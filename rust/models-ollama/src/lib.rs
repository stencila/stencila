use std::sync::Arc;

use cached::proc_macro::cached;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, MessageRole},
        images::Image,
        options::GenerationOptions,
    },
    models::LocalModel,
    Ollama,
};

use model::{
    common::{
        async_trait::async_trait,
        eyre::{eyre, Result},
        inflector::Inflector,
        tracing,
    },
    schema::{self, ImageObject, MessagePart},
    Model, ModelIO, ModelOutput, ModelTask, ModelType,
};

/// A model running on a Ollama (https://github.com/jmorganca/ollama/) server
///
/// To start an Ollama server:
///
/// ```sh
/// ollama serve
/// ```
///
/// On Linux, to stop the server:
///
/// ```sh
/// sudo service ollama stop
/// ```
///
/// A model is listed for each Ollama model that has previously been pulled.
pub struct OllamaModel {
    /// The Ollama name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

    /// The context length of the model
    context_length: usize,

    /// The Ollama API client
    client: Ollama,
}

impl OllamaModel {
    /// Create a Ollama-based model
    pub fn new(model: String, context_length: usize) -> Self {
        Self::new_with(model, context_length, None, None)
    }

    /// Create a Ollama-based model with options for address of server
    pub fn new_with(
        model: String,
        context_length: usize,
        host: Option<String>,
        port: Option<u16>,
    ) -> Self {
        let host = host.unwrap_or("http://localhost".to_string());
        let port = port.unwrap_or(11434);
        let client = Ollama::new(host, port);

        Self {
            model,
            context_length,
            client,
        }
    }
}

#[async_trait]
impl Model for OllamaModel {
    fn id(&self) -> String {
        format!("ollama/{}", self.model)
    }

    fn r#type(&self) -> ModelType {
        ModelType::Local
    }

    fn name(&self) -> String {
        let name = self.id();
        let name = name
            .rsplit_once('/')
            .map(|(.., name)| name.split_once(':').map_or(name, |(name, ..)| name))
            .unwrap_or(&name);
        name.to_title_case()
    }

    fn version(&self) -> String {
        let name = self.id();
        let version = name
            .split_once(':')
            .map(|(.., version)| version)
            .unwrap_or(&name);
        version.to_string()
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    fn supported_outputs(&self) -> &[ModelIO] {
        &[ModelIO::Text]
    }

    async fn perform_task(&self, task: &ModelTask) -> Result<ModelOutput> {
        let messages = task
            .messages
            .iter()
            .map(|message| {
                let role = match message.role.clone().unwrap_or_default() {
                    schema::MessageRole::Model => MessageRole::Assistant,
                    schema::MessageRole::System => MessageRole::System,
                    schema::MessageRole::User => MessageRole::User,
                };

                let mut content = String::new();
                let mut images = vec![];
                for part in &message.parts {
                    match part {
                        MessagePart::Text(text) => {
                            content += &text.value;
                        }
                        MessagePart::ImageObject(ImageObject { content_url, .. }) => {
                            if let Some(pos) = content_url.find(";base64,") {
                                let base64 = &content_url[(pos + 8)..];
                                images.push(Image::from_base64(base64))
                            } else {
                                tracing::warn!(
                                    "Image does not appear to have a DataURI so was ignored by model `{}`",
                                    self.id()
                                );
                            }
                        }
                        _ => {
                            tracing::warn!(
                                "Message part `{part}` is ignored by model `{}`",
                                self.id()
                            );
                        }
                    }
                }

                ChatMessage {
                    role,
                    content,
                    images: if images.is_empty() {
                        None
                    } else {
                        Some(images)
                    },
                }
            })
            .collect();

        let mut request = ChatMessageRequest::new(self.model.clone(), messages);

        // Map options to Ollama options
        let mut options = GenerationOptions::default();
        macro_rules! map_option {
            ($from:ident, $to:ident) => {
                if let Some(value) = &task.$from {
                    options = options.$to(value.clone());
                }
            };
            ($name:ident) => {
                map_option!($name, $name)
            };
        }
        macro_rules! ignore_option {
            ($name:ident) => {
                if task.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by model `{}` for text-to-text generation",
                        stringify!($name),
                        self.name()
                    )
                }
            };
        }
        map_option!(mirostat);
        map_option!(mirostat_eta);
        map_option!(mirostat_tau);
        map_option!(num_ctx);
        map_option!(num_gqa);
        map_option!(num_gpu);
        map_option!(num_thread);
        map_option!(repeat_last_n);
        map_option!(repeat_penalty);
        map_option!(temperature);
        map_option!(seed);
        if let Some(value) = &task.stop {
            options = options.stop(vec![value.clone()]);
        }
        if let Some(value) = task.max_tokens {
            options = options.num_predict(value as i32);
        }
        map_option!(tfs_z);
        map_option!(top_k);
        map_option!(top_p);
        ignore_option!(image_size);
        ignore_option!(image_quality);
        ignore_option!(image_style);

        request.options = Some(options);

        if task.dry_run {
            return ModelOutput::empty(self);
        }

        let response = self
            .client
            .send_chat_messages(request)
            .await
            .map_err(|error| eyre!(error))?;

        let text = response
            .message
            .map(|message| message.content)
            .unwrap_or_default();

        ModelOutput::from_text(self, &task.format, text).await
    }
}

/// Get a list of all available Ollama models
///
/// Fetches the list of Ollama models from the server and maps them
/// into models.
///
/// If there is no server listening on port 11434 (the default for Ollama)
/// returns an empty list.
///
/// Note that this uses a fixed assume context length for all models
/// (which will be probably be wrong for some). At the time of writing
/// there does not appear to be an easy way to get the actual context
/// length of an Ollama model (i.e. it is not in the API).
#[cached(time = 120, result = true)]
pub async fn list() -> Result<Vec<Arc<dyn Model>>> {
    if std::net::TcpStream::connect("127.0.0.1:11434").is_err() {
        return Ok(vec![]);
    }

    let models = list_ollama_models(0)
        .await?
        .into_iter()
        .map(|model| Arc::new(OllamaModel::new(model.name, 4096)) as Arc<dyn Model>)
        .collect();

    Ok(models)
}

/// Fetch the list of models
///
/// In-memory cached for 5 minutes to reduce the number of times that Ollama needs to
/// be started while allowing for new models to be pulled and to appear here.
#[cached(time = 3000, result = true)]
async fn list_ollama_models(_unused: u8) -> Result<Vec<LocalModel>> {
    Ok(Ollama::default().list_local_models().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{common::tokio, test_task_repeat_word};

    #[tokio::test]
    async fn list_models() -> Result<()> {
        // Just check this does not error since list may be empty is Ollama
        // not installed or has no models.
        list().await?;

        Ok(())
    }

    #[tokio::test]
    async fn perform_task() -> Result<()> {
        let list = list().await?;
        let Some(model) = list.first() else {
            return Ok(());
        };
        let output = model.perform_task(&test_task_repeat_word()).await?;

        assert_eq!(output.content.trim(), "HELLO".to_string());

        Ok(())
    }
}
