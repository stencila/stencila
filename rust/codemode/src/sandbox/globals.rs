use std::sync::{Arc, Mutex};

use rquickjs::{Ctx, function::Func};

use crate::modules::ToolSnapshot;
use crate::search::search_tools;
use crate::types::{DetailLevel, SearchToolsOptions};

use super::SandboxState;
use super::console::inject_console;
use super::polyfills::inject_polyfills;

/// JavaScript source that strips banned globals per spec §3.5.
///
/// Removes `eval` and neuters the `Function` constructor (string form).
/// QuickJS doesn't have fetch/XMLHttpRequest/WebSocket/require/process
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
const SET_TIMEOUT_JS: &str = r#"
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
"#;

/// JavaScript source that freezes the host bridge as a non-configurable,
/// non-writable property on `globalThis` per spec §11 bullet 4.
///
/// This prevents agent-authored code from tampering with the bridge.
const FREEZE_HOST_BRIDGE_JS: &str = r#"
(function() {
    const bridge = {
        listServers: globalThis.__codemode_list_servers,
        describeServer: globalThis.__codemode_describe_server,
        listTools: globalThis.__codemode_list_tools,
        getTool: globalThis.__codemode_get_tool,
        searchTools: globalThis.__codemode_search_tools,
    };

    // Remove raw helpers from global scope
    delete globalThis.__codemode_list_servers;
    delete globalThis.__codemode_describe_server;
    delete globalThis.__codemode_list_tools;
    delete globalThis.__codemode_get_tool;
    delete globalThis.__codemode_search_tools;

    // Install as frozen, non-configurable, non-writable property
    Object.defineProperty(globalThis, '__codemode_internal__', {
        value: Object.freeze(bridge),
        writable: false,
        configurable: false,
        enumerable: false
    });
})();
"#;

/// Set up all sandbox globals.
///
/// This configures the sandbox environment in the correct order:
/// 1. Inject `__codemode_result__` (initially null)
/// 2. Inject console capture
/// 3. Inject polyfills (URL, URLSearchParams, TextEncoder, TextDecoder)
/// 4. Inject host bridge functions for `@codemode/discovery`
/// 5. Inject setTimeout/clearTimeout
/// 6. Strip banned globals (eval, Function constructor)
pub(super) fn setup_globals(
    ctx: &Ctx<'_>,
    state: &Arc<Mutex<SandboxState>>,
    snapshot: &Arc<ToolSnapshot>,
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

    // 5. Inject setTimeout/clearTimeout
    ctx.eval::<(), _>(SET_TIMEOUT_JS)?;

    // 6. Strip banned globals
    ctx.eval::<(), _>(STRIP_GLOBALS_JS)?;

    Ok(())
}

/// Inject host bridge functions for discovery operations.
///
/// Each function captures an `Arc<ToolSnapshot>` and returns a JSON string.
/// After injection, `FREEZE_HOST_BRIDGE_JS` moves them into a frozen
/// `__codemode_internal__` object.
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

    // Freeze the bridge into a non-configurable, non-writable property
    ctx.eval::<(), _>(FREEZE_HOST_BRIDGE_JS)?;

    Ok(())
}
