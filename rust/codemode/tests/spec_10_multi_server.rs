//! Tests for §10: Multi-server orchestration and `codemode_run` end-to-end.
//!
//! Validates that a single execution can import multiple servers,
//! compose tool calls, use `Promise.all` for concurrency, and that
//! `codemode_run` correctly handles capabilities and error absorption.

mod common;

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use stencila_codemode::{
    CodemodeError, DiagnosticCode, DiagnosticSeverity, McpContent, McpServer, McpToolInfo,
    McpToolResult, RunRequest, codemode_run,
};

use common::{MockServer, run_request, simple_tool, tool_with_schema};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn database_server() -> Arc<dyn McpServer> {
    Arc::new(MockServer::new(
        "database",
        "Database Server",
        vec![simple_tool("query", "Run a database query")],
    ))
}

fn server_with_capabilities(id: &str, name: &str, caps: Vec<&str>) -> Arc<dyn McpServer> {
    let mut server = MockServer::new(id, name, vec![simple_tool("ping", "Ping")]);
    server.capabilities = Some(caps.into_iter().map(String::from).collect());
    Arc::new(server)
}

fn run_request_with_caps(code: &str, caps: Vec<&str>) -> RunRequest {
    RunRequest {
        code: code.into(),
        limits: None,
        requested_capabilities: Some(caps.into_iter().map(String::from).collect()),
    }
}

// ---------------------------------------------------------------------------
// §10 — Multi-server imports and composition
// ---------------------------------------------------------------------------

/// §10: Import multiple server modules and compose tool calls.
#[tokio::test]
async fn multi_server_import_and_compose() {
    let servers: Vec<Arc<dyn McpServer>> = vec![files_server(), database_server()];
    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { query } from "@codemode/servers/database";
        const fileResult = await readFile();
        const dbResult = await query();
        globalThis.__codemode_result__ = { file: fileResult, db: dbResult };
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result["file"], "Called readFile");
    assert_eq!(resp.result["db"], "Called query");
}

/// §10.1: Promise.all concurrent tool calls across servers.
#[tokio::test]
async fn promise_all_concurrent_tool_calls() {
    let servers: Vec<Arc<dyn McpServer>> = vec![files_server(), database_server()];
    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { query } from "@codemode/servers/database";
        const [fileResult, dbResult] = await Promise.all([readFile(), query()]);
        globalThis.__codemode_result__ = { file: fileResult, db: dbResult };
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result["file"], "Called readFile");
    assert_eq!(resp.result["db"], "Called query");
}

/// §10.1: Promise.all with multiple calls to same server.
#[tokio::test]
async fn promise_all_same_server_concurrent() {
    let servers: Vec<Arc<dyn McpServer>> = vec![files_server()];
    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile, search } from "@codemode/servers/files";
        const [r1, r2] = await Promise.all([readFile(), search({ query: "*.rs" })]);
        globalThis.__codemode_result__ = [r1, r2];
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result[0], "Called readFile");
    assert_eq!(resp.result[1], "Called search");
}

/// §10: Tool trace records calls from multiple servers in order.
#[tokio::test]
async fn multi_server_tool_trace() {
    let servers: Vec<Arc<dyn McpServer>> = vec![files_server(), database_server()];
    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { query } from "@codemode/servers/database";
        await readFile();
        await query();
        await readFile();
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    let trace = resp
        .tool_trace
        .as_ref()
        .expect("tool_trace should be present");
    assert_eq!(trace.len(), 3);
    assert_eq!(trace[0].server_id, "files");
    assert_eq!(trace[0].tool_name, "readFile");
    assert_eq!(trace[1].server_id, "database");
    assert_eq!(trace[1].tool_name, "query");
    assert_eq!(trace[2].server_id, "files");
    assert_eq!(trace[2].tool_name, "readFile");
}

/// §10: Failure in one server call throws ToolCallError but does not abort the script.
#[tokio::test]
async fn server_error_does_not_abort_script() {
    let mut error_server = MockServer::new(
        "failing",
        "Failing Server",
        vec![simple_tool("fail_tool", "Always fails")],
    );
    error_server.call_response = common::MockCallResponse::ErrorResult("server error".into());

    let servers: Vec<Arc<dyn McpServer>> = vec![files_server(), Arc::new(error_server)];

    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile } from "@codemode/servers/files";
        import { fail_tool } from "@codemode/servers/failing";
        let errorCaught = false;
        try {
            await fail_tool();
        } catch (e) {
            errorCaught = true;
        }
        const fileResult = await readFile();
        globalThis.__codemode_result__ = { errorCaught, fileResult };
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result["errorCaught"], true);
    assert_eq!(resp.result["fileResult"], "Called readFile");
}

