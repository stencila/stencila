use std::sync::Arc;

use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

use assistant::{
    common::{
        async_trait::async_trait,
        eyre::{eyre, Result},
        inflector::Inflector,
        tracing,
    },
    Assistant, AssistantIO, GenerateOptions, GenerateOutput, GenerateTask,
};

/// An assistant running on a Ollama (https://github.com/jmorganca/ollama/) server
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
/// An assistant is listed for each Ollama model that has previously been pulled.
pub struct OllamaAssistant {
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

impl OllamaAssistant {
    /// Create a Ollama-based assistant
    pub fn new(model: String, context_length: usize) -> Self {
        Self::new_with(model, context_length, None, None)
    }

    /// Create a Ollama-based assistant with options for address of server
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
impl Assistant for OllamaAssistant {
    fn id(&self) -> String {
        format!("ollama/{}", self.model)
    }

    fn publisher(&self) -> String {
        "Ollama".to_string()
    }

    fn name(&self) -> String {
        let id = self.id();
        let name = id
            .rsplit_once('/')
            .map(|(.., name)| name.split_once(':').map_or(name, |(name, ..)| name))
            .unwrap_or(&id);
        name.to_title_case()
    }

    fn version(&self) -> String {
        let id = self.id();
        let version = id
            .split_once(':')
            .map(|(.., version)| version)
            .unwrap_or(&id);
        version.to_string()
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        let mut request =
            GenerationRequest::new(self.model.clone(), task.user_prompt().to_string());

        if let Some(system_prompt) = task.system_prompt() {
            request.system = Some(system_prompt.into());
        }

        // Map options to Ollama options
        let mut opts = GenerationOptions::default();
        macro_rules! map_option {
            ($from:ident, $to:ident) => {
                if let Some(value) = &options.$from {
                    opts = opts.$to(value.clone());
                }
            };
            ($name:ident) => {
                map_option!($name, $name)
            };
        }
        macro_rules! ignore_option {
            ($name:ident) => {
                if options.$name.is_some() {
                    tracing::warn!(
                        "Option `{}` is ignored by assistant `{}` for text-to-text generation",
                        stringify!($name),
                        self.id()
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
        map_option!(stop);
        if let Some(value) = options.max_tokens {
            opts = opts.num_predict(value as i32);
        }
        map_option!(tfs_z);
        map_option!(top_k);
        map_option!(top_p);
        ignore_option!(image_size);
        ignore_option!(image_quality);
        ignore_option!(image_style);

        request.options = Some(opts);

        let response = self
            .client
            .generate(request)
            .await
            .map_err(|error| eyre!(error))?;

        let text = response.response;

        GenerateOutput::from_text(self, task, text).await
    }
}

/// Get a list of all available Ollama assistants
///
/// Fetches the list of Ollama models from the server and maps them
/// into assistants.
///
/// Note that this uses a fixed assume context length for all models
/// (which will be probably be wrong for some). At the time of writing
/// there does not appear to be an easy way to get the actual context
/// length of an Ollama model (i.e. it is not in the API).
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    let models = Ollama::default().list_local_models().await?;

    let assistants = models
        .into_iter()
        .map(|model| Arc::new(OllamaAssistant::new(model.name, 4096)) as Arc<dyn Assistant>)
        .collect();

    Ok(assistants)
}
