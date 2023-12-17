//! Custom Stencila agents specialized for specific tasks
//!
//! An agent is a combination of (a) a model, (b) a default prompt,
//! and (c) a set of default options. This crate defines some specialized
//! agents build on top of lower level, more generalized agents
//! in other crates and prompts defined in the top level `prompts` module.

use std::{
    fs::{read_dir, read_to_string},
    path::PathBuf,
    sync::Arc,
};

use agent::{
    common::{
        async_trait::async_trait,
        chrono::Utc,
        eyre::{bail, eyre, OptionExt, Result},
        itertools::Itertools,
        serde_json, serde_yaml,
    },
    merge::Merge,
    Agent, AgentIO, GenerateContext, GenerateOptions, GenerateDetails,
};
use agent_anthropic::AnthropicAgent;
use agent_ollama::OllamaAgent;
use agent_openai::OpenAIAgent;
use codecs::{EncodeOptions, Format};
use minijinja::{Environment, UndefinedBehavior};
use rust_embed::RustEmbed;

struct CustomAgent {
    /// The name of the agent
    name: String,

    /// Provider name
    provider: String,

    /// The name of the model provider e.g. openai
    model: String,

    /// The system prompt of the agent
    system_prompt: String,

    /// The name of the default prompt for the agent
    user_prompt_template: String,

    /// The set of default options to use for the agent
    #[allow(unused)]
    options: GenerateOptions,

    /// The base agent that this one extends
    base: Arc<dyn Agent>,
}

impl CustomAgent {
    /// Create a new custom agent
    fn new(
        name: &str,
        extends: &str,
        system_prompt: &str,
        user_prompt_template: &str,
        options: GenerateOptions,
    ) -> Result<Self> {
        let (provider, model) = extends
            .split("/")
            .collect_tuple()
            .ok_or_else(|| eyre!("Expected base agent name to have a forward slash"))?;

        let base = match provider {
            "anthropic" => Arc::new(AnthropicAgent::new(model.into())) as Arc<dyn Agent>,
            "ollama" => Arc::new(OllamaAgent::new(model)) as Arc<dyn Agent>,
            "openai" => Arc::new(OpenAIAgent::new(
                model.into(),
                vec![AgentIO::Text],
                vec![AgentIO::Text],
            )) as Arc<dyn Agent>,
            _ => bail!("Unknown provider agent: {provider}"),
        };

        Ok(Self {
            name: name.into(),
            provider: provider.into(),
            model: model.into(),
            system_prompt: system_prompt.into(),
            user_prompt_template: user_prompt_template.into(),
            options,
            base,
        })
    }

    /// Parse Markdown content into a custom agent
    fn parse(content: &str) -> Result<CustomAgent> {
        // Split a string into the three parts of a prompt: YAML header, system prompt and user prompt
        let (header, system_prompt, user_prompt_template) = content
            .splitn(4, "---")
            .map(|part| part.trim().to_string())
            .skip(1)
            .collect_tuple()
            .ok_or_else(|| eyre!("Content does not have at least three --- separators"))?;

        // Initially parse the header into JSON value and extract out name and extends
        let mut header: serde_json::Value = serde_yaml::from_str(&header)?;
        let Some(map) = header.as_object_mut() else {
            bail!("Expected header to be a YAML map")
        };
        let name = map
            .remove("name")
            .and_then(|value| value.as_str().map(String::from))
            .ok_or_eyre("Agent header should include the `name` of agent")?;
        let extends = header
            .get("extends")
            .and_then(|value| value.as_str().map(String::from))
            .ok_or_eyre("Agent header should include the name of the agent it `extends`")?;

        // Now transform the remainder of the header into [`GenerateOptions`]
        let options = serde_json::from_value(header)?;

        Self::new(
            &name,
            &extends,
            &system_prompt,
            &user_prompt_template,
            options,
        )
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
    async fn update_context(
        &self,
        mut context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<GenerateContext> {
        context.system_prompt = Some(self.system_prompt.clone());

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

        context.agent_name = Some(self.name());
        context.provider_name = Some(self.provider());
        context.model_name = Some(self.model());
        context.current_timestamp = Some(Utc::now().to_rfc3339());

        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Chainable);
        let user_prompt = env.render_str(&self.user_prompt_template, &context)?;
        context.user_prompt = Some(user_prompt);

        Ok(context)
    }
}

#[async_trait]
impl Agent for CustomAgent {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn provider(&self) -> String {
        self.provider.clone()
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn supported_inputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    fn supported_outputs(&self) -> &[AgentIO] {
        &[AgentIO::Text]
    }

    async fn text_to_text(
        &self,
        context: GenerateContext,
        options: &GenerateOptions,
    ) -> Result<(String, GenerateDetails)> {
        let options = self.merge_options(options);
        let context = self.update_context(context, &options).await?;

        let (text, mut details) = self.base.text_to_text(context, &options).await?;
        details.agent_chain.insert(0, self.name());

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
pub async fn list() -> Result<Vec<Arc<dyn Agent>>> {
    list_sync()
}

/// Get a list of all available custom agents (sync for easier testing)
fn list_sync() -> Result<Vec<Arc<dyn Agent>>> {
    let agents_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../agents");
    let mut agents = vec![];

    // Add all builtin prompts, erroring if there are any syntax errors in them
    for content in Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| file.data)) {
        let content = String::from_utf8_lossy(&content);
        let agent = CustomAgent::parse(&content)?;
        agents.push(Arc::new(agent) as Arc<dyn Agent>)
    }

    // If in development, also load example agents
    #[cfg(debug_assertions)]
    {
        for entry in read_dir(&agents_dir.join("example"))?.flatten() {
            let content = read_to_string(entry.path())?;
            let agent = CustomAgent::parse(&content)?;
            agents.push(Arc::new(agent) as Arc<dyn Agent>)
        }
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
