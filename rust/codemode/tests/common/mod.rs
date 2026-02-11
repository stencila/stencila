#![allow(dead_code)]

use std::sync::Arc;

use stencila_codemode::{
    CodemodeError, McpContent, McpServer, McpToolInfo, McpToolResult, Sandbox,
};

/// A mock MCP server for testing.
///
/// Provides configurable server metadata and a fixed set of tools.
pub struct MockServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub tools: Vec<McpToolInfo>,
}

impl MockServer {
    /// Create a minimal mock server with the given ID and tools.
    pub fn new(id: &str, name: &str, tools: Vec<McpToolInfo>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: None,
            capabilities: None,
            tools,
        }
    }

    /// Create a fully-described mock server.
    pub fn with_description(
        id: &str,
        name: &str,
        description: &str,
        version: &str,
        tools: Vec<McpToolInfo>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: Some(description.into()),
            version: Some(version.into()),
            capabilities: None,
            tools,
        }
    }
}

#[async_trait::async_trait]
impl McpServer for MockServer {
    fn server_id(&self) -> &str {
        &self.id
    }

    fn server_name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    fn capabilities(&self) -> Option<Vec<String>> {
        self.capabilities.clone()
    }

    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        Ok(self.tools.clone())
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        _input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError> {
        // Simple echo: return the tool name as text content
        Ok(McpToolResult {
            content: vec![McpContent::Text {
                text: format!("Called {tool_name}"),
            }],
            structured_content: None,
            is_error: false,
        })
    }
}

/// Create a simple tool info with just a name and description.
pub fn simple_tool(name: &str, description: &str) -> McpToolInfo {
    McpToolInfo {
        name: name.into(),
        description: Some(description.into()),
        input_schema: None,
        output_schema: None,
        annotations: None,
    }
}

/// Create a tool info with a full input schema.
pub fn tool_with_schema(name: &str, description: &str) -> McpToolInfo {
    McpToolInfo {
        name: name.into(),
        description: Some(description.into()),
        input_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        })),
        output_schema: None,
        annotations: Some(serde_json::json!({
            "readOnlyHint": true
        })),
    }
}

/// Create a sandbox with one mock server (convenience for tests needing a minimal server).
pub async fn sandbox_with_server() -> Sandbox {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "test-server",
        "Test Server",
        vec![simple_tool("ping", "Ping the server")],
    ));
    Sandbox::new(None, &[server])
        .await
        .expect("sandbox creation")
}

/// Create a sandbox with the given mock servers.
pub async fn sandbox_with_servers(servers: Vec<Arc<dyn McpServer>>) -> Sandbox {
    Sandbox::new(None, &servers)
        .await
        .expect("sandbox creation")
}
