use std::str::FromStr;

use codec_markdown_trait::to_markdown;
use context::Context;
use merge::Merge;

use codecs::{DecodeOptions, EncodeOptions, LossesResponse};
use common::{
    async_trait::async_trait,
    clap::{self, Args, ValueEnum},
    eyre::{bail, Result},
    inflector::Inflector,
    itertools::Itertools,
    regex::Regex,
    serde::{de::Error, Deserialize, Deserializer, Serialize},
    serde_with::skip_serializing_none,
    smart_default::SmartDefault,
    strum::Display,
    tracing,
};
use format::Format;
use schema::{
    transforms::{
        blocks_to_inlines, blocks_to_nodes, inlines_to_blocks, inlines_to_nodes, transform_block,
        transform_inline,
    },
    Article, AudioObject, Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Block, ImageObject,
    Inline, InstructionBlock, InstructionInline, InstructionMessage, Link, MessagePart,
    MessageRole, Node, NodeType, Organization, OrganizationOptions, PersonOrOrganization,
    SoftwareApplication, SoftwareApplicationOptions, StringOrNumber, SuggestionBlock,
    SuggestionInline, Timestamp, VideoObject, VisitorMut, WalkNode,
};

// Export crates for the convenience of dependant crates
pub use codecs;
pub use common;
pub use context;
pub use format;
pub use merge;
pub use schema;
pub use secrets;

