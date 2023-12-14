use codecs::{EncodeOptions, Format};
use common::{
    async_trait::async_trait,
    chrono::Utc,
    clap::{self, Args, ValueEnum},
    eyre::{bail, Result},
    serde::Serialize,
    serde_with::skip_serializing_none,
    strum::Display,
};

// Export crates for the convenience of dependant crates
pub use common;
use schema::Node;

use crate::Prompt;

/// The type of input or output an agent can consume or generate
#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
pub enum AgentIO {
    Text,
    Image,
    Audio,
    Video,
}

/// Context for a generation request
///
/// An instance of this context is created for each generation (aka completion)
/// request to an AI model. It is then included in the rendering context for
/// the prompt.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct GenerateContext {
    /// The instruction provided by the user
    user_instruction: String,

    /// The document that the instruction is contained within
    /// (usually an `Article`).
    document: Option<Node>,

    /// The content of the document in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    document_content: Option<String>,

    /// The specific node in the document to which the instruction
    /// applies (if any)
    node: Option<Node>,

    /// The content of the specific node in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    node_content: Option<String>,

    // The following fields do NOT need to be provided by the
    // caller of the generation functions. They are populated automatically
    // before the prompt is rendered.
    /// The name of the agent
    agent_name: String,

    /// The name of the model provider
    provider_name: String,

    /// The name of the model
    model_name: String,

    /// The name of the prompt
    prompt_name: String,

    /// The current timestamp in ISO format
    current_timestamp: String,
}

impl GenerateContext {
    pub fn new(instruction: &str, document: Option<Node>, node: Option<Node>) -> Self {
        Self {
            user_instruction: instruction.to_string(),
            document,
            node,
            ..Default::default()
        }
    }
}

/// Options for various agent generation methods
///
/// These options are across the various APIs used by various implementations of
/// the `Agent` trait. As such, each option is not necessarily supported by each
/// implementation, and implementations may differ in their application of, and
/// defaults for each option.
///
/// Ollama: https://github.com/jmorganca/ollama/blob/main/docs/modelfile.md#valid-parameters-and-values
/// OpenAI: https://platform.openai.com/docs/api-reference/parameter-details
///
/// Currently, the names and descriptions are based mainly on those documented for `ollama`
/// with some additions for OpenAI.
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Args, Serialize)]
#[serde(crate = "common::serde")]
pub struct GenerateOptions {
    /// The name of the prompt to use
    ///
    /// Each agent has a default prompt template but this option can be used to
    /// override that.
    #[arg(long = "prompt", short)]
    pub prompt_name: Option<String>,

    /// The format to convert the document content into when rendered into the prompt.
    #[arg(long)]
    pub document_format: Option<Format>,

    /// The format to convert the node content into when rendered into the prompt.
    #[arg(long)]
    pub node_format: Option<Format>,

    /// Enable Mirostat sampling for controlling perplexity.
    ///
    /// Supported by: Ollama                                                                                                                              
    #[arg(long)]
    pub mirostat: Option<u8>,

    /// Influences how quickly the algorithm responds to feedback from the generated text.
    ///
    /// A lower learning rate will result in slower adjustments, while a higher learning
    /// rate will make the algorithm more responsive.
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub mirostat_eta: Option<f32>,

    /// Controls the balance between coherence and diversity of the output.
    ///
    /// A lower value will result in more focused and coherent text.                                                                                                     
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub mirostat_tau: Option<f32>,

    /// Sets the size of the context window used to generate the next token.                                                                                                                                                             
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub num_ctx: Option<u32>,

    /// The number of GQA groups in the transformer layer.
    ///
    /// Required for some models, for example it is 8 for llama2:70b                                                                                                                                       
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub num_gqa: Option<u32>,

    /// The number of layers to send to the GPU(s).
    ///
    /// On macOS it defaults to 1 to enable metal support, 0 to disable.                                                                                                                                          
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub num_gpu: Option<u32>,

    /// Sets the number of threads to use during computation.
    ///
    /// By default, Ollama will detect this for optimal performance. It is recommended to set this
    /// value to the number of physical CPU cores your system has (as opposed to the logical
    /// number of cores).    
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub num_thread: Option<u32>,

    /// Sets how far back for the model to look back to prevent repetition.
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub repeat_last_n: Option<i32>,

    /// Sets how strongly to penalize repetitions.
    ///
    /// A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.                                                              
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    #[arg(long)]
    pub repeat_penalty: Option<f32>,

    /// The temperature of the model.
    ///
    /// Increasing the temperature will make the model answer more creatively.
    ///
    /// Supported by: Ollama, OpenAI Chat                   
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Sets the random number seed to use for generation.
    ///
    /// Setting this to a specific number will make the model generate the same text for the same prompt.                                                                            
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    #[arg(long)]
    pub seed: Option<i32>,

    /// Sets the stop sequences to use.
    ///
    /// When this pattern is encountered the LLM will stop generating text and return.                                   
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    #[arg(long)]
    pub stop: Option<String>,

    /// The maximum number of tokens to generate.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    ///
    /// Supported by: OpenAI Chat
    #[arg(long)]
    pub max_tokens: Option<u16>,

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output.
    ///
    /// A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.                                         
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub tfs_z: Option<f32>,

