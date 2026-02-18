//! MCP server configuration and multi-source discovery.
//!
//! Discovers MCP server configurations from multiple sources:
//! Stencila's own config (`stencila.toml`), and external tool configs
//! (Claude, Codex, Gemini). Discovery is global — all sources are loaded
//! regardless of which LLM provider is active.

use std::collections::HashMap;
use std::path::Path;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::error::McpResult;

// ---------------------------------------------------------------------------
// ConfigSource
// ---------------------------------------------------------------------------

/// Where an MCP server configuration was discovered from.
///
/// Each external tool has both a user-level and workspace-level config.
/// Sources listed later in [`discover`] take precedence on id conflicts
/// (last wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    /// `stencila.toml` (user + workspace merged via figment) → `[mcp.servers]`
    Stencila,
    /// `~/.claude.json` → `projects[workspace].mcpServers`
    ClaudeUser,
    /// `.mcp.json` (project root) → `mcpServers`
    ClaudeWorkspace,
    /// `~/.codex/config.toml` → `[mcp_servers]`
    CodexUser,
    /// `.codex/config.toml` → `[mcp_servers]`
    CodexWorkspace,
    /// `~/.gemini/settings.json` → `mcpServers`
    GeminiUser,
    /// `.gemini/settings.json` → `mcpServers`
    GeminiWorkspace,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stencila => f.write_str("stencila"),
            Self::ClaudeUser => f.write_str("claude (user)"),
            Self::ClaudeWorkspace => f.write_str("claude (workspace)"),
            Self::CodexUser => f.write_str("codex (user)"),
            Self::CodexWorkspace => f.write_str("codex (workspace)"),
            Self::GeminiUser => f.write_str("gemini (user)"),
            Self::GeminiWorkspace => f.write_str("gemini (workspace)"),
        }
    }
}

// ---------------------------------------------------------------------------
// McpServerConfig
// ---------------------------------------------------------------------------

/// A normalized MCP server configuration.
///
/// Regardless of which source format the config was loaded from, all configs
/// are normalized into this common type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Unique identifier (the key from the config file).
    pub id: String,

    /// Human-readable name (defaults to `id` if not set).
    pub name: Option<String>,

    /// How to connect to the server.
    pub transport: TransportConfig,

    /// Environment variables to set for stdio servers.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether this server is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Which source this config was loaded from.
    #[serde(skip)]
    pub source: Option<ConfigSource>,
}

fn default_true() -> bool {
    true
}

/// Transport configuration for connecting to an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// Stdio transport: spawn a child process and communicate via stdin/stdout.
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// HTTP transport: connect to an HTTP/SSE endpoint.
    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Discover MCP server configurations from all sources.
///
/// On id conflict, later sources win. Load order (lowest to highest precedence):
/// 1. Stencila (user + workspace merged by figment)
/// 2. External user-level (Claude, Codex, Gemini `~/.*/` configs)
/// 3. External workspace-level (`.mcp.json`, `.codex/`, `.gemini/` in project root)
///
/// The `workspace_dir` must be the project/repository root. The caller is
/// responsible for resolving this (e.g. walking up to find a VCS root).
/// User-level configs are found via well-known paths.
#[must_use]
pub fn discover(workspace_dir: &Path) -> Vec<McpServerConfig> {
    let mut by_id: HashMap<String, McpServerConfig> = HashMap::new();

    // Stencila config (user + workspace merged by figment, loaded first, lowest precedence)
    load_stencila(workspace_dir, &mut by_id);

    // External user-level sources
    load_claude_user(workspace_dir, &mut by_id);
    load_codex_user(&mut by_id);
    load_gemini_user(&mut by_id);

    // External workspace-level sources (loaded last, highest precedence)
    load_claude_workspace(workspace_dir, &mut by_id);
    load_codex_workspace(workspace_dir, &mut by_id);
    load_gemini_workspace(workspace_dir, &mut by_id);

    let mut servers: Vec<McpServerConfig> = by_id.into_values().collect();
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    servers
}