mod jinja;

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
    /// Create an inline instruction from some text
    pub fn inline_text<S: AsRef<str>>(text: S) -> Self {
        Instruction::Inline(InstructionInline {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text(text.into())],
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    /// Create an inline instruction from some text with content
    pub fn inline_text_with<S: AsRef<str>, C: IntoIterator<Item = Inline>>(
        text: S,
        content: C,
    ) -> Self {
        Instruction::Inline(InstructionInline {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text(text.into())],
                ..Default::default()
            }],
            content: Some(content.into_iter().collect()),
            ..Default::default()
        })
    }

    /// Create a block instruction from some text
    pub fn block_text<S: AsRef<str>>(text: S) -> Self {
        Instruction::Block(InstructionBlock {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text(text.into())],
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    /// Create a block instruction from some text with content
    pub fn block_text_with<S: AsRef<str>, C: IntoIterator<Item = Block>>(
        text: S,
        content: C,
    ) -> Self {
        Instruction::Block(InstructionBlock {
            messages: vec![InstructionMessage {
                parts: vec![MessagePart::Text(text.into())],
                ..Default::default()
            }],
            content: Some(content.into_iter().collect()),
            ..Default::default()
        })
    }

    /// Get the assignee of the instruction (if any)
    pub fn assignee(&self) -> Option<&str> {
        match self {
            Instruction::Block(block) => block.assignee.as_deref(),
            Instruction::Inline(inline) => inline.assignee.as_deref(),
        }
    }

    /// Get the messages of the instruction
    pub fn messages(&self) -> Vec<InstructionMessage> {
        let mut messages = match self {
            Instruction::Block(block) => block.messages.clone(),
            Instruction::Inline(inline) => inline.messages.clone(),
        };

        match self {
            Instruction::Block(node) => {
                for suggestion in node.suggestions.iter().flatten() {
                    // Note: this encodes suggestion content to Markdown. Using the
                    // format used by the particular assistant e.g. HTML may be more appropriate
                    let md = to_markdown(&suggestion.content);
                    messages.push(InstructionMessage::assistant(md));

                    if let Some(feedback) = &suggestion.feedback {
                        messages.push(InstructionMessage::user(feedback));
                    }
                }
            }
            Instruction::Inline(node) => {
                for suggestion in node.suggestions.iter().flatten() {
                    let md = to_markdown(&suggestion.content);
                    messages.push(InstructionMessage::assistant(md));

                    if let Some(feedback) = &suggestion.feedback {
                        messages.push(InstructionMessage::user(feedback));
                    }
                }
            }
        }

        messages
    }

    /// Get the text of the instruction
    ///
    /// This joins all the text parts from all the messages in the instruction that are
    /// not from assistants. It is generally better to use messages individually but
    /// this is provided for convenience in circumstances when the aggregate text suffices.
    pub fn text(&self) -> String {
        self.messages()
            .iter()
            .filter(|message| !matches!(message.role, Some(MessageRole::Assistant)))
            .flat_map(|message| message.parts.iter())
            .filter_map(|part| match part {
                MessagePart::Text(text) => Some(text.to_value_string()),
                _ => None,
            })
            .join(" ")
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
/// Used to assign to different types of assistants
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
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

/// A wrapper for Embeddings.
/// It starts out empty, and must be filled in.
/// We store both the text and the vectors so that we can easily
/// retrieve them in tests.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct Embeddings {
    texts: Option<Vec<String>>,
    vectors: Option<Vec<Vec<f32>>>,
}

impl Embeddings {
    pub fn is_empty(&self) -> bool {
        self.vectors.is_none()
    }

    pub fn iter_items(&self) -> impl Iterator<Item = (&str, &[f32])> {
        match (self.texts.as_ref(), self.vectors.as_ref()) {
            (Some(texts), Some(vectors)) => texts
                .iter()
                .zip(vectors.iter())
                .map(|(text, vector)| (text.as_str(), vector.as_slice()))
                .collect::<Vec<(&str, &[f32])>>()
                .into_iter(),
            _ => vec![].into_iter(),
        }
    }

    /// Create embeddings for a list of instructions texts
    pub fn build<S>(&mut self, texts: Vec<S>) -> Result<()>
    where
        S: AsRef<str> + Send + Sync,
    {
        // Store a copy of the strings.
        self.texts = Some(texts.iter().map(|s| s.as_ref().to_string()).collect());

        #[cfg(feature = "fastembed")]
        {
            use app::DirType;
            use common::{eyre::eyre, once_cell::sync::OnceCell};
            use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
            use std::path::PathBuf;

            // Informal perf tests during development indicated that using
            // a static improved speed substantially (rather than reloading for each call)
            static MODEL: OnceCell<TextEmbedding> = OnceCell::new();

            let model = match MODEL.get_or_try_init(|| {
                TextEmbedding::try_new(InitOptions {
                    // This model was chosen good performance for a small size.
                    // For benchmarks see https://huggingface.co/spaces/mteb/leaderboard
                    model_name: EmbeddingModel::BGESmallENV15,
                    cache_dir: app::get_app_dir(DirType::Cache, true)
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join("fastembed"),
                    ..Default::default()
                })
            }) {
                Ok(model) => model,
                Err(error) => bail!(error),
            };

            self.vectors = Some(model.embed(texts, None).map_err(|error| eyre!(error))?);
        }

        Ok(())
    }

    /// Return the best score of matching between two sets of embeddings.
    /// If either has not been filled, then we get None.
    pub fn score_match(&self, other: &Embeddings) -> Option<f32> {
        self.vectors
            .as_ref()
            .zip(other.vectors.as_ref())
            .map(|(v1, v2)| {
                let mut best = 0.0f32;
                for e1 in v1 {
                    for e2 in v2 {
                        let score = Self::calculate_similarity(e1, e2);
                        if score > best {
                            best = score;
                        }
                    }
                }
                best
            })
    }

    /// Calculate the cosine similarity of two embeddings
    pub fn calculate_similarity(a: &[f32], b: &[f32]) -> f32 {
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
    instruction: Instruction,

    /// The instruction text provided for convenient access in prompt template
    instruction_text: String,

    /// The instruction embedding
    ///
    /// Not serialized because not necessary to send to plugin assistants.
    #[serde(skip)]
    instruction_embedding: Embeddings,

    /// The content of the instruction in the format specified
    /// in the `GenerateOptions` (defaulting to Markdown)
    content_formatted: Option<String>,

    /// The context of the instruction
    ///
    /// This is available to assistants so that they can tailor
    /// their responses given the broader context of the document
    /// that the instruction is within. For specialized assistants
    /// it is available as the variable `context` within the
    /// system prompt template.
    context: Option<Context>,

    /// The input type of the task
    pub input: ModelIO,

    /// The output type of the task
    pub output: ModelIO,

    /// The desired output format of the task
    pub format: Format,

    /// The context length of assistant performing the task
    ///
    /// Provided here so that user prompt templates can change rendering
    /// to take into account the specific context length of the specific base
    /// assistant being used for the task.
    context_length: Option<usize>,

    /// An optional system prompt
    system_prompt: Option<String>,
}

impl GenerateTask {
    /// Create a generation task from an instruction
    pub fn new(instruction: Instruction, context: Option<Context>) -> Self {
        // Pull the text out of the instruction here.
        // TODO: Should we just build the embeddings here?
        let text = instruction.text().to_string();
        Self {
            instruction,
            instruction_text: text,
            context,
            ..Default::default()
        }
    }

    /// Get the task's instruction
    pub fn instruction(&self) -> &Instruction {
        &self.instruction
    }

    /// Get the task's instruction mutably
    pub fn instruction_mut(&mut self) -> &mut Instruction {
        &mut self.instruction
    }

    /// Get the messages of the task's instruction
    pub fn instruction_messages(&self) -> Vec<InstructionMessage> {
        self.instruction.messages()
    }

    /// Get the similarity between the text of the instruction and some other, precalculated embedding
    ///
    /// Will populate `self.instruction_embedding` if necessary, so that this only needs to
    /// be done once for each call to this function (e.g. when calculating similarity with
    /// a number of other embeddings).
    pub fn instruction_similarity(&mut self, embeddings: &Embeddings) -> Result<f32> {
        if self.instruction_embedding.is_empty() {
            self.instruction_embedding
                .build(vec![&self.instruction_text])?;
        }

        Ok(self
            .instruction_embedding
            .score_match(embeddings)
            .unwrap_or(0.1))
    }

    /// Get the task's desired input type
    pub fn input(&self) -> &ModelIO {
        &self.input
    }

    /// Get the task's desired output type
    pub fn output(&self) -> &ModelIO {
        &self.output
    }

    /// Get the task's desired output format
    pub fn format(&self) -> &Format {
        &self.format
    }

    /// Get the task's system prompt
    pub fn system_prompt(&self) -> &Option<String> {
        &self.system_prompt
    }

    /// Prepare the task to be executed by a particular assistant
    #[tracing::instrument(skip_all)]
    pub async fn prepare(
        &mut self,
        assistant: Option<&dyn Model>,
        content_format: Option<&Format>,
        system_prompt: Option<&String>,
    ) -> Result<()> {
        // Update properties of the task related to the assistant that
        // will perform the task
        if let Some(assistant) = assistant {
            self.context_length = Some(assistant.context_length());
        }

        // Encode content to desired format
        let encode_options = EncodeOptions {
            // Do not use compact encodings
            compact: Some(false),
            // Reduce log level for losses. Consider further reducing to `Ignore`.
            losses: LossesResponse::Debug,
            ..Default::default()
        };
        if let Some(nodes) = self.instruction().content() {
            let mut content = String::new();
            for node in nodes {
                content += &codecs::to_string(
                    &node,
                    Some(EncodeOptions {
                        format: content_format.cloned().or(Some(Format::Markdown)),
                        ..encode_options.clone()
                    }),
                )
                .await?;
            }
            self.content_formatted = Some(content);
        }

        // Render the system prompt with this task as context
        if let Some(prompt) = system_prompt {
            let rendered = jinja::render_template(prompt, self)?;

            // This is very useful for debugging rendering of system prompts.
            // Please consider that before removing :)
            tracing::debug!("System prompt rendered:\n\n{rendered}");

            self.system_prompt = Some(rendered);
        }

        Ok(())
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

    /// Prepare a generation task (e.g. render a system prompt) but do not actually generate content
    ///
    /// Assistant implementations should respect this option by returning an empty `GenerateOutput`
    /// from `perform_task` at the last possible moment before generation (usually just before an API request is made).
    #[arg(long)]
    #[serde(default)]
    #[merge(strategy = merge::bool::overwrite_false)]
    pub dry_run: bool,

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

    /// The type of node that each decoded node should be transformed to
    #[serde(
        deserialize_with = "deserialize_option_node_type",
        default,
        skip_serializing
    )]
    pub transform_nodes: Option<NodeType>,

    /// The pattern for the type of node that filtered for after transform in applied
    #[serde(
        deserialize_with = "deserialize_option_regex",
        default,
        skip_serializing
    )]
    pub filter_nodes: Option<Regex>,

    /// The number of nodes to take after filtering
    pub take_nodes: Option<usize>,

    /// A pattern for the type and number of nodes that should be generated
    #[serde(
        deserialize_with = "deserialize_option_regex",
        default,
        skip_serializing
    )]
    pub assert_nodes: Option<Regex>,
}

