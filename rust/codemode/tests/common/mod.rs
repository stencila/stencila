#![allow(dead_code)]

use std::sync::Arc;

use stencila_codemode::{
    CodemodeError, Limits, McpContent, McpServer, McpToolInfo, McpToolResult, RunRequest, Sandbox,
};

/// Type alias for custom call_tool handler closures.
type CallToolHandler =
    Arc<dyn Fn(&str, serde_json::Value) -> Result<McpToolResult, CodemodeError> + Send + Sync>;

/// Controls how `MockServer::call_tool()` responds.
pub enum MockCallResponse {
    /// Echo the tool name as text (default behavior).
    Echo,
    /// Return structured content with the given JSON value.
    StructuredContent(serde_json::Value),
    /// Return an error result with the given message.
    ErrorResult(String),
    /// Return multiple content blocks.
    MultiContent(Vec<McpContent>),
    /// Return a custom response via a closure.
    Custom(CallToolHandler),
}

impl Default for MockCallResponse {
    fn default() -> Self {
        Self::Echo
    }
}

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
    pub call_response: MockCallResponse,
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
            call_response: MockCallResponse::Echo,
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
            call_response: MockCallResponse::Echo,
        }
    }

    /// Set the call response behavior for this mock server.
    pub fn with_call_response(mut self, response: MockCallResponse) -> Self {
        self.call_response = response;
        self
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
        input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError> {
        match &self.call_response {
            MockCallResponse::Echo => Ok(McpToolResult {
                content: vec![McpContent::Text {
                    text: format!("Called {tool_name}"),
                }],
                structured_content: None,
                is_error: false,
            }),
            MockCallResponse::StructuredContent(value) => Ok(McpToolResult {
                content: vec![],
                structured_content: Some(value.clone()),
                is_error: false,
            }),
            MockCallResponse::ErrorResult(msg) => Ok(McpToolResult {
                content: vec![McpContent::Text { text: msg.clone() }],
                structured_content: None,
                is_error: true,
            }),
            MockCallResponse::MultiContent(content) => Ok(McpToolResult {
                content: content.clone(),
                structured_content: None,
                is_error: false,
            }),
            MockCallResponse::Custom(f) => f(tool_name, input),
        }
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

/// Create a tool info with a custom input schema.
pub fn tool_with_custom_schema(
    name: &str,
    description: &str,
    input_schema: serde_json::Value,
) -> McpToolInfo {
    McpToolInfo {
        name: name.into(),
        description: Some(description.into()),
        input_schema: Some(input_schema),
        output_schema: None,
        annotations: None,
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

/// Create a sandbox with the given mock servers and limits.
pub async fn sandbox_with_limits(servers: Vec<Arc<dyn McpServer>>, limits: Limits) -> Sandbox {
    Sandbox::new(Some(&limits), &servers)
        .await
        .expect("sandbox creation")
}

/// Create a minimal `RunRequest` with no limits or capabilities.
pub fn run_request(code: &str) -> RunRequest {
    RunRequest {
        code: code.into(),
        limits: None,
        requested_capabilities: None,
    }
}