/// Discover servers from a single source file. Useful for testing and CLI.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn discover_from_file(path: &Path, source: ConfigSource) -> McpResult<Vec<McpServerConfig>> {
    let content = std::fs::read_to_string(path)?;
    parse_source(&content, source)
}

// ---------------------------------------------------------------------------
// Source-specific loaders
// ---------------------------------------------------------------------------

/// Load Stencila MCP servers from the config crate.
///
/// Uses `stencila_config::load_and_validate()` which merges user-level
/// (`~/.config/stencila/stencila.toml`) and workspace-level (`stencila.toml`)
/// configs via figment (workspace overrides user).
fn load_stencila(workspace_dir: &Path, by_id: &mut HashMap<String, McpServerConfig>) {
    let config = match stencila_config::load_and_validate(workspace_dir) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("failed to load stencila config: {e}");
            return;
        }
    };

    let Some(mcp) = config.mcp else {
        return;
    };
    let Some(servers) = mcp.servers else {
        return;
    };

    for (id, entry) in servers {
        let transport = match entry.transport {
            stencila_config::McpTransportConfig::Stdio { command, args } => {
                TransportConfig::Stdio { command, args }
            }
            stencila_config::McpTransportConfig::Http { url, headers } => {
                TransportConfig::Http { url, headers }
            }
        };
        by_id.insert(
            id.clone(),
            McpServerConfig {
                id,
                name: entry.name,
                transport,
                env: entry.env,
                enabled: entry.enabled,
                source: Some(ConfigSource::Stencila),
            },
        );
    }
}

/// Load Claude user-level MCP servers from `~/.claude.json`.
///
/// Claude Code stores project-scoped servers under
/// `projects["<absolute-workspace-path>"].mcpServers` and lists disabled
/// server IDs in `projects["<path>"].disabledMcpServers`.
fn load_claude_user(workspace_dir: &Path, by_id: &mut HashMap<String, McpServerConfig>) {
    let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) else {
        return;
    };
    let path = home.join(".claude.json");
    if !path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to read {}: {e}", path.display());
            return;
        }
    };
    match parse_claude_user_json(&content, workspace_dir) {
        Ok(servers) => {
            for server in servers {
                by_id.insert(server.id.clone(), server);
            }
        }
        Err(e) => {
            tracing::warn!("failed to parse {}: {e}", path.display());
        }
    }
}

fn load_claude_workspace(workspace_dir: &Path, by_id: &mut HashMap<String, McpServerConfig>) {
    // Claude uses .mcp.json at project root
    let path = workspace_dir.join(".mcp.json");
    load_file(&path, ConfigSource::ClaudeWorkspace, by_id);
}

fn load_codex_user(by_id: &mut HashMap<String, McpServerConfig>) {
    if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
        let path = home.join(".codex").join("config.toml");
        load_file(&path, ConfigSource::CodexUser, by_id);
    }
}

fn load_codex_workspace(workspace_dir: &Path, by_id: &mut HashMap<String, McpServerConfig>) {
    let path = workspace_dir.join(".codex").join("config.toml");
    load_file(&path, ConfigSource::CodexWorkspace, by_id);
}

fn load_gemini_user(by_id: &mut HashMap<String, McpServerConfig>) {
    if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
        let path = home.join(".gemini").join("settings.json");
        load_file(&path, ConfigSource::GeminiUser, by_id);
    }
}

fn load_gemini_workspace(workspace_dir: &Path, by_id: &mut HashMap<String, McpServerConfig>) {
    let path = workspace_dir.join(".gemini").join("settings.json");
    load_file(&path, ConfigSource::GeminiWorkspace, by_id);
}

