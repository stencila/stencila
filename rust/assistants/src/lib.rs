use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use assistant::{
    common::{
        async_recursion::async_recursion,
        eyre::{bail, eyre, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
        uuid::Uuid,
    },
    schema::{
        walk::{VisitorMut, WalkControl, WalkNode},
        Block, ExecutionError, Inline, Node,
    },
    Assistant, GenerateOptions, GenerateOutput, GenerateTask, Instruction, Nodes,
};

pub use assistant;

pub mod testing;
mod testing_db;

/// Get a list of available assistants in descending preference rank
pub async fn list() -> Vec<Arc<dyn Assistant>> {
    let futures = (0..=5).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", assistant_anthropic::list().await),
            1 => ("Google", assistant_google::list().await),
            2 => ("Mistral", assistant_mistral::list().await),
            3 => ("Ollama", assistant_ollama::list().await),
            4 => ("OpenAI", assistant_openai::list().await),
            5 => ("Stencila", assistant_custom::list()),
            _ => return vec![],
        };

        match result {
            Ok(list) => list,
            Err(error) => {
                tracing::error!("While listing {provider} assistants: {error}");
                vec![]
            }
        }
    });

    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .sorted_by(
            |a, b| match a.preference_rank().cmp(&b.preference_rank()).reverse() {
                Ordering::Equal => a.id().cmp(&b.id()),
                ordering => ordering,
            },
        )
        .collect_vec()
}

/// Perform an instruction and return the generated content as a string
#[tracing::instrument(skip_all)]
#[async_recursion]
pub async fn perform_instruction(
    instruction: Instruction,
    document: Option<Node>,
    options: &GenerateOptions,
) -> Result<GenerateOutput> {
    let mut task = GenerateTask::with_doc(instruction, document.clone());

    let assistants = list().await;

    let assistant = if let Some(assignee) = task.instruction.assignee() {
        // Get the assistant assigned to the task

        let id = if assignee.contains('/') {
            assignee.to_string()
        } else {
            ["stencila/", assignee].concat()
        };

        assistants
            .iter()
            .find(|assistant| assistant.id() == id)
            .ok_or_else(|| eyre!("No assistant with id `{id}`"))?
    } else {
        // Get the assistant with the highest suitability score for the task

        // It is tempting to use the position_max iterator method here but, in the case of
        // ties, that returns the assistant with the higher index (ie. lower preference), whereas
        // we want the one with the lowest index.
        let mut best = (0., 0);
        for (index, assistant) in assistants.iter().enumerate() {
            let score = assistant.suitability_score(&mut task)?;
            if score > best.0 {
                best = (score, index);
            }
        }

        let (max, index) = best;
        if max == 0. {
            bail!("Unable to delegate the task, no assistants with suitable capabilities")
        }

        assistants
            .get(index)
            .ok_or_else(|| eyre!("Best assistant not in list of assistants!?"))?
    };

    // Perform the task
    let mut output = assistant.perform_task(&task, options).await?;

    // Collect instructions within the generated nodes
    let mut collector = InstructionCollector::default();
    match &mut output.nodes {
        Nodes::Blocks(nodes) => nodes.walk_mut(&mut collector),
        Nodes::Inlines(nodes) => nodes.walk_mut(&mut collector),
    }

    // Perform inner instructions in parallel and put results into a hash map
    // so they can be applied to the instructions
    let futures = collector.instructions.into_iter().map(|(id, instruction)| {
        let document = document.clone();
        async move {
            (
                id,
                perform_instruction(instruction, document, options).await,
            )
        }
    });
    let results = join_all(futures).await.into_iter().collect();

    // Apply the results to the instructions
    let mut applier = ResultApplier { results };
    match &mut output.nodes {
        Nodes::Blocks(nodes) => nodes.walk_mut(&mut applier),
        Nodes::Inlines(nodes) => nodes.walk_mut(&mut applier),
    }

    Ok(output)
}

/// A node visitor which collects instructions
#[derive(Default)]
struct InstructionCollector {
    /// A map of instructions by their id
    instructions: HashMap<String, Instruction>,
}

impl VisitorMut for InstructionCollector {
    fn visit_inline_mut(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::InstructionInline(instruction) = inline {
            let id = instruction
                .id
                .get_or_insert_with(|| Uuid::new_v4().to_string())
                .clone();
            let instruction = Instruction::from(instruction.clone());
            self.instructions.insert(id, instruction);
        }
        WalkControl::Continue
    }

    fn visit_block_mut(&mut self, block: &mut Block) -> WalkControl {
        if let Block::InstructionBlock(instruction) = block {
            let id = instruction
                .id
                .get_or_insert_with(|| Uuid::new_v4().to_string())
                .clone();
            let instruction = Instruction::from(instruction.clone());
            self.instructions.insert(id, instruction);
        }
        WalkControl::Continue
    }
}

/// A node visitor which applies generation results to instructions
struct ResultApplier {
    /// A map of generation results by instruction id
    results: HashMap<String, Result<GenerateOutput>>,
}

impl VisitorMut for ResultApplier {
    fn visit_inline_mut(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::InstructionInline(instruction) = inline {
            if let Some(result) = instruction
                .id
                .as_ref()
                .and_then(|id| self.results.remove(id))
            {
                match result {
                    Ok(output) => {
                        instruction.messages.push(output.to_message());
                        instruction.options.suggestion = Some(output.to_suggestion_inline())
                    }
                    Err(error) => {
                        instruction.options.execution_errors =
                            Some(vec![ExecutionError::new(error.to_string())]);
                    }
                }
            } else {
                tracing::debug!("Instruction has no id and/or result: {instruction:?}")
            }
        }
        WalkControl::Continue
    }

    fn visit_block_mut(&mut self, block: &mut Block) -> WalkControl {
        if let Block::InstructionBlock(instruction) = block {
            if let Some(result) = instruction
                .id
                .as_ref()
                .and_then(|id| self.results.remove(id))
            {
                match result {
                    Ok(output) => {
                        instruction.messages.push(output.to_message());
                        instruction.options.suggestion = Some(output.to_suggestion_block());
                    }
                    Err(error) => {
                        instruction.options.execution_errors =
                            Some(vec![ExecutionError::new(error.to_string())]);
                    }
                }
            } else {
                tracing::debug!("Instruction has no id and/or result: {instruction:?}")
            }
        }
        WalkControl::Continue
    }
}
