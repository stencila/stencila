//! Custom Stencila assistants specialized for specific tasks
//!
//! An assistant is a combination of (a) a model, (b) a default prompt,
//! and (c) a set of default options. This crate defines some specialized
//! assistants build on top of lower level, more generalized assistants
//! in other crates and prompts defined in the top level `prompts` module.

use std::{fs::read_to_string, sync::Arc};

use assistant::{GenerateContent, Nodes};
#[cfg(not(debug_assertions))]
use cached::proc_macro::once;
use minijinja::{Environment, UndefinedBehavior};
use rust_embed::RustEmbed;

use app::{get_app_dir, DirType};
use assistant::common::eyre;
use assistant::schema::{Block, CodeBlock, Cord, MessagePart, NodeType};
use assistant::{
    codecs::{self, EncodeOptions, Format, LossesResponse},
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        glob::glob,
        inflector::Inflector,
        itertools::Itertools,
        regex::Regex,
        serde::{de::Error, Deserialize, Deserializer, Serialize},
        serde_yaml, tracing,
    },
    deserialize_option_regex,
    merge::Merge,
    schema::InstructionMessage,
    Assistant, AssistantIO, Embeddings, GenerateOptions, GenerateOutput, GenerateTask, Instruction,
    InstructionType,
};

/// Default preference rank
const PREFERENCE_RANK: u8 = 50;

/// Default ordered list of delegates
///
/// Ordering of text-to-text assistants loosely based on https://huggingface.co/spaces/lmsys/chatbot-arena-leaderboard
/// but with more recent models in a series always preferred over older models
/// in the same series.
///
/// Local models are at the end of the list on the assumption that
/// if an API key is available for one of the other remote providers then
/// that will usually be preferred.
const DELEGATES: &[&str] = &[
    // Text-to-text
    "openai/gpt-4-1106-preview",
    "openai/gpt-4-0613",
    "openai/gpt-4-0314",
    "anthropic/claude-2.1",
    "anthropic/claude-2.0",
    "anthropic/claude-instant-1.2",
    "mistral/mistral-medium",
    "google/gemini-pro",
    "openai/gpt-3.5-turbo-1106",
    "openai/gpt-3.5-turbo-0613",
    "openai/gpt-3.5-turbo-0301",
    "mistral/mistral-small",
    "mistral/mistral-tiny",
    "ollama/llama2:latest",
    // Text-to-image,
    "openai/dall-e-3",
    "openai/dall-e-2",
];

/// This structure eases the process of creating a specialized assistant
/// by providing a shorthand for the type of nodes expected to be returned
/// by the instruction.
/// For now, it is simple, just a node type and a boolean indicating whether
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String", crate = "assistant::common::serde")]
pub struct ExpectedNodes {
    node_type: NodeType,
    repeated: bool,
}

impl ExpectedNodes {
    /// Create a regex for the comma separated list of expected node type names
    fn as_regex(&self, use_repeat: bool) -> Result<Regex> {
        let pattern = if use_repeat && self.repeated {
            format!("^({},?)+$", self.node_type)
        } else {
            format!("^{}$", self.node_type)
        };
        Ok(Regex::new(&pattern)?)
    }

    /// Update the options based on the expected nodes.
    fn apply(&self, options: &mut GenerateOptions) -> Result<()> {
        if options.transform_nodes.is_none() {
            options.transform_nodes = Some(self.node_type);
        }
        if options.filter_nodes.is_none() {
            options.filter_nodes = Some(self.as_regex(false)?);
        }

        if options.take_nodes.is_none() && !self.repeated {
            options.take_nodes = Some(1);
        }

        if options.assert_nodes.is_none() {
            options.assert_nodes = Some(self.as_regex(true)?);
        }
        Ok(())
    }
}

// Providing these conversions means we don't need a specialized Serialize and
// Deserialize implementation for the `ExpectedNodes` struct.
// And they can be used more widely.
impl From<ExpectedNodes> for String {
    fn from(en: ExpectedNodes) -> Self {
        let mut result = en.node_type.to_string();
        if en.repeated {
            result.push('+');
        }
        result
    }
}

