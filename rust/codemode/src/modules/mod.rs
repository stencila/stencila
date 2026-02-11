pub(crate) mod discovery;
pub(crate) mod errors;
pub(crate) mod server;

use std::collections::HashSet;
use std::sync::Arc;

use rquickjs::{Ctx, Module, loader};

use crate::error::CodemodeError;
use crate::identifiers::resolve_export_collisions;
use crate::traits::{McpServer, McpToolInfo};
use crate::types::{DetailLevel, ServerDescription, ServerInfo, ToolDefinition};

/// A pre-fetched snapshot of all server and tool information.
///
/// Built once before sandbox creation (since `McpServer::tools()` is async).
/// The snapshot is shared via `Arc` between the module loader and host bridge.
pub(crate) struct ToolSnapshot {
    pub servers: Vec<ServerToolset>,
}

/// All information about a single server and its tools.
pub(crate) struct ServerToolset {
    /// Original server ID (from `McpServer::server_id()`).
    #[allow(dead_code)] // Preserved for debugging and future tracing
    pub original_id: String,
    /// Normalized ID for module paths (per §5.0.1).
    pub normalized_id: String,
    pub server_name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub tools: Vec<SnapshotTool>,
}

/// A single tool within a snapshot.
pub(crate) struct SnapshotTool {
    /// Canonical MCP tool name.
    pub name: String,
    /// JS export identifier (after collision resolution per §6.1).
    pub export_name: String,
    pub description: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
}

impl ToolSnapshot {
    /// Build a snapshot by pre-fetching tools from all servers.
    ///
    /// Normalizes server IDs (§5.0.1), resolves export name collisions (§6.1),
    /// and stores a frozen view of the tool definitions.
    pub(crate) async fn build(servers: &[Arc<dyn McpServer>]) -> Result<Self, CodemodeError> {
        let mut server_toolsets = Vec::with_capacity(servers.len());

        // Normalize server IDs and resolve collisions.
        // The collision resolver sorts alphabetically, so we build a HashMap
        // to look up normalized IDs while preserving the original server order.
        let original_ids: Vec<&str> = servers.iter().map(|s| s.server_id()).collect();
        let normalized_pairs = resolve_server_id_collisions(&original_ids)?;
        let id_map: std::collections::HashMap<&str, &str> = normalized_pairs
            .iter()
            .map(|(orig, norm)| (orig.as_str(), norm.as_str()))
            .collect();

        for server in servers {
            let normalized_id = (*id_map.get(server.server_id()).ok_or_else(|| {
                CodemodeError::InvalidServerId {
                    server_id: server.server_id().to_string(),
                }
            })?)
            .to_string();
            let mcp_tools = server.tools().await?;
            let tools = resolve_tool_exports(&mcp_tools);

            server_toolsets.push(ServerToolset {
                original_id: server.server_id().to_string(),
                normalized_id,
                server_name: server.server_name().to_string(),
                description: server.description().map(String::from),
                version: server.version().map(String::from),
                capabilities: server.capabilities(),
                tools,
            });
        }

        Ok(Self {
            servers: server_toolsets,
        })
    }

    /// Create an empty snapshot (no servers).
    pub(crate) fn empty() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// List all servers as `ServerInfo`.
    pub(crate) fn list_servers(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|s| ServerInfo {
                server_id: s.normalized_id.clone(),
                server_name: s.server_name.clone(),
                capabilities: s.capabilities.clone(),
            })
            .collect()
    }

    /// Get extended information about a server.
    pub(crate) fn describe_server(&self, normalized_id: &str) -> Option<ServerDescription> {
        self.servers
            .iter()
            .find(|s| s.normalized_id == normalized_id)
            .map(|s| ServerDescription {
                server_id: s.normalized_id.clone(),
                server_name: s.server_name.clone(),
                capabilities: s.capabilities.clone(),
                description: s.description.clone(),
                version: s.version.clone(),
            })
    }

    /// List tools for a server at the given detail level.
    pub(crate) fn list_tools(
        &self,
        normalized_id: &str,
        detail: DetailLevel,
    ) -> Option<Vec<ToolDefinition>> {
        let server = self
            .servers
            .iter()
            .find(|s| s.normalized_id == normalized_id)?;
        Some(
            server
                .tools
                .iter()
                .map(|t| tool_to_definition(t, detail))
                .collect(),
        )
    }

    /// Get a single tool definition.
    pub(crate) fn get_tool(&self, normalized_id: &str, tool_name: &str) -> Option<ToolDefinition> {
        let server = self
            .servers
            .iter()
            .find(|s| s.normalized_id == normalized_id)?;
        let tool = server.tools.iter().find(|t| t.name == tool_name)?;
        Some(tool_to_definition(tool, DetailLevel::Full))
    }

    /// Check whether a server with the given normalized ID exists.
    pub(crate) fn has_server(&self, normalized_id: &str) -> bool {
        self.servers
            .iter()
            .any(|s| s.normalized_id == normalized_id)
    }

    /// Get the set of known normalized server IDs (for module resolution).
    pub(crate) fn known_server_ids(&self) -> HashSet<String> {
        self.servers
            .iter()
            .map(|s| s.normalized_id.clone())
            .collect()
    }
}

