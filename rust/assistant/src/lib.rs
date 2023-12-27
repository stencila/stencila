use std::path::PathBuf;

use fastembed::{EmbeddingBase, EmbeddingModel, FlagEmbedding, InitOptions};
use merge::Merge;

use common::{
    async_trait::async_trait,
    clap::{self, Args, ValueEnum},
    eyre::{bail, eyre, Result},
    once_cell::sync::OnceCell,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    smart_default::SmartDefault,
    strum::Display,
};
use format::Format;
use schema::{
    transforms::{blocks_to_nodes, inlines_to_nodes},
    InstructionBlock, InstructionInline, Node,
};

// Export crates for the convenience of dependant crates
pub use common;
pub use format;
pub use merge;
pub use schema;

/// The type of input or output an assistant can consume or generate
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum, Display, Serialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum AssistantIO {
    #[default]
    Text,
    Image,
    Audio,
    Video,
    Node,
}

/// An instruction created within a document
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Instruction {
    /// The user created an `InstructionBlock` node
    Block(InstructionBlock),

    /// The user created an `InstructionInline` node
    Inline(InstructionInline),
}

impl Default for Instruction {
    fn default() -> Self {
        Self::Block(InstructionBlock::default())
    }
}

impl From<InstructionBlock> for Instruction {
    fn from(instruct: InstructionBlock) -> Self {
        Self::Block(instruct)
    }
}

impl From<InstructionInline> for Instruction {
    fn from(instruct: InstructionInline) -> Self {
        Self::Inline(instruct)
    }
}

impl Instruction {
    /// Get the text of the instruction
    pub fn text(&self) -> &str {
        match self {
            Instruction::Block(block) => block.text.as_str(),
            Instruction::Inline(inline) => inline.text.as_str(),
        }
    }

    /// Get the content of the instruction (if any)
    pub fn content(&self) -> Option<Vec<Node>> {
        match self {
            Instruction::Block(InstructionBlock {
                content: Some(content),
                ..
            }) => Some(blocks_to_nodes(content.clone())),

            Instruction::Inline(InstructionInline {
                content: Some(content),
                ..
            }) => Some(inlines_to_nodes(content.clone())),

            _ => None,
        }
    }
}

/// A enumeration of the type of instructions
///
/// Used to delegate to different types of assistants
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum InstructionType {
    InsertBlocks,
    InsertInlines,
    ModifyBlocks,
    ModifyInlines,
}

impl From<&Instruction> for InstructionType {
    fn from(instruct: &Instruction) -> Self {
        use InstructionType::*;
        match instruct {
            Instruction::Block(InstructionBlock { content: None, .. }) => InsertBlocks,

            Instruction::Block(InstructionBlock {
                content: Some(..), ..
            }) => ModifyBlocks,

            Instruction::Inline(InstructionInline { content: None, .. }) => InsertInlines,

            Instruction::Inline(InstructionInline {
                content: Some(..), ..
            }) => ModifyInlines,
        }
    }
}

/// A task to generate content
///
/// A task is created for each generation (aka completion)
/// request to an AI model. It is then included in the rendering context for
/// the prompt.
///
/// The ordering of fields follows the order that they are usually included
/// in the request to the AI model: the system prompt, the document context,
/// the user instruction (including any specific content it has), and
/// finally the rendered user_prompt.
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct GenerateTask {
    /// An optional system prompt
    pub system_prompt: Option<String>,

    /// The document that the instruction is contained within
    /// (usually an `Article`).
    pub document: Option<Node>,

    /// The content of the document in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    pub document_formatted: Option<String>,

    /// The instruction provided by the user
    pub instruction: Instruction,

    /// The instruction text provided for convenient access in the
    /// user prompt template
    instruction_text: Option<String>,

    /// The instruction embedding
    instruction_embedding: Option<Vec<f32>>,

    /// The content of the instruction in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    pub content_formatted: Option<String>,

    /// The user prompt usually generated from the `instruction_text` and
    /// other fields by rendering a user prompt template
    pub user_prompt: Option<String>,

    /// The input type of the task
    pub input: AssistantIO,

    /// The output type of the task
    pub output: AssistantIO,
}