impl TryFrom<&str> for ExpectedNodes {
    type Error = eyre::Report;

    fn try_from(s: &str) -> Result<Self> {
        let repeated = s.ends_with('+');
        let node_type_str = if repeated { &s[..s.len() - 1] } else { s };
        let node_type = node_type_str
            .parse::<NodeType>()
            .map_err(|_| eyre::eyre!("Invalid NodeType: {}", node_type_str))?;

        Ok(ExpectedNodes {
            node_type,
            repeated,
        })
    }
}

/// Default format
const FORMAT: Format = Format::Markdown;

/// Default maximum retries
const MAX_RETRIES: u8 = 1;

/// Debug mode
#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "assistant::common::serde")]
enum Debug {
    // No debugging
    #[default]
    No,
    /// Echo the rendered system prompt as a Markdown code block
    /// in the suggestion. Useful for debugging system prompt templates.
    Echo,
}

/// A custom assistant
/// TODO: Remove this when the options are being used.
#[allow(dead_code)]
#[derive(Default, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    deny_unknown_fields,
    crate = "assistant::common::serde"
)]
pub struct SpecializedAssistant {
    /// The id of the assistant
    #[serde(skip_deserializing)]
    id: String,

    /// The version of the assistant
    version: String,

    /// A description of the custom assistant
    #[allow(unused)]
    #[serde(skip_deserializing)]
    description: String,

    /// The names of the assistants this assistant will delegate
    /// to in descending order of preference
    ///
    /// The default ordered list of delegates can be prepended
    /// using this options. If the last item is `only` then the
    /// list will be limited to those specified.
    #[serde(
        deserialize_with = "deserialize_delegates",
        default = "default_delegates"
    )]
    delegates: Vec<String>,

    /// The debug mode to use when executing the assistant
    #[serde(default)]
    debug: Debug,

    /// The type of input for the generation task delegated
    /// to base assistants
    task_input: Option<AssistantIO>,

    /// The type of output for the generation task delegated
    /// to base assistants
    task_output: Option<AssistantIO>,

    /// An indication of the context length
    ///
    /// At runtime, the context length of the assistant delegated to is
    /// used (for example to trim prompts).
    context_length: Option<usize>,

    /// The preference rank of the custom assistant
    ///
    /// Defaults to 50 so that custom assistants are by default
    /// preferred over generic assistants.
    preference_rank: Option<u8>,

    /// The type of instruction the assistant executes
    instruction_type: Option<InstructionType>,

    /// A description of the kinds of nodes expected to be returned by the instruction.
    expected_nodes: Option<ExpectedNodes>,

    /// Regexes to match in the instruction text
    #[serde(deserialize_with = "deserialize_option_vec_regex", default)]
    instruction_regexes: Option<Vec<Regex>>,

    /// Examples of instructions used to generate a suitability score
    instruction_examples: Option<Vec<String>>,

    /// Embeddings of the instructions examples
    #[serde(skip_deserializing)]
    instruction_embeddings: Embeddings,

    /// A regex to match against a comma separated list of the
    /// node types in the instruction content
    #[serde(deserialize_with = "deserialize_option_regex", default)]
    content_nodes: Option<Regex>,

    /// Regexes to match in the text of the instruction content
    #[serde(deserialize_with = "deserialize_option_vec_regex", default)]
    content_regexes: Option<Vec<Regex>>,

    /// The format to convert various parts of the document and generated content
    ///
    /// Generally this single format is applied to the `document`, the `content` of
    /// the instruction, and to the generated content. However, these can be specified
    /// separately using `document_format`, `content_format`, and `generated_format`
    /// respectively.
    format: Option<Format>,

    /// The format to convert the document content into when rendered into the prompt.
    document_format: Option<Format>,

    /// The format to convert the instruction content (if any) into when rendered into the prompt.
    content_format: Option<Format>,

    /// The format of the generated content
    generated_format: Option<Format>,

    /// The system prompt of the assistant
    #[serde(skip_deserializing)]
    system_prompt: Option<String>,

    /// The template environment for the system prompt (if any)
    #[serde(skip_deserializing)]
    template_env: Option<Environment<'static>>,

    /// The maximum number of retries for generating valid nodes
    max_retries: Option<u8>,

    /// The default options to use for the assistant
    #[serde(flatten)]
    options: GenerateOptions,
}

