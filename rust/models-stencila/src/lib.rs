use std::{env, sync::Arc};

use cached::proc_macro::cached;

use model::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        inflector::Inflector,
        itertools::Itertools,
        reqwest::Client,
        serde::{Deserialize, Serialize},
    },
    secrets, Model, ModelIO, ModelOutput, ModelTask, ModelType,
};

/// The base URL for the Stencila Cloud API
///
/// Can be overridden by setting the STENCILA_API_URL environment variable.
const BASE_URL: &str = "https://api.stencila.cloud";

fn base_url() -> String {
    env::var("STENCILA_API_URL").unwrap_or_else(|_| BASE_URL.to_string())
}

/// The name of the env var or secret for the API key
const API_KEY: &str = "STENCILA_API_TOKEN";

/// A model available via Stencila Cloud
#[derive(Default, Deserialize)]
#[serde(crate = "model::common::serde")]
pub struct StencilaModel {
    /// The name of the provider e.g. openai
    provider: String,

    /// The provider's identifier for the model e.g. gpt-4o-mini-2024-07-18
    identifier: String,

    /// The name of the model e.g. GPT
    name: String,

    /// The version of the model e.g. 4o-mini-2024-07-18
    version: String,

    /// The HTTP client for performing tasks via the Stencila Cloud API
    #[serde(skip)]
    client: Client,
}

#[async_trait]
impl Model for StencilaModel {
    fn id(&self) -> String {
        [&self.provider, "/", &self.identifier].concat()
    }

    fn provider(&self) -> String {
        match self.provider.as_str() {
            "openai" => "OpenAI".to_string(),
            provider => provider.to_title_case(),
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn r#type(&self) -> ModelType {
        if self.provider == "stencila" && self.identifier == "auto" {
            ModelType::Router
        } else {
            ModelType::Proxy
        }
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let response = self
            .client
            .post(format!("{}/models/task", base_url()))
            .bearer_auth(secrets::env_or_get(API_KEY)?)
            .json(&PerformTaskRequest {
                provider: self.provider.clone(),
                identifier: self.identifier.clone(),
                task: task.clone(),
            })
            .send()
            .await?;

        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        let output: ModelOutput = response.json().await?;

        Ok(output)
    }
}

#[derive(Serialize)]
#[serde(crate = "model::common::serde")]
struct PerformTaskRequest {
    provider: String,
    identifier: String,
    task: ModelTask,
}

/// Get a list of all models available via Stencila Cloud.
#[cached(time = 3600, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    let response = Client::new()
        .get(format!("{}/models", base_url()))
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        let message = response.text().await?;
        bail!("{error}: {message}");
    }

    let models: Vec<StencilaModel> = response.json().await?;

    Ok(models
        .into_iter()
        .map(|model| Arc::new(model) as Arc<dyn Model>)
        .collect_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{common::tokio, test_task_repeat_word};

    #[tokio::test]
    async fn list_models() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping test on CI until deployed");
            return Ok(());
        }

        let list = list().await?;
        assert!(!list.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_auto() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "stencila".to_string(),
            identifier: "auto".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_anthropic() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "anthropic".to_string(),
            identifier: "claude-3-haiku-20240307".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_google() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "google".to_string(),
            identifier: "gemini-1.5-flash-001".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_mistral() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "mistral".to_string(),
            identifier: "mistral-small-2402".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_openai() -> Result<()> {
        if secrets::env_or_get(API_KEY).is_err() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "openai".to_string(),
            identifier: "gpt-4o-mini-2024-07-18".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }
}
