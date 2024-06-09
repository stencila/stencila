#![recursion_limit = "256"]

use std::sync::Arc;

use model::{
    common::{
        eyre::{bail, eyre, Result},
        futures::future::join_all,
        itertools::Itertools,
        tracing,
    },
    GenerateTask, Model,
};

pub mod cli;

/// Get a list of available models
pub async fn list() -> Vec<Arc<dyn Model>> {
    let futures = (0..=4).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", models_anthropic::list().await),
            1 => ("Google", models_google::list().await),
            2 => ("Mistral", models_mistral::list().await),
            3 => ("Ollama", models_ollama::list().await),
            4 => ("OpenAI", models_openai::list().await),
            _ => return vec![],
        };

        match result {
            Ok(list) => list,
            Err(error) => {
                tracing::error!("While listing {provider} models: {error}");
                vec![]
            }
        }
    });

    join_all(futures).await.into_iter().flatten().collect_vec()
}

/// Get a model
pub async fn get(name: &str, task: &GenerateTask) -> Result<Arc<dyn Model>> {
    let models = list().await;

    let model = models
        .iter()
        .find(|model| model.name() == name)
        .ok_or_else(|| eyre!("No model with name `{name}`"))?;

    // Check that the model supports the task
    if !model.supports_task(task) {
        bail!("The assigned model `{name}` does not support this task")
    }

    tracing::debug!("Using model with name: {}", model.name());

    Ok(model.clone())
}