pub fn deserialize_option_node_type<'de, D>(deserializer: D) -> Result<Option<NodeType>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<String>::deserialize(deserializer)? {
        Some(value) => Some(
            NodeType::from_str(&value)
                .map_err(|error| D::Error::custom(format!("invalid node type: {error}")))?,
        ),
        None => None,
    })
}

pub fn deserialize_option_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<String>::deserialize(deserializer)? {
        Some(value) => Some(
            Regex::new(&value)
                .map_err(|error| D::Error::custom(format!("invalid regex: {error}")))?,
        ),
        None => None,
    })
}

/// Output generated for a task
///
/// Yes, this could have been named `GeneratedOutput`! But it wasn't to
/// maintain consistency with `GenerateTask` and `GenerateOptions`.
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, crate = "common::serde")]
pub struct GenerateOutput {
    /// The assistants that were involved in generating the output
    pub authors: Vec<AuthorRole>,

    /// The kind of output in the content
    ///
    /// Used to determine how to handle the `content` before
    /// decoding it into nodes.
    pub kind: GenerateKind,

    /// The format of the generated content
    ///
    /// Used by to decode the generated `content` into a set of
    /// Stencila Schema nodes.
    pub format: Format,

    /// The content generated by the assistant
    pub content: String,

    /// Stencila Schema nodes generated by the assistant or decoded from `content`
    pub nodes: Nodes,
}

