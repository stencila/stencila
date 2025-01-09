use std::sync::Arc;

use cached::proc_macro::cached;

use model::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        inflector::Inflector,
        itertools::Itertools,
        reqwest::Client,
        serde::{Deserialize, Serialize},
    },
    Model, ModelAvailability, ModelIO, ModelOutput, ModelTask, ModelType,
};

/// A model available via Stencila Cloud
#[derive(Default, Clone, Serialize, Deserialize)]
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
        if self.provider == "stencila" && self.identifier == "router" {
            ModelType::Router
        } else {
            ModelType::Proxied
        }
    }

    fn availability(&self) -> ModelAvailability {
        cloud::api_key()
            .as_ref()
            .map(|_| ModelAvailability::Available)
            .unwrap_or(ModelAvailability::RequiresKey)
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let token = cloud::api_key().ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or key chain entry found. Get one at https://stencila.cloud/."))?;

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        let response = self
            .client
            .post(format!("{}/models/task", cloud::base_url()))
            .bearer_auth(token)
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

/// Get a list of all models available from Stencila Cloud.
///
/// Memoized for one minutes to avoid loading from disk cache too frequently
/// but allowing user to set API key while process is running.
#[cached(time = 60, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    Ok(list_stencila_models(0)
        .await?
        .into_iter()
        .map(|model| Arc::new(model) as Arc<dyn Model>)
        .collect_vec())
}

/// Fetch the list of models
///
/// In-memory cached for six hours to reduce requests to remote API.
#[cached(time = 21_600, result = true)]
async fn list_stencila_models(_unused: u8) -> Result<Vec<StencilaModel>> {
    let response = Client::new()
        .get(format!("{}/models", cloud::base_url()))
        .send()
        .await?;

    if let Err(error) = response.error_for_status_ref() {
        let message = response.text().await?;
        bail!("{error}: {message}");
    }

    Ok(response.json().await?)
}

#[cfg(test)]
#[allow(clippy::print_stderr)]
mod tests {
    use super::*;
    use model::{common::tokio, test_task_repeat_word};

    #[tokio::test]
    async fn list_models() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping test on CI until deployed");
            return Ok(());
        }

        let list = list().await?;
        assert!(!list.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_router() -> Result<()> {
        if cloud::api_key().is_none() {
            return Ok(());
        }

        let model = StencilaModel {
            provider: "stencila".to_string(),
            identifier: "router".to_string(),
            ..Default::default()
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;
        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn perform_task_anthropic() -> Result<()> {
        if cloud::api_key().is_none() {
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
        if cloud::api_key().is_none() {
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
        if cloud::api_key().is_none() {
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
        if cloud::api_key().is_none() {
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
