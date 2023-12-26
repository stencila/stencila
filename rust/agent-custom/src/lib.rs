//! Custom Stencila agents specialized for specific tasks
//!
//! An agent is a combination of (a) a model, (b) a default prompt,
//! and (c) a set of default options. This crate defines some specialized
//! agents build on top of lower level, more generalized agents
//! in other crates and prompts defined in the top level `prompts` module.

use std::sync::Arc;

use agent::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        itertools::Itertools,
        regex::Regex,
        serde::Deserialize,
        serde_yaml,
    },
    merge::Merge,
    Agent, AgentIO, GenerateDetails, GenerateOptions, GenerateTask, InstructionType, GenerateOutput,
};
use codec_text_trait::TextCodec;
use codecs::{EncodeOptions, Format};
use minijinja::{Environment, UndefinedBehavior};
use rust_embed::RustEmbed;

/// Specifications for a custom agent read in from YAML header in Markdown
#[derive(Deserialize)]
#[serde(
    rename_all = "kebab-case",
    deny_unknown_fields,
    crate = "agent::common::serde"
)]
struct CustomAgentHeader {
    /// The name of the agent
    name: String,

    /// A description of the custom agent
    #[allow(unused)]
    description: String,

    /// The names of the agents this agent will delegate
    /// to in descending order of preference
    delegates: Vec<String>,

    /// The preference rank of the custom agent
    ///
    /// Defaults to 50 so that custom agents are by default
    /// preferred over generic agents.
    preference_rank: Option<u8>,

    /// The type of instruction the agent executes
    instruction_type: Option<InstructionType>,

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<String>>,

    /// A regex to match against a comma separated list of the
    /// node types in the instruction content.
    content_nodes: Option<String>,

    /// Regexes to match in the text of the instruction content
    content_regexes: Option<Vec<String>>,

    /// Default generate options
    #[serde(flatten)]
    options: GenerateOptions,
}

/// A custom agent
#[derive(Default)]
struct CustomAgent {
    /// The name of the agent
    name: String,

    /// The names of the agents this agent will delegate
    /// to in descending order of preference
    delegates: Vec<String>,

    /// The preference rank of the custom agent
    preference_rank: u8,

    /// The type of instruction the agent executes
    instruction_type: Option<InstructionType>,

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<Regex>>,

    /// A regex to match against a comma separated list of the
    /// node types in the instruction content
    content_nodes: Option<Regex>,

    /// Regexes to match in the text of the instruction content
    content_regexes: Option<Vec<Regex>>,

    /// The system prompt of the agent
    system_prompt: String,

    /// The template for rendering the user prompt
    user_prompt_template: String,

    /// The default options to use for the agent
    #[allow(unused)]
    options: GenerateOptions,
}

impl CustomAgent {
    /// Parse Markdown content into a custom agent
    fn parse(content: &str) -> Result<CustomAgent> {
        // Split a string into the three parts of a prompt: YAML header, system prompt and user prompt
        let (header, system_prompt, user_prompt_template) = content
            .splitn(4, "---")
            .map(|part| part.trim().to_string())
            .skip(1)
            .collect_tuple()
            .ok_or_else(|| eyre!("Content does not have at least three --- separators"))?;

        // Parse the header into JSON value and extract out name and extends
        let header: CustomAgentHeader = serde_yaml::from_str(&header)?;

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

        Ok(Self {
            name: header.name,
            delegates: header.delegates,
            preference_rank: header.preference_rank.unwrap_or(50),
            instruction_type: header.instruction_type,
            instruction_regexes,
            content_nodes,
            content_regexes,
            system_prompt,
            user_prompt_template,
            options: header.options,
        })
    }

    /// Merge options supplied to generation functions into the default options for this custom agent
    fn merge_options(&self, options: &GenerateOptions) -> GenerateOptions {
        let mut merged_options = self.options.clone();
        merged_options.merge(options.clone());
        merged_options
    }