fn default_delegates() -> Vec<String> {
    DELEGATES
        .iter()
        .map(|delegate| delegate.to_string())
        .collect()
}

fn deserialize_delegates<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut defaults: Vec<String> = default_delegates();

    if let Some(mut specified) = Option::<Vec<String>>::deserialize(deserializer)? {
        if let Some("none") = specified.first().map(|id| id.as_str()) {
            return Ok(Vec::new());
        } else if let Some("only") = specified.last().map(|id| id.as_str()) {
            specified.pop();
        } else {
            defaults.retain(|delegate| !specified.contains(delegate));
            specified.append(&mut defaults);
        }
        Ok(specified)
    } else {
        Ok(defaults)
    }
}

fn deserialize_option_vec_regex<'de, D>(deserializer: D) -> Result<Option<Vec<Regex>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<Vec<String>>::deserialize(deserializer)? {
        Some(value) => Some(
            value
                .into_iter()
                .map(|regex| Regex::new(&regex))
                .collect::<Result<Vec<Regex>, _>>()
                .map_err(|error| D::Error::custom(format!("invalid regex: {error}")))?,
        ),
        None => None,
    })
}

/// Expand a `minijinja` error to include the sources of the error (location etc)
fn minijinja_error_to_eyre(error: minijinja::Error) -> eyre::Report {
    let mut error = &error as &dyn std::error::Error;
    let mut message = format!("{error:#}");
    while let Some(source) = error.source() {
        message.push_str(&format!("\n{:#}", source));
        error = source;
    }
    eyre!(message)
}

const SYSTEM_PROMPT_TEMPLATE_NAME: &str = "system_prompt";

impl SpecializedAssistant {
    // Added for testing
    // TODO: Wrap these in test / debug assertions?
    pub fn instruction_examples(&self) -> &Option<Vec<String>> {
        &self.instruction_examples
    }

    pub fn instruction_embeddings(&self) -> &Embeddings {
        &self.instruction_embeddings
    }

    pub fn instruction_type(&self) -> &Option<InstructionType> {
        &self.instruction_type
    }

