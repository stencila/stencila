//! MCP pool setup and direct tool injection.
//!
//! When `enable_mcp` is active, each tool from connected MCP servers is
//! registered directly in the agent's [`ToolRegistry`]. The LLM sees each
//! tool individually and calls them by name.
//!
//! [`ToolRegistry`]: crate::registry::ToolRegistry

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde_json::json;
use stencila_mcp::{ConnectionPool, McpContent, McpServer, McpToolInfo, McpToolResult, discover};
use stencila_models3::types::tool::ToolDefinition;

use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};

// ---------------------------------------------------------------------------
// Pool setup
// ---------------------------------------------------------------------------

/// Discover MCP servers and create a connection pool.
///
/// Returns the pool and a list of `(server_id, error)` for servers that
/// failed to connect.
pub async fn setup_mcp_pool(
    workspace_dir: &Path,
) -> (Arc<ConnectionPool>, Vec<(String, stencila_mcp::McpError)>) {
    let configs = discover(workspace_dir);
    let pool = Arc::new(ConnectionPool::new(configs));
    let errors = pool.connect_all().await;
    (pool, errors)
}

// ---------------------------------------------------------------------------
// Tool naming
// ---------------------------------------------------------------------------

/// Sanitize a string for use in a tool name.
///
/// Replaces any character that is not `[a-zA-Z0-9_]` with `_`. This ensures
/// MCP server IDs and tool names (which often contain hyphens, dots, or
/// other characters) produce valid tool names per the `[a-zA-Z][a-zA-Z0-9_]*`
/// constraint.
#[must_use]
fn sanitize_for_tool_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Build a namespaced tool name: `mcp__{server_id}__{tool_name}`.
///
/// Double underscores avoid collisions with tool names that contain single
/// underscores. Server IDs and tool names are sanitized to match the
/// `[a-zA-Z][a-zA-Z0-9_]*` constraint (max 64 characters).
#[must_use]
pub fn mcp_tool_name(server_id: &str, tool_name: &str) -> String {
    let raw = format!(
        "mcp__{}__{}",
        sanitize_for_tool_name(server_id),
        sanitize_for_tool_name(tool_name)
    );
    // Truncate to 64 characters (the ToolDefinition max)
    if raw.len() > 64 {
        raw[..64].to_string()
    } else {
        raw
    }
}

// ---------------------------------------------------------------------------
// Tool registration
// ---------------------------------------------------------------------------

/// Register all tools from connected MCP servers into the profile's registry.
///
/// Returns a system prompt metadata section describing the registered MCP
/// tools, or an empty string if no tools were registered.
pub async fn register_mcp_tools(
    profile: &mut dyn ProviderProfile,
    pool: &Arc<ConnectionPool>,
) -> String {
    let servers = pool.connected_servers().await;
    if servers.is_empty() {
        return String::new();
    }

    let mut tool_descriptions: Vec<String> = Vec::new();

    // Track registered names across all servers to detect collisions from
    // sanitization/truncation. Value is `"server_id/tool_name"` for diagnostics.
    let mut registered_origins: HashMap<String, String> = HashMap::new();

    for server in &servers {
        let server_id = server.server_id().to_string();
        let server_name = server.server_name().to_string();

        let tools = match server.tools().await {
            Ok(tools) => tools,
            Err(e) => {
                tracing::warn!(server_id, "failed to list tools from MCP server: {e}");
                continue;
            }
        };

        let mut registered_names: Vec<String> = Vec::new();
        for info in &tools {
            let full_name = mcp_tool_name(&server_id, &info.name);

            // Detect collision: a different server/tool already mapped to this name.
            let origin = format!("{server_id}/{}", info.name);
            if let Some(existing) = registered_origins.get(&full_name) {
                tracing::warn!(
                    full_name,
                    new = origin,
                    existing = existing.as_str(),
                    "MCP tool name collision after sanitization — skipping duplicate"
                );
                continue;
            }

            let registered =
                build_registered_mcp_tool(&full_name, &server_id, &info.name, info, pool);

            if let Err(e) = profile.tool_registry_mut().register(registered) {
                tracing::warn!(full_name, "failed to register MCP tool: {e}");
            } else {
                registered_origins.insert(full_name, origin);
                registered_names.push(info.name.clone());
            }
        }

        // Only add to prompt metadata if at least one tool was registered
        if !registered_names.is_empty() {
            tool_descriptions.push(format!(
                "- **{server_name}** (`{server_id}`): {}",
                registered_names.join(", ")
            ));
        }
    }

    if tool_descriptions.is_empty() {
        return String::new();
    }

    format!(
        "# MCP Tools\n\n\
         The following MCP server tools are available as direct tool calls. \
         Tool names are prefixed with `mcp__<server>__`.\n\n\
         {}\n",
        tool_descriptions.join("\n")
    )
}