// ---------------------------------------------------------------------------
// §3.3.4 — codemode_run never errors
// ---------------------------------------------------------------------------

/// §3.3.4: codemode_run returns RunResponse even on syntax error.
#[tokio::test]
async fn codemode_run_syntax_error_returns_response() {
    let resp = codemode_run(
        &run_request("this is not valid @@@ javascript"),
        &[],
        &HashSet::new(),
    )
    .await;

    assert_eq!(resp.result, serde_json::Value::Null);
    assert!(!resp.diagnostics.is_empty());
    assert_eq!(resp.diagnostics[0].code, DiagnosticCode::SyntaxError);
}

/// §3.3.4: codemode_run returns RunResponse with logs + diagnostics on exception.
#[tokio::test]
async fn codemode_run_exception_preserves_logs() {
    let resp = codemode_run(
        &run_request(
            r#"
        console.log("before error");
        throw new Error("boom");
    "#,
        ),
        &[],
        &HashSet::new(),
    )
    .await;

    assert_eq!(resp.result, serde_json::Value::Null);
    assert_eq!(resp.logs.len(), 1);
    assert_eq!(resp.logs[0].message, "before error");
    assert!(!resp.diagnostics.is_empty());
    assert!(resp.diagnostics[0].message.contains("boom"));
}

