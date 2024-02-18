#![recursion_limit = "256"]

use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use assistant::{
    common::{
        async_recursion::async_recursion,
        eyre::{bail, eyre, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
    },
    schema::{
        walk::{VisitorMut, WalkControl, WalkNode},
        Block, ExecutionMessage, ExecutionStatus, Inline, MessageLevel, Node, NodeId,
    },
    Assistant, GenerateOptions, GenerateOutput, GenerateTask, Instruction,
};

pub use assistant;

pub mod testing;

/// Get a list of available assistants in descending preference rank
pub async fn list() -> Vec<Arc<dyn Assistant>> {
    let futures = (0..=5).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", assistant_anthropic::list().await),
            1 => ("Google", assistant_google::list().await),
            2 => ("Mistral", assistant_mistral::list().await),
            3 => ("Ollama", assistant_ollama::list().await),
            4 => ("OpenAI", assistant_openai::list().await),
            5 => ("specialized", assistant_specialized::list()),
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

/// Perform all the instructions within a root document
#[tracing::instrument(skip_all)]
pub async fn perform_document<'doc>(
    document: &'doc mut Node,
    options: &GenerateOptions,
) -> Result<()> {
    let clone = document.clone();
    perform_instructions(document, Some(&clone), options).await
}

/// Perform the instructions within a node
#[tracing::instrument(skip_all)]
pub async fn perform_instructions<'doc, T>(
    node: &mut T,
    document: Option<&'doc Node>,
    options: &GenerateOptions,
) -> Result<()>
where
    T: WalkNode,
{
    // Collect instructions within the node/s
    let mut collector = InstructionCollector::default();
    node.walk_mut(&mut collector);

    // Perform instructions in parallel and put results into a hash map
    // so they can be applied to the instructions
    let futures = collector
        .instructions
        .into_iter()
        .map(|(id, instruction)| async move {
            (
                id,
                perform_instruction(instruction, document, options).await,
            )
        });
    let results = join_all(futures).await.into_iter().collect();

    // Apply the results to the collected instructions
    let mut applier = ResultApplier { results };
    node.walk_mut(&mut applier);

    Ok(())
}

/// Perform an instruction and return the generated output
#[tracing::instrument(skip_all)]
#[async_recursion]
pub async fn perform_instruction<'doc: 'async_recursion>(
    instruction: Instruction,
    document: Option<&'doc Node>,
    options: &GenerateOptions,
) -> Result<GenerateOutput> {
    let mut task = GenerateTask {
        instruction,
        document,
        ..Default::default()
    };

    let assistant = match get_assistant(&mut task).await {
        Err(error) => {
            tracing::warn!("While getting assistant: {}", error);
            return Err(error);
        }
        Ok(assistant) => assistant,
    };

    // Perform the task
    let mut output = match assistant.perform_task(&task, options).await {
        Err(error) => {
            tracing::warn!("While performing task: {}", error);
            return Err(error);
        }
        Ok(output) => output,
    };

    // Recursively perform any instructions within the output nodes
    perform_instructions(&mut output.nodes, document, options).await?;

    Ok(output)
}

/// Get the assistant for a task
///
/// If the task's instruction has an `assignee` (and assignee exists and supports the
/// task) then returns that assistant. Otherwise returns the assignee with the highest
/// suitability score for the task.
pub async fn get_assistant<'doc>(task: &mut GenerateTask<'doc>) -> Result<Arc<dyn Assistant>> {
    let assistants = list().await;

    let assistant = if let Some(assignee) = task.instruction.assignee() {
        let id = if assignee.contains('/') {
            assignee.to_string()
        } else {
            ["stencila/", assignee].concat()
        };

        let assistant = assistants
            .iter()
            .find(|assistant| assistant.id() == id)
            .ok_or_else(|| eyre!("No assistant with id `{id}`"))?;

        // Check that the assignee supports the task
        if !assistant.supports_task(task) {
            bail!("The assigned assistant `{id}` does not support this task")
        }

        tracing::debug!("Using assistant matching id: {}", assistant.id());
        assistant
    } else {
        // It is tempting to use the position_max iterator method here but, in the case of
        // ties, that returns the assistant with the higher index (ie. lower preference), whereas
        // we want the one with the lowest index.
        let mut best = (0., 0);
        for (index, assistant) in assistants.iter().enumerate() {
            let score = assistant.suitability_score(task)?;
            if score > best.0 {
                best = (score, index);
            }
        }

        let (max, index) = best;
        if max == 0. {
            bail!("Unable to delegate the task, no assistants with suitable capabilities")
        }

        let assistant = assistants
            .get(index)
            .ok_or_else(|| eyre!("Best assistant not in list of assistants!?"))?;

        tracing::debug!(
            "Found assistant {}, with best score {}",
            assistant.id(),
            max
        );
        assistant
    };

    Ok(assistant.clone())
}

/// A node visitor which collects instructions
#[derive(Default)]
struct InstructionCollector {
    /// A map of instructions by their id
    instructions: HashMap<NodeId, Instruction>,
}

impl VisitorMut for InstructionCollector {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::InstructionInline(instruction) = inline {
            let id = instruction.node_id();
            let instruction = Instruction::from(instruction.clone());
            self.instructions.insert(id, instruction);
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::InstructionBlock(instruction) = block {
            let id = instruction.node_id();
            let instruction = Instruction::from(instruction.clone());
            self.instructions.insert(id, instruction);
        }
        WalkControl::Continue
    }
}

/// A node visitor which applies generation results to instructions
struct ResultApplier {
    /// A map of generation results by instruction id
    results: HashMap<NodeId, Result<GenerateOutput>>,
}

impl VisitorMut for ResultApplier {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::InstructionInline(instruction) = inline {
            if let Some(result) = self.results.remove(&instruction.node_id()) {
                match result {
                    Ok(output) => {
                        instruction.messages.push(output.to_message());
                        instruction.options.suggestion =
                            Some(output.to_suggestion_inline(instruction.content.is_none()));

                        instruction.options.execution_status = Some(ExecutionStatus::Succeeded);
                    }
                    Err(error) => {
                        tracing::error!("Instruction failed: {}", error.to_string());
                        instruction.options.execution_status = Some(ExecutionStatus::Failed);
                        instruction.options.execution_messages = Some(vec![ExecutionMessage::new(
                            MessageLevel::Error,
                            error.to_string(),
                        )]);
                    }
                }
            } else {
                tracing::debug!("Instruction has no id and/or result: {instruction:?}")
            }
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::InstructionBlock(instruction) = block {
            if let Some(result) = self.results.remove(&instruction.node_id()) {
                match result {
                    Ok(output) => {
                        instruction.messages.push(output.to_message());
                        instruction.options.suggestion =
                            Some(output.to_suggestion_block(instruction.content.is_none()));

                        instruction.options.execution_status = Some(ExecutionStatus::Succeeded);
                    }
                    Err(error) => {
                        instruction.options.execution_status = Some(ExecutionStatus::Failed);
                        instruction.options.execution_messages = Some(vec![ExecutionMessage::new(
                            MessageLevel::Error,
                            error.to_string(),
                        )]);
                    }
                }
            } else {
                tracing::debug!("Instruction has no id and/or result: {instruction:?}")
            }
        }
        WalkControl::Continue
    }
}