/// Convert a snapshot tool to a `ToolDefinition` at the given detail level.
pub(crate) fn tool_to_definition(tool: &SnapshotTool, detail: DetailLevel) -> ToolDefinition {
    match detail {
        DetailLevel::Name => ToolDefinition {
            tool_name: tool.name.clone(),
            export_name: tool.export_name.clone(),
            description: None,
            annotations: None,
            input_schema: None,
            output_schema: None,
        },
        DetailLevel::Description => ToolDefinition {
            tool_name: tool.name.clone(),
            export_name: tool.export_name.clone(),
            description: tool.description.clone(),
            annotations: tool.annotations.clone(),
            input_schema: None,
            output_schema: None,
        },
        DetailLevel::Full => ToolDefinition {
            tool_name: tool.name.clone(),
            export_name: tool.export_name.clone(),
            description: tool.description.clone(),
            annotations: tool.annotations.clone(),
            input_schema: tool.input_schema.clone(),
            output_schema: tool.output_schema.clone(),
        },
    }
}

/// Resolve server ID collisions using normalization.
fn resolve_server_id_collisions(ids: &[&str]) -> Result<Vec<(String, String)>, CodemodeError> {
    crate::identifiers::resolve_server_collisions(ids)
}

/// Resolve tool export name collisions for a set of MCP tools.
fn resolve_tool_exports(tools: &[McpToolInfo]) -> Vec<SnapshotTool> {
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    let resolved = resolve_export_collisions(&names);

    // Build a mapping from canonical name → export name
    let export_map: std::collections::HashMap<&str, &str> = resolved
        .iter()
        .map(|(canonical, export)| (canonical.as_str(), export.as_str()))
        .collect();

    tools
        .iter()
        .map(|t| {
            let export_name =
                (*export_map.get(t.name.as_str()).unwrap_or(&t.name.as_str())).to_string();
            SnapshotTool {
                name: t.name.clone(),
                export_name,
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
                output_schema: t.output_schema.clone(),
                annotations: t.annotations.clone(),
            }
        })
        .collect()
}

// ============================================================
// Module resolver and loader for `@codemode/*` imports
// ============================================================

/// Resolves `@codemode/*` module specifiers.
///
/// Recognized modules:
/// - `@codemode/discovery`
/// - `@codemode/errors`
/// - `@codemode/servers/<normalized-id>` (for known servers)
pub(crate) struct CodemodeResolver {
    known_servers: HashSet<String>,
}

impl CodemodeResolver {
    pub(crate) fn new(snapshot: &ToolSnapshot) -> Self {
        Self {
            known_servers: snapshot.known_server_ids(),
        }
    }
}

impl loader::Resolver for CodemodeResolver {
    fn resolve(&mut self, _ctx: &Ctx<'_>, base: &str, name: &str) -> rquickjs::Result<String> {
        match name {
            "@codemode/discovery" | "@codemode/errors" => Ok(name.to_string()),
            _ if name.starts_with("@codemode/servers/") => {
                let id = &name["@codemode/servers/".len()..];
                if self.known_servers.contains(id) {
                    Ok(name.to_string())
                } else {
                    Err(rquickjs::Error::new_resolving(base, name))
                }
            }
            _ => Err(rquickjs::Error::new_resolving(base, name)),
        }
    }
}

/// Loads `@codemode/*` modules by generating JS source.
///
/// - `@codemode/discovery` → static JS calling host bridge functions
/// - `@codemode/errors` → static JS defining error class hierarchy
/// - `@codemode/servers/<id>` → generated JS per-server module with tool bindings
pub(crate) struct CodemodeLoader {
    snapshot: Arc<ToolSnapshot>,
}

impl CodemodeLoader {
    pub(crate) fn new(snapshot: &Arc<ToolSnapshot>) -> Self {
        Self {
            snapshot: Arc::clone(snapshot),
        }
    }
}

impl loader::Loader for CodemodeLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let source = match name {
            "@codemode/discovery" => discovery::JS_SOURCE.to_string(),
            "@codemode/errors" => errors::JS_SOURCE.to_string(),
            _ if name.starts_with("@codemode/servers/") => {
                let id = &name["@codemode/servers/".len()..];
                let toolset = self
                    .snapshot
                    .servers
                    .iter()
                    .find(|s| s.normalized_id == id)
                    .ok_or_else(|| {
                        rquickjs::Error::new_loading_message(
                            name,
                            &format!("No server with ID '{id}'"),
                        )
                    })?;
                server::generate_module(toolset)
            }
            _ => return Err(rquickjs::Error::new_loading(name)),
        };
        Module::declare(ctx.clone(), name, source)
    }
}