    /// Return
    /// Parse Markdown content into a custom assistant
    fn parse(id: &str, content: &str) -> Result<Self> {
        // Split a string into parts and ensure that there is at least a header
        let mut parts = content
            .split("---\n")
            .map(|part| part.trim().to_string())
            .skip(1);
        let Some(header) = parts.next() else {
            bail!("Assistant file should have a YAML header delimited by ---");
        };

        // Parse header into an assistant
        let mut assistant: SpecializedAssistant = serde_yaml::from_str(&header)?;
        assistant.id = id.to_string();
        assistant.description = parts.next().unwrap_or_else(|| "No description".to_string());

        // If the system prompt is blank then make it None
        let prompt = parts.next().and_then(|prompt| {
            let trimmed = prompt.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        // If there is a system prompt then instantiate a template environment for it
        if let Some(prompt) = prompt {
            assistant.template_env = Some(Self::template_environment(prompt.clone())?);
            assistant.system_prompt = Some(prompt);
        }

        assistant.init()?;

        Ok(assistant)
    }

    /// Create a template environment for rendering prompts
    fn template_environment(prompt: String) -> Result<Environment<'static>> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);

        env.add_filter("trim_start_chars", |content: &str, length: u32| -> String {
            let current_length = content.chars().count();
            content
                .chars()
                .skip(current_length.saturating_sub(length as usize))
                .take(length as usize)
                .collect()
        });

        env.add_filter("trim_end_chars", |content: &str, length: u32| -> String {
            content.chars().take(length as usize).collect()
        });

        env.add_template_owned(SYSTEM_PROMPT_TEMPLATE_NAME, prompt)
            .map_err(minijinja_error_to_eyre)?;

        Ok(env)
    }

    /// Initialize the assistant
    pub fn init(&mut self) -> Result<()> {
        // Calculate embeddings if necessary
        if let Some(examples) = &self.instruction_examples {
            self.instruction_embeddings.build(examples.clone())?;
        }

        // Apply expected nodes to options, updating them if necessary
        if let Some(expected_nodes) = &self.expected_nodes {
            expected_nodes.apply(&mut self.options)?;
        }

        Ok(())
    }

    /// Merge a `GenerateTask` with the relevant options of this assistant
    ///
    /// This should be called before selecting an assistant to delegate to
    /// (since the input and output type of the task influences that)
    fn merge_task(&self, task: &GenerateTask) -> GenerateTask {
        let mut task = task.clone();

        if let Some(input) = self.task_input {
            task.input = input;
        }

        if let Some(output) = self.task_output {
            task.output = output;
        }

        task.format = self
            .generated_format
            .clone()
            .or(self.format.clone())
            .unwrap_or(FORMAT);

        task
    }

    /// Merge options supplied to generation functions into the default options for this custom assistant
    fn merge_options(&self, options: &GenerateOptions) -> GenerateOptions {
        let mut merged_options = self.options.clone();
        merged_options.merge(options.clone());
        merged_options
    }

    /// Prepare a `GenerateTask` with the system prompt, rendered user prompt of
    /// this assistant, and other details
    #[tracing::instrument(skip_all)]
    async fn prepare_task(
        &self,
        mut task: GenerateTask,
        delegate: Option<&dyn Assistant>,
    ) -> Result<GenerateTask> {
        if let Some(system_prompt) = &self.system_prompt {
            task.system_prompt = Some(system_prompt.clone());
        }
        // Encode document and content with these defaults
        let encode_options = EncodeOptions {
            // Do not use compact encodings
            compact: Some(false),
            // Reduce log level for losses. Consider further reducing to `Ignore`.
            losses: LossesResponse::Debug,
            ..Default::default()
        };
        if let Some(nodes) = &task.instruction.content() {
            let mut content = String::new();
            for node in nodes {
                content += &codecs::to_string(
                    node,
                    Some(EncodeOptions {
                        format: self
                            .content_format
                            .clone()
                            .or(self.format.clone())
                            .or(Some(FORMAT)),
                        ..encode_options.clone()
                    }),
                )
                .await?;
            }
            task.content_formatted = Some(content);
        }

        // Update other properties of the task related to the delegate (is any)
        if let Some(delegate) = delegate {
            task.context_length = Some(delegate.context_length());
        }

        // If the assistant has a template env (and this a system prompt)
        // then render the prompt with the task as its context
        if let Some(env) = &self.template_env {
            let template = env.get_template(SYSTEM_PROMPT_TEMPLATE_NAME)?;
            let prompt = template
                .render(&task)
                .map_err(minijinja_error_to_eyre)?
                .trim()
                .to_string();
            task.system_prompt = Some(prompt);
        }

        Ok(task)
    }

    /// Get the first assistant in the list of delegates capable to performing task
    #[tracing::instrument(skip_all)]
    async fn first_available_delegate(&self, task: &GenerateTask) -> Result<Arc<dyn Assistant>> {
        for id in &self.delegates {
            let (provider, _model) = id
                .split('/')
                .collect_tuple()
                .ok_or_else(|| eyre!("Expected delegate assistant name to have a forward slash"))?;

            let list = match provider {
                "anthropic" => assistant_anthropic::list().await?,
                "google" => assistant_google::list().await?,
                "mistral" => assistant_mistral::list().await?,
                "ollama" => assistant_ollama::list().await?,
                "openai" => assistant_openai::list().await?,
                _ => bail!("Unknown assistant provider: {provider}"),
            };

            if let Some(assistant) = list
                .into_iter()
                .find(|assistant| &assistant.id() == id)
                .take()
            {
                if assistant.supports_task(task) {
                    return Ok(assistant);
                }
            }
        }

        bail!("Unable to delegate task, none of the assistants listed in `delegates` are available or capable of performing task: {}", self.delegates.join(", "))
    }
}

#[async_trait]
impl Assistant for SpecializedAssistant {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> String {
        let id = self.id();
        let name = id.rsplit_once('/').map(|(.., name)| name).unwrap_or(&id);
        name.to_title_case()
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn context_length(&self) -> usize {
        self.context_length.unwrap_or_default()
    }