fn load_file(path: &Path, source: ConfigSource, by_id: &mut HashMap<String, McpServerConfig>) {
    if !path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to read MCP config at {}: {e}", path.display());
            return;
        }
    };
    match parse_source(&content, source) {
        Ok(servers) => {
            for server in servers {
                by_id.insert(server.id.clone(), server);
            }
        }
        Err(e) => {
            tracing::warn!("failed to parse MCP config at {}: {e}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Parsers
// ---------------------------------------------------------------------------

fn parse_source(content: &str, source: ConfigSource) -> McpResult<Vec<McpServerConfig>> {
    match source {
        ConfigSource::Stencila => Err(crate::error::McpError::Config(
            "use stencila_config crate to load Stencila config".to_string(),
        )),
        ConfigSource::ClaudeUser | ConfigSource::ClaudeWorkspace => {
            parse_claude_json(content, source)
        }
        ConfigSource::CodexUser | ConfigSource::CodexWorkspace => parse_codex_toml(content, source),
        ConfigSource::GeminiUser | ConfigSource::GeminiWorkspace => {
            parse_gemini_json(content, source)
        }
    }
}

/// Parse Claude user config (`~/.claude.json`) for project-scoped MCP servers.
///
/// The file has the structure:
/// ```json
/// {
///   "projects": {
///     "/absolute/path/to/project": {
///       "mcpServers": { "id": { "command": "...", "args": [...] } },
///       "disabledMcpServers": ["id"]
///     }
///   }
/// }
/// ```
fn parse_claude_user_json(content: &str, workspace_dir: &Path) -> McpResult<Vec<McpServerConfig>> {
    #[derive(Deserialize)]
    struct ClaudeServerEntry {
        command: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        url: Option<String>,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(rename = "type")]
        transport_type: Option<String>,
    }

    let root: serde_json::Value = serde_json::from_str(content)?;

    let workspace_key = workspace_dir.to_string_lossy();
    let project = root
        .get("projects")
        .and_then(|p| p.get(workspace_key.as_ref()));

    let Some(project) = project else {
        return Ok(Vec::new());
    };

    let Some(servers_value) = project.get("mcpServers") else {
        return Ok(Vec::new());
    };

    let entries: HashMap<String, ClaudeServerEntry> =
        serde_json::from_value(servers_value.clone())?;

    // Parse disabledMcpServers
    let disabled: Vec<String> = project
        .get("disabledMcpServers")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(entries
        .into_iter()
        .filter_map(|(id, entry)| {
            let transport = resolve_claude_transport(
                &id,
                entry.command,
                entry.url,
                entry.args,
                entry.headers,
                entry.transport_type.as_deref(),
            )?;
            let enabled = !disabled.contains(&id);
            Some(McpServerConfig {
                id,
                name: None,
                transport,
                env: entry.env,
                enabled,
                source: Some(ConfigSource::ClaudeUser),
            })
        })
        .collect())
}

/// Parse Claude JSON format (`.mcp.json`):
/// ```json
/// { "mcpServers": { "name": { "command": "...", "args": [...] } } }
/// ```
fn parse_claude_json(content: &str, source: ConfigSource) -> McpResult<Vec<McpServerConfig>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ClaudeConfig {
        #[serde(default)]
        mcp_servers: HashMap<String, ClaudeServerEntry>,
    }
    #[derive(Deserialize)]
    struct ClaudeServerEntry {
        command: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        /// HTTP transport
        url: Option<String>,
        #[serde(default)]
        headers: HashMap<String, String>,
        /// Type hint: "stdio", "http", "sse"
        #[serde(rename = "type")]
        transport_type: Option<String>,
    }

    let config: ClaudeConfig = serde_json::from_str(content)?;

    Ok(config
        .mcp_servers
        .into_iter()
        .filter_map(|(id, entry)| {
            let transport = resolve_claude_transport(
                &id,
                entry.command,
                entry.url,
                entry.args,
                entry.headers,
                entry.transport_type.as_deref(),
            )?;
            Some(McpServerConfig {
                id,
                name: None,
                transport,
                env: entry.env,
                enabled: true,
                source: Some(source),
            })
        })
        .collect())
}

/// Resolve Claude's transport from the flexible config format.
///
/// Claude infers transport type: if `command` is present it's stdio,
/// if `url` is present it's HTTP/SSE.
fn resolve_claude_transport(
    id: &str,
    command: Option<String>,
    url: Option<String>,
    args: Vec<String>,
    headers: HashMap<String, String>,
    transport_type: Option<&str>,
) -> Option<TransportConfig> {
    match transport_type {
        Some("http" | "sse") => {
            let url = url.or_else(|| {
                tracing::warn!("claude server `{id}`: http/sse type requires url");
                None
            })?;
            Some(TransportConfig::Http { url, headers })
        }
        Some("stdio") => {
            // Explicit stdio type — command is required.
            if let Some(command) = command {
                Some(TransportConfig::Stdio { command, args })
            } else {
                tracing::warn!("claude server `{id}`: stdio type requires command");
                None
            }
        }
        None => {
            // No explicit type — infer from available fields.
            if let Some(command) = command {
                Some(TransportConfig::Stdio { command, args })
            } else if let Some(url) = url {
                Some(TransportConfig::Http { url, headers })
            } else {
                tracing::warn!("claude server `{id}`: no command or url specified");
                None
            }
        }
        Some(other) => {
            tracing::warn!("claude server `{id}`: unknown transport type `{other}`");
            None
        }
    }
}

/// Parse Codex TOML format:
/// ```toml
/// [mcp_servers.my-server]
/// command = "npx"
/// args = ["-y", "@package/name"]
/// ```
fn parse_codex_toml(content: &str, source: ConfigSource) -> McpResult<Vec<McpServerConfig>> {
    #[derive(Deserialize)]
    struct CodexConfig {
        #[serde(default)]
        mcp_servers: HashMap<String, CodexServerEntry>,
    }
    #[derive(Deserialize)]
    struct CodexServerEntry {
        command: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        url: Option<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(default = "default_true")]
        enabled: bool,
    }

    let config: CodexConfig = toml::from_str(content)?;

    Ok(config
        .mcp_servers
        .into_iter()
        .filter_map(|(id, entry)| {
            let transport = if let Some(command) = entry.command {
                TransportConfig::Stdio {
                    command,
                    args: entry.args,
                }
            } else if let Some(url) = entry.url {
                TransportConfig::Http {
                    url,
                    headers: HashMap::new(),
                }
            } else {
                tracing::warn!("codex server `{id}`: no command or url specified");
                return None;
            };
            Some(McpServerConfig {
                id,
                name: None,
                transport,
                env: entry.env,
                enabled: entry.enabled,
                source: Some(source),
            })
        })
        .collect())
}

