#![recursion_limit = "256"]

use std::{cmp::Ordering, sync::Arc};

use assistant::{
    common::{
        eyre::{bail, eyre, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
    },
    context::Context,
    Assistant, GenerateOptions, GenerateOutput, GenerateTask, Instruction,
};

pub use assistant;

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

/// Execute an instruction
pub async fn execute_instruction<T>(
    instruction: T,
    context: Context,
    options: GenerateOptions,
) -> Result<GenerateOutput>
where
    Instruction: From<T>,
{
    let instruction = Instruction::from(instruction);

    let mut task = GenerateTask {
        instruction,
        context: Some(context),
        ..Default::default()
    };

    let assistant = get_assistant(&mut task).await?;
    assistant.perform_task(&task, &options).await
}

/// Get the assistant for a task
///
/// If the task's instruction has an `assignee` (and assignee exists and supports the
/// task) then returns that assistant. Otherwise returns the assignee with the highest
/// suitability score for the task.
pub async fn get_assistant(task: &mut GenerateTask) -> Result<Arc<dyn Assistant>> {
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
