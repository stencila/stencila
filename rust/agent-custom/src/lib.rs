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
    Agent, AgentIO, GenerateDetails, GenerateOptions, GenerateTask, InstructionType,
};
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

    /// The type of instruction the agent executes
    instruction_type: Option<InstructionType>,

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<String>>,

    /// The preference rank of the custom agent
    ///
    /// Defaults to 50 so that custom agents are by
    /// default preferred over generic agents
    preference_rank: Option<u8>,

    /// Default generate options
    #[serde(flatten)]
    options: GenerateOptions
}

/// A custom agent
struct CustomAgent {
    /// The name of the agent
    name: String,

    /// The names of the agents this agent will delegate
    /// to in descending order of preference
    delegates: Vec<String>,

    /// The type of instruction the agent executes
    instruction_type: Option<InstructionType>,

    /// Regexes to match in the instruction text
    instruction_regexes: Option<Vec<Regex>>,

    /// The preference rank of the custom agent
    preference_rank: u8,

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
        let regexes = match header.instruction_regexes {
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
            instruction_type: header.instruction_type,
            instruction_regexes: regexes,
            preference_rank: header.preference_rank.unwrap_or(50),
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

    /// Get an agent to delegate a task to
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
            if !regexes
                .iter()
                .any(|regex| regex.is_match(task.instruction.text()))
            {
                return false;
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

    async fn text_to_text(
        &self,
        task: GenerateTask,
        options: &GenerateOptions,
    ) -> Result<(String, GenerateDetails)> {
        let options = self.merge_options(options);
        let task = self.update_task(task, &options).await?;

        let agent = self.delegate().await?;

        let (text, mut details) = agent.text_to_text(task, &options).await?;
        details.agents.insert(0, self.name());

        Ok((text, details))
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
    list_sync()
}

/// Get a list of all available custom agents
///
/// Sorts in descending order of delegation rank.
fn list_sync() -> Result<Vec<Arc<dyn Agent>>> {
    let mut agents = vec![];

    // Add all builtin agents
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
    use super::*;

    #[test]
    fn builtin_and_example_agents_can_be_parsed() -> Result<()> {
        list_sync()?;

        Ok(())
    }
}
