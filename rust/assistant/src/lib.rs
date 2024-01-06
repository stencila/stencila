use std::path::PathBuf;

use codecs::DecodeOptions;
use fastembed::{EmbeddingBase, EmbeddingModel, FlagEmbedding, InitOptions};
use merge::Merge;

use common::{
    async_trait::async_trait,
    clap::{self, Args, ValueEnum},
    eyre::{bail, eyre, Result},
    inflector::Inflector,
    once_cell::sync::OnceCell,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    smart_default::SmartDefault,
    strum::Display,
};
use format::Format;
use node_authorship::author_roles;
use schema::{
    transforms::{blocks_to_inlines, blocks_to_nodes, inlines_to_blocks, inlines_to_nodes},
    Article, AudioObject, AuthorRole, AuthorRoleName, Block, ImageObject, Inline, InstructionBlock,
    InstructionInline, Link, Node, Organization, OrganizationOptions, PersonOrOrganization,
    PersonOrOrganizationOrSoftwareApplication, SoftwareApplication, SoftwareApplicationOptions,
    StringOrNumber, VideoObject,
};

// Export crates for the convenience of dependant crates
pub use codecs;
pub use common;
pub use format;
pub use merge;
pub use node_authorship;
pub use schema;

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
    /// Get the assignee of the instruction (if any)
    pub fn assignee(&self) -> Option<&str> {
        match self {
            Instruction::Block(block) => block.assignee.as_deref(),
            Instruction::Inline(inline) => inline.assignee.as_deref(),
        }
    }

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
/// A task is created for each generation request to an AI model.
/// It is then included in the rendering context for the prompt.
///
/// Only properties not required within rendered templates should
/// have `#[serde(skip)]`.
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct GenerateTask {
    /// The instruction provided by the user
    pub instruction: Instruction,

    /// The input type of the task
    pub input: AssistantIO,

    /// The output type of the task
    pub output: AssistantIO,

    /// The desired output format of the task
    pub format: Format,

    /// The context length of assistant performing the task
    ///
    /// Provided here so that user prompt templates can change rendering
    /// to take into account the specific context length of the specific base
    /// assistant being used for the task.
    pub context_length: Option<usize>,

    /// The document that the instruction is contained within
    /// (usually an `Article`).
    #[serde(skip)]
    pub document: Option<Node>,

    /// The content of the document in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    pub document_formatted: Option<String>,

    /// The instruction text provided for convenient access in the
    /// user prompt template
    pub instruction_text: Option<String>,

    /// The instruction embedding
    #[serde(skip)]
    pub instruction_embedding: Option<Vec<f32>>,

    /// The content of the instruction in the format specified
    /// in the `GenerateOptions` (defaulting to HTML)
    pub content_formatted: Option<String>,

    /// An optional system prompt
    pub system_prompt: Option<String>,

    /// The user prompt usually generated from the `instruction_text` and
    /// other fields by rendering a user prompt template
    pub user_prompt: Option<String>,
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

        Ok(Self::calculate_similarity(embedding, other))
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
/// For details, see:
///
/// Google: https://ai.google.dev/api/rest/v1beta/GenerationConfig
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

    /// Enable Mirostat sampling for controlling perplexity.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub mirostat: Option<u8>,

    /// Influences how quickly the algorithm responds to feedback from the generated text.
    ///
    /// A lower learning rate will result in slower adjustments, while a higher learning
    /// rate will make the algorithm more responsive.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub mirostat_eta: Option<f32>,

    /// Controls the balance between coherence and diversity of the output.
    ///
    /// A lower value will result in more focused and coherent text.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub mirostat_tau: Option<f32>,

    /// Sets the size of the context window used to generate the next token.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub num_ctx: Option<u32>,

    /// The number of GQA groups in the transformer layer.
    ///
    /// Required for some models, for example it is 8 for llama2:70b
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub num_gqa: Option<u32>,

    /// The number of layers to send to the GPU(s).
    ///
    /// On macOS it defaults to 1 to enable metal support, 0 to disable.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub num_gpu: Option<u32>,

    /// Sets the number of threads to use during computation.
    ///
    /// By default, Ollama will detect this for optimal performance. It is recommended to set this
    /// value to the number of physical CPU cores your system has (as opposed to the logical
    /// number of cores).
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub num_thread: Option<u32>,

    /// Sets how far back for the model to look back to prevent repetition.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub repeat_last_n: Option<i32>,

    /// Sets how strongly to penalize repetitions.
    ///
    /// A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.
    ///
    /// Supported by Ollama, OpenAI Chat.
    #[arg(long)]
    pub repeat_penalty: Option<f32>,

    /// The temperature of the model.
    ///
    /// Increasing the temperature will make the model answer more creatively.
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Sets the random number seed to use for generation.
    ///
    /// Setting this to a specific number will make the model generate the same text for the same prompt.
    #[arg(long)]
    pub seed: Option<i32>,

    /// Sets the stop sequences to use.
    ///
    /// When this pattern is encountered the LLM will stop generating text and return.
    #[arg(long)]
    pub stop: Option<String>,

    /// The maximum number of tokens to generate.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    #[arg(long)]
    pub max_tokens: Option<u16>,

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output.
    ///
    /// A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.
    ///
    /// Supported by Ollama.
    #[arg(long)]
    pub tfs_z: Option<f32>,

    /// Reduces the probability of generating nonsense.
    ///
    /// A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.
    #[arg(long)]
    pub top_k: Option<u32>,

    /// Works together with top-k.
    ///
    /// A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more
    /// focused and conservative text.
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
    /// Supported by `openai/dall-e-3`.
    #[arg(long)]
    pub image_quality: Option<String>,

    /// The style of the generated images. Must be one of `vivid` or `natural`.
    ///
    /// Vivid causes the model to lean towards generating hyper-real and dramatic images.
    /// Natural causes the model to produce more natural, less hyper-real looking images.
    ///
    /// Supported by `openai/dall-e-3`.
    #[arg(long)]
    pub image_style: Option<String>,
}

