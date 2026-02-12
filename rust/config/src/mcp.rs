use std::collections::HashMap;

use eyre::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// MCP (Model Context Protocol) server configuration.
///
/// Defines MCP servers that agents can connect to for tool access.
///
/// ```toml
/// [mcp.servers.filesystem]
/// transport.type = "stdio"
/// transport.command = "npx"
/// transport.args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
///
/// [mcp.servers.remote-api]
/// transport.type = "http"
/// transport.url = "https://api.example.com/mcp"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct McpConfig {
    /// MCP server definitions, keyed by server identifier.
    pub servers: Option<HashMap<String, McpServerEntry>>,
}

/// Configuration for a single MCP server.
///
/// ```toml
/// [mcp.servers.my-server]
/// transport.type = "stdio"
/// transport.command = "npx"
/// transport.args = ["-y", "@package/name"]
/// env = { API_KEY = "..." }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct McpServerEntry {
    /// Human-readable name for this server.
    ///
    /// Defaults to the server identifier (the TOML key) if not set.
    pub name: Option<String>,

    /// How to connect to the server.
    pub transport: McpTransportConfig,

    /// Environment variables to set for stdio servers.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether this server is enabled.
    ///
    /// Set to `false` to temporarily disable a server without removing
    /// its configuration.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Transport configuration for connecting to an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransportConfig {
    /// Stdio transport: spawn a child process and communicate via stdin/stdout.
    Stdio {
        /// The command to run (e.g. `"npx"`, `"python3"`).
        command: String,
        /// Command-line arguments.
        #[serde(default)]
        args: Vec<String>,
    },
    /// HTTP transport: connect to an HTTP/SSE endpoint.
    Http {
        /// The server URL.
        url: String,
        /// Additional HTTP headers.
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

impl McpConfig {
    /// Validate the MCP configuration.
    pub fn validate(&self) -> Result<()> {
        // Currently no validation beyond serde (deny_unknown_fields handles
        // unknown keys, required fields are enforced by the type system).
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_server_roundtrip() -> Result<(), toml::de::Error> {
        let toml = r#"
[servers.filesystem]
transport.type = "stdio"
transport.command = "npx"
transport.args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
"#;
        let config: McpConfig = toml::from_str(toml)?;
        let servers = config.servers.expect("should have servers");
        let server = &servers["filesystem"];
        assert!(matches!(&server.transport, McpTransportConfig::Stdio { command, .. } if command == "npx"));
        assert!(server.enabled);
        assert!(server.name.is_none());
        Ok(())
    }

    #[test]
    fn http_server_roundtrip() -> Result<(), toml::de::Error> {
        let toml = r#"
[servers.remote]
name = "Remote API"
transport.type = "http"
transport.url = "https://api.example.com/mcp"
transport.headers = { Authorization = "Bearer token" }
"#;
        let config: McpConfig = toml::from_str(toml)?;
        let servers = config.servers.expect("should have servers");
        let server = &servers["remote"];
        assert_eq!(server.name.as_deref(), Some("Remote API"));
        assert!(matches!(&server.transport, McpTransportConfig::Http { url, .. } if url == "https://api.example.com/mcp"));
        Ok(())
    }

    #[test]
    fn disabled_server() -> Result<(), toml::de::Error> {
        let toml = r#"
[servers.test]
transport.type = "stdio"
transport.command = "test"
enabled = false
"#;
        let config: McpConfig = toml::from_str(toml)?;
        let servers = config.servers.expect("should have servers");
        assert!(!servers["test"].enabled);
        Ok(())
    }

    #[test]
    fn with_env_vars() -> Result<(), toml::de::Error> {
        let toml = r#"
[servers.test]
transport.type = "stdio"
transport.command = "server"
env = { API_KEY = "secret", DEBUG = "true" }
"#;
        let config: McpConfig = toml::from_str(toml)?;
        let servers = config.servers.expect("should have servers");
        assert_eq!(servers["test"].env.len(), 2);
        assert_eq!(servers["test"].env["API_KEY"], "secret");
        Ok(())
    }

    #[test]
    fn empty_section() -> Result<(), toml::de::Error> {
        let config: McpConfig = toml::from_str("")?;
        assert!(config.servers.is_none());
        Ok(())
    }

    #[test]
    fn no_servers() -> Result<(), toml::de::Error> {
        let toml = "[servers]";
        let config: McpConfig = toml::from_str(toml)?;
        let servers = config.servers.expect("should have servers");
        assert!(servers.is_empty());
        Ok(())
    }
}
