use common::{
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    smart_default::SmartDefault,
    strum::Display,
};
use format::Format;
use schema::{InstructionMessage, InstructionModel, InstructionType};

/// The kind of generative model task
#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum ModelTaskKind {
    /// Given a list of input messages, generate the next message in a conversation
    ///
    /// Example APIs include:
    ///
    /// - OpenAI Chat Completion: https://platform.openai.com/docs/api-reference/chat
    /// - Anthropic Messages: https://docs.anthropic.com/en/api/messages
    #[default]
    MessageGeneration,

    /// Given an input message (text and/or image), generate an image
    ///
    /// Example APIs include:
    ///
    /// - OpenAI Images: https://platform.openai.com/docs/api-reference/images
    /// - Anthropic Messages: https://docs.anthropic.com/en/api/messages
    ImageGeneration,
}

/// A task to generate content
///
/// A task is created for each generation request to an AI model.
/// It is then included in the rendering context for the prompt.
///
/// Only properties not required within rendered templates should
/// have `#[serde(skip)]`.
///
/// Options for various assistant generation methods
///
/// These options are across the various APIs used by various implementations of
/// the `Assistant` trait. As such, each option is not necessarily supported by each
/// implementation, and implementations may differ in their application of, and
/// defaults for each option.
///
/// For details, see:
///
/// Google: https://ai.google.dev/api/rest/v1beta/GenerationConfig
/// Ollama: https://github.com/jmorganca/ollama/blob/main/docs/modelfile.md#valid-parameters-and-values
/// OpenAI: https://platform.openai.com/docs/api-reference/parameter-details
///
/// Currently, the names and descriptions are based mainly on those documented for `ollama`
/// with some additions for OpenAI.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    deny_unknown_fields,
    crate = "common::serde"
)]
pub struct ModelTask {
    /// The type of instruction this task is for
    pub instruction_type: Option<InstructionType>,

    /// The model selection and execution options set by the user
    pub instruction_model: Option<InstructionModel>,

    /// The list of input messages
    pub messages: Vec<InstructionMessage>,

    /// The kind of model task
    pub kind: ModelTaskKind,

    /// The desired format of the generated content
    pub format: Format,

    /// Enable Mirostat sampling for controlling perplexity.
    ///
    /// Supported by Ollama.
    pub mirostat: Option<u8>,

    /// Influences how quickly the algorithm responds to feedback from the generated text.
    ///
    /// A lower learning rate will result in slower adjustments, while a higher learning
    /// rate will make the algorithm more responsive.
    ///
    /// Supported by Ollama.
    pub mirostat_eta: Option<f32>,

    /// Controls the balance between coherence and diversity of the output.
    ///
    /// A lower value will result in more focused and coherent text.
    ///
    /// Supported by Ollama.
    pub mirostat_tau: Option<f32>,

    /// Sets the size of the context window used to generate the next token.
    ///
    /// Supported by Ollama.
    pub num_ctx: Option<u32>,

    /// The number of GQA groups in the transformer layer.
    ///
    /// Required for some models, for example it is 8 for llama2:70b
    ///
    /// Supported by Ollama.
    pub num_gqa: Option<u32>,

    /// The number of layers to send to the GPU(s).
    ///
    /// On macOS it defaults to 1 to enable metal support, 0 to disable.
    ///
    /// Supported by Ollama.
    pub num_gpu: Option<u32>,

    /// Sets the number of threads to use during computation.
    ///
    /// By default, Ollama will detect this for optimal performance. It is recommended to set this
    /// value to the number of physical CPU cores your system has (as opposed to the logical
    /// number of cores).
    ///
    /// Supported by Ollama.
    pub num_thread: Option<u32>,

    /// Sets how far back for the model to look back to prevent repetition.
    ///
    /// Supported by Ollama.
    pub repeat_last_n: Option<i32>,

    /// Sets how strongly to penalize repetitions.
    ///
    /// A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.
    ///
    /// Supported by Ollama, OpenAI Chat.
    pub repeat_penalty: Option<f32>,

    /// The temperature of the model.
    ///
    /// Increasing the temperature will make the model answer more creatively.
    pub temperature: Option<f32>,

    /// Sets the random number seed to use for generation.
    ///
    /// Setting this to a specific number will make the model generate the same text for the same prompt.
    pub seed: Option<i32>,

    /// Sets the stop sequences to use.
    ///
    /// When this pattern is encountered the LLM will stop generating text and return.
    pub stop: Option<String>,

    /// The maximum number of tokens to generate.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    pub max_tokens: Option<u16>,

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output.
    ///
    /// A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.
    ///
    /// Supported by Ollama.
    pub tfs_z: Option<f32>,

    /// Reduces the probability of generating nonsense.
    ///
    /// A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.
    pub top_k: Option<u32>,

    /// Works together with top-k.
    ///
    /// A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more
    /// focused and conservative text.
    pub top_p: Option<f32>,

    /// The size of the generated images.
    ///
    /// Supported by `openai/dall-e-3` and `openai/dall-e-2`.
    /// Must be one of `256x256`, `512x512`, or `1024x1024` for `dall-e-2`.
    /// Must be one of `1024x1024`, `1792x1024`, or `1024x1792` for `dall-e-3` models.
    pub image_size: Option<(u16, u16)>,

    /// The quality of the image that will be generated.
    ///
    /// Supported by `openai/dall-e-3`.
    pub image_quality: Option<String>,

    /// The style of the generated images. Must be one of `vivid` or `natural`.
    ///
    /// Vivid causes the model to lean towards generating hyper-real and dramatic images.
    /// Natural causes the model to produce more natural, less hyper-real looking images.
    ///
    /// Supported by `openai/dall-e-3`.
    pub image_style: Option<String>,

    /// Prepare the task but do not actually generate content
    ///
    /// Model implementations should respect this option by returning an empty `ModelOutput`
    /// at the last possible moment before generation (usually just before an API request is made).
    #[serde(default)]
    pub dry_run: bool,
}

impl ModelTask {
    pub fn new(
        instruction_type: InstructionType,
        instruction_model: Option<InstructionModel>,
        messages: Vec<InstructionMessage>,
    ) -> Self {
        // Extract and transform any model execution options
        let temperature = instruction_model
            .as_ref()
            .and_then(|model| model.temperature)
            .map(|temp| (temp as f32 / 100.).min(1.));

        Self {
            instruction_type: Some(instruction_type),
            instruction_model,
            messages,
            temperature,
            ..Default::default()
        }
    }
}
