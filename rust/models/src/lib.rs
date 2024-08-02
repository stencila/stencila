#![recursion_limit = "256"]

use std::sync::Arc;

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

pub mod cli;

/// Get a list of available models
pub async fn list() -> Vec<Arc<dyn Model>> {
    let futures = (0..=5).map(|provider| async move {
        let (provider, result) = match provider {
            0 => ("Anthropic", models_anthropic::list().await),
            1 => ("Google", models_google::list().await),
            2 => ("Mistral", models_mistral::list().await),
            3 => ("Ollama", models_ollama::list().await),
            4 => ("OpenAI", models_openai::list().await),
            5 => ("Plugins", plugins::models::list().await),
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

/// Get a score for a model
///
/// Based on https://artificialanalysis.ai/leaderboards/models
/// from 2024-07-30. Only the top models from supported
/// models are listed here.
fn score(name: &str) -> f32 {
    match name {
        "anthropic/claude-3-5-sonnet-20240620" => 98.0,
        "anthropic/claude-3-opus-20240229" => 93.0,
        "anthropic/claude-3-haiku-20240307" => 74.0,
        "google/gemini-1.5-pro-001" => 95.,
        "google/gemini-1.5-flash-001" => 84.,
        "openai/gpt-4o-2024-05-13" => 100.,
        "openai/gpt-4-turbo-2024-04-09" => 94.,
        "openai/gpt-4o-mini-2024-07-18" => 88.,
        "mistral/mistral-large-2407" => 76.,
        "mistral/mistral-medium-2312" => 70.,
        "mistral/mistral-small-2402" => 71.,
        _ => 50.,
    }
}

/// Select a model based on selection criteria of the `InstructionModel`
pub async fn select(task: &ModelTask) -> Result<Arc<dyn Model>> {
    // Get the list of available models
    let models = list().await;

    // Filter according to whether the task is supported, and the selection criteria
    let regex = match task
        .instruction_model
        .as_ref()
        .and_then(|model| model.id_pattern.as_deref())
    {
        Some(pattern) => {
            let regex = pattern.replace('.', r"\.").replace('*', "(.*?)");
            Some(Regex::new(&regex)?)
        }
        None => None,
    };
    let mut models = models
        .into_iter()
        .filter(|model| {
            if !model.supports_task(task) {
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
        bail!("No models available that match criteria")
    }

    if models.len() == 1 {
        // Return early with the only model
        return Ok(models.swap_remove(0));
    }

    // Score models, getting max score as we go
    let mut max_score = 0.;
    let mut model_scores = Vec::new();
    for model in models.into_iter() {
        let score = score(&model.id());
        if score > max_score {
            max_score = score;
        }
        model_scores.push((model, score))
    }

    // Filter out models below min score
    let min_score = task
        .instruction_model
        .as_ref()
        .and_then(|model| model.minimum_score)
        .map(|min| min as f32)
        .unwrap_or(95.);

    let mut models = model_scores
        .into_iter()
        .filter_map(|(model, score)| ((score / max_score) * 100. >= min_score).then_some(model))
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
