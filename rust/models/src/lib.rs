#![recursion_limit = "256"]

use std::{cmp::Ordering, sync::Arc};

use model::{
    common::{
        eyre::{bail, Result},
        futures::future::join_all,
        itertools::Itertools,
        rand::{self, Rng},
        regex::Regex,
        tracing,
    },
    Model, ModelOutput, ModelTask,
};

pub use model::{ModelAvailability, ModelSpecification, ModelType};

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

    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .sorted_by(|a, b| match a.r#type().cmp(&b.r#type()) {
            Ordering::Equal => a.id().cmp(&b.id()),
            order => order,
        })
        .collect_vec()
}

/// Get an overall score for a model
///
/// Task specific, and more frequently updated, scores are available by
/// using the stencila/auto router alias.
fn score(id: &str) -> u32 {
    match id {
        // Automatic selection based on the task type, assistant and users
        "stencila/auto" => 100,
        // Only the top models from supported providers are listed here.
        "anthropic/claude-3-5-sonnet-20240620" => 98,
        "anthropic/claude-3-opus-20240229" => 93,
        "anthropic/claude-3-haiku-20240307" => 74,
        "google/gemini-1.5-pro-001" => 95,
        "google/gemini-1.5-flash-001" => 84,
        "openai/gpt-4o-2024-05-13" => 100,
        "openai/gpt-4o-2024-08-06" => 100,
        "openai/gpt-4-turbo-2024-04-09" => 94,
        "openai/gpt-4o-mini-2024-07-18" => 88,
        "mistral/mistral-large-2407" => 76,
        "mistral/mistral-medium-2312" => 70,
        "mistral/mistral-small-2402" => 71,
        _ => 50,
    }
}

/// Select a model based on selection criteria of the `ModelParameters`
#[tracing::instrument(skip_all)]
pub async fn select(task: &ModelTask) -> Result<Arc<dyn Model>> {
    tracing::trace!("Selecting a model for task");

    // Get the list models
    let models = list().await;

    // Check that there is at least one model available so we can advise the user
    // that they might need to provide an API key or run a model locally
    if !models.iter().any(|any| any.is_available()) {
        let message =
            "No AI models available. Please sign in to Stencila Cloud, set STENCILA_API_TOKEN, or configure local models.";

        // Log message so it is visible in console or an the LSP client
        tracing::error!(message);

        // Throw message so it is set as an `ExecutionMessage` on the `Instruction`
        bail!(message)
    }

    // If a model router is available and the task does not specify a id pattern
    // then use the first router
    if task
        .model_parameters
        .as_ref()
        .and_then(|pars| pars.model_ids.clone())
        .is_none()
    {
        if let Some(model) = models
            .iter()
            .find(|model| model.is_available() && matches!(model.r#type(), ModelType::Router))
        {
            return Ok(model.clone());
        }
    }

    // Filter according to whether the task is supported, and the selection criteria
    let regex = match task
        .model_parameters
        .as_ref()
        .and_then(|model| model.model_ids.as_deref())
    {
        Some(pattern) => {
            // TODO: don't join here
            let regex = pattern.join("").replace('.', r"\.").replace('*', "(.*?)");
            Some(Regex::new(&regex)?)
        }
        None => None,
    };
    let mut models = models
        .into_iter()
        .filter(|model| {
            if !model.is_available() || !model.supports_task(task) {
                return false;
            }

            if let Some(regex) = &regex {
                if !regex.is_match(&model.id()) {
                    return false;
                }
            }

            true
        })
        .collect_vec();

    if models.is_empty() {
        // This gets set as an `ExecutionMessage` on the `Instruction`
        bail!("No AI models available that support this command")
    }

    if models.len() == 1 {
        // Return early with the only model
        return Ok(models.swap_remove(0));
    }

    // Score models, getting max score as we go
    let mut max_score = 0u32;
    let mut model_scores = Vec::new();
    for model in models.into_iter() {
        let score = score(&model.id());
        if score > max_score {
            max_score = score;
        }
        model_scores.push((model, score))
    }

    let Some(min_score) = task
        .model_parameters
        .as_ref()
        .and_then(|model| model.minimum_score)
    else {
        // No min score defined so just return the model with best score with ties broken by type
        model_scores.sort_by(|a, b| match a.1.cmp(&b.1) {
            Ordering::Equal => a.0.r#type().cmp(&b.0.r#type()),
            order => order,
        });
        return Ok(model_scores.pop().expect("should be at least one model").0);
    };

    // Filter out models below min score
    let mut models = model_scores
        .into_iter()
        .filter_map(|(model, score)| {
            ((score as f32 / max_score as f32) * 100. >= min_score as f32).then_some(model)
        })
        .collect_vec();

    // Randomly select one of the filtered models
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..models.len());
    Ok(models.swap_remove(index))
}

/// Perform a model task
pub async fn perform_task(task: ModelTask) -> Result<ModelOutput> {
    let model = select(&task).await?;
    model.perform_task(&task).await
}
