use common::{
    async_trait::async_trait,
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    itertools::Itertools,
    strum::Display,
};

// Export crates for the convenience of dependant crates
pub use common;

/// The type of input or output an agent can consume or generate
#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
pub enum AgentIO {
    Text,
    Image,
    Audio,
    Video,
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
#[derive(Default)]
pub struct GenerateOptions {
    /// The system prompt to use
    pub system_prompt: Option<String>,

    /// Enable Mirostat sampling for controlling perplexity.
    ///
    /// Supported by: Ollama                                                                                                                              
    pub mirostat: Option<u8>,

    /// Influences how quickly the algorithm responds to feedback from the generated text.
    ///
    /// A lower learning rate will result in slower adjustments, while a higher learning
    /// rate will make the algorithm more responsive.
    ///
    /// Supported by: Ollama                    
    pub mirostat_eta: Option<f32>,

    /// Controls the balance between coherence and diversity of the output.
    ///
    /// A lower value will result in more focused and coherent text.                                                                                                     
    ///
    /// Supported by: Ollama                    
    pub mirostat_tau: Option<f32>,

    /// Sets the size of the context window used to generate the next token.                                                                                                                                                             
    ///
    /// Supported by: Ollama                    
    pub num_ctx: Option<u32>,

    /// The number of GQA groups in the transformer layer.
    ///
    /// Required for some models, for example it is 8 for llama2:70b                                                                                                                                       
    ///
    /// Supported by: Ollama                    
    pub num_gqa: Option<u32>,

    /// The number of layers to send to the GPU(s).
    ///
    /// On macOS it defaults to 1 to enable metal support, 0 to disable.                                                                                                                                          
    ///
    /// Supported by: Ollama                    
    pub num_gpu: Option<u32>,

    /// Sets the number of threads to use during computation.
    ///
    /// By default, Ollama will detect this for optimal performance. It is recommended to set this
    /// value to the number of physical CPU cores your system has (as opposed to the logical
    /// number of cores).    
    ///
    /// Supported by: Ollama                    
    pub num_thread: Option<u32>,

    /// Sets how far back for the model to look back to prevent repetition.
    ///
    /// Supported by: Ollama                    
    pub repeat_last_n: Option<i32>,

    /// Sets how strongly to penalize repetitions.
    ///
    /// A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.                                                              
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    pub repeat_penalty: Option<f32>,

    /// The temperature of the model.
    ///
    /// Increasing the temperature will make the model answer more creatively.
    ///
    /// Supported by: Ollama, OpenAI Chat                   
    pub temperature: Option<f32>,

    /// Sets the random number seed to use for generation.
    ///
    /// Setting this to a specific number will make the model generate the same text for the same prompt.                                                                            
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    pub seed: Option<i32>,

    /// Sets the stop sequences to use.
    ///
    /// When this pattern is encountered the LLM will stop generating text and return.                                   
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    pub stop: Option<String>,

    /// The maximum number of tokens to generate.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    ///
    /// Supported by: OpenAI Chat
    pub max_tokens: Option<u16>,

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output.
    ///
    /// A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.                                         
    ///
    /// Supported by: Ollama                    
    pub tfs_z: Option<f32>,

    /// Maximum number of tokens to predict when generating text.                                                                                                                                    
    ///
    /// Supported by: Ollama                    
    pub num_predict: Option<i32>,

    /// Reduces the probability of generating nonsense.
    ///
    /// A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.                                                                    
    ///
    /// Supported by: Ollama                    
    pub top_k: Option<u32>,

    /// Works together with top-k.
    ///
    /// A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more
    /// focused and conservative text.                                                            
    ///
    /// Supported by: Ollama, OpenAI Chat                    
    pub top_p: Option<f32>,

    /// The size of the generated images.
    ///
    /// Supported by `openai/dall-e-3` and `openai/dall-e-2`.
    /// Must be one of `256x256`, `512x512`, or `1024x1024` for `dall-e-2`.
    /// Must be one of `1024x1024`, `1792x1024`, or `1024x1792` for `dall-e-3` models.
    pub image_size: Option<(u16, u16)>,

    /// The quality of the image that will be generated.
    ///
    /// Supported for `openai/dall-e-3`.
    pub image_quality: Option<String>,

    /// The style of the generated images. Must be one of `vivid` or `natural`.
    /// Vivid causes the model to lean towards generating hyper-real and dramatic images.
    /// Natural causes the model to produce more natural, less hyper-real looking images.
    /// Supported by `openai/dall-e-3`.
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
     * Get the fully qualified name of the agent
     */
    fn name(&self) -> String;

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
     * Generate text in response to an instruction
     */
    #[allow(unused)]
    async fn text_to_text(
        &self,
        instruction: &str,
        options: Option<GenerateOptions>,
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
        chat: &[&str],
        options: Option<GenerateOptions>,
    ) -> Result<String> {
        let instruction = chat.iter().map(|message| message.trim()).join("\n\n");
        self.text_to_text(&instruction, options).await
    }

    /**
     * Generate image in response to an instruction
     *
     * TODO: This should return a Vec<u8> (the bytes of the image)
     * as well as the format of the image(?).
     */
    #[allow(unused)]
    async fn text_to_image(
        &self,
        instruction: &str,
        options: Option<GenerateOptions>,
    ) -> Result<String> {
        unsupported!(self, "Text to image")
    }
}