/// Output generated for a task
///
/// Yes, this could have been named `GeneratedOutput`! But it wasn't to
/// maintain consistency with `GenerateTask` and `GenerateOptions`.
pub struct GenerateOutput {
    /// The content generated by the assistant
    pub content: GenerateContent,

    /// The format of the generated content
    pub format: Format,

    /// Stencila Schema nodes derived from the generated content
    pub nodes: Nodes,
}

impl GenerateOutput {
    /// Create a `GenerateOutput` from text
    ///
    /// If the output format of the task in unknown (i.e. was not specified)
    /// then assumes it is Markdown.
    pub async fn from_text(
        assistant: &dyn Assistant,
        task: &GenerateTask,
        text: String,
    ) -> Result<Self> {
        let format = match task.format {
            Format::Unknown => Format::Markdown,
            _ => task.format,
        };

        let node = codecs::from_str(
            &text,
            Some(DecodeOptions {
                format: Some(format),
                ..Default::default()
            }),
        )
        .await?;

        let Node::Article(Article{mut content,..}) = node else {
            bail!("Expected decoded node to be an article, got `{node}`")
        };

        author_roles(
            &mut content,
            vec![assistant.to_author_role(AuthorRoleName::Generator)],
        );

        let instruction_type = InstructionType::from(&task.instruction);
        let nodes = if matches!(
            instruction_type,
            InstructionType::InsertBlocks | InstructionType::ModifyBlocks
        ) {
            Nodes::Blocks(content)
        } else {
            Nodes::Inlines(blocks_to_inlines(content))
        };

        Ok(Self {
            content: GenerateContent::Text(text),
            format,
            nodes,
        })
    }