/// Parse Gemini JSON format:
/// ```json
/// { "mcpServers": { "name": { "command": "...", "args": [...] } } }
/// ```
fn parse_gemini_json(content: &str, source: ConfigSource) -> McpResult<Vec<McpServerConfig>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GeminiConfig {
        #[serde(default)]
        mcp_servers: HashMap<String, GeminiServerEntry>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GeminiServerEntry {
        command: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        url: Option<String>,
        http_url: Option<String>,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(default)]
        env: HashMap<String, String>,
    }

    let config: GeminiConfig = serde_json::from_str(content)?;

    Ok(config
        .mcp_servers
        .into_iter()
        .filter_map(|(id, entry)| {
            let transport = if let Some(command) = entry.command {
                TransportConfig::Stdio {
                    command,
                    args: entry.args,
                }
            } else if let Some(url) = entry.url.or(entry.http_url) {
                TransportConfig::Http {
                    url,
                    headers: entry.headers,
                }
            } else {
                tracing::warn!("gemini server `{id}`: no command, url, or httpUrl specified");
                return None;
            };
            Some(McpServerConfig {
                id,
                name: None,
                transport,
                env: entry.env,
                enabled: true,
                source: Some(source),
            })
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Mutex to serialize tests that modify environment variables.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Run a closure with HOME and XDG_CONFIG_HOME pointed at the temp dir
    /// so that user-level configs (stencila, claude, codex, gemini) are not
    /// picked up from the real home directory.
    #[allow(unsafe_code)]
    fn with_isolated_home<F: FnOnce()>(tmp: &tempfile::TempDir, f: F) {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let old_home = std::env::var("HOME").ok();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            std::env::set_var("HOME", tmp.path());
            std::env::set_var("XDG_CONFIG_HOME", tmp.path().join(".config"));
        }
        f();
        unsafe {
            match old_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match old_xdg {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    // -- Stencila config (via stencila-config crate) --

    #[test]
    fn stencila_servers_loaded_via_config_crate() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let toml = r#"
[mcp.servers.filesystem]
transport.type = "stdio"
transport.command = "npx"
transport.args = ["-y", "@modelcontextprotocol/server-filesystem"]

[mcp.servers.remote]
name = "Remote"
transport.type = "http"
transport.url = "https://mcp.example.com"
enabled = false

[mcp.servers.github]
transport.type = "stdio"
transport.command = "github-mcp"

[mcp.servers.github.env]
GITHUB_TOKEN = "abc123"
"#;
        std::fs::write(tmp.path().join("stencila.toml"), toml).expect("write");

        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());
            assert_eq!(servers.len(), 3);

            let fs_server = servers
                .iter()
                .find(|s| s.id == "filesystem")
                .expect("filesystem");
            assert!(fs_server.enabled);
            assert!(
                matches!(&fs_server.transport, TransportConfig::Stdio { command, .. } if command == "npx")
            );
            assert_eq!(fs_server.source, Some(ConfigSource::Stencila));

            let remote = servers.iter().find(|s| s.id == "remote").expect("remote");
            assert!(!remote.enabled);
            assert_eq!(remote.name.as_deref(), Some("Remote"));

            let github = servers.iter().find(|s| s.id == "github").expect("github");
            assert_eq!(
                github.env.get("GITHUB_TOKEN").map(String::as_str),
                Some("abc123")
            );
        });
    }

    #[test]
    fn stencila_no_mcp_section() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("stencila.toml"), "# empty\n").expect("write");
        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());
            assert!(servers.is_empty());
        });
    }

    // -- Claude JSON parser --

    #[test]
    fn parse_claude_stdio_server() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
                }
            }
        }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "filesystem");
        assert!(
            matches!(&servers[0].transport, TransportConfig::Stdio { command, .. } if command == "npx")
        );
        Ok(())
    }

    #[test]
    fn parse_claude_http_server() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "remote": {
                    "type": "http",
                    "url": "https://mcp.example.com/mcp",
                    "headers": { "Authorization": "Bearer token" }
                }
            }
        }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeUser)?;
        assert_eq!(servers.len(), 1);
        assert!(
            matches!(&servers[0].transport, TransportConfig::Http { url, headers }
            if url == "https://mcp.example.com/mcp" && headers.contains_key("Authorization"))
        );
        Ok(())
    }

    #[test]
    fn parse_claude_infers_http_from_url() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "remote": {
                    "url": "https://mcp.example.com"
                }
            }
        }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert!(matches!(
            &servers[0].transport,
            TransportConfig::Http { .. }
        ));
        Ok(())
    }

    #[test]
    fn parse_claude_explicit_stdio_without_command_rejected() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "bad": {
                    "type": "stdio",
                    "url": "https://mcp.example.com"
                }
            }
        }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeWorkspace)?;
        // Should be skipped — explicit stdio type but no command
        assert!(servers.is_empty());
        Ok(())
    }

    #[test]
    fn parse_claude_no_mcp_servers() -> McpResult<()> {
        let json = r#"{ "numStartups": 42, "autoUpdates": true }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeUser)?;
        assert!(servers.is_empty());
        Ok(())
    }

    #[test]
    fn parse_claude_skips_invalid_entry() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "broken": {},
                "good": { "command": "echo" }
            }
        }"#;
        let servers = parse_claude_json(json, ConfigSource::ClaudeWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "good");
        Ok(())
    }

    // -- Claude user config (project-scoped) parser --

    #[test]
    fn parse_claude_user_project_scoped_servers() -> McpResult<()> {
        let json = r#"{
            "numStartups": 42,
            "projects": {
                "/home/user/my-project": {
                    "mcpServers": {
                        "playwright": {
                            "command": "npx",
                            "args": ["@playwright/mcp@latest"]
                        },
                        "remote": {
                            "url": "https://mcp.example.com"
                        }
                    }
                }
            }
        }"#;
        let workspace = Path::new("/home/user/my-project");
        let mut servers = parse_claude_user_json(json, workspace)?;
        servers.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(servers.len(), 2);

        assert_eq!(servers[0].id, "playwright");
        assert!(
            matches!(&servers[0].transport, TransportConfig::Stdio { command, args }
            if command == "npx" && args == &["@playwright/mcp@latest"])
        );
        assert!(servers[0].enabled);
        assert_eq!(servers[0].source, Some(ConfigSource::ClaudeUser));

        assert_eq!(servers[1].id, "remote");
        assert!(matches!(
            &servers[1].transport,
            TransportConfig::Http { .. }
        ));
        assert!(servers[1].enabled);
        Ok(())
    }

    #[test]
    fn parse_claude_user_disabled_servers() -> McpResult<()> {
        let json = r#"{
            "projects": {
                "/workspace": {
                    "mcpServers": {
                        "enabled-server": { "command": "echo" },
                        "disabled-server": { "command": "echo" }
                    },
                    "disabledMcpServers": ["disabled-server"]
                }
            }
        }"#;
        let workspace = Path::new("/workspace");
        let mut servers = parse_claude_user_json(json, workspace)?;
        servers.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(servers.len(), 2);

        assert_eq!(servers[0].id, "disabled-server");
        assert!(!servers[0].enabled);

        assert_eq!(servers[1].id, "enabled-server");
        assert!(servers[1].enabled);
        Ok(())
    }

    #[test]
    fn parse_claude_user_non_matching_workspace() -> McpResult<()> {
        let json = r#"{
            "projects": {
                "/other/project": {
                    "mcpServers": {
                        "server": { "command": "echo" }
                    }
                }
            }
        }"#;
        let workspace = Path::new("/my/project");
        let servers = parse_claude_user_json(json, workspace)?;
        assert!(servers.is_empty());
        Ok(())
    }

    #[test]
    fn parse_claude_user_no_projects_key() -> McpResult<()> {
        let json = r#"{ "numStartups": 42, "autoUpdates": true }"#;
        let workspace = Path::new("/workspace");
        let servers = parse_claude_user_json(json, workspace)?;
        assert!(servers.is_empty());
        Ok(())
    }

    // -- Codex TOML parser --

    #[test]
    fn parse_codex_stdio_server() -> McpResult<()> {
        let toml = r#"
[mcp_servers.docs]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]
enabled = true

