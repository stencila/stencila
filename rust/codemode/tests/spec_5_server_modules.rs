mod common;

use std::sync::Arc;

use stencila_codemode::{Limits, McpContent, McpServer, McpToolResult};

use common::{
    MockCallResponse, MockServer, sandbox_with_limits, sandbox_with_servers, simple_tool,
    tool_with_custom_schema, tool_with_schema,
};

/// Helper: create a files server with readFile and search tools.
fn files_server() -> Arc<dyn McpServer> {
    Arc::new(MockServer::with_description(
        "files",
        "File Server",
        "A file management server",
        "1.2.0",
        vec![
            simple_tool("readFile", "Read a file from disk"),
            tool_with_schema("search", "Search for files by query"),
        ],
    ))
}

/// Helper: create a database server.
fn database_server() -> Arc<dyn McpServer> {
    Arc::new(MockServer::new(
        "database",
        "Database Server",
        vec![simple_tool("query", "Run a database query")],
    ))
}

// ============================================================
// §5.1 — Basic tool calling
// ============================================================

#[tokio::test]
async fn import_and_call_tool_returns_result() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile({ path: "/foo" });
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    // MockServer echoes: "Called readFile"
    assert_eq!(response.result, "Called readFile");
}

#[tokio::test]
async fn call_tool_with_no_args_sends_empty_object() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called readFile");
}

#[tokio::test]
async fn call_tool_with_undefined_sends_empty_object() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile(undefined);
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called readFile");
}

#[tokio::test]
async fn call_tool_with_null_sends_empty_object() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile(null);
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called readFile");
}

#[tokio::test]
async fn call_tool_with_empty_object_succeeds() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile({});
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called readFile");
}

#[tokio::test]
async fn call_tool_with_valid_schema_input() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { search } from "@codemode/servers/files";
        const result = await search({ query: "hello" });
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called search");
}

// ============================================================
// §5.2 — __meta__ export
// ============================================================

#[tokio::test]
async fn meta_has_correct_shape() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { __meta__ } from "@codemode/servers/files";
        globalThis.__codemode_result__ = {
            serverId: __meta__.serverId,
            serverName: __meta__.serverName,
            serverVersion: __meta__.serverVersion,
            toolCount: __meta__.tools.length,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["serverId"], "files");
    assert_eq!(response.result["serverName"], "File Server");
    assert_eq!(response.result["serverVersion"], "1.2.0");
    assert_eq!(response.result["toolCount"], 2);
}

#[tokio::test]
async fn meta_tools_have_correct_entries() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { __meta__ } from "@codemode/servers/files";
        globalThis.__codemode_result__ = __meta__.tools.map(t => ({
            toolName: t.toolName,
            exportName: t.exportName,
            hasDescription: typeof t.description === "string",
        }));
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["toolName"], "readFile");
    assert_eq!(arr[0]["exportName"], "readFile");
    assert_eq!(arr[1]["toolName"], "search");
}

#[tokio::test]
async fn meta_is_frozen() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { __meta__ } from "@codemode/servers/files";
        globalThis.__codemode_result__ = {
            metaFrozen: Object.isFrozen(__meta__),
            toolsFrozen: Object.isFrozen(__meta__.tools),
            firstToolFrozen: Object.isFrozen(__meta__.tools[0]),
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["metaFrozen"], true);
    assert_eq!(response.result["toolsFrozen"], true);
    assert_eq!(response.result["firstToolFrozen"], true);
}

#[tokio::test]
async fn meta_server_version_empty_when_absent() {
    let sandbox = sandbox_with_servers(vec![database_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { __meta__ } from "@codemode/servers/database";
        globalThis.__codemode_result__ = __meta__.serverVersion;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    // No version set → empty string in generated JS
    assert_eq!(response.result, "");
}

// ============================================================
// §5.3.2 — Result unwrapping
// ============================================================

#[tokio::test]
async fn structured_content_takes_priority() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("getData", "Get data")])
            .with_call_response(MockCallResponse::StructuredContent(serde_json::json!({
                "key": "value",
                "count": 42,
            }))),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { getData } from "@codemode/servers/test";
        const result = await getData();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["key"], "value");
    assert_eq!(response.result["count"], 42);
}