impl GenerateTask {
    /// Create a [`GenerateContext`] with an instruction & document
    pub fn new(instruction: Instruction, document: Option<Node>) -> Self {
        Self {
            instruction,
            document,
            ..Default::default()
        }
    }

    /// Get the text of the instruction
    ///
    /// Will populate `self.instruction_text` if necessary for use
    /// in user prompt templates
    pub fn instruction_text(&mut self) -> &str {
        self.instruction_text
            .get_or_insert_with(|| self.instruction.text().to_string())
    }

    /// Get the similarity between the text of the instruction and some other, precalculated embedding
    ///
    /// Will populate `self.instruction_embedding` is necessary, so that this only needs to
    /// be done once for each call to this function (e.g. when calculating similarity with
    /// a number of other embeddings).
    pub fn instruction_similarity(&mut self, other: &[f32]) -> Result<f32> {
        let embedding = match &self.instruction_embedding {
            Some(embedding) => embedding,
            None => {
                let text = self.instruction_text();
                let mut embedding = Self::create_embeddings(vec![text])?;
                self.instruction_embedding = Some(embedding.swap_remove(0));
                self.instruction_embedding.as_ref().unwrap()
            }
        };

        Ok(Self::calculate_similarity(&embedding, other))
    }

    /// Get the user prompt of the context
    ///
    /// If the `user_prompt` field has been set (e.g. a template rendered into it)
    /// then that is returned. Otherwise, the raw text of the `instruction` is
    /// returned.
    pub fn user_prompt(&self) -> &str {
        self.user_prompt
            .as_deref()
            .unwrap_or(self.instruction.text())
    }

    /// Get the system prompt of the context (if any)
    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    /// Create embeddings for a list of instructions texts
    pub fn create_embeddings<S>(texts: Vec<S>) -> Result<Vec<Vec<f32>>>
    where
        S: AsRef<str> + Send + Sync,
    {
        // Informal perf tests during development indicated that using
        // a static improved speed substantially (rather than reloading for each call)
        static MODEL: OnceCell<FlagEmbedding> = OnceCell::new();

        let model = match MODEL.get_or_try_init(|| {
            FlagEmbedding::try_new(InitOptions {
                // This model was chosen good performance for a small size.
                // For benchmarks see https://huggingface.co/spaces/mteb/leaderboard
                model_name: EmbeddingModel::BGESmallENV15,
                cache_dir: app::cache_dir(true)
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("models")
                    .join("bge-small-en-v1.5"),
                ..Default::default()
            })
        }) {
            Ok(model) => model,
            Err(error) => bail!(error),
        };

        model.embed(texts, None).map_err(|error| eyre!(error))
    }

    /// Calculate the cosine similarity of two embeddings
    fn calculate_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|&x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|&y| y * y).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }
}

/// Options for various assistant generation methods
///
/// These options are across the various APIs used by various implementations of
/// the `Assistant` trait. As such, each option is not necessarily supported by each
/// implementation, and implementations may differ in their application of, and
/// defaults for each option.
///
/// Ollama: https://github.com/jmorganca/ollama/blob/main/docs/modelfile.md#valid-parameters-and-values
/// OpenAI: https://platform.openai.com/docs/api-reference/parameter-details
///
/// Currently, the names and descriptions are based mainly on those documented for `ollama`
/// with some additions for OpenAI.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, Merge, Args, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    deny_unknown_fields,
    crate = "common::serde"
)]
pub struct GenerateOptions {
    /// The name of the assistant to use
    ///
    /// Specify this option when you want to use a specific assistant and skip
    /// the assistant delegation algorithm.
    #[arg(long)]
    pub assistant: Option<String>,

    /// The format to convert the document content into when rendered into the prompt.
    #[arg(long, default_value = "html")]
    #[default(Some(Format::Html))]
    pub document_format: Option<Format>,

    /// The format to convert the instruction content (if any) into when rendered into the prompt.
    #[arg(long)]
    #[arg(long, default_value = "html")]
    #[default(Some(Format::Html))]
    pub content_format: Option<Format>,

    /// The format of the generated content
    ///
    /// Used to decode (i.e. parse) the generated content into an array of
    /// Stencila Schema nodes.
    #[arg(long)]
    #[arg(long, default_value = "html")]
    #[default(Some(Format::Html))]
    pub generated_format: Option<Format>,