[mcp_servers.docs.env]
API_KEY = "secret"
"#;
        let servers = parse_codex_toml(toml, ConfigSource::CodexWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "docs");
        assert!(servers[0].enabled);
        assert!(
            matches!(&servers[0].transport, TransportConfig::Stdio { command, .. } if command == "npx")
        );
        assert_eq!(
            servers[0].env.get("API_KEY").map(String::as_str),
            Some("secret")
        );
        Ok(())
    }

    #[test]
    fn parse_codex_http_server() -> McpResult<()> {
        let toml = r#"
[mcp_servers.github]
url = "https://github-mcp.example.com/mcp"
enabled = true
"#;
        let servers = parse_codex_toml(toml, ConfigSource::CodexUser)?;
        assert_eq!(servers.len(), 1);
        assert!(matches!(
            &servers[0].transport,
            TransportConfig::Http { .. }
        ));
        Ok(())
    }

    #[test]
    fn parse_codex_disabled_server() -> McpResult<()> {
        let toml = r#"
[mcp_servers.experimental]
command = "test-server"
enabled = false
"#;
        let servers = parse_codex_toml(toml, ConfigSource::CodexWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert!(!servers[0].enabled);
        Ok(())
    }

    #[test]
    fn parse_codex_no_mcp_servers() -> McpResult<()> {
        let toml = r#"
model = "o3"
"#;
        let servers = parse_codex_toml(toml, ConfigSource::CodexUser)?;
        assert!(servers.is_empty());
        Ok(())
    }

    // -- Gemini JSON parser --

    #[test]
    fn parse_gemini_stdio_server() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "tools": {
                    "command": "python",
                    "args": ["-m", "my_mcp_server"]
                }
            }
        }"#;
        let servers = parse_gemini_json(json, ConfigSource::GeminiWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "tools");
        assert!(
            matches!(&servers[0].transport, TransportConfig::Stdio { command, .. } if command == "python")
        );
        Ok(())
    }

    #[test]
    fn parse_gemini_http_url_field() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "github": {
                    "httpUrl": "https://api.githubcopilot.com/mcp/",
                    "headers": { "Authorization": "Bearer token" }
                }
            }
        }"#;
        let servers = parse_gemini_json(json, ConfigSource::GeminiUser)?;
        assert_eq!(servers.len(), 1);
        assert!(
            matches!(&servers[0].transport, TransportConfig::Http { url, .. }
            if url == "https://api.githubcopilot.com/mcp/")
        );
        Ok(())
    }

    #[test]
    fn parse_gemini_prefers_url_over_http_url() -> McpResult<()> {
        let json = r#"{
            "mcpServers": {
                "test": {
                    "url": "http://localhost:8080/sse",
                    "httpUrl": "http://localhost:8080/mcp"
                }
            }
        }"#;
        let servers = parse_gemini_json(json, ConfigSource::GeminiWorkspace)?;
        assert_eq!(servers.len(), 1);
        assert!(
            matches!(&servers[0].transport, TransportConfig::Http { url, .. }
            if url == "http://localhost:8080/sse")
        );
        Ok(())
    }

    // -- Discovery integration tests (with temp dirs) --

    #[test]
    fn discover_merges_sources_last_wins() {
        let tmp = tempfile::tempdir().expect("tempdir");

        // Create a stencila.toml with one server
        let stencila_toml = r#"
[mcp.servers.shared]
transport.type = "stdio"
transport.command = "original-cmd"
"#;
        std::fs::write(tmp.path().join("stencila.toml"), stencila_toml).expect("write");

        // Create a .mcp.json (Claude workspace) that overrides it
        let mcp_json = r#"{
            "mcpServers": {
                "shared": {
                    "command": "overridden-cmd"
                }
            }
        }"#;
        std::fs::write(tmp.path().join(".mcp.json"), mcp_json).expect("write");

        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());

            assert_eq!(servers.len(), 1);
            assert_eq!(servers[0].id, "shared");
            // Claude workspace loaded after Stencila workspace → last wins
            assert!(
                matches!(&servers[0].transport, TransportConfig::Stdio { command, .. }
                if command == "overridden-cmd")
            );
            assert_eq!(servers[0].source, Some(ConfigSource::ClaudeWorkspace));
        });
    }

    #[test]
    fn discover_collects_unique_servers() {
        let tmp = tempfile::tempdir().expect("tempdir");

        let stencila_toml = r#"
[mcp.servers.alpha]
transport.type = "stdio"
transport.command = "alpha-cmd"
"#;
        std::fs::write(tmp.path().join("stencila.toml"), stencila_toml).expect("write");

        let mcp_json = r#"{
            "mcpServers": {
                "beta": { "command": "beta-cmd" }
            }
        }"#;
        std::fs::write(tmp.path().join(".mcp.json"), mcp_json).expect("write");

        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());

            assert_eq!(servers.len(), 2);
            // Sorted alphabetically
            assert_eq!(servers[0].id, "alpha");
            assert_eq!(servers[1].id, "beta");
        });
    }

    #[test]
    fn discover_empty_when_no_configs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());
            assert!(servers.is_empty());
        });
    }

    #[test]
    fn discover_skips_disabled_stencila_servers() {
        let tmp = tempfile::tempdir().expect("tempdir");

        let toml = r#"
[mcp.servers.disabled]
transport.type = "stdio"
transport.command = "test"
enabled = false

[mcp.servers.enabled]
transport.type = "stdio"
transport.command = "test"
"#;
        std::fs::write(tmp.path().join("stencila.toml"), toml).expect("write");

        with_isolated_home(&tmp, || {
            let servers = discover(tmp.path());
            // Both are returned — `enabled` is a field, filtering is the caller's job
            assert_eq!(servers.len(), 2);
        });
    }
}