/// §3.3.4: codemode_run end-to-end with server import, tool call, and result.
#[tokio::test]
async fn codemode_run_end_to_end_with_tool_call() {
    let servers: Vec<Arc<dyn McpServer>> = vec![files_server()];
    let resp = codemode_run(
        &run_request(
            r#"
        import { readFile } from "@codemode/servers/files";
        const result = await readFile();
        globalThis.__codemode_result__ = result;
    "#,
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!("Called readFile"));
}

// ---------------------------------------------------------------------------
// §3.2.3 — requestedCapabilities checking
// ---------------------------------------------------------------------------

/// §3.2.3: Matching capabilities produce no warnings.
#[tokio::test]
async fn requested_capabilities_matched_no_warning() {
    let servers: Vec<Arc<dyn McpServer>> =
        vec![server_with_capabilities("fs", "FS", vec!["file-system"])];
    let resp = codemode_run(
        &run_request_with_caps("globalThis.__codemode_result__ = 'ok'", vec!["file-system"]),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!("ok"));
}

/// §3.2.3: Unmatched capability emits a warning diagnostic.
#[tokio::test]
async fn requested_capabilities_unmatched_emits_warning() {
    let servers: Vec<Arc<dyn McpServer>> =
        vec![server_with_capabilities("fs", "FS", vec!["file-system"])];
    let resp = codemode_run(
        &run_request_with_caps(
            "globalThis.__codemode_result__ = 'ok'",
            vec!["file-system", "database"],
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert_eq!(resp.result, serde_json::json!("ok"));
    assert_eq!(resp.diagnostics.len(), 1);
    assert_eq!(resp.diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(
        resp.diagnostics[0].code,
        DiagnosticCode::CapabilityUnavailable
    );
    assert!(resp.diagnostics[0].message.contains("database"));
}

/// §3.2.3: Multiple unmatched capabilities produce multiple warnings.
#[tokio::test]
async fn requested_capabilities_multiple_unmatched() {
    let resp = codemode_run(
        &run_request_with_caps(
            "globalThis.__codemode_result__ = 'ok'",
            vec!["alpha", "beta"],
        ),
        &[],
        &HashSet::new(),
    )
    .await;

    assert_eq!(resp.result, serde_json::json!("ok"));
    let warnings: Vec<_> = resp
        .diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Warning)
        .collect();
    assert_eq!(warnings.len(), 2);
    assert_eq!(warnings[0].code, DiagnosticCode::CapabilityUnavailable);
    assert!(warnings[0].message.contains("alpha"));
    assert_eq!(warnings[1].code, DiagnosticCode::CapabilityUnavailable);
    assert!(warnings[1].message.contains("beta"));
}

/// §3.2.3: Capability matched by any one server satisfies the check.
#[tokio::test]
async fn requested_capability_matched_by_any_server() {
    let servers: Vec<Arc<dyn McpServer>> = vec![
        server_with_capabilities("s1", "S1", vec!["cap-a"]),
        server_with_capabilities("s2", "S2", vec!["cap-b"]),
    ];
    let resp = codemode_run(
        &run_request_with_caps(
            "globalThis.__codemode_result__ = 'ok'",
            vec!["cap-a", "cap-b"],
        ),
        &servers,
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
}

/// §3.2.3: Empty requestedCapabilities produces no diagnostics.
#[tokio::test]
async fn empty_requested_capabilities_no_diagnostics() {
    let resp = codemode_run(
        &run_request_with_caps("globalThis.__codemode_result__ = 'ok'", vec![]),
        &[],
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.is_empty());
}

/// §3.2.3: Capability warnings appear before execution diagnostics.
#[tokio::test]
async fn capability_warnings_precede_execution_diagnostics() {
    let resp = codemode_run(
        &run_request_with_caps("throw new Error('boom')", vec!["missing-cap"]),
        &[],
        &HashSet::new(),
    )
    .await;

    assert!(resp.diagnostics.len() >= 2);
    // Warning should come first
    assert_eq!(resp.diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(
        resp.diagnostics[0].code,
        DiagnosticCode::CapabilityUnavailable
    );
    assert!(resp.diagnostics[0].message.contains("missing-cap"));
    // Then the execution error
    assert_eq!(resp.diagnostics[1].severity, DiagnosticSeverity::Error);
}

// ---------------------------------------------------------------------------
// §8.1 via codemode_run — dirty-server refresh wiring
// ---------------------------------------------------------------------------

/// Minimal mock server that tracks refresh calls and can stage new tools.
struct RefreshableMockServer {
    id: String,
    tools: Mutex<Vec<McpToolInfo>>,
    pending_tools: Mutex<Option<Vec<McpToolInfo>>>,
    refresh_count: AtomicU32,
}

impl RefreshableMockServer {
    fn new(id: &str, tools: Vec<McpToolInfo>) -> Self {
        Self {
            id: id.into(),
            tools: Mutex::new(tools),
            pending_tools: Mutex::new(None),
            refresh_count: AtomicU32::new(0),
        }
    }

    fn stage_tools(&self, tools: Vec<McpToolInfo>) {
        *self.pending_tools.lock().expect("lock") = Some(tools);
    }

    fn refresh_count(&self) -> u32 {
        self.refresh_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl McpServer for RefreshableMockServer {
    fn server_id(&self) -> &str {
        &self.id
    }

    fn server_name(&self) -> &str {
        &self.id
    }

    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError> {
        Ok(self.tools.lock().expect("lock").clone())
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        _input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError> {
        Ok(McpToolResult {
            content: vec![McpContent::Text {
                text: format!("Called {tool_name}"),
            }],
            structured_content: None,
            is_error: false,
        })
    }

    fn supports_list_changed(&self) -> bool {
        true
    }

    async fn refresh_tools(&self) -> Result<(), CodemodeError> {
        self.refresh_count.fetch_add(1, Ordering::SeqCst);
        if let Some(new_tools) = self.pending_tools.lock().expect("lock").take() {
            *self.tools.lock().expect("lock") = new_tools;
        }
        Ok(())
    }
}

/// §8.1 via codemode_run: dirty server is refreshed, new tool is callable.
#[tokio::test]
async fn codemode_run_refreshes_dirty_server() {
    let server = Arc::new(RefreshableMockServer::new(
        "dynamic",
        vec![simple_tool("ping", "Ping")],
    ));
    server.stage_tools(vec![
        simple_tool("ping", "Ping"),
        simple_tool("pong", "Pong"),
    ]);

    let mut dirty = HashSet::new();
    dirty.insert("dynamic".to_string());

    let resp = codemode_run(
        &run_request(
            r#"
            import { pong } from "@codemode/servers/dynamic";
            globalThis.__codemode_result__ = await pong();
        "#,
        ),
        &[server.clone() as Arc<dyn McpServer>],
        &dirty,
    )
    .await;

    assert!(resp.diagnostics.is_empty(), "{:?}", resp.diagnostics);
    assert_eq!(resp.result, serde_json::json!("Called pong"));
    assert_eq!(server.refresh_count(), 1);
}