impl GenerateOutput {
    /// Create an empty `GenerateOutput`
    ///
    /// Usually only used when the `--dry-run` flag is used.
    pub fn empty(assistant: &dyn Model) -> Result<Self> {
        Ok(Self {
            authors: vec![assistant.to_author_role(AuthorRoleName::Generator)],
            kind: GenerateKind::Text,
            format: Format::Unknown,
            content: (String::new()),
            nodes: Nodes::Blocks(vec![]),
        })
    }

    /// Create a `GenerateOutput` from text
    ///
    /// If the output format of the task in unknown (i.e. was not specified)
    /// then assumes it is Markdown.
    pub async fn from_text(
        assistant: &dyn Model,
        format: &Format,
        instruction: &Instruction,
        options: &GenerateOptions,
        text: String,
    ) -> Result<Self> {
        let format = match format {
            Format::Unknown => Format::Markdown,
            _ => format.clone(),
        };

        // Decode text to an article
        let node = codecs::from_str(
            &text,
            Some(DecodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            }),
        )
        .await?;
        let Node::Article(Article { content, .. }) = node else {
            bail!("Expected decoded node to be an article, got `{node}`")
        };

        // Transform content to blocks or inlines depending upon instruction type
        let nodes = if matches!(
            InstructionType::from(instruction),
            InstructionType::InsertBlocks | InstructionType::ModifyBlocks
        ) {
            Nodes::Blocks(content)
        } else {
            Nodes::Inlines(blocks_to_inlines(content))
        };

        let unfiltered_types = match &nodes {
            Nodes::Blocks(nodes) => nodes.iter().map(|node| node.to_string()).join(","),
            Nodes::Inlines(nodes) => nodes.iter().map(|node| node.to_string()).join(","),
        };

        // Transform the nodes to the expected type if specified
        let nodes = if let Some(node_type) = options.transform_nodes {
            match nodes {
                Nodes::Blocks(nodes) => Nodes::Blocks(
                    nodes
                        .into_iter()
                        .map(|node| transform_block(node, node_type))
                        .collect(),
                ),
                Nodes::Inlines(nodes) => Nodes::Inlines(
                    nodes
                        .into_iter()
                        .map(|node| transform_inline(node, node_type))
                        .collect(),
                ),
            }
        } else {
            nodes
        };