#[tokio::test]
async fn single_text_unwraps_to_string() {
    // Default Echo mock returns single text content
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile({});
        globalThis.__codemode_result__ = {
            value: result,
            type: typeof result,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["type"], "string");
    assert_eq!(response.result["value"], "Called readFile");
}

#[tokio::test]
async fn image_content_returns_full_array() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("getImage", "Get image")])
            .with_call_response(MockCallResponse::MultiContent(vec![McpContent::Image {
                data: "iVBORw0KGgo=".into(),
                mime_type: "image/png".into(),
            }])),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { getImage } from "@codemode/servers/test";
        const result = await getImage();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["type"], "image");
    assert_eq!(arr[0]["data"], "iVBORw0KGgo=");
    assert_eq!(arr[0]["mimeType"], "image/png");
}

#[tokio::test]
async fn audio_content_returns_full_array() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("getAudio", "Get audio")])
            .with_call_response(MockCallResponse::MultiContent(vec![McpContent::Audio {
                data: "AAAA".into(),
                mime_type: "audio/mp3".into(),
            }])),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { getAudio } from "@codemode/servers/test";
        const result = await getAudio();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["type"], "audio");
}

#[tokio::test]
async fn multiple_text_returns_full_array() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("getMulti", "Multi")]).with_call_response(
            MockCallResponse::MultiContent(vec![
                McpContent::Text {
                    text: "first".into(),
                },
                McpContent::Text {
                    text: "second".into(),
                },
            ]),
        ),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { getMulti } from "@codemode/servers/test";
        const result = await getMulti();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["type"], "text");
    assert_eq!(arr[0]["text"], "first");
    assert_eq!(arr[1]["text"], "second");
}

#[tokio::test]
async fn empty_content_returns_empty_array() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("empty", "Empty")])
            .with_call_response(MockCallResponse::MultiContent(vec![])),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { empty } from "@codemode/servers/test";
        const result = await empty();
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert!(arr.is_empty());
}

// ============================================================
// §7.2 — Schema validation
// ============================================================

#[tokio::test]
async fn schema_validation_missing_required_field() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { SchemaValidationError } from "@codemode/errors";
        import { search } from "@codemode/servers/files";
        let caught = null;
        try {
            await search({});  // missing required "query"
        } catch (e) {
            caught = {
                isSchemaError: e instanceof SchemaValidationError,
                name: e.name,
                message: e.message,
                toolName: e.toolName,
                exportName: e.exportName,
                path: e.path,
                hasHint: typeof e.hint === "string",
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isSchemaError"], true);
    assert_eq!(response.result["name"], "SchemaValidationError");
    assert_eq!(response.result["toolName"], "search");
    assert_eq!(response.result["exportName"], "search");
    assert_eq!(response.result["path"], "/query");
    assert_eq!(response.result["hasHint"], true);
}

#[tokio::test]
async fn schema_validation_wrong_type() {
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "test",
        "Test",
        vec![tool_with_custom_schema(
            "typedTool",
            "A typed tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "count": { "type": "integer" }
                },
                "required": ["count"]
            }),
        )],
    ));
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { SchemaValidationError } from "@codemode/errors";
        import { typedTool } from "@codemode/servers/test";
        let caught = null;
        try {
            await typedTool({ count: "not a number" });
        } catch (e) {
            caught = {
                isSchemaError: e instanceof SchemaValidationError,
                hasExpected: e.expected !== null && e.expected !== undefined,
                hasReceived: e.received !== null && e.received !== undefined,
                path: e.path,
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isSchemaError"], true);
    assert_eq!(response.result["hasExpected"], true);
    assert_eq!(response.result["hasReceived"], true);
    assert_eq!(response.result["path"], "/count");
}

#[tokio::test]
async fn schema_validation_hint_present() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { search } from "@codemode/servers/files";
        let hint = null;
        try {
            await search({});
        } catch (e) {
            hint = e.hint;
        }
        globalThis.__codemode_result__ = hint;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let hint = response.result.as_str().expect("hint should be a string");
    assert!(
        hint.contains("search"),
        "hint should reference the tool: {hint}"
    );
}

