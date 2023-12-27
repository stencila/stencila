//! Custom Stencila assistants specialized for specific tasks
//!
//! An assistant is a combination of (a) a model, (b) a default prompt,
//! and (c) a set of default options. This crate defines some specialized
//! assistants build on top of lower level, more generalized assistants
//! in other crates and prompts defined in the top level `prompts` module.

use std::sync::Arc;

use minijinja::{Environment, UndefinedBehavior};
use rust_embed::RustEmbed;

use assistant::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        itertools::Itertools,
        regex::Regex,
        serde::Deserialize,
        serde_yaml,
    },
    merge::Merge,
    Assistant, AssistantIO, GenerateDetails, GenerateOptions, GenerateOutput, GenerateTask,
    InstructionType,
};
use codec_text_trait::TextCodec;
use codecs::{EncodeOptions, Format};

/// Specifications for a custom assistant read in from YAML header in Markdown
#[derive(Default, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    deny_unknown_fields,
    crate = "assistant::common::serde"
)]
struct CustomAssistantHeader {
    /// The name of the assistant
    name: String,

    /// A description of the custom assistant
    #[allow(unused)]
    description: String,

    /// The names of the assistants this assistant will delegate
    /// to in descending order of preference
    delegates: Vec<String>,

    /// The context length of the first delegate
    ///
    /// This is indicative only and defaults to 4096.
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

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<String>>,

    /// Examples of instructions to use for generating suitability score based on
    /// similarity with the actual instruction
    instruction_examples: Option<Vec<String>>,

    /// A regex to match against a comma separated list of the
    /// node types in the instruction content.
    content_nodes: Option<String>,

    /// Regexes to match in the text of the instruction content
    content_regexes: Option<Vec<String>>,

    /// Default generate options
    #[serde(flatten)]
    options: GenerateOptions,
}

/// A custom assistant
#[derive(Default)]
struct CustomAssistant {
    /// The id of the assistant
    id: String,

    /// The ids of the assistants this assistant will delegate
    /// to in descending order of preference
    delegates: Vec<String>,

    /// An indication of the context length. The actual context
    /// length for a task will depend upon the assistant delegated to.
    context_length: usize,

    /// The preference rank of the custom assistant
    preference_rank: u8,

    /// The type of instruction the assistant executes
    instruction_type: Option<InstructionType>,

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<Regex>>,

    /// Embeddings of the instructions phrases used to generate
    /// a suitability score based on similarity to instruction
    instruction_embeddings: Option<Vec<Vec<f32>>>,

    /// A regex to match against a comma separated list of the
    /// node types in the instruction content
    content_nodes: Option<Regex>,

    /// Regexes to match in the text of the instruction content
    content_regexes: Option<Vec<Regex>>,

    /// The system prompt of the assistant
    system_prompt: String,

    /// The template for rendering the user prompt
    user_prompt_template: String,

    /// The default options to use for the assistant
    options: GenerateOptions,
}

impl CustomAssistant {
    /// Parse Markdown content into a custom assistant
    fn parse(content: &str) -> Result<Self> {
        // Split a string into the three parts of a prompt: YAML header, system prompt and user prompt
        let (header, system_prompt, user_prompt_template) = content
            .splitn(4, "---")
            .map(|part| part.trim().to_string())
            .skip(1)
            .collect_tuple()
            .ok_or_else(|| eyre!("Content does not have at least three --- separators"))?;

        // Parse the header into JSON value and extract out name and extends
        let header: CustomAssistantHeader = serde_yaml::from_str(&header)?;

        Self::try_new(header, system_prompt, user_prompt_template)
    }

