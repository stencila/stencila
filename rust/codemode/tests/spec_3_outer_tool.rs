use stencila_codemode::{
    DetailLevel, Diagnostic, DiagnosticCode, DiagnosticSeverity, Limits, ListToolsOptions,
    LogEntry, LogLevel, RunRequest, RunResponse, SearchResultEntry, SearchResults,
    SearchToolsOptions, ServerDescription, ServerInfo, ToolDefinition, ToolSummary, ToolTraceEntry,
};

// ============================================================
// §3 — RunRequest / RunResponse serialization round-trips
// ============================================================

#[test]
fn run_request_serialization() {
    let req = RunRequest {
        code: "console.log('hello')".into(),
        limits: Some(Limits {
            timeout_ms: Some(5000),
            max_memory_bytes: Some(10_000_000),
            max_log_bytes: Some(50_000),
            max_tool_calls: Some(20),
        }),
        requested_capabilities: Some(vec!["filesystem".into()]),
    };

    let json = serde_json::to_value(&req).expect("serialize");
    assert_eq!(json["code"], "console.log('hello')");
    assert_eq!(json["limits"]["timeoutMs"], 5000);
    assert_eq!(json["limits"]["maxMemoryBytes"], 10_000_000);
    assert_eq!(json["limits"]["maxLogBytes"], 50_000);
    assert_eq!(json["limits"]["maxToolCalls"], 20);
    assert_eq!(json["requestedCapabilities"][0], "filesystem");

    let deser: RunRequest = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.code, "console.log('hello')");
    let limits = deser.limits.expect("limits present");
    assert_eq!(limits.timeout_ms, Some(5000));
}

#[test]
fn run_request_minimal() {
    let json = serde_json::json!({ "code": "1 + 1" });
    let req: RunRequest = serde_json::from_value(json).expect("deserialize");
    assert_eq!(req.code, "1 + 1");
    assert!(req.limits.is_none());
    assert!(req.requested_capabilities.is_none());
}

#[test]
fn run_response_serialization() {
    let resp = RunResponse {
        logs: vec![LogEntry {
            level: LogLevel::Log,
            message: "hello".into(),
            time_ms: 42,
        }],
        result: serde_json::json!({"answer": 42}),
        diagnostics: vec![Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::SyntaxError,
            message: "Unexpected token".into(),
            hint: Some("Check your syntax".into()),
            path: Some("1:5".into()),
            error_class: None,
        }],
        tool_trace: Some(vec![ToolTraceEntry {
            server_id: "server-a".into(),
            tool_name: "readFile".into(),
            duration_ms: 150,
            ok: true,
            error: None,
        }]),
    };

    let json = serde_json::to_value(&resp).expect("serialize");

    // Check log entry
    assert_eq!(json["logs"][0]["level"], "log");
    assert_eq!(json["logs"][0]["message"], "hello");
    assert_eq!(json["logs"][0]["timeMs"], 42);

    // Check diagnostic
    assert_eq!(json["diagnostics"][0]["severity"], "error");
    assert_eq!(json["diagnostics"][0]["code"], "SYNTAX_ERROR");
    assert_eq!(json["diagnostics"][0]["message"], "Unexpected token");
    assert_eq!(json["diagnostics"][0]["hint"], "Check your syntax");

    // Check tool trace
    assert_eq!(json["toolTrace"][0]["serverId"], "server-a");
    assert_eq!(json["toolTrace"][0]["toolName"], "readFile");
    assert_eq!(json["toolTrace"][0]["durationMs"], 150);
    assert_eq!(json["toolTrace"][0]["ok"], true);

    // Round-trip
    let deser: RunResponse = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.logs.len(), 1);
    assert_eq!(deser.result["answer"], 42);
    assert_eq!(deser.diagnostics.len(), 1);
    assert_eq!(deser.diagnostics[0].code, DiagnosticCode::SyntaxError);
}

#[test]
fn run_response_default() {
    let resp = RunResponse::default();
    assert!(resp.logs.is_empty());
    assert_eq!(resp.result, serde_json::Value::Null);
    assert!(resp.diagnostics.is_empty());
    assert!(resp.tool_trace.is_none());
}

#[test]
fn log_level_serialization() {
    assert_eq!(
        serde_json::to_value(LogLevel::Debug).expect("serialize"),
        "debug"
    );
    assert_eq!(
        serde_json::to_value(LogLevel::Log).expect("serialize"),
        "log"
    );
    assert_eq!(
        serde_json::to_value(LogLevel::Warn).expect("serialize"),
        "warn"
    );
    assert_eq!(
        serde_json::to_value(LogLevel::Error).expect("serialize"),
        "error"
    );
}

#[test]
fn diagnostic_severity_serialization() {
    assert_eq!(
        serde_json::to_value(DiagnosticSeverity::Error).expect("serialize"),
        "error"
    );
    assert_eq!(
        serde_json::to_value(DiagnosticSeverity::Warning).expect("serialize"),
        "warning"
    );
    assert_eq!(
        serde_json::to_value(DiagnosticSeverity::Info).expect("serialize"),
        "info"
    );
}

#[test]
fn diagnostic_code_serialization() {
    assert_eq!(
        serde_json::to_value(DiagnosticCode::SyntaxError).expect("serialize"),
        "SYNTAX_ERROR"
    );
    assert_eq!(
        serde_json::to_value(DiagnosticCode::UncaughtException).expect("serialize"),
        "UNCAUGHT_EXCEPTION"
    );
    assert_eq!(
        serde_json::to_value(DiagnosticCode::ImportFailure).expect("serialize"),
        "IMPORT_FAILURE"
    );
    assert_eq!(
        serde_json::to_value(DiagnosticCode::SandboxLimit).expect("serialize"),
        "SANDBOX_LIMIT"
    );
}