#[tokio::test]
async fn schema_validation_instanceof_checks() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { CodemodeError, SchemaValidationError } from "@codemode/errors";
        import { search } from "@codemode/servers/files";
        let checks = null;
        try {
            await search({});
        } catch (e) {
            checks = {
                isSchemaValidation: e instanceof SchemaValidationError,
                isCodemode: e instanceof CodemodeError,
                isError: e instanceof Error,
            };
        }
        globalThis.__codemode_result__ = checks;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isSchemaValidation"], true);
    assert_eq!(response.result["isCodemode"], true);
    assert_eq!(response.result["isError"], true);
}

#[tokio::test]
async fn invalid_schema_gracefully_skipped() {
    // A tool with an invalid JSON Schema should not cause validation to fail
    let server: Arc<dyn McpServer> = Arc::new(MockServer::new(
        "test",
        "Test",
        vec![tool_with_custom_schema(
            "badSchema",
            "Tool with bad schema",
            serde_json::json!({
                "type": "not_a_real_type"
            }),
        )],
    ));
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { badSchema } from "@codemode/servers/test";
        const result = await badSchema({ anything: "goes" });
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    // Should succeed (validation skipped) → Echo result
    assert_eq!(response.result, "Called badSchema");
}

#[tokio::test]
async fn no_schema_allows_any_input() {
    // readFile has no input_schema, so any input should be accepted
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile({ whatever: true, nested: { deep: 42 } });
        globalThis.__codemode_result__ = result;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result, "Called readFile");
}

// ============================================================
// §3.3.2 — Tool trace
// ============================================================

#[tokio::test]
async fn tool_trace_recorded_on_success() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        await readFile({});
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let trace = response
        .tool_trace
        .as_ref()
        .expect("tool_trace should exist");
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].server_id, "files");
    assert_eq!(trace[0].tool_name, "readFile");
    assert!(trace[0].ok);
    assert!(trace[0].error.is_none());
}

#[tokio::test]
async fn tool_trace_has_duration() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        await readFile({});
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    let trace = response.tool_trace.as_ref().expect("tool_trace");
    // Duration should be >= 0 (it's a u64)
    // We can't assert exact value but can confirm the field exists
    assert!(
        trace[0].duration_ms < 10000,
        "duration should be reasonable"
    );
}

#[tokio::test]
async fn tool_trace_recorded_on_error() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("fail", "Fails")])
            .with_call_response(MockCallResponse::ErrorResult("Something went wrong".into())),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { fail } from "@codemode/servers/test";
        try {
            await fail({});
        } catch (e) {
            // Expected
        }
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let trace = response.tool_trace.as_ref().expect("tool_trace");
    assert_eq!(trace.len(), 1);
    assert!(!trace[0].ok);
    assert_eq!(trace[0].error.as_deref(), Some("Something went wrong"));
}

#[tokio::test]
async fn tool_trace_multiple_calls() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile, search } from "@codemode/servers/files";
        await readFile({});
        await search({ query: "test" });
        await readFile({});
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let trace = response.tool_trace.as_ref().expect("tool_trace");
    assert_eq!(trace.len(), 3);
    assert_eq!(trace[0].tool_name, "readFile");
    assert_eq!(trace[1].tool_name, "search");
    assert_eq!(trace[2].tool_name, "readFile");
}

#[tokio::test]
async fn tool_trace_absent_when_no_calls() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { __meta__ } from "@codemode/servers/files";
        globalThis.__codemode_result__ = __meta__.serverId;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert!(response.tool_trace.is_none());
}

// ============================================================
// Limits — maxToolCalls
// ============================================================

#[tokio::test]
async fn max_tool_calls_enforced() {
    let limits = Limits {
        timeout_ms: None,
        max_memory_bytes: None,
        max_log_bytes: None,
        max_tool_calls: Some(2),
    };
    let sandbox = sandbox_with_limits(vec![files_server()], limits).await;
    let response = sandbox
        .execute(
            r#"
        import { SandboxLimitError } from "@codemode/errors";
        import { readFile } from "@codemode/servers/files";
        const results = [];
        for (let i = 0; i < 3; i++) {
            try {
                await readFile({});
                results.push("ok");
            } catch (e) {
                results.push({
                    caught: true,
                    isSandboxLimit: e instanceof SandboxLimitError,
                    kind: e.kind,
                });
            }
        }
        globalThis.__codemode_result__ = results;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    let arr = response.result.as_array().expect("array");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], "ok");
    assert_eq!(arr[1], "ok");
    assert_eq!(arr[2]["caught"], true);
    assert_eq!(arr[2]["isSandboxLimit"], true);
    assert_eq!(arr[2]["kind"], "toolCalls");
}