    /// Maximum number of tokens to predict when generating text.                                                                                                                                    
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub num_predict: Option<i32>,

    /// Reduces the probability of generating nonsense.
    ///
    /// A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.                                                                    
    ///
    /// Supported by: Ollama                    
    #[arg(long)]
    pub top_k: Option<u32>,

    /// Works together with top-k.
    ///
    /// A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more
    /// focused and conservative text.                                                            
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    #[arg(long)]
    pub top_p: Option<f32>,

    /// The size of the generated images.
    ///
    /// Supported by `openai/dall-e-3` and `openai/dall-e-2`.
    /// Must be one of `256x256`, `512x512`, or `1024x1024` for `dall-e-2`.
    /// Must be one of `1024x1024`, `1792x1024`, or `1024x1792` for `dall-e-3` models.
    #[arg(skip)]
    pub image_size: Option<(u16, u16)>,

    /// The quality of the image that will be generated.
    ///
    /// Supported for `openai/dall-e-3`.
    #[arg(long)]
    pub image_quality: Option<String>,

    /// The style of the generated images. Must be one of `vivid` or `natural`.
    /// Vivid causes the model to lean towards generating hyper-real and dramatic images.
    /// Natural causes the model to produce more natural, less hyper-real looking images.
    /// Supported by `openai/dall-e-3`.
    #[arg(long)]
    pub image_style: Option<String>,
}

/// Macro to return an error for a method that is not supported by an agent
macro_rules! unsupported {
    ($self:expr, $what:literal) => {
        bail!("{} is not supported by agent `{}`", $what, $self.name())
    };
}

/// An AI agent
///
/// Provides a common, shared interface for the various AI models
/// and APIs used. Agent implementations should override
/// `supports_generating` and other methods.
#[async_trait]
pub trait Agent: Sync + Send {
    /**
     * Get the name of the agent
     *
     * This name should be unique amongst agents. This default
     * implementation combines the provider and model name but this
     * should be overridden if necessary for uniqueness.
     */
    fn name(&self) -> String {
        format!("{}/{}", self.provider(), self.model())
    }

    /**
     * Get the name of the model provider that the agent uses
     */
    fn provider(&self) -> String;

    /**
     * Get the name of the model that the agent uses
     */
    fn model(&self) -> String;

    /**
     * Get the name of the default prompt for this agent
     *
     * This prompt will be used if none is provided by the
     * user.
     */
    fn prompt(&self) -> String {
        "default".to_string()
    }

    /**
     * Get a list of input types this agent supports
     */
    fn supported_inputs(&self) -> &[AgentIO] {
        &[]
    }

    /**
     * Get a list of output types this agent supports
     */
    fn supported_outputs(&self) -> &[AgentIO] {
        &[]
    }

    /**
     * Get a list of output types this agent supports
     */
    fn supports_from_to(&self, input: AgentIO, output: AgentIO) -> bool {
        self.supported_inputs().contains(&input) && self.supported_outputs().contains(&output)
    }

    /**
     * Render a prompt with some context
     *
     * If no prompt name is provided, the default prompt for the agent is used.
     *
     * Additional context variables are added.
     *
     * Returns a tuple of the system and user prompt.
     */
    async fn render_prompt(
        &self,
        mut context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<(String, String)> {
        if let Some(document) = &context.document {
            context.document_content = Some(
                codecs::to_string(
                    document,
                    Some(EncodeOptions {
                        format: options.document_format.or(Some(Format::Html)),
                        ..Default::default()
                    }),
                )
                .await?,
            )
        };

        if let Some(node) = &context.node {
            context.node_content = Some(
                codecs::to_string(
                    node,
                    Some(EncodeOptions {
                        format: options.node_format.or(Some(Format::Html)),
                        ..Default::default()
                    }),
                )
                .await?,
            )
        };

        let prompt_name = options.prompt_name.clone().unwrap_or_else(|| self.prompt());

        context.agent_name = self.name();
        context.provider_name = self.provider();
        context.model_name = self.model();
        context.prompt_name = prompt_name.clone();
        context.current_timestamp = Utc::now().to_rfc3339();

        let prompt = Prompt::load(&prompt_name)?;
        prompt.render_with(context)
    }

    /**
     * Generate text in response to an instruction
     */
    #[allow(unused)]
    async fn text_to_text(
        &self,
        context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<String> {
        unsupported!(self, "Text to text")
    }

    /**
     * Generate text to complete a chat conversation
     *
     * Takes a sequence of messages, alternating between a
     * user prompts and agent responses, and returns a new text message
     * in the chat sequence.
     *
     * If the agent's model does not explicitly support a chat mode
     * (i.e. if this method is not overridden) this default implementation
     * simply concatenates the chat messages into blank line separated
     * text and passes it to the `text_to_text` method.
     */
    #[allow(unused)]
    async fn chat_to_text(
        &self,
        context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<String> {
        unsupported!(self, "Chat to text")
    }

    /**
     * Generate image in response to an instruction
     *
     * TODO: This should return a Vec<u8> (the bytes of the image)
     * as well as the format of the image(?).
     */
    #[allow(unused)]
    async fn text_to_image(&self, instruction: &str, options: &GenerateOptions) -> Result<String> {
        unsupported!(self, "Text to image")
    }
}