    /// Update context supplied to generation functions with the system prompt and
    /// user prompt template defined in this agent
    ///
    /// Currently, this uses `minijinja` but the intension is to use Stencila Markdown
    /// to render them in the future.
    async fn update_task(
        &self,
        mut task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<GenerateTask> {
        task.system_prompt = Some(self.system_prompt.clone());

        task.instruction_text = task.instruction.text().to_string();

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

    /// Get the first agent that is available in the list of delegates
    async fn delegate(&self) -> Result<Arc<dyn Agent>> {
        for name in &self.delegates {
            let (provider, _model) = name
                .split('/')
                .collect_tuple()
                .ok_or_else(|| eyre!("Expected delegate agent name to have a forward slash"))?;

            let list = match provider {
                "anthropic" => agent_anthropic::list().await?,
                "ollama" => agent_ollama::list().await?,
                "openai" => agent_openai::list().await?,
                _ => bail!("Unknown agent provider: {provider}"),
            };

            if let Some(agent) = list.into_iter().find(|agent| &agent.name() == name).take() {
                return Ok(agent);
            }
        }

        bail!("Unable to delegate task, none of the agents listed in `delegates` are available")
    }
}

#[async_trait]
impl Agent for CustomAgent {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn provider(&self) -> String {
        "stencila".to_string()
    }

    fn model(&self) -> String {
        "delegated".to_string()
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

    fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
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

        let agent = self.delegate().await?;

        let (output, mut details) = agent.perform_task(task, &options).await?;
        details.agents.insert(0, self.name());

        Ok((output, details))
    }
}

/// Builtin agents
///
/// During development these are loaded directly from the `agents/builtin`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../agents/builtin"]
struct Builtin;

/// Get a list of all available custom agents
///
/// Memoized in production for performance, but not in debug (so that
/// custom agents can be reloaded from disk).
/// TODO: caching
//#[cfg_attr(not(debug_assertions), cached(time = 3600))]
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    list_builtin()
}

/// Get a list of all builtin agents
///
/// Sorts in descending order of delegation rank.
fn list_builtin() -> Result<Vec<Arc<dyn Agent>>> {
    let mut agents = vec![];

    for (name, content) in
        Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
    {
        let content = String::from_utf8_lossy(&content);
        let agent = CustomAgent::parse(&content)
            .map_err(|error| eyre!("While parsing `{name}`: {error}"))?;
        agents.push(Arc::new(agent) as Arc<dyn Agent>)
    }

    Ok(agents)
}

#[cfg(test)]
mod tests {
    use agent::{
        schema::{
            shortcuts::{p, t},
            InstructionBlock, InstructionInline,
        },
        Instruction,
    };

    use super::*;

    #[test]
    fn builtin_agents_can_be_parsed() -> Result<()> {
        list_builtin()?;

        Ok(())
    }

    #[test]
    fn supports_task_works_as_expected() -> Result<()> {
        let tasks = [
            GenerateTask {
                instruction: Instruction::from(InstructionInline {
                    text: String::from("modify-inlines-regex-nodes-regex"),
                    content: Some(vec![t("the"), t(" keyword")]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionBlock {
                    text: String::from("modify-blocks-regex-nodes"),
                    content: Some(vec![p([])]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionBlock {
                    text: String::from("insert-blocks-regex"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionInline {
                    text: String::from("modify-inlines-regex"),
                    content: Some(vec![t("")]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionBlock {
                    text: String::from("insert-blocks"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionBlock {
                    text: String::from("modify-blocks"),
                    content: Some(vec![p([])]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionInline {
                    text: String::from("insert-inlines"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            GenerateTask {
                instruction: Instruction::from(InstructionInline {
                    text: String::from("modify-inlines"),
                    content: Some(vec![t("")]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ];

        let agents = [
            // Agents with regexes and content nodes and content regexes specified
            CustomAgent {
                name: "modify-inlines-regex-nodes-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![Regex::new("^modify-inlines-regex-nodes-regex$")?]),
                content_nodes: Some(Regex::new("^(Text,?)+$")?),
                content_regexes: Some(vec![Regex::new("keyword")?]),
                ..Default::default()
            },
            // Agents with regexes and content nodes specified
            CustomAgent {
                name: "modify-blocks-regex-nodes".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                instruction_regexes: Some(vec![Regex::new("^modify-blocks-regex-nodes$")?]),
                content_nodes: Some(Regex::new("^Paragraph$")?),
                ..Default::default()
            },
            // Agents with regexes specified
            CustomAgent {
                name: "insert-blocks-regex".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                instruction_regexes: Some(vec![Regex::new("^insert-blocks-regex$")?]),
                ..Default::default()
            },
            CustomAgent {
                name: "modify-inlines-regex".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                instruction_regexes: Some(vec![
                    Regex::new("foo")?,
                    Regex::new("^modify-inlines-regex$")?,
                ]),
                ..Default::default()
            },
            // Generic agents
            CustomAgent {
                name: "insert-blocks".to_string(),
                instruction_type: Some(InstructionType::InsertBlocks),
                ..Default::default()
            },
            CustomAgent {
                name: "modify-blocks".to_string(),
                instruction_type: Some(InstructionType::ModifyBlocks),
                ..Default::default()
            },
            CustomAgent {
                name: "insert-inlines".to_string(),
                instruction_type: Some(InstructionType::InsertInlines),
                ..Default::default()
            },
            CustomAgent {
                name: "modify-inlines".to_string(),
                instruction_type: Some(InstructionType::ModifyInlines),
                ..Default::default()
            },
        ];

        // Iterate over tasks (in reverse order, generic to specific) and ensure that the agents
        // that it matches against has the name equal to the instruction text of the task
        for task in tasks.iter().rev() {
            let task_name = task.instruction.text();

            let mut matched = false;
            for agent in &agents {
                if agent.supports_task(task) {
                    let agent_name = agent.name.as_str();
                    if agent_name != task_name {
                        bail!(
                            "Task `{task_name}` was unexpectedly matched by agent `{agent_name}`"
                        );
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                bail!("Task `{task_name}` was not matched by any agent");
            }
        }

        Ok(())
    }
}