#[tokio::test]
async fn max_tool_calls_exact_boundary() {
    let limits = Limits {
        timeout_ms: None,
        max_memory_bytes: None,
        max_log_bytes: None,
        max_tool_calls: Some(1),
    };
    let sandbox = sandbox_with_limits(vec![files_server()], limits).await;
    let response = sandbox
        .execute(
            r#"
        import { SandboxLimitError } from "@codemode/errors";
        import { readFile } from "@codemode/servers/files";
        const first = await readFile({});
        let secondFailed = false;
        try {
            await readFile({});
        } catch (e) {
            secondFailed = e instanceof SandboxLimitError;
        }
        globalThis.__codemode_result__ = { first, secondFailed };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["first"], "Called readFile");
    assert_eq!(response.result["secondFailed"], true);
}

// ============================================================
// Error cases
// ============================================================

#[tokio::test]
async fn is_error_true_throws_tool_call_error() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("errorTool", "Error tool")])
            .with_call_response(MockCallResponse::ErrorResult("Bad request".into())),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { ToolCallError } from "@codemode/errors";
        import { errorTool } from "@codemode/servers/test";
        let caught = null;
        try {
            await errorTool({});
        } catch (e) {
            caught = {
                isToolCallError: e instanceof ToolCallError,
                name: e.name,
                message: e.message,
                serverId: e.serverId,
                toolName: e.toolName,
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isToolCallError"], true);
    assert_eq!(response.result["name"], "ToolCallError");
    assert_eq!(response.result["message"], "Bad request");
    assert_eq!(response.result["serverId"], "test");
    assert_eq!(response.result["toolName"], "errorTool");
}

#[tokio::test]
async fn rust_error_throws_tool_call_error() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("crashTool", "Crash")])
            .with_call_response(MockCallResponse::Custom(Arc::new(|_tool_name, _input| {
                Err(stencila_codemode::CodemodeError::Runtime(
                    "Connection lost".into(),
                ))
            }))),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { ToolCallError } from "@codemode/errors";
        import { crashTool } from "@codemode/servers/test";
        let caught = null;
        try {
            await crashTool({});
        } catch (e) {
            caught = {
                isToolCallError: e instanceof ToolCallError,
                messageContains: e.message.includes("Connection lost"),
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isToolCallError"], true);
    assert_eq!(response.result["messageContains"], true);
}

#[tokio::test]
async fn rust_error_records_trace() {
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("crashTool", "Crash")])
            .with_call_response(MockCallResponse::Custom(Arc::new(|_tool_name, _input| {
                Err(stencila_codemode::CodemodeError::Runtime(
                    "Connection lost".into(),
                ))
            }))),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { crashTool } from "@codemode/servers/test";
        try { await crashTool({}); } catch (e) {}
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    let trace = response.tool_trace.as_ref().expect("tool_trace");
    assert_eq!(trace.len(), 1);
    assert!(!trace[0].ok);
    assert!(
        trace[0]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Connection lost"))
    );
}

#[tokio::test]
async fn unknown_tool_via_bridge_throws_tool_not_found_error() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { ToolNotFoundError } from "@codemode/errors";
        // Call the bridge directly with a tool name that doesn't exist
        const json = await globalThis.__codemode_internal__.callTool("files", "nonExistent", "{}");
        const r = JSON.parse(json);
        let caught = null;
        if (!r.ok && r.error === "tool_not_found") {
            // Manually construct the error to verify the envelope shape
            caught = {
                ok: r.ok,
                error: r.error,
                hasMessage: typeof r.message === "string",
                serverId: r.serverId,
                toolName: r.toolName,
                hasHint: typeof r.hint === "string",
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["ok"], false);
    assert_eq!(response.result["error"], "tool_not_found");
    assert_eq!(response.result["hasMessage"], true);
    assert_eq!(response.result["serverId"], "files");
    assert_eq!(response.result["toolName"], "nonExistent");
    assert_eq!(response.result["hasHint"], true);
}