        // Filter nodes if regex specified
        let nodes = if let Some(regex) = &options.filter_nodes {
            match nodes {
                Nodes::Blocks(nodes) => Nodes::Blocks(
                    nodes
                        .into_iter()
                        .filter(|node| regex.is_match(&node.to_string()))
                        .collect(),
                ),
                Nodes::Inlines(nodes) => Nodes::Inlines(
                    nodes
                        .into_iter()
                        .filter(|node| regex.is_match(&node.to_string()))
                        .collect(),
                ),
            }
        } else {
            nodes
        };

        // Take a certain number of nodes is specified
        let nodes = if let Some(take) = options.take_nodes {
            match nodes {
                Nodes::Blocks(nodes) => Nodes::Blocks(nodes.into_iter().take(take).collect()),
                Nodes::Inlines(nodes) => Nodes::Inlines(nodes.into_iter().take(take).collect()),
            }
        } else {
            nodes
        };

        // Assert the number and type of nodes if specified
        if let Some(regex) = &options.assert_nodes {
            let list = match &nodes {
                Nodes::Blocks(nodes) => nodes.iter().map(|node| node.to_string()).join(","),
                Nodes::Inlines(nodes) => nodes.iter().map(|node| node.to_string()).join(","),
            };
            if !regex.is_match(&list) {
                bail!(
                    "Expected types of generated {format} to match pattern `{regex}`, got `{unfiltered_types}`"
                )
            }
        }

        Ok(Self {
            authors: vec![assistant.to_author_role(AuthorRoleName::Generator)],
            kind: GenerateKind::Text,
            format,
            content: text,
            nodes,
        })
    }

    /// Create a `GenerateOutput` from a URL with a specific media type
    pub async fn from_url(assistant: &dyn Model, media_type: &str, url: String) -> Result<Self> {
        let format = Format::from_media_type(media_type).unwrap_or(Format::Unknown);

        let media_type = Some(media_type.to_string());

        let node = if format.is_audio() {
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

        let nodes = Nodes::Inlines(vec![node]);

        Ok(Self {
            authors: vec![assistant.to_author_role(AuthorRoleName::Generator)],
            kind: GenerateKind::Url,
            format,
            content: url,
            nodes,
        })
    }

    /// Update output after it has been deserialized from a plugin based assistant
    pub async fn from_plugin(
        other: Self,
        assistant: &dyn Model,
        format: &Format,
        instruction: &Instruction,
        options: &GenerateOptions,
    ) -> Result<Self> {
        let mut output = if !other.nodes.is_empty() {
            other
        } else {
            match other.kind {
                GenerateKind::Text => {
                    Self::from_text(assistant, format, instruction, options, other.content).await?
                }
                GenerateKind::Url => {
                    Self::from_url(assistant, &format.media_type(), other.content).await?
                }
            }
        };

        output
            .authors
            .push(assistant.to_author_role(AuthorRoleName::Generator));

        Ok(output)
    }

    /// Create a `Message` from the output that can be added to the `messages` property
    /// of the instruction
    pub fn to_message(&self) -> InstructionMessage {
        let authors = self
            .authors
            .iter()
            .filter_map(|role| match &role.author {
                AuthorRoleAuthor::SoftwareApplication(app) => Some(app.clone()),
                _ => None,
            })
            .map(Author::SoftwareApplication)
            .collect();
        let authors = Some(authors);

        let parts = vec![match &self.kind {
            GenerateKind::Text => MessagePart::Text(self.content.clone().into()),
            GenerateKind::Url => {
                let url = self.content.clone();
                if self.format.is_audio() {
                    MessagePart::AudioObject(AudioObject::new(url))
                } else if self.format.is_image() {
                    MessagePart::ImageObject(ImageObject::new(url))
                } else if self.format.is_video() {
                    MessagePart::VideoObject(VideoObject::new(url))
                } else {
                    MessagePart::Text(url.into())
                }
            }
        }];

        InstructionMessage {
            authors,
            parts,
            ..Default::default()
        }
    }

    /// Create a `SuggestionInline` from the output that can be used for the `suggestion`
    /// property of the instruction
    pub fn to_suggestion_inline(self) -> SuggestionInline {
        SuggestionInline::new(self.nodes.into_inlines())
    }

    /// Create a `SuggestionBlock` from the output that can be used for the `suggestion`
    /// property of the instruction
    pub fn to_suggestion_block(self) -> SuggestionBlock {
        SuggestionBlock::new(self.nodes.into_blocks())
    }
}