    fn supports_task(&self, task: &GenerateTask) -> bool {
        // If instruction type is specified then the instruction must match
        if let Some(instruction_type) = self.instruction_type {
            if instruction_type != InstructionType::from(&task.instruction) {
                return false;
            }
        }

        true
    }

    fn supported_inputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    fn supported_outputs(&self) -> &[AssistantIO] {
        &[AssistantIO::Text]
    }

    fn suitability_score(&self, task: &mut GenerateTask) -> Result<f32> {
        if !self.supports_task(task) {
            return Ok(0.0);
        }

        task.instruction_similarity(&self.instruction_embeddings)
    }

    fn preference_rank(&self) -> u8 {
        self.preference_rank.unwrap_or(PREFERENCE_RANK)
    }

    #[tracing::instrument(skip_all)]
    async fn perform_task(
        &self,
        task: &GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateOutput> {
        let task = self.merge_task(task);
        let options = self.merge_options(options);

        let output = if matches!(self.debug, Debug::Echo) {
            // Debug echo so just render the prompt into a Markdown
            // code block in the output

            let task = self.prepare_task(task, None).await?;
            let prompt = task.system_prompt.clone().unwrap_or_default();

            GenerateOutput {
                prompter: None,
                generator: self.to_software_application(),
                content: GenerateContent::Text(prompt.clone()),
                format: Format::Markdown,
                nodes: Nodes::Blocks(vec![Block::CodeBlock(CodeBlock {
                    code: Cord::new(prompt.clone()),
                    ..Default::default()
                })]),
            }
        } else if self.delegates.is_empty() {
            // No delegates, so simply render the template into output.
            // This differs from `Debug::Echo` in that the prompt is decoded into nodes
            // (including transformations associated with `expected_nodes`) in the call to `from_text`.

            let task = self.prepare_task(task, None).await?;
            let prompt = task.system_prompt.clone().unwrap_or_default();

            GenerateOutput::from_text(self, &task, &options, prompt).await?
        } else {
            // Get the first available assistant to delegate to
            let delegate = self.first_available_delegate(&task).await?;

            // Update the task, to render template etc based on the delegate, before performing it
            let mut task = self.prepare_task(task, Some(delegate.as_ref())).await?;

            // Try once, and then up to `max_retries`, breaking early if successful
            let max_retries = self.max_retries.unwrap_or(MAX_RETRIES);
            for retry in 0..=max_retries {
                let result: Result<GenerateOutput> = delegate.perform_task(&task, &options).await;
                match result {
                    Ok(mut output) => {
                        // Assign this assistant as the prompter for the output so it can be recorded
                        // in messages and output nodes
                        output.assign_prompter(self);

                        return Ok(output);
                    }
                    Err(error) => {
                        if retry >= max_retries {
                            return Err(error);
                        }

                        tracing::debug!("Error on retry {retry}: {error}");

                        // Add the error to the instruction messages so that the assistant
                        // can use it to try to correct
                        let message = InstructionMessage {
                            parts: vec![MessagePart::Text(format!("Error: {error}").into())],
                            ..Default::default()
                        };
                        match &mut task.instruction {
                            Instruction::Block(instr) => instr.messages.push(message),
                            Instruction::Inline(instr) => instr.messages.push(message),
                        }
                    }
                }
            }

            // Should not be reached but in case it is...
            bail!("Maximum number of retries reached")
        };

        Ok(output)
    }
}

/// Builtin assistants
///
/// During development these are loaded directly from the `assistants/builtin`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../assistants/builtin"]
struct Builtin;

/// Get a list of all available specialized assistants
///
/// Memoized in production for performance (i.e not parsing files or creating
/// embeddings), but not in debug (so that custom assistants can be reloaded from disk).
#[cfg_attr(not(debug_assertions), once(result = true))]
pub fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    let mut list = list_builtin()?;
    list.append(&mut list_local()?);
    Ok(list)
}

/// Get a list of the specialized assistants.
/// Useful for testing.
pub fn list_builtin_as_specialized() -> Result<Vec<SpecializedAssistant>> {
    let mut assistants = vec![];

    for (name, content) in
        Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
    {
        let id = format!("stencila/{}", name.strip_suffix(".md").unwrap_or(&name));
        let content = String::from_utf8_lossy(&content);
        let assistant = SpecializedAssistant::parse(&id, &content)
            .map_err(|error| eyre!("While parsing `{name}`: {error}"))?;
        assistants.push(assistant)
    }
    Ok(assistants)
}

/// Get a list of all builtin specialized assistants as Assistant trait objects
fn list_builtin() -> Result<Vec<Arc<dyn Assistant>>> {
    list_builtin_as_specialized().map(|assistants| {
        assistants
            .into_iter()
            .map(|assistant| Arc::new(assistant) as Arc<dyn Assistant>)
            .collect()
    })
}

/// Get a list of all local specialized assistants
fn list_local() -> Result<Vec<Arc<dyn Assistant>>> {
    let mut assistants = vec![];

    let dir = get_app_dir(DirType::Assistants, false)?;

    tracing::debug!(
        "Attempting to reading assistants from `{}` (if it exists)",
        dir.display()
    );

    if !dir.exists() {
        return Ok(assistants);
    }

    for path in glob(&dir.join("*.md").to_string_lossy())?.flatten() {
        let Some(name) = path.file_name().map(|name| name.to_string_lossy()) else {
            continue;
        };
        let id = format!("local/{}", name.strip_suffix(".md").unwrap_or(&name));

        let content = read_to_string(&path)?;

        let assistant = SpecializedAssistant::parse(&id, &content)
            .map_err(|error| eyre!("While parsing `{}`: {error}", path.display()))?;
        assistants.push(Arc::new(assistant) as Arc<dyn Assistant>)
    }

    Ok(assistants)
}

#[cfg(test)]
mod tests {
    use assistant::{
        schema::shortcuts::{p, t},
        Instruction,
    };

