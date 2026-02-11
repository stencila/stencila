use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rquickjs::{Ctx, function::Func};

use crate::error::{CodemodeError, LimitKind};
use crate::modules::ToolSnapshot;
use crate::search::search_tools;
use crate::traits::{McpContent, McpServer};
use crate::types::{DetailLevel, SearchToolsOptions, ToolTraceEntry};

use super::SandboxState;
use super::console::inject_console;
use super::polyfills::inject_polyfills;

/// JavaScript source that strips banned globals per spec §3.5.
///
/// Removes `eval` and neuters the `Function` constructor (string form).
/// `QuickJS` doesn't have fetch/XMLHttpRequest/WebSocket/require/process
/// natively, so we just confirm their absence by deleting them if present.
const STRIP_GLOBALS_JS: &str = r#"
(function() {
    "use strict";

    // Delete eval
    delete globalThis.eval;

    // Delete network/Node.js globals (should not exist in QuickJS, but be safe)
    delete globalThis.fetch;
    delete globalThis.XMLHttpRequest;
    delete globalThis.WebSocket;
    delete globalThis.setInterval;
    delete globalThis.require;
    delete globalThis.process;

    // Neuter Function constructor (prevent string-based code generation)
    const OrigFunction = Function;
    const SafeFunction = function() {
        throw new TypeError('Function constructor is not allowed in codemode sandbox');
    };
    SafeFunction.prototype = OrigFunction.prototype;
    OrigFunction.prototype.constructor = SafeFunction;
    Object.defineProperty(globalThis, 'Function', {
        value: SafeFunction,
        writable: false,
        configurable: false
    });
})();
"#;

/// JavaScript source for setTimeout/clearTimeout implementation.
///
/// Uses the microtask queue (`Promise.resolve().then()`) to defer callbacks.
/// Actual time delays are not supported in Phase 2 — the delay parameter
/// is accepted but ignored. This will be upgraded in later phases.
const SET_TIMEOUT_JS: &str = r"
(function() {
    let nextId = 1;
    const active = new Map();

    globalThis.setTimeout = function(callback, _delay) {
        const id = nextId++;
        active.set(id, true);
        Promise.resolve().then(() => {
            if (active.has(id)) {
                active.delete(id);
                try { callback(); } catch(e) { console.error(String(e)); }
            }
        });
        return id;
    };

    globalThis.clearTimeout = function(id) {
        active.delete(id);
    };
})();
";

/// JavaScript source that freezes the host bridge as a non-configurable,
/// non-writable property on `globalThis` per spec §11 bullet 4.
///
/// This prevents agent-authored code from tampering with the bridge.
const FREEZE_HOST_BRIDGE_JS: &str = r"
(function() {
    const bridge = {
        listServers: globalThis.__codemode_list_servers,
        describeServer: globalThis.__codemode_describe_server,
        listTools: globalThis.__codemode_list_tools,
        getTool: globalThis.__codemode_get_tool,
        searchTools: globalThis.__codemode_search_tools,
        callTool: globalThis.__codemode_call_tool,
    };

    // Remove raw helpers from global scope
    delete globalThis.__codemode_list_servers;
    delete globalThis.__codemode_describe_server;
    delete globalThis.__codemode_list_tools;
    delete globalThis.__codemode_get_tool;
    delete globalThis.__codemode_search_tools;
    delete globalThis.__codemode_call_tool;

    // Install as frozen, non-configurable, non-writable property
    Object.defineProperty(globalThis, '__codemode_internal__', {
        value: Object.freeze(bridge),
        writable: false,
        configurable: false,
        enumerable: false
    });
})();
";