/// The kind of content generated for a task
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum GenerateKind {
    /// Generated text in a text format
    #[default]
    Text,

    /// Generated content at a URL
    Url,
}

/// Generated nodes
///
/// This enum allows us to differentiate between different types of
/// generated nodes associated with different types of instructions
/// (block or inline)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Nodes {
    Blocks(Vec<Block>),
    Inlines(Vec<Inline>),
}

impl Default for Nodes {
    fn default() -> Self {
        Self::Blocks(Vec::new())
    }
}

impl Nodes {
    pub fn is_empty(&self) -> bool {
        match self {
            Nodes::Blocks(blocks) => blocks.is_empty(),
            Nodes::Inlines(inlines) => inlines.is_empty(),
        }
    }

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

impl WalkNode for Nodes {
    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        match self {
            Nodes::Blocks(nodes) => nodes.walk_mut(visitor),
            Nodes::Inlines(nodes) => nodes.walk_mut(visitor),
        }
    }
}

/// The type of input or output an assistant can consume or generate
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum, Display, Deserialize, Serialize,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum ModelIO {
    #[default]
    Text,
    Image,
    Audio,
    Video,
    Nodes,
}

/// An AI assistant
///
/// Provides a common, shared interface for the various AI models
/// and APIs used. Assistant implementations should override
/// `supports_generating` and other methods.
#[async_trait]
pub trait Model: Sync + Send {
    /// Get the name of the assistant
    ///
    /// The name should be unique amongst assistants.
    /// The name should follow the pattern <PUBLISHER>/<MODEL>.
    fn name(&self) -> String;