    /// Create a `GenerateOutput` from a URL with a specific media type
    pub async fn from_url(
        assistant: &dyn Assistant,
        _task: &GenerateTask,
        media_type: &str,
        url: String,
    ) -> Result<Self> {
        let format = Format::from_media_type(media_type).unwrap_or(Format::Unknown);

        let media_type = Some(media_type.to_string());

        let mut node = if format.is_audio() {
            Inline::AudioObject(AudioObject {
                content_url: url.clone(),
                media_type,
                ..Default::default()
            })
        } else if format.is_image() {
            Inline::ImageObject(ImageObject {
                content_url: url.clone(),
                media_type,
                ..Default::default()
            })
        } else if format.is_video() {
            Inline::VideoObject(VideoObject {
                content_url: url.clone(),
                media_type,
                ..Default::default()
            })
        } else {
            Inline::Link(Link {
                target: url.clone(),
                ..Default::default()
            })
        };

        author_roles(
            &mut node,
            vec![assistant.to_author_role(AuthorRoleName::Generator)],
        );

        let nodes = Nodes::Inlines(vec![node]);

        Ok(Self {
            content: GenerateContent::Url(url),
            format,
            nodes,
        })
    }

    /// Create a `GenerateOutput` from Base64 encoded data of a specific media type
    pub async fn from_base64(
        assistant: &dyn Assistant,
        _task: &GenerateTask,
        media_type: &str,
        data: String,
    ) -> Result<Self> {
        let url = format!("{};base64,{}", media_type, data);

        let format = Format::from_media_type(media_type).unwrap_or(Format::Unknown);

        let media_type = Some(media_type.to_string());

        let mut node = if format.is_audio() {
            Inline::AudioObject(AudioObject {
                content_url: url,
                media_type,
                ..Default::default()
            })
        } else if format.is_image() {
            Inline::ImageObject(ImageObject {
                content_url: url,
                media_type,
                ..Default::default()
            })
        } else if format.is_video() {
            Inline::VideoObject(VideoObject {
                content_url: url,
                media_type,
                ..Default::default()
            })
        } else {
            Inline::Link(Link {
                target: url,
                ..Default::default()
            })
        };

        author_roles(
            &mut node,
            vec![assistant.to_author_role(AuthorRoleName::Generator)],
        );

        let nodes = Nodes::Inlines(vec![node]);

        Ok(Self {
            content: GenerateContent::Base64(data),
            format,
            nodes,
        })
    }

    /// Display the generated output as Markdown
    ///
    /// This is mainly intended for the `stencila ai repl` and `stencila ai test`
    /// commands. It provides an easily human readable representation of the output
    /// of a generative task.
    pub fn display(&self) -> String {
        let mut md = String::new();

        // Display the output's generated content
        md += &format!(
            "The generated {format}:\n\n",
            format = if self.format.is_unknown() {
                "content"
            } else {
                self.format.name()
            }
        );
        md += &match &self.content {
            GenerateContent::Text(text) => text.to_string(),
            GenerateContent::Url(url) => format!("![]({url})"),
            GenerateContent::Base64(data) => format!(
                "![]({media_type};base64,{data})",
                media_type = self.format.media_type()
            ),
        };
        md += "\n";

        // Display nodes as a JSON code block
        let nodes = format!(
            "\nThis content was decoded to these nodes:\n\n```json\n{}\n```\n",
            serde_json::to_string_pretty(&self.nodes).unwrap_or_default()
        );
        md += &nodes;

        md
    }
}

/// The content generated for a task
pub enum GenerateContent {
    /// Generated text
    Text(String),

    /// Generated content at a URL
    Url(String),

    /// Generated content as Base64 encoded bytes
    Base64(String),
}

/// Generated nodes
///
/// This enum allows us to differentiate between different types of
/// generated nodes associated with different types of instructions
/// (block or inline)
#[derive(Serialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Nodes {
    Blocks(Vec<Block>),
    Inlines(Vec<Inline>),
}

impl Nodes {
    /// Move nodes into blocks
    pub fn into_blocks(self) -> Vec<Block> {
        match self {
            Nodes::Blocks(blocks) => blocks,
            Nodes::Inlines(inlines) => inlines_to_blocks(inlines),
        }
    }

    /// Move nodes into inlines
    pub fn into_inlines(self) -> Vec<Inline> {
        match self {
            Nodes::Blocks(blocks) => blocks_to_inlines(blocks),
            Nodes::Inlines(inlines) => inlines,
        }
    }
}

/// The type of input or output an assistant can consume or generate
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum, Display, Deserialize, Serialize,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum AssistantIO {
    #[default]
    Text,
    Image,
    Audio,
    Video,
}

