//! Minimal JSON-RPC 2.0 and MCP protocol types.
//!
//! These types represent the wire format for MCP communication. They are used
//! by the transport layer to serialize requests and deserialize responses.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// JSON-RPC 2.0
// ---------------------------------------------------------------------------

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request with the given id and method.
    #[must_use]
    pub fn new(id: u64, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 notification (no `id` field — no response expected).
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: &'static str,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification.
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response (deserialized from the server).
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// A server-initiated notification (no `id` field).
///
/// These arrive asynchronously from the server (e.g. `notifications/tools/listChanged`).
#[derive(Debug, Clone, Deserialize)]
pub struct ServerNotification {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// MCP Protocol Types
// ---------------------------------------------------------------------------

/// The result of the MCP `initialize` handshake.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ServerCapabilities,
    #[serde(default)]
    pub server_info: Option<ServerInfo>,
}

/// Server capabilities declared during initialization.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
}

/// Capability metadata for the tools namespace.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

/// Server identity from the `initialize` response.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

/// A tool definition from `tools/list`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
}

/// The result envelope from `tools/list`.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpToolDefinition>,
}

/// A single content block in a `tools/call` response.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum McpContentBlock {
    Text {
        text: String,
    },
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// The result envelope from `tools/call`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    #[serde(default)]
    pub content: Vec<McpContentBlock>,
    pub structured_content: Option<serde_json::Value>,
    #[serde(default)]
    pub is_error: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_request_serializes() -> Result<(), serde_json::Error> {
        let req = JsonRpcRequest::new(1, "tools/list", None);
        let json = serde_json::to_value(&req)?;
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["method"], "tools/list");
        assert!(json.get("params").is_none());
        Ok(())
    }

    #[test]
    fn json_rpc_request_with_params_serializes() -> Result<(), serde_json::Error> {
        let params = serde_json::json!({"name": "test"});
        let req = JsonRpcRequest::new(42, "tools/call", Some(params));
        let json = serde_json::to_value(&req)?;
        assert_eq!(json["id"], 42);
        assert_eq!(json["params"]["name"], "test");
        Ok(())
    }

    #[test]
    fn json_rpc_response_deserializes_result() -> Result<(), serde_json::Error> {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json)?;
        assert_eq!(resp.id, Some(1));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        Ok(())
    }

    #[test]
    fn json_rpc_response_deserializes_error() -> Result<(), serde_json::Error> {
        let json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json)?;
        assert!(resp.result.is_none());
        let err = resp.error.as_ref().expect("should have error");
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid request");
        Ok(())
    }

    #[test]
    fn server_notification_deserializes() -> Result<(), serde_json::Error> {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/tools/listChanged"}"#;
        let notif: ServerNotification = serde_json::from_str(json)?;
        assert_eq!(notif.method, "notifications/tools/listChanged");
        assert!(notif.params.is_none());
        Ok(())
    }

    #[test]
    fn initialize_result_deserializes() -> Result<(), serde_json::Error> {
        let json = r#"{
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": { "listChanged": true }
            },
            "serverInfo": { "name": "test-server", "version": "1.0.0" }
        }"#;
        let result: InitializeResult = serde_json::from_str(json)?;
        assert_eq!(result.protocol_version, "2025-03-26");
        let tools = result
            .capabilities
            .tools
            .as_ref()
            .expect("tools capability");
        assert!(tools.list_changed);
        let info = result.server_info.as_ref().expect("server info");
        assert_eq!(info.name, "test-server");
        assert_eq!(info.version.as_deref(), Some("1.0.0"));
        Ok(())
    }

    #[test]
    fn tools_list_result_deserializes() -> Result<(), serde_json::Error> {
        let json = r#"{
            "tools": [{
                "name": "read_file",
                "description": "Read a file",
                "inputSchema": { "type": "object" }
            }]
        }"#;
        let result: ToolsListResult = serde_json::from_str(json)?;
        assert_eq!(result.tools.len(), 1);
        assert_eq!(result.tools[0].name, "read_file");
        assert!(result.tools[0].input_schema.is_some());
        Ok(())
    }

    #[test]
    fn tool_call_result_deserializes_text() -> Result<(), serde_json::Error> {
        let json = r#"{
            "content": [{ "type": "text", "text": "hello" }],
            "isError": false
        }"#;
        let result: ToolCallResult = serde_json::from_str(json)?;
        assert_eq!(result.content.len(), 1);
        assert!(!result.is_error);
        assert!(matches!(&result.content[0], McpContentBlock::Text { text } if text == "hello"));
        Ok(())
    }

    #[test]
    fn tool_call_result_deserializes_error() -> Result<(), serde_json::Error> {
        let json = r#"{
            "content": [{ "type": "text", "text": "not found" }],
            "isError": true
        }"#;
        let result: ToolCallResult = serde_json::from_str(json)?;
        assert!(result.is_error);
        Ok(())
    }

    #[test]
    fn tool_call_result_with_structured_content() -> Result<(), serde_json::Error> {
        let json = r#"{
            "content": [],
            "structuredContent": { "data": [1, 2, 3] }
        }"#;
        let result: ToolCallResult = serde_json::from_str(json)?;
        assert!(result.structured_content.is_some());
        Ok(())
    }

    #[test]
    fn mcp_content_block_image() -> Result<(), serde_json::Error> {
        let json = r#"{ "type": "image", "data": "base64...", "mimeType": "image/png" }"#;
        let block: McpContentBlock = serde_json::from_str(json)?;
        assert!(matches!(block, McpContentBlock::Image { .. }));
        Ok(())
    }
}
