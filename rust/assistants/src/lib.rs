use std::{cmp::Ordering, sync::Arc};

use assistant::{
    common::{
        eyre::{bail, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
    },
    Assistant, GenerateDetails, GenerateOptions, GenerateOutput, GenerateTask,
};

pub use assistant;

pub mod testing;
mod testing_db;

/// Get a list of available assistants in descending preference rank
pub async fn list() -> Vec<Arc<dyn Assistant>> {
    let futures = (0..=5).into_iter().map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", assistant_anthropic::list().await),
            1 => ("Google", assistant_google::list().await),
            2 => ("Mistral", assistant_mistral::list().await),
            3 => ("Ollama", assistant_ollama::list().await),
            4 => ("OpenAI", assistant_openai::list().await),
            5 => ("Stencila", assistant_custom::list().await),
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

/// Perform a `GenerateTask` and return the generated content as a string
pub async fn perform_task(
    mut task: GenerateTask,
    options: &GenerateOptions,
) -> Result<(GenerateOutput, GenerateDetails)> {
    let assistants = list().await;

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

    let Some(assistant) = assistants.get(index) else {
        bail!("Best assistant not in list of assistants!?")
    };

    assistant.perform_task(task, options).await
}