    /// Create from header and prompts
    fn try_new(
        header: CustomAssistantHeader,
        system_prompt: String,
        user_prompt_template: String,
    ) -> Result<Self> {
        // Parse any regexes
        let instruction_regexes = match header.instruction_regexes {
            Some(regexes) => Some(
                regexes
                    .into_iter()
                    .map(|regex| Regex::new(&regex))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            None => None,
        };
        let content_nodes = match header.content_nodes {
            Some(regex) => Some(Regex::new(&regex)?),
            None => None,
        };
        let content_regexes = match header.content_regexes {
            Some(regexes) => Some(
                regexes
                    .into_iter()
                    .map(|regex| Regex::new(&regex))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            None => None,
        };

        // Generate any embeddings
        let instruction_embeddings = match header.instruction_examples {
            Some(phrases) => Some(GenerateTask::create_embeddings(phrases)?),
            None => None,
        };

        Ok(Self {
            id: header.name,
            delegates: header.delegates,
            context_length: header.context_length.unwrap_or(4_096),
            preference_rank: header.preference_rank.unwrap_or(50),
            instruction_type: header.instruction_type,
            instruction_regexes,
            instruction_embeddings,
            content_nodes,
            content_regexes,
            system_prompt,
            user_prompt_template,
            options: header.options,
        })
    }

    /// Merge options supplied to generation functions into the default options for this custom assistant
    fn merge_options(&self, options: &GenerateOptions) -> GenerateOptions {
        let mut merged_options = self.options.clone();
        merged_options.merge(options.clone());
        merged_options
    }

    /// Update context supplied to generation functions with the system prompt and
    /// user prompt template defined in this assistant
    async fn update_task(
        &self,
        mut task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateTask> {
        task.system_prompt = Some(self.system_prompt.clone());

        // This will populate the task.instruction_text if necessary
        task.instruction_text();

        if let Some(document) = &task.document {
            task.document_formatted = Some(
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

        if let Some(nodes) = &task.instruction.content() {
            let mut content = String::new();
            for node in nodes {
                content += &codecs::to_string(
                    node,
                    Some(EncodeOptions {
                        format: options.content_format.or(Some(Format::Html)),
                        ..Default::default()
                    }),
                )
                .await?;
            }
            task.content_formatted = Some(content);
        }

        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Chainable);
        let user_prompt = env.render_str(&self.user_prompt_template, &task)?;
        task.user_prompt = Some(user_prompt);

        Ok(task)
    }

    /// Get the first assistant that is available in the list of delegates
    async fn delegate(&self) -> Result<Arc<dyn Assistant>> {
        for id in &self.delegates {
            let (provider, _model) = id
                .split('/')
                .collect_tuple()
                .ok_or_else(|| eyre!("Expected delegate assistant name to have a forward slash"))?;

            let list = match provider {
                "anthropic" => assistant_anthropic::list().await?,
                "ollama" => assistant_ollama::list().await?,
                "openai" => assistant_openai::list().await?,
                _ => bail!("Unknown assistant provider: {provider}"),
            };

            if let Some(assistant) = list
                .into_iter()
                .find(|assistant| &assistant.id() == id)
                .take()
            {
                return Ok(assistant);
            }
        }

        bail!("Unable to delegate task, none of the assistants listed in `delegates` are available")
    }
}

#[async_trait]
impl Assistant for CustomAssistant {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn context_length(&self) -> usize {
        self.context_length
    }

    fn supports_task(&self, task: &GenerateTask) -> bool {
        // If instruction type is specified then the instruction must match
        if let Some(instruction_type) = self.instruction_type {
            if instruction_type != InstructionType::from(&task.instruction) {
                return false;
            }
        }

        // If instruction regexes are specified then at least one must match
        if let Some(regexes) = &self.instruction_regexes {
            let text = task.instruction.text();
            if !regexes.iter().any(|regex| regex.is_match(text)) {
                return false;
            }
        }

        if let Some(content) = task.instruction.content() {
            // If content node type regex specified then, create a comma
            // separated list of node types, and ensure that the regex matches it
            if let Some(regex) = &self.content_nodes {
                let list = content.iter().map(|node| node.to_string()).join(",");
                if !regex.is_match(&list) {
                    return false;
                }
            }

            // If context regexes are specified then, extract the text of the content, and
            // ensure that at least one regex matches
            if let Some(regexes) = &self.content_regexes {
                let (text, ..) = content.to_text();
                if !regexes.iter().any(|regex| regex.is_match(&text)) {
                    return false;
                }
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

        let Some(instruction_embeddings) = &self.instruction_embeddings else {
            return Ok(0.1);
        };

        // Suitability score is the maximum cosine similarity between the instruction
        // and the phrases registered for this assistant
        let mut score = 0.;
        for embedding in instruction_embeddings {
            let similarity = task.instruction_similarity(embedding)?;
            if similarity > score {
                score = similarity
            }
        }

        Ok(score)
    }

    fn preference_rank(&self) -> u8 {
        self.preference_rank
    }

    async fn perform_task(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(GenerateOutput, GenerateDetails)> {
        let options = self.merge_options(options);
        let task = self.update_task(task, &options).await?;

        let assistant = self.delegate().await?;

        let (output, mut details) = assistant.perform_task(task, &options).await?;
        details.assistants.insert(0, self.id());

        Ok((output, details))
    }
}

/// Builtin assistants
///
/// During development these are loaded directly from the `assistants/builtin`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../assistants/builtin"]
struct Builtin;

/// Get a list of all available custom assistants
///
/// Memoized in production for performance, but not in debug (so that
/// custom assistants can be reloaded from disk).
/// TODO: caching
//#[cfg_attr(not(debug_assertions), cached(time = 3600))]
pub async fn list() -> Result<Vec<Arc<dyn Assistant>>> {
    list_builtin()
}

/// Get a list of all builtin assistants
///
/// Sorts in descending order of delegation rank.
fn list_builtin() -> Result<Vec<Arc<dyn Assistant>>> {
    let mut assistants = vec![];

    for (name, content) in
        Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
    {
        let content = String::from_utf8_lossy(&content);
        let assistant = CustomAssistant::parse(&content)
            .map_err(|error| eyre!("While parsing `{name}`: {error}"))?;
        assistants.push(Arc::new(assistant) as Arc<dyn Assistant>)
    }

    Ok(assistants)
}

#[cfg(test)]
mod tests {
    use assistant::{
        schema::{
            shortcuts::{p, t},
            InstructionBlock, InstructionInline,
        },
        Instruction,
    };

    use super::*;

    #[test]
    fn builtin_assistants_can_be_parsed() -> Result<()> {
        list_builtin()?;

        Ok(())
    }

    #[test]
    fn supports_task_works_as_expected() -> Result<()> {
        let tasks = [
            GenerateTask::new(
                Instruction::from(InstructionInline {
                    text: String::from("modify-inlines-regex-nodes-regex"),
                    content: Some(vec![t("the"), t(" keyword")]),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionBlock {
                    text: String::from("modify-blocks-regex-nodes"),
                    content: Some(vec![p([])]),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionBlock {
                    text: String::from("insert-blocks-regex"),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionInline {
                    text: String::from("modify-inlines-regex"),
                    content: Some(vec![t("")]),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionBlock {
                    text: String::from("insert-blocks"),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionBlock {
                    text: String::from("modify-blocks"),
                    content: Some(vec![p([])]),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionInline {
                    text: String::from("insert-inlines"),
                    ..Default::default()
                }),
                None,
            ),
            GenerateTask::new(
                Instruction::from(InstructionInline {
                    text: String::from("modify-inlines"),
                    content: Some(vec![t("")]),
                    ..Default::default()
                }),
                None,
            ),
        ];

        let assistants = [
            // Assistants with regexes and content nodes and content regexes specified
            CustomAssistant {
                id: "modify-inlines-regex-nodes-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![Regex::new("^modify-inlines-regex-nodes-regex$")?]),
                content_nodes: Some(Regex::new("^(Text,?)+$")?),
                content_regexes: Some(vec![Regex::new("keyword")?]),
                ..Default::default()
            },
            // Assistants with regexes and content nodes specified
            CustomAssistant {
                id: "modify-blocks-regex-nodes".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                instruction_regexes: Some(vec![Regex::new("^modify-blocks-regex-nodes$")?]),
                content_nodes: Some(Regex::new("^Paragraph$")?),
                ..Default::default()
            },
            // Assistants with regexes specified
            CustomAssistant {
                id: "insert-blocks-regex".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                instruction_regexes: Some(vec![Regex::new("^insert-blocks-regex$")?]),
                ..Default::default()
            },
            CustomAssistant {
                id: "modify-inlines-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![
                    Regex::new("foo")?,
                    Regex::new("^modify-inlines-regex$")?,
                ]),
                ..Default::default()
            },
            // Generic assistants
            CustomAssistant {
                id: "insert-blocks".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                ..Default::default()
            },
            CustomAssistant {
                id: "modify-blocks".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                ..Default::default()
            },
            CustomAssistant {
                id: "insert-inlines".to_string(),
                instruction_type: Some(InstructionType::InsertInlines),
                ..Default::default()
            },
            CustomAssistant {
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
        let mut task_improve_wording = GenerateTask::new(
            Instruction::from(InstructionInline {
                text: String::from("improve wording"),
                ..Default::default()
            }),
            None,
        );
        let mut task_the_improve_wording_of_this = GenerateTask::new(
            Instruction::from(InstructionInline {
                text: String::from("improve the wording of this"),
                ..Default::default()
            }),
            None,
        );
        let mut task_make_table = GenerateTask::new(
            Instruction::from(InstructionInline {
                text: String::from("make a 4x4 table"),
                ..Default::default()
            }),
            None,
        );

        let assistant_improve_wording = CustomAssistant::try_new(
            CustomAssistantHeader {
                instruction_examples: Some(vec![String::from("improve wording")]),
                ..Default::default()
            },
            String::new(),
            String::new(),
        )?;

        let score_perfect =
            assistant_improve_wording.suitability_score(&mut task_improve_wording)?;
        assert!(score_perfect > 0.9999);

        let score_high =
            assistant_improve_wording.suitability_score(&mut task_the_improve_wording_of_this)?;
        assert!(score_high < score_perfect);

        let score_low = assistant_improve_wording.suitability_score(&mut task_make_table)?;
        assert!(score_low < score_high);

        Ok(())
    }
}