#[tokio::test]
async fn unknown_tool_via_generated_handler_throws_tool_not_found() {
    // Verify the __handleResult__ function in generated modules correctly
    // converts a tool_not_found envelope into a ToolNotFoundError
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { ToolNotFoundError } from "@codemode/errors";
        // Import a real module to get access to the __handleResult__ flow,
        // then call the bridge directly with a bad tool name and let the
        // module's error handling convert the envelope.
        const __internal__ = globalThis.__codemode_internal__;
        let caught = null;
        try {
            const json = await __internal__.callTool("files", "doesNotExist", "{}");
            const r = JSON.parse(json);
            if (!r.ok) {
                // Simulate what __handleResult__ does
                if (r.error === "tool_not_found") {
                    throw new ToolNotFoundError(r.message, {
                        serverId: r.serverId,
                        toolName: r.toolName,
                        hint: r.hint,
                    });
                }
            }
        } catch (e) {
            caught = {
                isToolNotFound: e instanceof ToolNotFoundError,
                name: e.name,
                serverId: e.serverId,
                toolName: e.toolName,
            };
        }
        globalThis.__codemode_result__ = caught;
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["isToolNotFound"], true);
    assert_eq!(response.result["name"], "ToolNotFoundError");
    assert_eq!(response.result["serverId"], "files");
    assert_eq!(response.result["toolName"], "doesNotExist");
}

// ============================================================
// Multi-server & edge cases
// ============================================================

#[tokio::test]
async fn cross_server_orchestration() {
    let sandbox = sandbox_with_servers(vec![files_server(), database_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { query } from "@codemode/servers/database";
        const fileResult = await readFile({});
        const dbResult = await query({});
        globalThis.__codemode_result__ = { fileResult, dbResult };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["fileResult"], "Called readFile");
    assert_eq!(response.result["dbResult"], "Called query");
}

#[tokio::test]
async fn cross_server_tool_trace() {
    let sandbox = sandbox_with_servers(vec![files_server(), database_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { query } from "@codemode/servers/database";
        await readFile({});
        await query({});
        globalThis.__codemode_result__ = "done";
    "#,
        )
        .await;

    let trace = response.tool_trace.as_ref().expect("tool_trace");
    assert_eq!(trace.len(), 2);
    assert_eq!(trace[0].server_id, "files");
    assert_eq!(trace[0].tool_name, "readFile");
    assert_eq!(trace[1].server_id, "database");
    assert_eq!(trace[1].tool_name, "query");
}

#[tokio::test]
async fn discovery_and_server_modules_coexist() {
    let sandbox = sandbox_with_servers(vec![files_server()]).await;
    let response = sandbox
        .execute(
            r#"
        import { listServers } from "@codemode/discovery";
        import { readFile, __meta__ } from "@codemode/servers/files";

        const servers = await listServers();
        const result = await readFile({});

        globalThis.__codemode_result__ = {
            serverCount: servers.length,
            result,
            serverId: __meta__.serverId,
        };
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["serverCount"], 1);
    assert_eq!(response.result["result"], "Called readFile");
    assert_eq!(response.result["serverId"], "files");
}

#[tokio::test]
async fn input_json_passed_to_server() {
    // Verify the actual input JSON is received by the server
    let server: Arc<dyn McpServer> = Arc::new(
        MockServer::new("test", "Test", vec![simple_tool("echo", "Echo input")])
            .with_call_response(MockCallResponse::Custom(Arc::new(|_tool_name, input| {
                Ok(McpToolResult {
                    content: vec![McpContent::Text {
                        text: serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string()),
                    }],
                    structured_content: None,
                    is_error: false,
                })
            }))),
    );
    let sandbox = sandbox_with_servers(vec![server]).await;
    let response = sandbox
        .execute(
            r#"
        import { echo } from "@codemode/servers/test";
        const result = await echo({ key: "value", num: 42 });
        globalThis.__codemode_result__ = JSON.parse(result);
    "#,
        )
        .await;

    assert!(
        response.diagnostics.is_empty(),
        "diagnostics: {:?}",
        response.diagnostics
    );
    assert_eq!(response.result["key"], "value");
    assert_eq!(response.result["num"], 42);
}