/// An AI assistant
///
/// Provides a common, shared interface for the various AI models
/// and APIs used. Assistant implementations should override
/// `supports_generating` and other methods.
#[async_trait]
pub trait Assistant: Sync + Send {
    /// Get the id of the assistant
    ///
    /// This id should be unique amongst assistants. For base assistants which directly
    /// use a large language model (as opposed to a custom assistant that delegates)
    /// the id should follow the pattern <PUBLISHER>/<MODEL>.
    fn id(&self) -> String;

    /// Get the name of the publisher of the assistant
    ///
    /// This default implementation returns the title cased name
    /// before the first forward slash in the id. Derived assistants
    /// should override if necessary.
    fn publisher(&self) -> String {
        let id = self.id();
        let publisher = id
            .split_once('/')
            .map(|(publisher, ..)| publisher)
            .unwrap_or(&id);
        publisher.to_title_case()
    }

    /// Get the name of the assistant
    ///
    /// This default implementation returns the title cased name
    /// after the last forward slash but before the first dash in the id.
    /// Derived assistants should override if necessary.
    fn name(&self) -> String {
        let id = self.id();
        let name = id
            .rsplit_once('/')
            .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
            .unwrap_or(&id);
        name.to_title_case()
    }

    /// Get the version of the assistant
    ///
    /// This default implementation returns the version after the
    /// first dash in the id. Derived assistants should override if necessary.
    fn version(&self) -> String {
        let id = self.id();
        let version = id
            .split_once('-')
            .map(|(.., version)| version)
            .unwrap_or_default();
        version.to_string()
    }

    /// Create an `AuthorRole` node for this assistant
    fn to_author_role(&self, role_name: AuthorRoleName) -> AuthorRole {
        AuthorRole::new(
            PersonOrOrganizationOrSoftwareApplication::SoftwareApplication(
                self.to_software_application(),
            ),
            role_name,
        )
    }

    /// Create a `SoftwareApplication` node representing this assistant
    fn to_software_application(&self) -> SoftwareApplication {
        SoftwareApplication {
            id: Some(self.id()),
            name: self.name(),
            options: Box::new(SoftwareApplicationOptions {
                version: Some(StringOrNumber::String(self.version())),
                publisher: Some(PersonOrOrganization::Organization(Organization {
                    options: Box::new(OrganizationOptions {
                        name: Some(self.publisher()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Get the context length of the assistant
    ///
    /// Used by custom assistants to dynamically adjust the content of prompts
    /// based on the context length of the underlying model being delegated to.
    fn context_length(&self) -> usize;

    /// Does the assistant support a specific task
    ///
    /// This default implementation is based solely on whether the assistants
    /// supports the input/output combination of the task. Overrides may
    /// add other criteria such as the type of the task's instruction.
    fn supports_task(&self, task: &GenerateTask) -> bool {
        self.supports_from_to(task.input, task.output)
    }

    /// Get a list of input types this assistant supports
    fn supported_inputs(&self) -> &[AssistantIO] {
        &[]
    }

    /// Get a list of output types this assistant supports
    fn supported_outputs(&self) -> &[AssistantIO] {
        &[]
    }

    /// Whether this assistant support a specific input/output combination
    fn supports_from_to(&self, input: AssistantIO, output: AssistantIO) -> bool {
        self.supported_inputs().contains(&input) && self.supported_outputs().contains(&output)
    }

    /// A score of the suitability of this assistant for a performing a task
    ///
    /// A score between 0 and 1. A task will be delegated to the assistant
    /// with the highest suitability score for that task.
    /// Tied scores are broken using [`Self::preference_rank`].
    ///
    /// This default implementation returns 0.1 if the assistant supports the
    /// task, 0.0 otherwise.
    fn suitability_score(&self, task: &mut GenerateTask) -> Result<f32> {
        Ok(if self.supports_task(task) { 0.1 } else { 0.0 })
    }

    /// The relative rank of preference to delegate tasks to this assistant
    ///
    /// Used to break ties when to assistants have the same suitability score
    /// for a task.
    fn preference_rank(&self) -> u8 {
        0
    }

    /// Perform a generation task
    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput>;
}
