//! Agent definition loading and discovery.
//!
//! Discovers agents from three sources (lowest to highest precedence):
//! 1. CLI tools detected on PATH (`claude`, `codex`, `gemini`)
//! 2. User config `~/.config/stencila/agents/` (`AGENT.md` files)
//! 3. Workspace `.stencila/agents/` (`AGENT.md` files)
//!
//! CLI-detected agents are created in-memory without requiring an `AGENT.md`
//! on disk. Workspace and user agents are parsed from YAML frontmatter +
//! optional Markdown body.

use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Agent, Node, NodeType};

/// Where an agent was discovered from.
///
/// Variant order determines sort priority: workspace agents sort first,
/// then user, then CLI-detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum AgentSource {
    /// `.stencila/agents/` in the workspace
    Workspace,
    /// `~/.config/stencila/agents/` (user-level)
    User,
    /// Auto-detected from a CLI tool found on PATH
    #[cfg_attr(feature = "cli", clap(name = "cli"))]
    CliDetected,
}

impl std::fmt::Display for AgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace => f.write_str("workspace"),
            Self::User => f.write_str("user"),
            Self::CliDetected => f.write_str("cli"),
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

    /// Create an in-memory agent instance (no file on disk).
    fn in_memory(inner: Agent, source: AgentSource) -> Self {
        Self {
            inner,
            path: PathBuf::new(),
            home: PathBuf::new(),
            source: Some(source),
        }
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
    /// Returns `None` if the file has no body after the frontmatter, or if
    /// this is an in-memory agent with no backing file.
    pub async fn instructions(&self) -> Result<Option<String>> {
        if self.path.as_os_str().is_empty() {
            return Ok(None);
        }

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

/// CLI tools that can be auto-detected on PATH and exposed as agents.
const CLI_AGENT_DEFS: &[(&str, &str, &str)] = &[
    (
        "claude",
        "Claude Code CLI with local settings and standard system prompt",
        "claude-cli",
    ),
    (
        "codex",
        "Codex CLI with local settings and standard system prompt",
        "codex-cli",
    ),
    (
        "gemini",
        "Gemini CLI with local settings and standard system prompt",
        "gemini-cli",
    ),
];

/// Build in-memory agent instances for CLI tools found on PATH.
fn detect_cli_agents() -> Vec<AgentInstance> {
    CLI_AGENT_DEFS
        .iter()
        .filter(|(binary, _, _)| crate::cli_providers::is_cli_available(binary))
        .map(|(name, description, provider)| {
            let agent = stencila_schema::Agent {
                name: name.to_string(),
                description: description.to_string(),
                provider: Some(provider.to_string()),
                ..Default::default()
            };
            AgentInstance::in_memory(agent, AgentSource::CliDetected)
        })
        .collect()
}

/// Discover agents from workspace and user config, with workspace taking precedence.
///
/// Agents are discovered from (lowest to highest precedence):
/// 1. CLI tools detected on PATH (`claude`, `codex`, `gemini`)
/// 2. User config `~/.config/stencila/agents/`
/// 3. Workspace `.stencila/agents/` (via closest `.stencila` dir)
///
/// When the same agent name appears in multiple locations, the higher-precedence
/// version wins.
pub async fn discover(cwd: &Path) -> Vec<AgentInstance> {
    let mut by_name: std::collections::HashMap<String, AgentInstance> =
        std::collections::HashMap::new();

    // CLI-detected agents first (lowest precedence)
    for agent in detect_cli_agents() {
        by_name.insert(agent.name.clone(), agent);
    }

    // User agents second (overwrites CLI-detected)
    if let Some(user_dir) = user_agents_dir() {
        for agent in list(&user_dir).await {
            by_name.insert(agent.name.clone(), agent.with_source(AgentSource::User));
        }
    }

    // Workspace agents last (highest precedence, overwrites user and CLI)
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
    agents.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then_with(|| a.name.cmp(&b.name))
    });
    agents
}

/// Find an agent by name across CLI-detected, user config, and workspace agents.
pub async fn get_by_name(cwd: &Path, name: &str) -> Result<AgentInstance> {
    let mut found: Option<AgentInstance> = None;

    // Check CLI-detected agents first (lowest precedence)
    if let Some(agent) = detect_cli_agents().into_iter().find(|a| a.name == name) {
        found = Some(agent);
    }

    // Check user agents (overwrites CLI-detected if found)
    if let Some(user_dir) = user_agents_dir()
        && let Ok(agent) = get(&user_dir, name).await
    {
        found = Some(agent.with_source(AgentSource::User));
    }

    // Check workspace agents (highest precedence, overwrites user if found)
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
        create_workspace_agent(tmp.path(), "shared", "workspace version");

        let agents = discover(tmp.path()).await;

        let shared = agents.iter().find(|a| a.name == "shared").expect("found");
        assert_eq!(shared.source(), Some(AgentSource::Workspace));
    }

    #[tokio::test]
    async fn discover_no_workspace_or_user_agents() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agents = discover(tmp.path()).await;
        // Only CLI-detected agents (if any CLIs are on PATH)
        assert!(
            agents
                .iter()
                .all(|a| a.source() == Some(AgentSource::CliDetected))
        );
    }

    #[tokio::test]
    async fn discover_multiple_agents_sorted() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workspace_agent(tmp.path(), "beta", "second agent");
        create_workspace_agent(tmp.path(), "alpha", "first agent");

        let agents = discover(tmp.path()).await;

        let workspace: Vec<_> = agents
            .iter()
            .filter(|a| a.source() == Some(AgentSource::Workspace))
            .collect();
        assert_eq!(workspace.len(), 2);
        assert_eq!(workspace[0].name, "alpha");
        assert_eq!(workspace[1].name, "beta");
    }

    #[tokio::test]
    async fn discover_workspace_overrides_cli_detected() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Create a workspace agent with the same name as a CLI tool
        create_workspace_agent(tmp.path(), "claude", "custom claude");

        let agents = discover(tmp.path()).await;

        if let Some(claude) = agents.iter().find(|a| a.name == "claude") {
            // Workspace should win over CLI-detected
            assert_eq!(claude.source(), Some(AgentSource::Workspace));
            assert_eq!(claude.description, "custom claude");
        }
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