    /// Get the provider of the assistant
    fn r#type(&self) -> ModelType {
        ModelType::Builtin
    }

    /// Get the availability of the assistant
    fn availability(&self) -> ModelAvailability {
        ModelAvailability::Available
    }

    /// Is the assistant currently available?
    fn is_available(&self) -> bool {
        matches!(self.availability(), ModelAvailability::Available)
    }

    /// Get the name of the publisher of the assistant
    ///
    /// This default implementation returns the title cased name
    /// before the first forward slash in the name. Derived assistants
    /// should override if necessary.
    fn publisher(&self) -> String {
        let name = self.name();
        let publisher = name
            .split_once('/')
            .map(|(publisher, ..)| publisher)
            .unwrap_or(&name);
        publisher.to_title_case()
    }

    /// Get the title of the assistant
    ///
    /// This default implementation returns the title cased name
    /// after the last forward slash but before the first dash in the name.
    /// Derived assistants should override if necessary.
    fn title(&self) -> String {
        let name = self.name();
        let name = name
            .rsplit_once('/')
            .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
            .unwrap_or(&name);
        name.to_title_case()
    }

    /// Get the version of the assistant
    ///
    /// This default implementation returns the version after the
    /// first dash in the name. Derived assistants should override if necessary.
    fn version(&self) -> String {
        let name = self.name();
        let version = name
            .split_once('-')
            .map(|(.., version)| version)
            .unwrap_or_default();
        version.to_string()
    }

    /// A description of the assistant in Markdown
    fn description(&self) -> Option<String> {
        None
    }

    /// Create an `AuthorRole` node for this assistant
    fn to_author_role(&self, role_name: AuthorRoleName) -> AuthorRole {
        let mut role = AuthorRole::new(
            AuthorRoleAuthor::SoftwareApplication(self.to_software_application()),
            role_name,
        );
        role.last_modified = Some(Timestamp::now());
        role
    }

    /// Create a `SoftwareApplication` node identifying this assistant
    ///
    /// Intended for usage in the `authors` property of inner document
    /// nodes where it is desirable to have minimal identifying information
    /// only.
    fn to_software_application(&self) -> SoftwareApplication {
        SoftwareApplication {
            id: Some(self.name()),
            name: self.title(),
            options: Box::new(SoftwareApplicationOptions {
                version: Some(StringOrNumber::String(self.version())),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Create a `SoftwareApplication` node representing this assistant
    ///
    /// Intended for usage in the `authors` or `contributors` property
    /// of the root `CreativeWork`.
    fn to_software_application_complete(&self) -> SoftwareApplication {
        SoftwareApplication {
            id: Some(self.name()),
            name: self.title(),
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
    /// based on the context length of the underlying model being used.
    fn context_length(&self) -> usize {
        0
    }

    /// Does the assistant support a specific task
    ///
    /// This default implementation is based solely on whether the assistants
    /// supports the input/output combination of the task. Overrides may
    /// add other criteria such as the type of the task's instruction.
    fn supports_task(&self, task: &GenerateTask) -> bool {
        self.supports_from_to(task.input, task.output)
    }

    /// Get a list of input types this assistant supports
    fn supported_inputs(&self) -> &[ModelIO] {
        &[]
    }

    /// Get a list of output types this assistant supports
    fn supported_outputs(&self) -> &[ModelIO] {
        &[]
    }

    /// Whether this assistant support a specific input/output combination
    fn supports_from_to(&self, input: ModelIO, output: ModelIO) -> bool {
        self.supported_inputs().contains(&input) && self.supported_outputs().contains(&output)
    }

    /// A score of the suitability of this assistant for a performing a task
    ///
    /// A score between 0 and 1. A task will be assigned to the assistant
    /// with the highest suitability score for that task.
    /// Tied scores are broken using [`Self::preference_rank`].
    ///
    /// This default implementation returns 0.1 if the assistant supports the
    /// task, 0.0 otherwise.
    fn suitability_score(&self, task: &mut GenerateTask) -> Result<f32> {
        Ok(if self.supports_task(task) { 0.1 } else { 0.0 })
    }

    /// The relative rank of preference to assign tasks to this assistant
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

/// The provider of a model
pub enum ModelType {
    Builtin,
    Local,
    Remote,
    Plugin(String),
}

/// The availability of a model on the current machine
#[derive(Display, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum ModelAvailability {
    /// Available on this machine
    Available,
    /// Available on this machine but requires installation
    Installable,
    /// Not available on this machine
    Unavailable,
    /// Available on this machine but disabled
    Disabled,
}

/// Generate a test task which has system, user and assistant messages
///
/// Used for tests of implementations of the `Assistant` trait to check that
/// the system prompt, and each user and assistant message, are being sent to
/// and processed by the assistant.
#[allow(unused)]
pub fn test_task_repeat_word() -> GenerateTask {
    GenerateTask {
        system_prompt: Some(
            "When asked to repeat a word, you should repeat it in ALL CAPS. Do not provide any other notes, explanation or content.".to_string(),
        ),
        instruction: Instruction::from(InstructionInline {
            messages: vec![
                InstructionMessage {
                    role: Some(MessageRole::User),
                    parts: vec![MessagePart::Text("Say the word \"Hello\".".into())],
                    ..Default::default()
                },
                InstructionMessage {
                    role: Some(MessageRole::Assistant),
                    parts: vec![MessagePart::Text("Hello".into())],
                    ..Default::default()
                },
                InstructionMessage {
                    role: Some(MessageRole::User),
                    parts: vec![MessagePart::Text("Repeat the word.".into())],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[allow(unused)]
#[cfg(test)]

mod tests {
    use super::*;

    #[cfg(feature = "fastembed")]
    #[test]
    fn create_embeddings_and_compare_them() -> Result<()> {
        let mut e1 = Embeddings::default();
        let mut e2 = Embeddings::default();
        let mut e3 = Embeddings::default();
        let e4 = Embeddings::default();
        e1.build(vec!["insert an equation", "insert some math"])?;
        e2.build(vec!["insert some code"])?;
        e3.build(vec!["edit this text"])?;

        let s1 = e1.score_match(&e2).expect("Should have a score");
        let s2 = e2.score_match(&e3).expect("Should have a score");
        let s3 = e1.score_match(&e3).expect("Should have a score");
        assert!(s1 > s2);
        assert!(s2 > s3);
        assert_eq!(e4.score_match(&e3), None);

        Ok(())
    }
}