/// Set up all sandbox globals.
///
/// This configures the sandbox environment in the correct order:
/// 1. Inject `__codemode_result__` (initially null)
/// 2. Inject console capture
/// 3. Inject polyfills (URL, `URLSearchParams`, `TextEncoder`, `TextDecoder`)
/// 4. Inject host bridge functions (discovery + callTool)
/// 5. Inject setTimeout/clearTimeout
/// 6. Strip banned globals (eval, Function constructor)
pub(super) fn setup_globals(
    ctx: &Ctx<'_>,
    state: &Arc<Mutex<SandboxState>>,
    snapshot: &Arc<ToolSnapshot>,
    servers: &Arc<HashMap<String, Arc<dyn McpServer>>>,
) -> Result<(), rquickjs::Error> {
    // 1. Set __codemode_result__ to null
    ctx.globals().set(
        "__codemode_result__",
        rquickjs::Value::new_null(ctx.clone()),
    )?;

    // 2. Inject console capture
    inject_console(ctx, state)?;

    // 3. Inject polyfills
    inject_polyfills(ctx)?;

    // 4. Inject host bridge functions then freeze into __codemode_internal__
    inject_host_bridge(ctx, snapshot)?;
    inject_call_tool_bridge(ctx, state, snapshot, servers)?;

    // 5. Inject setTimeout/clearTimeout
    ctx.eval::<(), _>(SET_TIMEOUT_JS)?;

    // Freeze the bridge (must come after all bridge functions are injected)
    ctx.eval::<(), _>(FREEZE_HOST_BRIDGE_JS)?;

    // 6. Strip banned globals
    ctx.eval::<(), _>(STRIP_GLOBALS_JS)?;

    Ok(())
}