/// Build a [`RegisteredTool`] for a single MCP tool.
fn build_registered_mcp_tool(
    full_name: &str,
    server_id: &str,
    tool_name: &str,
    info: &McpToolInfo,
    pool: &Arc<ConnectionPool>,
) -> RegisteredTool {
    let definition = ToolDefinition {
        name: full_name.into(),
        description: info
            .description
            .clone()
            .unwrap_or_else(|| format!("MCP tool: {tool_name}")),
        parameters: info
            .input_schema
            .clone()
            .unwrap_or_else(|| json!({"type": "object"})),
        strict: false,
    };

    let pool = Arc::clone(pool);
    let sid = server_id.to_string();
    let tname = tool_name.to_string();

    let executor: ToolExecutorFn = Box::new(move |args, _env| {
        let pool = Arc::clone(&pool);
        let sid = sid.clone();
        let tname = tname.clone();
        Box::pin(async move {
            let server = pool.get(&sid).await?;
            let result = server.call_tool(&tname, args).await?;
            Ok(mcp_result_to_tool_output(&result))
        })
    });

    RegisteredTool::new(definition, executor)
}

// ---------------------------------------------------------------------------
// Result conversion
// ---------------------------------------------------------------------------

/// Convert an [`McpToolResult`] into a [`ToolOutput`].
///
/// Text content blocks are concatenated. If `is_error` is set, the output
/// is prefixed with `"[ERROR] "`. Image/audio content is currently skipped
/// (rare for MCP tools).
#[must_use]
pub fn mcp_result_to_tool_output(result: &McpToolResult) -> ToolOutput {
    // Prefer structured content if present
    if let Some(ref structured) = result.structured_content {
        let text = serde_json::to_string_pretty(structured).unwrap_or_default();
        return if result.is_error {
            ToolOutput::Text(format!("[ERROR] {text}"))
        } else {
            ToolOutput::Text(text)
        };
    }

    let mut text_parts: Vec<&str> = Vec::new();
    for content in &result.content {
        if let McpContent::Text { text } = content {
            text_parts.push(text);
        }
    }

    let text = text_parts.join("\n");
    if result.is_error {
        ToolOutput::Text(format!("[ERROR] {text}"))
    } else {
        ToolOutput::Text(text)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_invalid_chars() {
        assert_eq!(sanitize_for_tool_name("my-server"), "my_server");
        assert_eq!(sanitize_for_tool_name("a.b.c"), "a_b_c");
        assert_eq!(sanitize_for_tool_name("ok_name"), "ok_name");
        assert_eq!(sanitize_for_tool_name(""), "");
    }

    #[test]
    fn mcp_tool_name_sanitizes_hyphens() {
        assert_eq!(
            mcp_tool_name("my-server", "list_files"),
            "mcp__my_server__list_files"
        );
    }

    #[test]
    fn mcp_tool_name_sanitizes_dots() {
        assert_eq!(
            mcp_tool_name("github.com", "create-pr"),
            "mcp__github_com__create_pr"
        );
    }

    #[test]
    fn mcp_tool_name_truncates_long_names() {
        let long_server = "a".repeat(30);
        let long_tool = "b".repeat(30);
        let name = mcp_tool_name(&long_server, &long_tool);
        assert!(
            name.len() <= 64,
            "tool name should be at most 64 chars, got {}",
            name.len()
        );
        assert!(name.starts_with("mcp__"));
    }

    #[test]
    fn mcp_tool_name_handles_empty_parts() {
        assert_eq!(mcp_tool_name("", "tool"), "mcp____tool");
        assert_eq!(mcp_tool_name("server", ""), "mcp__server__");
    }

    #[test]
    fn sanitization_collision_is_detectable() {
        // Different server IDs that sanitize to the same name — callers
        // should detect this via the `registered_origins` map.
        let a = mcp_tool_name("my-server", "list");
        let b = mcp_tool_name("my.server", "list");
        assert_eq!(a, b, "sanitization should collapse these to the same name");
    }

    #[test]
    fn result_to_output_text_content() {
        let result = McpToolResult {
            content: vec![McpContent::Text {
                text: "hello".into(),
            }],
            structured_content: None,
            is_error: false,
        };
        let output = mcp_result_to_tool_output(&result);
        assert_eq!(output.as_text(), "hello");
    }

    #[test]
    fn result_to_output_multiple_text_blocks() {
        let result = McpToolResult {
            content: vec![
                McpContent::Text {
                    text: "line 1".into(),
                },
                McpContent::Text {
                    text: "line 2".into(),
                },
            ],
            structured_content: None,
            is_error: false,
        };
        let output = mcp_result_to_tool_output(&result);
        assert_eq!(output.as_text(), "line 1\nline 2");
    }

    #[test]
    fn result_to_output_error_prefix() {
        let result = McpToolResult {
            content: vec![McpContent::Text {
                text: "something went wrong".into(),
            }],
            structured_content: None,
            is_error: true,
        };
        let output = mcp_result_to_tool_output(&result);
        assert_eq!(output.as_text(), "[ERROR] something went wrong");
    }

    #[test]
    fn result_to_output_structured_content_preferred() {
        let structured = serde_json::json!({"key": "value"});
        let result = McpToolResult {
            content: vec![McpContent::Text {
                text: "fallback".into(),
            }],
            structured_content: Some(structured.clone()),
            is_error: false,
        };
        let output = mcp_result_to_tool_output(&result);
        let expected = serde_json::to_string_pretty(&structured).unwrap_or_default();
        assert_eq!(output.as_text(), expected);
    }

    #[test]
    fn result_to_output_structured_content_with_error() {
        let structured = serde_json::json!({"error": "bad request"});
        let result = McpToolResult {
            content: vec![],
            structured_content: Some(structured.clone()),
            is_error: true,
        };
        let output = mcp_result_to_tool_output(&result);
        let expected = format!(
            "[ERROR] {}",
            serde_json::to_string_pretty(&structured).unwrap_or_default()
        );
        assert_eq!(output.as_text(), expected);
    }

    #[test]
    fn result_to_output_empty_content() {
        let result = McpToolResult {
            content: vec![],
            structured_content: None,
            is_error: false,
        };
        let output = mcp_result_to_tool_output(&result);
        assert_eq!(output.as_text(), "");
    }

    #[test]
    fn result_to_output_skips_non_text() {
        let result = McpToolResult {
            content: vec![
                McpContent::Text {
                    text: "before".into(),
                },
                McpContent::Image {
                    data: "base64data".into(),
                    mime_type: "image/png".into(),
                },
                McpContent::Text {
                    text: "after".into(),
                },
            ],
            structured_content: None,
            is_error: false,
        };
        let output = mcp_result_to_tool_output(&result);
        assert_eq!(output.as_text(), "before\nafter");
    }
}