#[test]
fn optional_fields_omitted_when_none() {
    let resp = RunResponse::default();
    let json = serde_json::to_value(&resp).expect("serialize");
    assert!(json.get("toolTrace").is_none());
}

#[test]
fn tool_trace_error_field() {
    let entry = ToolTraceEntry {
        server_id: "s".into(),
        tool_name: "t".into(),
        duration_ms: 10,
        ok: false,
        error: Some("timeout".into()),
    };
    let json = serde_json::to_value(&entry).expect("serialize");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"], "timeout");
}

// ============================================================
// §4.2 — Discovery type serialization round-trips
// ============================================================

#[test]
fn server_info_serialization() {
    let info = ServerInfo {
        server_id: "google-drive".into(),
        server_name: "Google Drive".into(),
        capabilities: Some(vec!["filesystem".into(), "search".into()]),
    };
    let json = serde_json::to_value(&info).expect("serialize");
    assert_eq!(json["serverId"], "google-drive");
    assert_eq!(json["serverName"], "Google Drive");
    assert_eq!(json["capabilities"][0], "filesystem");

    let deser: ServerInfo = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.server_id, "google-drive");
    assert_eq!(deser.capabilities.as_ref().map(Vec::len), Some(2));
}

#[test]
fn server_info_optional_fields_omitted() {
    let info = ServerInfo {
        server_id: "s".into(),
        server_name: "S".into(),
        capabilities: None,
    };
    let json = serde_json::to_value(&info).expect("serialize");
    assert!(json.get("capabilities").is_none());
}

#[test]
fn server_description_serialization() {
    let desc = ServerDescription {
        server_id: "s".into(),
        server_name: "S".into(),
        capabilities: None,
        description: Some("A server".into()),
        version: Some("1.0.0".into()),
    };
    let json = serde_json::to_value(&desc).expect("serialize");
    assert_eq!(json["description"], "A server");
    assert_eq!(json["version"], "1.0.0");

    let deser: ServerDescription = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.description.as_deref(), Some("A server"));
}

#[test]
fn tool_summary_serialization() {
    let summary = ToolSummary {
        tool_name: "readFile".into(),
        export_name: "readFile".into(),
        description: Some("Read a file".into()),
        annotations: Some(serde_json::json!({"readOnlyHint": true})),
    };
    let json = serde_json::to_value(&summary).expect("serialize");
    assert_eq!(json["toolName"], "readFile");
    assert_eq!(json["exportName"], "readFile");
    assert_eq!(json["annotations"]["readOnlyHint"], true);

    let deser: ToolSummary = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.tool_name, "readFile");
}

#[test]
fn tool_definition_serialization() {
    let def = ToolDefinition {
        tool_name: "search".into(),
        export_name: "search".into(),
        description: Some("Search files".into()),
        annotations: None,
        input_schema: Some(serde_json::json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        })),
        output_schema: None,
    };
    let json = serde_json::to_value(&def).expect("serialize");
    assert_eq!(json["inputSchema"]["type"], "object");
    assert!(json.get("outputSchema").is_none());
    assert!(json.get("annotations").is_none());

    let deser: ToolDefinition = serde_json::from_value(json).expect("deserialize");
    assert!(deser.input_schema.is_some());
}

#[test]
fn search_results_serialization() {
    let results = SearchResults {
        query: "read".into(),
        results: vec![SearchResultEntry {
            server_id: "s1".into(),
            tool_name: "readFile".into(),
            export_name: "readFile".into(),
            description: Some("Read a file".into()),
            annotations: None,
            input_schema: None,
            output_schema: None,
        }],
    };
    let json = serde_json::to_value(&results).expect("serialize");
    assert_eq!(json["query"], "read");
    assert_eq!(json["results"][0]["serverId"], "s1");
    assert_eq!(json["results"][0]["toolName"], "readFile");

    let deser: SearchResults = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.results.len(), 1);
}

#[test]
fn detail_level_serialization() {
    assert_eq!(
        serde_json::to_value(DetailLevel::Name).expect("serialize"),
        "name"
    );
    assert_eq!(
        serde_json::to_value(DetailLevel::Description).expect("serialize"),
        "description"
    );
    assert_eq!(
        serde_json::to_value(DetailLevel::Full).expect("serialize"),
        "full"
    );

    // Default is Description
    assert_eq!(DetailLevel::default(), DetailLevel::Description);
}

#[test]
fn list_tools_options_serialization() {
    let opts = ListToolsOptions {
        detail: Some(DetailLevel::Full),
    };
    let json = serde_json::to_value(&opts).expect("serialize");
    assert_eq!(json["detail"], "full");

    let deser: ListToolsOptions = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.detail, Some(DetailLevel::Full));
}

#[test]
fn search_tools_options_serialization() {
    let opts = SearchToolsOptions {
        detail: Some(DetailLevel::Name),
        server_id: Some("s1".into()),
        limit: Some(10),
    };
    let json = serde_json::to_value(&opts).expect("serialize");
    assert_eq!(json["detail"], "name");
    assert_eq!(json["serverId"], "s1");
    assert_eq!(json["limit"], 10);

    let deser: SearchToolsOptions = serde_json::from_value(json).expect("deserialize");
    assert_eq!(deser.limit, Some(10));
}

#[test]
fn search_tools_options_defaults_omit_none() {
    let opts = SearchToolsOptions::default();
    let json = serde_json::to_value(&opts).expect("serialize");
    assert!(json.get("detail").is_none());
    assert!(json.get("serverId").is_none());
    assert!(json.get("limit").is_none());
}