    /// A pattern to coerce the generated document nodes into
    ///
    /// TODO: Not yet implemented!
    #[arg(long)]
    pub coerce_nodes: Option<String>,

    /// A pattern for the type and number of nodes that should be generated
    ///
    /// TODO: Not yet implemented!
    #[arg(long)]
    pub assert_nodes: Option<String>,

    /// The maximum number of retries for generating valid nodes
    ///
    /// TODO: Not yet implemented!
    #[arg(long, default_value = "1")]
    #[default(Some(1))]
    pub max_retries: Option<u32>,

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

/// Output generated for a task
pub enum GenerateOutput {
    Text(String),
    // TODO: Use bytes of an image here
    Image(String),
}

impl GenerateOutput {
    pub fn into_string(self) -> Option<String> {
        use GenerateOutput::*;
        match self {
            Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        use GenerateOutput::*;
        match self {
            Text(text) => text.as_bytes(),
            Image(image) => image.as_bytes(),
        }
    }
}

/// Details returned along with whatever is generated
#[derive(Debug, Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct GenerateDetails {
    /// The chain of assistants used in the generation
    pub assistants: Vec<String>,

    /// The model (a.k.a system) fingerprint at the time of generation
    /// for the last assistant in the chain
    pub fingerprint: Option<String>,

    /// The options used for the generation
    pub options: GenerateOptions,

    /// The task for the generation
    ///
    /// This may differ from the original task e.g. optional fields
    /// populated by rendering prompt templates
    pub task: GenerateTask,
}

/// An AI assistant
///
/// Provides a common, shared interface for the various AI models
/// and APIs used. Assistant implementations should override
/// `supports_generating` and other methods.
#[async_trait]
pub trait Assistant: Sync + Send {
    /**
     * Get the name of the assistant
     *
     * This name should be unique amongst assistants. This default
     * implementation combines the provider and model name but this
     * should be overridden if necessary for uniqueness.
     */
    fn name(&self) -> String {
        format!("{}/{}", self.provider(), self.model())
    }

    /**
     * Get the name of the model provider that the assistant uses
     */
    fn provider(&self) -> String;

    /**
     * Get the name of the model that the assistant uses
     */
    fn model(&self) -> String;

    /**
     * Does the assistant support a specific task
     *
     * This default implementation is based solely on whether the assistants
     * supports the input/output combination of the task. Overrides may
     * add other criteria such as the type of the task's instruction.
     */
    #[allow(unused)]
    fn supports_task(&self, task: &GenerateTask) -> bool {
        self.supports_from_to(task.input, task.output)
    }

    /**
     * Get a list of input types this assistant supports
     */
    fn supported_inputs(&self) -> &[AssistantIO] {
        &[]
    }

    /**
     * Get a list of output types this assistant supports
     */
    fn supported_outputs(&self) -> &[AssistantIO] {
        &[]
    }

    /**
     * Whether this assistant support a specific input/output combination
     */
    fn supports_from_to(&self, input: AssistantIO, output: AssistantIO) -> bool {
        self.supported_inputs().contains(&input) && self.supported_outputs().contains(&output)
    }

    /**
     * A score of the suitability of this assistant for a performing a task
     *
     * A score between 0 and 1. A task will be delegated to the assistant
     * with the highest suitability score for that task.
     * Tied scores are broken using [`Self::preference_rank`].
     *
     * This default implementation returns 0.1 if the assistant supports the
     * task, 0.0 otherwise.
     */
    fn suitability_score(&self, task: &mut GenerateTask) -> Result<f32> {
        Ok(if self.supports_task(task) { 0.1 } else { 0.0 })
    }

    /**
     * The relative rank of preference to delegate tasks to this assistant
     *
     * Used to break ties when to assistants have the same suitability score
     * for a task.
     */
    fn preference_rank(&self) -> u8 {
        0
    }

    /**
     * Perform a task
     *
     * Will call the appropriate method of the assistant, erroring if it does not
     * support the input/output combination of the task (this will normally
     * have been checked prior to calling this method)
     */
    async fn perform_task(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)>;
}