/// Inject host bridge functions for discovery operations.
///
/// Each function captures an `Arc<ToolSnapshot>` and returns a JSON string.
/// The freeze step happens later in `setup_globals` after all bridge functions
/// (including callTool) are registered.
fn inject_host_bridge(ctx: &Ctx<'_>, snapshot: &Arc<ToolSnapshot>) -> Result<(), rquickjs::Error> {
    // listServers() → JSON string of ServerInfo[]
    let snap = Arc::clone(snapshot);
    ctx.globals().set(
        "__codemode_list_servers",
        Func::from(move || -> String {
            let servers = snap.list_servers();
            serde_json::to_string(&servers).unwrap_or_else(|_| "[]".into())
        }),
    )?;

    // describeServer(id) → JSON string of ServerDescription | null
    let snap = Arc::clone(snapshot);
    ctx.globals().set(
        "__codemode_describe_server",
        Func::from(move |id: String| -> String {
            match snap.describe_server(&id) {
                Some(desc) => serde_json::to_string(&desc).unwrap_or_else(|_| "null".into()),
                None => "null".into(),
            }
        }),
    )?;

    // listTools(serverId, detail) → JSON string of ToolDefinition[] | null
    let snap = Arc::clone(snapshot);
    ctx.globals().set(
        "__codemode_list_tools",
        Func::from(move |server_id: String, detail: String| -> String {
            let detail_level = match detail.as_str() {
                "name" => DetailLevel::Name,
                "full" => DetailLevel::Full,
                _ => DetailLevel::Description,
            };
            match snap.list_tools(&server_id, detail_level) {
                Some(tools) => serde_json::to_string(&tools).unwrap_or_else(|_| "null".into()),
                None => "null".into(),
            }
        }),
    )?;

    // getTool(serverId, toolName) → JSON string with discriminated result:
    //   {"found": <ToolDefinition>} | {"error": "server_not_found"} | {"error": "tool_not_found"}
    let snap = Arc::clone(snapshot);
    ctx.globals().set(
        "__codemode_get_tool",
        Func::from(move |server_id: String, tool_name: String| -> String {
            if !snap.has_server(&server_id) {
                return r#"{"error":"server_not_found"}"#.into();
            }
            match snap.get_tool(&server_id, &tool_name) {
                Some(def) => serde_json::to_string(&def)
                    .unwrap_or_else(|_| r#"{"error":"tool_not_found"}"#.into()),
                None => r#"{"error":"tool_not_found"}"#.into(),
            }
        }),
    )?;

    // searchTools(query, optsJson) → JSON string of SearchResults
    let snap = Arc::clone(snapshot);
    ctx.globals().set(
        "__codemode_search_tools",
        Func::from(move |query: String, opts_json: String| -> String {
            let options: SearchToolsOptions = serde_json::from_str(&opts_json).unwrap_or_default();
            let results = search_tools(&snap, &query, &options);
            serde_json::to_string(&results)
                .unwrap_or_else(|_| r#"{"query":"","results":[]}"#.into())
        }),
    )?;

    Ok(())
}

/// Inject the `callTool` async bridge function.
///
/// Registers `__codemode_call_tool(serverId, toolName, inputJson)` as an async
/// function that returns a JSON envelope string. The function is async because
/// it calls the MCP server (which is an async operation).
fn inject_call_tool_bridge(
    ctx: &Ctx<'_>,
    state: &Arc<Mutex<SandboxState>>,
    snapshot: &Arc<ToolSnapshot>,
    servers: &Arc<HashMap<String, Arc<dyn McpServer>>>,
) -> Result<(), rquickjs::Error> {
    let state = Arc::clone(state);
    let snapshot = Arc::clone(snapshot);
    let servers = Arc::clone(servers);

    ctx.globals().set(
        "__codemode_call_tool",
        Func::from(rquickjs::function::Async(
            move |server_id: String, tool_name: String, input_json: String| {
                let state = Arc::clone(&state);
                let snapshot = Arc::clone(&snapshot);
                let servers = Arc::clone(&servers);
                async move {
                    call_tool_bridge(
                        &state,
                        &snapshot,
                        &servers,
                        &server_id,
                        &tool_name,
                        &input_json,
                    )
                    .await
                }
            },
        )),
    )?;

    Ok(())
}

/// Execute the full call-tool sequence and return a JSON envelope string.
///
/// Steps:
/// 1. Look up tool in snapshot to find its input schema
/// 2. Parse input JSON
/// 3. Validate input against schema (if present)
/// 4. Check `tool_call_count` vs `max_tool_calls`
/// 5. Call `server.call_tool()`
/// 6. Record `ToolTraceEntry`
/// 7. Unwrap result per §5.3.2 or return error envelope
async fn call_tool_bridge(
    state: &Arc<Mutex<SandboxState>>,
    snapshot: &Arc<ToolSnapshot>,
    servers: &Arc<HashMap<String, Arc<dyn McpServer>>>,
    server_id: &str,
    tool_name: &str,
    input_json: &str,
) -> String {
    // Look up the server
    let server = match servers.get(server_id) {
        Some(s) => Arc::clone(s),
        None => {
            return tool_not_found_envelope(server_id, tool_name);
        }
    };

    // Find the tool in the snapshot to get its input schema and export name
    let tool_info = snapshot
        .servers
        .iter()
        .find(|s| s.normalized_id == server_id)
        .and_then(|s| s.tools.iter().find(|t| t.name == tool_name));

    let (input_schema, export_name) = match tool_info {
        Some(t) => (t.input_schema.as_ref(), t.export_name.as_str()),
        None => {
            return tool_not_found_envelope(server_id, tool_name);
        }
    };

    // Parse input JSON
    let input: serde_json::Value = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => {
            return schema_validation_envelope(
                tool_name,
                export_name,
                &format!("Invalid JSON input: {e}"),
                None,
                None,
                None,
            );
        }
    };

    // Validate against schema (if present)
    if let Some(schema) = input_schema
        && let Err(failure) = validate_input(&input, schema)
    {
        return schema_validation_envelope(
            tool_name,
            export_name,
            &failure.message,
            failure.path.as_deref(),
            failure.expected.as_deref(),
            failure.received.as_deref(),
        );
    }

    // Check tool call limit (scoped lock — must not hold across .await)
    {
        let Ok(mut s) = state.lock() else {
            return error_envelope(
                "tool_call",
                "Internal error: state lock poisoned",
                server_id,
                tool_name,
            );
        };

        if let Some(max) = s.max_tool_calls
            && s.tool_call_count >= max
        {
            return sandbox_limit_envelope(LimitKind::ToolCalls);
        }

        s.tool_call_count += 1;
    }

    // Call the tool (async)
    let start = Instant::now();
    let result = server.call_tool(tool_name, input).await;
    #[allow(clippy::cast_possible_truncation)] // duration in ms will not exceed u64
    let duration_ms = start.elapsed().as_millis() as u64;

    // Record trace and build envelope
    match result {
        Ok(tool_result) if tool_result.is_error => {
            let msg = extract_error_message(&tool_result.content);
            record_trace(state, server_id, tool_name, duration_ms, Some(&msg));
            error_envelope("tool_call", &msg, server_id, tool_name)
        }
        Ok(tool_result) => {
            record_trace(state, server_id, tool_name, duration_ms, None);
            let value = unwrap_result(&tool_result);
            success_envelope(&value)
        }
        Err(CodemodeError::Authentication {
            server_id: sid,
            message: msg,
        }) => {
            record_trace(state, server_id, tool_name, duration_ms, Some(&msg));
            authentication_envelope(&sid, &msg)
        }
        Err(e) => {
            let msg = e.to_string();
            record_trace(state, server_id, tool_name, duration_ms, Some(&msg));
            error_envelope("tool_call", &msg, server_id, tool_name)
        }
    }
}

/// Record a tool trace entry in the sandbox state.
///
/// If `error` is `Some`, the trace is marked as failed (ok=false).
/// Silently ignored if the state lock is poisoned (not worth aborting for tracing).
fn record_trace(
    state: &Arc<Mutex<SandboxState>>,
    server_id: &str,
    tool_name: &str,
    duration_ms: u64,
    error: Option<&str>,
) {
    if let Ok(mut s) = state.lock() {
        s.tool_trace.push(ToolTraceEntry {
            server_id: server_id.to_string(),
            tool_name: tool_name.to_string(),
            duration_ms,
            ok: error.is_none(),
            error: error.map(String::from),
        });
    }
}

/// Validation failure details.
struct ValidationFailure {
    message: String,
    path: Option<String>,
    expected: Option<String>,
    received: Option<String>,
}

/// Validate input against a JSON Schema.
///
/// Returns `Ok(())` if valid, or `Err(ValidationFailure)` with details.
/// If the schema itself is invalid, validation is gracefully skipped.
fn validate_input(
    input: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), ValidationFailure> {
    let Ok(validator) = jsonschema::validator_for(schema) else {
        return Ok(()); // Gracefully skip if schema is invalid
    };

    let mut errors = validator.iter_errors(input);
    if let Some(error) = errors.next() {
        let instance_path = error.instance_path().to_string();
        let path = if instance_path.is_empty() {
            // For required-property errors at root, extract the property name
            // from the message (format: `"propName" is a required property`)
            extract_required_property_path(&error.to_string())
        } else {
            Some(instance_path)
        };

        // Try to extract expected type from the schema at the error path
        let expected = extract_expected(&error);
        let received = extract_received(input, &error);

        Err(ValidationFailure {
            message: error.to_string(),
            path,
            expected,
            received,
        })
    } else {
        Ok(())
    }
}

/// Extract a JSON pointer path from a required-property error message.
///
/// The jsonschema crate produces messages like `"query" is a required property`
/// for missing required fields. When this occurs at the root level (empty
/// `instance_path`), we extract the property name and return `"/query"`.
fn extract_required_property_path(message: &str) -> Option<String> {
    if message.contains("is a required property") {
        // Extract property name from: "propName" is a required property
        message.split('"').nth(1).map(|name| format!("/{name}"))
    } else {
        None
    }
}

/// Extract the expected type/constraint from a validation error.
fn extract_expected(error: &jsonschema::ValidationError<'_>) -> Option<String> {
    // The error message often contains the expected type
    let msg = error.to_string();
    if msg.contains("is not of type") {
        // Extract the type name from e.g. `42 is not of type "string"`
        msg.split("is not of type ")
            .nth(1)
            .map(|s| s.trim_matches('"').to_string())
    } else if msg.contains("is a required property") {
        Some("required".to_string())
    } else {
        None
    }
}

/// Extract a description of the received value for the validation error.
fn extract_received(
    input: &serde_json::Value,
    error: &jsonschema::ValidationError<'_>,
) -> Option<String> {
    // Navigate to the instance at the error path
    let instance_path = error.instance_path().as_str();
    let value = if instance_path.is_empty() {
        input
    } else {
        // Walk the JSON path
        let mut current = input;
        for segment in instance_path.split('/').skip(1) {
            current = match current {
                serde_json::Value::Object(map) => map.get::<str>(segment)?,
                serde_json::Value::Array(arr) => match segment.parse::<usize>() {
                    Ok(i) => arr.get(i)?,
                    Err(_) => return None,
                },
                _ => return None,
            };
        }
        current
    };

    Some(json_type_name(value).to_string())
}

/// Get a human-readable type name for a JSON value.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Unwrap an MCP tool result per §5.3.2.
///
/// Priority:
/// 1. `structured_content` (if present, return as-is)
/// 2. Single text content → return the string directly
/// 3. Single non-text or multiple content blocks → serialize full result
fn unwrap_result(result: &crate::traits::McpToolResult) -> serde_json::Value {
    // 1. Structured content takes priority
    if let Some(structured) = &result.structured_content {
        return structured.clone();
    }

    // 2. Single text content → return as string
    if result.content.len() == 1
        && let McpContent::Text { text } = &result.content[0]
    {
        return serde_json::Value::String(text.clone());
    }

    // 3. Multiple content blocks or non-text → serialize full
    serialize_full_result(&result.content)
}

/// Serialize content blocks as a JSON array.
fn serialize_full_result(content: &[McpContent]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = content
        .iter()
        .map(|c| match c {
            McpContent::Text { text } => serde_json::json!({
                "type": "text",
                "text": text,
            }),
            McpContent::Image { data, mime_type } => serde_json::json!({
                "type": "image",
                "data": data,
                "mimeType": mime_type,
            }),
            McpContent::Audio { data, mime_type } => serde_json::json!({
                "type": "audio",
                "data": data,
                "mimeType": mime_type,
            }),
        })
        .collect();

    serde_json::Value::Array(items)
}

/// Extract an error message from MCP content blocks.
fn extract_error_message(content: &[McpContent]) -> String {
    for c in content {
        if let McpContent::Text { text } = c {
            return text.clone();
        }
    }
    "Tool returned an error".to_string()
}

// ============================================================
// Envelope builders
// ============================================================

/// Build a success envelope: `{"ok": true, "value": <value>}`
fn success_envelope(value: &serde_json::Value) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "value": value,
    }))
    .unwrap_or_else(|_| r#"{"ok":true,"value":null}"#.into())
}