    use super::*;

    #[test]
    fn builtin_assistants_can_be_parsed() -> Result<()> {
        list_builtin()?;

        Ok(())
    }

    #[test]
    fn test_expected_nodes_conversion() {
        let test_cases = [
            ("Paragraph", NodeType::Paragraph, false),
            ("CodeBlock+", NodeType::CodeBlock, true),
            ("Cite", NodeType::Cite, false),
            ("Claim+", NodeType::Claim, true),
        ];

        for &(input, expected_node_type, expected_repeated) in &test_cases {
            match ExpectedNodes::try_from(input) {
                Ok(en) => {
                    assert_eq!(en.node_type, expected_node_type);
                    assert_eq!(en.repeated, expected_repeated);
                }
                Err(e) => panic!("Failed to convert from str: {}", e),
            }

            let en = ExpectedNodes {
                node_type: expected_node_type,
                repeated: expected_repeated,
            };
            let output: String = en.into();
            assert_eq!(output, input)
        }
    }

    #[test]
    fn expected_nodes_fills_out_options() -> Result<()> {
        let mut assistant = SpecializedAssistant {
            id: "insert-blocks".to_string(),
            instruction_type: Some(InstructionType::InsertBlocks),
            expected_nodes: Some(ExpectedNodes {
                node_type: NodeType::Paragraph,
                repeated: true,
            }),
            ..Default::default()
        };
        assistant.init()?;

        assert_eq!(assistant.options.transform_nodes, Some(NodeType::Paragraph));

        let rx = assistant
            .options
            .filter_nodes
            .ok_or_else(|| eyre!("Expected filter_nodes to be Some"))?;

        assert_eq!("^Paragraph$", rx.as_str());

        assert_eq!(assistant.options.take_nodes, None);

        let rx = assistant
            .options
            .assert_nodes
            .ok_or_else(|| eyre!("Expected assert_nodes to be Some"))?;

        assert_eq!("^(Paragraph,?)+$", rx.as_str());

        Ok(())
    }

