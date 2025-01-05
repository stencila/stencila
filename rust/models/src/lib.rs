#![recursion_limit = "256"]

use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use model::common::{
    eyre::{bail, Result},
    futures::future::join_all,
    itertools::Itertools,
    tracing,
};

pub use model::{
    Model, ModelAvailability, ModelOutput, ModelOutputKind, ModelSpecification, ModelTask,
    ModelType,
};

pub mod cli;

/// Get a list of available models
pub async fn list() -> Vec<Arc<dyn Model>> {
    let futures = (0..=6).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", models_anthropic::list().await),
            1 => ("Google", models_google::list().await),
            2 => ("Mistral", models_mistral::list().await),
            3 => ("Ollama", models_ollama::list().await),
            4 => ("OpenAI", models_openai::list().await),
            5 => ("Plugins", plugins::models::list().await),
            6 => ("Stencila", models_stencila::list().await),
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

    let list = join_all(futures).await.into_iter().flatten();

    // Ensure that ids are unique, taking those with lower rank (higher
    // preference) type: Local < Remote < Proxied. This avoids having
    // proxied models clashing with remote models when user has both
    // Stencila API key and other provider API keys set
    let mut unique = HashMap::new();
    for model in list {
        unique
            .entry(model.id())
            .and_modify(|existing: &mut Arc<dyn Model>| {
                if existing.r#type() > model.r#type() {
                    *existing = model.clone();
                }
            })
            .or_insert(model);
    }

    unique
        .into_values()
        .sorted_by(|a, b| match (a.r#type(), b.r#type()) {
            (ModelType::Router, _) => Ordering::Less,
            (_, ModelType::Router) => Ordering::Greater,
            _ => match a.provider().cmp(&b.provider()) {
                Ordering::Equal => match a.name().cmp(&b.name()) {
                    Ordering::Equal => a.version().cmp(&b.version()).reverse(),
                    order => order,
                },
                order => order,
            },
        })
        .collect_vec()
}

/// Select a model based on selection criteria of the `ModelParameters`
#[tracing::instrument(skip_all)]
pub async fn select(task: &ModelTask) -> Result<Arc<dyn Model>> {
    tracing::trace!("Selecting a model for task");

    // Get the list models
    let models = list().await;

    // Check that there is at least one model available so we can advise the user
    // that they might need to provide an API key or run a model locally
    if !models.iter().any(|model| model.is_available()) {
        let message =
            "No AI models available. Please sign in to Stencila Cloud, set STENCILA_API_TOKEN, or configure local models.";

        // Log message so it is visible in console or an the LSP client
        tracing::error!(message);

        // Throw message so it is set as an `ExecutionMessage` on the `Instruction`
        bail!(message)
    }

    // If the task includes model ids, use the first model matching on of the
    // ids (there should normally only be one) and error if not found
    if let Some(model_ids) = task
        .model_parameters
        .as_ref()
        .and_then(|pars| pars.model_ids.as_ref())
    {
        for model in models {
            if !model.is_available() {
                continue;
            }
            for id in model_ids {
                if id == "*" || model.id().contains(id) {
                    return Ok(model);
                }
            }
        }

        bail!("No model with id matching '{}'", model_ids.join(","))
    }

    // If the task does not specify model ids and a model router is available
    // then use the first router
    if let Some(model) = models
        .iter()
        .find(|model| model.is_available() && matches!(model.r#type(), ModelType::Router))
    {
        return Ok(model.clone());
    }

    // Filter according to whether the task is supported
    let mut models = models
        .into_iter()
        .filter(|model| {
            if !model.is_available() || !model.supports_task(task) {
                return false;
            }
            true
        })
        .collect_vec();

    if models.is_empty() {
        bail!("No AI models available that support this task")
    }

    Ok(models.swap_remove(0))
}

/// Perform a model task
#[tracing::instrument(skip_all)]
pub async fn perform_task(task: ModelTask) -> Result<ModelOutput> {
    tracing::debug!("Performing model task");

    let model = select(&task).await?;
    model.perform_task(&task).await
}
