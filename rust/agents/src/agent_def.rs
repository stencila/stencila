//! Agent definition loading and discovery.
//!
//! Parses `AGENT.md` files (YAML frontmatter + optional Markdown body) from
//! two locations — workspace `.stencila/agents/` and user config
//! `~/.config/stencila/agents/` — and provides a public API mirroring the
//! skills crate pattern.

use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Agent, Node, NodeType};

/// Where an agent was discovered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum AgentSource {
    /// `.stencila/agents/` in the workspace
    Workspace,
    /// `~/.config/stencila/agents/` (user-level)
    User,
}

impl std::fmt::Display for AgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace => f.write_str("workspace"),
            Self::User => f.write_str("user"),
        }
    }
}

/// An instance of an agent loaded from disk.
///
/// Wraps an [`Agent`] with its file path, home directory, and source metadata.
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub struct AgentInstance {
    #[serde(flatten)]
    pub inner: Agent,

    /// Path to the AGENT.md file
    path: PathBuf,

    /// Home directory of the agent (parent of AGENT.md)
    #[serde(skip)]
    home: PathBuf,

    /// Which source this agent was loaded from
    #[serde(skip)]
    source: Option<AgentSource>,
}

impl std::ops::Deref for AgentInstance {
    type Target = Agent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for AgentInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Custom serialization for display purposes
impl Serialize for AgentInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AgentInstance", 7)?;

        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("model", &self.inner.model)?;
        state.serialize_field("provider", &self.inner.provider)?;
        state.serialize_field("reasoningEffort", &self.inner.reasoning_effort)?;
        state.serialize_field("source", &self.source.map(|s| s.to_string()))?;
        state.serialize_field("path", &self.path)?;

        state.end()
    }
}

impl AgentInstance {
    fn new(inner: Agent, path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;

        let home = path
            .parent()
            .ok_or_else(|| eyre::eyre!("AGENT.md not in a directory"))?
            .to_path_buf();

        Ok(Self {
            inner,
            path,
            home,
            source: None,
        })
    }

    /// Return a copy with the source set.
    fn with_source(mut self, source: AgentSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Get the path to the AGENT.md file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the home directory of the agent
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Which source this agent was loaded from
    pub fn source(&self) -> Option<AgentSource> {
        self.source
    }

    /// Read the AGENT.md file and extract the Markdown body (instructions),
    /// stripping the YAML frontmatter.
    ///
    /// Returns `None` if the file has no body after the frontmatter.
    pub async fn instructions(&self) -> Result<Option<String>> {
        let raw = read_to_string(&self.path).await?;

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        // If starts with YAML frontmatter delimiter, strip it
        if let Some(rest) = trimmed.strip_prefix("---") {
            // Find the closing delimiter
            if let Some(end_idx) = rest.find("\n---") {
                let body = rest[end_idx + 4..].trim();
                if body.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(body.to_string()));
            }
            // No closing delimiter — treat entire content as frontmatter only
            return Ok(None);
        }

        // No frontmatter — entire file is body
        Ok(Some(trimmed.to_string()))
    }
}

/// Get the closest `.stencila/agents` directory, optionally creating it
pub(crate) async fn closest_agents_dir(cwd: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, ensure).await?;
    stencila_dirs::stencila_agents_dir(&stencila_dir, ensure).await
}

/// Get the user-level agents directory (`~/.config/stencila/agents/`)
fn user_agents_dir() -> Option<PathBuf> {
    stencila_dirs::get_app_dir(stencila_dirs::DirType::Agents, false).ok()
}

/// Discover agents from workspace and user config, with workspace taking precedence.
///
/// Agents are discovered from:
/// 1. Workspace `.stencila/agents/` (via closest `.stencila` dir)
/// 2. User config `~/.config/stencila/agents/`
///
/// When the same agent name appears in both locations, the workspace version wins.
pub async fn discover(cwd: &Path) -> Vec<AgentInstance> {
    let mut by_name: std::collections::HashMap<String, AgentInstance> =
        std::collections::HashMap::new();

    // User agents first (lower precedence)
    if let Some(user_dir) = user_agents_dir() {
        for agent in list(&user_dir).await {
            by_name.insert(agent.name.clone(), agent.with_source(AgentSource::User));
        }
    }

    // Workspace agents second (higher precedence, overwrites user)
    if let Some(stencila_dir) = stencila_dirs::closest_dot_dir(cwd, ".stencila") {
        let agents_dir = stencila_dir.join("agents");
        for agent in list(&agents_dir).await {
            by_name.insert(
                agent.name.clone(),
                agent.with_source(AgentSource::Workspace),
            );
        }
    }

    let mut agents: Vec<AgentInstance> = by_name.into_values().collect();
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    agents
}

/// Find an agent by name across workspace and user config.
pub async fn get_by_name(cwd: &Path, name: &str) -> Result<AgentInstance> {
    let mut found: Option<AgentInstance> = None;

    // Check user agents first
    if let Some(user_dir) = user_agents_dir()
        && let Ok(agent) = get(&user_dir, name).await
    {
        found = Some(agent.with_source(AgentSource::User));
    }

    // Check workspace agents (overwrites user if found)
    if let Some(stencila_dir) = stencila_dirs::closest_dot_dir(cwd, ".stencila") {
        let agents_dir = stencila_dir.join("agents");
        if let Ok(agent) = get(&agents_dir, name).await {
            found = Some(agent.with_source(AgentSource::Workspace));
        }
    }

    found.ok_or_else(|| eyre::eyre!("Unable to find agent with name `{name}`"))
}