/// Build a tool-not-found envelope.
fn tool_not_found_envelope(server_id: &str, tool_name: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": "tool_not_found",
        "message": format!("Tool '{}' not found on server '{}'", tool_name, server_id),
        "serverId": server_id,
        "toolName": tool_name,
        "hint": "Use listTools() to see available tools on this server.",
    }))
    .unwrap_or_else(|_| {
        r#"{"ok":false,"error":"tool_not_found","message":"Tool not found"}"#.into()
    })
}

/// Build a schema validation error envelope.
fn schema_validation_envelope(
    tool_name: &str,
    export_name: &str,
    message: &str,
    path: Option<&str>,
    expected: Option<&str>,
    received: Option<&str>,
) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": "schema_validation",
        "message": message,
        "toolName": tool_name,
        "exportName": export_name,
        "path": path,
        "expected": expected,
        "received": received,
        "hint": format!("Check the input schema for '{}'.", export_name),
    }))
    .unwrap_or_else(|_| {
        r#"{"ok":false,"error":"schema_validation","message":"Schema validation failed"}"#.into()
    })
}

/// Build an authentication error envelope.
fn authentication_envelope(server_id: &str, message: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": "authentication",
        "message": message,
        "serverId": server_id,
        "hint": "Check the server's credentials or API key.",
    }))
    .unwrap_or_else(|_| {
        r#"{"ok":false,"error":"authentication","message":"Authentication failed"}"#.into()
    })
}

/// Build a generic error envelope (`tool_call` errors).
fn error_envelope(error_type: &str, message: &str, server_id: &str, tool_name: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": error_type,
        "message": message,
        "serverId": server_id,
        "toolName": tool_name,
        "hint": "Check the tool call parameters and try again.",
    }))
    .unwrap_or_else(|_| {
        format!(r#"{{"ok":false,"error":"{error_type}","message":"Error occurred"}}"#)
    })
}

/// Build a sandbox limit error envelope.
fn sandbox_limit_envelope(kind: LimitKind) -> String {
    let kind_str = match kind {
        LimitKind::Timeout => "timeout",
        LimitKind::Memory => "memory",
        LimitKind::LogBytes => "logBytes",
        LimitKind::ToolCalls => "toolCalls",
    };
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": "sandbox_limit",
        "message": format!("Sandbox limit exceeded: {kind}"),
        "kind": kind_str,
        "hint": "Reduce the number of tool calls or increase maxToolCalls.",
    }))
    .unwrap_or_else(|_| {
        r#"{"ok":false,"error":"sandbox_limit","message":"Sandbox limit exceeded"}"#.into()
    })
}
