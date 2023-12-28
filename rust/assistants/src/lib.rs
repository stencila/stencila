use std::sync::Arc;

use assistant::{
    common::{
        eyre::{bail, Result},
        tracing,
    },
    Assistant, GenerateDetails, GenerateOptions, GenerateOutput, GenerateTask,
};

pub use assistant;

pub mod testing;
mod testing_db;

/// Get a list of available assistants in descending preference rank
pub async fn list() -> Vec<Arc<dyn Assistant>> {
    let mut all = Vec::new();

    match assistant_custom::list().await {
        Ok(mut assistants) => all.append(&mut assistants),
        Err(error) => tracing::error!("While listing Stencila assistants: {error}"),
    }

    match assistant_openai::list().await {
        Ok(mut assistants) => all.append(&mut assistants),
        Err(error) => tracing::error!("While listing OpenAI assistants: {error}"),
    }

    match assistant_anthropic::list().await {
        Ok(mut assistants) => all.append(&mut assistants),
        Err(error) => tracing::error!("While listing Anthropic assistants: {error}"),
    }

    match assistant_mistral::list().await {
        Ok(mut assistants) => all.append(&mut assistants),
        Err(error) => tracing::error!("While listing Mistral assistants: {error}"),
    }

    match assistant_ollama::list().await {
        Ok(mut assistants) => all.append(&mut assistants),
        Err(error) => tracing::error!("While listing Ollama assistants: {error}"),
    }

    all.sort_by(|a, b| a.preference_rank().cmp(&b.preference_rank()).reverse());

    all
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