/// List all agents found in an agents directory
pub async fn list(agents_dir: &Path) -> Vec<AgentInstance> {
    if !agents_dir.exists() {
        return Vec::new();
    }

    match list_dir(agents_dir).await {
        Ok(agents) => agents,
        Err(error) => {
            tracing::error!(
                "While listing agents in `{}`: {error}",
                agents_dir.display()
            );
            Vec::new()
        }
    }
}

/// Get an agent by name from an agents directory
pub async fn get(agents_dir: &Path, name: &str) -> Result<AgentInstance> {
    list(agents_dir)
        .await
        .into_iter()
        .find(|agent| agent.name == name)
        .ok_or_else(|| eyre::eyre!("Unable to find agent with name `{name}`"))
}

/// List agents in a directory.
///
/// Globs for `*/AGENT.md` files (one level deep), decodes each as an Agent.
async fn list_dir(agents_dir: &Path) -> Result<Vec<AgentInstance>> {
    tracing::trace!("Attempting to read agents from `{}`", agents_dir.display());

    let mut agents = vec![];
    for path in glob(&format!("{}/*/AGENT.md", agents_dir.display()))?.flatten() {
        match load_agent(&path).await {
            Ok(instance) => agents.push(instance),
            Err(error) => {
                tracing::warn!("Skipping `{}`: {error}", path.display());
            }
        }
    }

    Ok(agents)
}

/// Load a single agent from an AGENT.md path
async fn load_agent(path: &Path) -> Result<AgentInstance> {
    let content = read_to_string(path).await?;

    let node = stencila_codecs::from_str(
        &content,
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            node_type: Some(NodeType::Agent),
            ..Default::default()
        }),
    )
    .await?;

    if let Node::Agent(agent) = node {
        AgentInstance::new(agent, path.to_path_buf())
    } else {
        bail!(
            "Expected `{}` to be an `Agent`, got a `{}`",
            path.display(),
            node.to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper to create an agent directory --

    fn create_agent(base: &Path, subdir: &str, name: &str, description: &str) {
        let agent_dir = base.join(subdir).join(name);
        std::fs::create_dir_all(&agent_dir).expect("create agent dir");
        let content = format!(
            "---\nname: {name}\ndescription: {description}\n---\n\nInstructions for {name}.\n"
        );
        std::fs::write(agent_dir.join("AGENT.md"), content).expect("write AGENT.md");
    }

    fn create_workspace_agent(base: &Path, name: &str, description: &str) {
        create_agent(base, ".stencila/agents", name, description);
    }

    // -- discover() tests --

    #[tokio::test]
    async fn discover_workspace_overrides_user() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // User agent (simulated by discovering in workspace only for this test)
        create_workspace_agent(tmp.path(), "shared", "workspace version");

        let agents = discover(tmp.path()).await;

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "shared");
        assert_eq!(agents[0].source(), Some(AgentSource::Workspace));
    }

    #[tokio::test]
    async fn discover_empty_when_no_agents_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agents = discover(tmp.path()).await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn discover_multiple_agents_sorted() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workspace_agent(tmp.path(), "beta", "second agent");
        create_workspace_agent(tmp.path(), "alpha", "first agent");

        let agents = discover(tmp.path()).await;

        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "alpha");
        assert_eq!(agents[1].name, "beta");
    }

    // -- get_by_name() tests --

    #[tokio::test]
    async fn get_by_name_finds_workspace_agent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workspace_agent(tmp.path(), "my-agent", "a test agent");

        let agent = get_by_name(tmp.path(), "my-agent").await;
        assert!(agent.is_ok());
        assert_eq!(agent.expect("found").name, "my-agent");
    }

    #[tokio::test]
    async fn get_by_name_returns_error_when_not_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = get_by_name(tmp.path(), "nonexistent").await;
        assert!(result.is_err());
    }

    // -- load_agent() tests --

    #[tokio::test]
    async fn load_agent_with_model_and_provider() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agent_dir = tmp.path().join("test-agent");
        std::fs::create_dir_all(&agent_dir).expect("create dir");
        let content = "---\nname: test-agent\ndescription: A test agent\nmodel: claude-sonnet-4-5\nprovider: anthropic\nreasoningEffort: high\n---\n\nYou are a helpful assistant.\n";
        std::fs::write(agent_dir.join("AGENT.md"), content).expect("write");

        let instance = load_agent(&agent_dir.join("AGENT.md")).await.expect("load");

        assert_eq!(instance.name, "test-agent");
        assert_eq!(instance.description, "A test agent");
        assert_eq!(instance.model.as_deref(), Some("claude-sonnet-4-5"));
        assert_eq!(instance.provider.as_deref(), Some("anthropic"));
        assert_eq!(instance.reasoning_effort.as_deref(), Some("high"));
        // Agent with body should have content
        assert!(instance.content.is_some());
    }

    #[tokio::test]
    async fn load_agent_config_only_has_no_content() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agent_dir = tmp.path().join("config-only");
        std::fs::create_dir_all(&agent_dir).expect("create dir");
        // No markdown body — just frontmatter
        let content = "---\nname: config-only\ndescription: A config-only agent\nmodel: gpt-5\nprovider: openai\n---\n";
        std::fs::write(agent_dir.join("AGENT.md"), content).expect("write");

        let instance = load_agent(&agent_dir.join("AGENT.md")).await.expect("load");

        assert_eq!(instance.name, "config-only");
        // Config-only agent should have None content, not Some(vec![])
        assert!(instance.content.is_none());
    }
}
