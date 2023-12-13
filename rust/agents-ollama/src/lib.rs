use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{eyre, Result},
        tracing,
    },
    Agent, AgentIO, GenerateOptions,
};

/// An agent running on a Ollama (https://github.com/jmorganca/ollama/) server
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
/// An agent is listed for each Ollama model that has previously been pulled.
struct OllamaAgent {
    /// The Ollama name for a model including any tag e.g. "llama2:13b"
    ///
    /// Used as the required `model` parameter in each request to `POST /api/generate`
    /// (along with `prompt`).
    model: String,

    /// The Ollama API client
    client: Ollama,
}

impl OllamaAgent {
    /// Create a Ollama-based agent
    pub fn new(model: &str) -> Self {
        Self::new_with(model, None, None)
    }

    /// Create a Ollama-based agent with options for address of server
    pub fn new_with(model: &str, host: Option<String>, port: Option<u16>) -> Self {
        let host = host.unwrap_or("http://localhost".to_string());
        let port = port.unwrap_or(11434);
        let client = Ollama::new(host, port);

        Self {
            model: model.into(),
            client,
        }
    }
}

#[async_trait]
impl Agent for OllamaAgent {
    fn name(&self) -> String {
        format!("ollama-{}", self.model)
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    async fn text_to_text(
        &self,
        instruction: &str,
        options: Option<GenerateOptions>,
    ) -> Result<String> {
        let mut request = GenerationRequest::new(self.model.clone(), instruction.into());
        if let Some(options) = options {
            request.system = options.prompt_name;

            // Map options to Ollama options
            let mut opts = GenerationOptions::default();
            macro_rules! map_option {
                ($from:ident, $to:ident) => {
                    if let Some(value) = options.$from {
                        opts = opts.$to(value);
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
                            "Option `{}` is ignored by agent `{}` for text-to-text generation",
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
            map_option!(stop);
            ignore_option!(max_tokens);
            map_option!(tfs_z);
            map_option!(num_predict);
            map_option!(top_k);
            map_option!(top_p);
            ignore_option!(image_size);
            ignore_option!(image_quality);
            ignore_option!(image_style);

            request.options = Some(opts);
        }

        let response = self
            .client
            .generate(request)
            .await
            .map_err(|error| eyre!(error))?;

        Ok(response.response)
    }
}

/// Get a list of all available Ollama agents
///
/// Fetches the list of Ollama models from the server and maps them
/// into agents.
pub async fn list() -> Result<Vec<Box<dyn Agent>>> {
    let models = Ollama::default().list_local_models().await?;
    let agents = models
        .iter()
        .map(|model| Box::new(OllamaAgent::new(&model.name)) as Box<dyn Agent>)
        .collect();
    Ok(agents)
}