    #[test]
    fn supports_task_works_as_expected() -> Result<()> {
        let tasks = [
            /*
            TODO: temporarily commented out while instruction regex in flux

            GenerateTask::new(Instruction::inline_text_with(
                "modify-inlines-regex-nodes-regex",
                [t("the"), t(" keyword")],
            )),
            GenerateTask::new(Instruction::block_text_with(
                "modify-blocks-regex-nodes",
                [p([])],
            )),
            GenerateTask::new(Instruction::block_text("insert-blocks-regex")),
            GenerateTask::new(Instruction::inline_text_with(
                "modify-inlines-regex",
                [t("")],
            )),
            */
            GenerateTask::new(Instruction::block_text("insert-blocks")),
            GenerateTask::new(Instruction::block_text_with("modify-blocks", [p([])])),
            GenerateTask::new(Instruction::inline_text("insert-inlines")),
            GenerateTask::new(Instruction::inline_text_with("modify-inlines", [t("")])),
        ];

        let assistants = [
            /*
            // Assistants with regexes and content nodes and content regexes specified
            SpecializedAssistant {
                id: "modify-inlines-regex-nodes-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![Regex::new("^modify-inlines-regex-nodes-regex$")?]),
                content_nodes: Some(Regex::new("^(Text,?)+$")?),
                content_regexes: Some(vec![Regex::new("keyword")?]),
                ..Default::default()
            },
            // Assistants with regexes and content nodes specified
            SpecializedAssistant {
                id: "modify-blocks-regex-nodes".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                instruction_regexes: Some(vec![Regex::new("^modify-blocks-regex-nodes$")?]),
                content_nodes: Some(Regex::new("^Paragraph$")?),
                ..Default::default()
            },
            // Assistants with regexes specified
            SpecializedAssistant {
                id: "insert-blocks-regex".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                instruction_regexes: Some(vec![Regex::new("^insert-blocks-regex$")?]),
                ..Default::default()
            },
            SpecializedAssistant {
                id: "modify-inlines-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![
                    Regex::new("foo")?,
                    Regex::new("^modify-inlines-regex$")?,
                ]),
                ..Default::default()
            },
            */
            // Generic assistants
            SpecializedAssistant {
                id: "insert-blocks".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                ..Default::default()
            },
            SpecializedAssistant {
                id: "modify-blocks".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                ..Default::default()
            },
            SpecializedAssistant {
                id: "insert-inlines".to_string(),
                instruction_type: Some(InstructionType::InsertInlines),
                ..Default::default()
            },
            SpecializedAssistant {
                id: "modify-inlines".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                ..Default::default()
            },
        ];

        // Iterate over tasks (in reverse order, generic to specific) and ensure that the assistants
        // that it matches against has the name equal to the instruction text of the task
        for task in tasks.iter().rev() {
            let task_name = task.instruction.text();

            let mut matched = false;
            for assistant in &assistants {
                if assistant.supports_task(task) {
                    let assistant_name = assistant.id.as_str();
                    if assistant_name != task_name {
                        bail!(
                            "Task `{task_name}` was unexpectedly matched by assistant `{assistant_name}`"
                        );
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                bail!("Task `{task_name}` was not matched by any assistant");
            }
        }

        Ok(())
    }

    //#[ignore]
    #[test]
    fn suitability_score_works_as_expected() -> Result<()> {
        let mut task_improve_wording =
            GenerateTask::new(Instruction::inline_text("improve wording"));
        let mut task_the_improve_wording_of_this =
            GenerateTask::new(Instruction::inline_text("improve the wording of this"));
        let mut task_make_table = GenerateTask::new(Instruction::inline_text("make a 4x4 table"));

        let mut assistant_improve_wording = SpecializedAssistant {
            instruction_examples: Some(vec![String::from("improve wording")]),
            ..Default::default()
        };
        assistant_improve_wording.init()?;

        let score_perfect =
            assistant_improve_wording.suitability_score(&mut task_improve_wording)?;
        println!("{}", score_perfect);
        assert!(score_perfect > 0.9999);

        let score_high =
            assistant_improve_wording.suitability_score(&mut task_the_improve_wording_of_this)?;
        assert!(score_high < score_perfect);

        let score_low = assistant_improve_wording.suitability_score(&mut task_make_table)?;
        assert!(score_low < score_high);

        Ok(())
    }
}
