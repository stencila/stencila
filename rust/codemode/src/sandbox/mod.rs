mod console;
mod globals;
mod limits;
mod polyfills;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Module, Promise, Value,
    async_with, module::Evaluated,
};

use crate::error::CodemodeError;
use crate::modules::{CodemodeLoader, CodemodeResolver, ToolSnapshot};
use stencila_mcp::McpServer;
use crate::types::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, Limits, LogEntry, RunResponse, ToolTraceEntry,
};

/// Shared mutable state for the sandbox execution.
///
/// Accessed by the console capture, tool call tracking, and limit enforcement.
pub(crate) struct SandboxState {
    pub logs: Vec<LogEntry>,
    pub diagnostics: Vec<Diagnostic>,
    pub tool_trace: Vec<ToolTraceEntry>,
    pub log_bytes: u64,
    pub tool_call_count: u32,
    pub start_time: Instant,
    pub max_log_bytes: Option<u64>,
    pub max_tool_calls: Option<u32>,
    pub log_truncated: bool,
    /// Whether a memory limit was configured (used for error classification).
    pub has_memory_limit: bool,
}

impl SandboxState {
    fn new(limits: Option<&Limits>) -> Self {
        Self {
            logs: Vec::new(),
            diagnostics: Vec::new(),
            tool_trace: Vec::new(),
            log_bytes: 0,
            tool_call_count: 0,
            start_time: Instant::now(),
            max_log_bytes: limits.and_then(|l| l.max_log_bytes),
            max_tool_calls: limits.and_then(|l| l.max_tool_calls),
            log_truncated: false,
            has_memory_limit: limits.and_then(|l| l.max_memory_bytes).is_some(),
        }
    }

    #[allow(clippy::cast_possible_truncation)] // duration in ms will not exceed u64
    pub(crate) fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Push an error diagnostic with the given code and message.
    fn push_error(&mut self, code: DiagnosticCode, message: String) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code,
            message,
            hint: error_hint(code),
            path: None,
            error_class: None,
        });
    }
}

/// A sandboxed JavaScript execution environment.
///
/// Each `Sandbox` instance is fresh (per spec §1.4) and can execute
/// a single piece of JavaScript code, collecting logs, diagnostics,
/// and the result value.
pub struct Sandbox {
    runtime: AsyncRuntime,
    context: AsyncContext,
    state: Arc<Mutex<SandboxState>>,
    timeout_ms: Option<u64>,
    snapshot: Arc<ToolSnapshot>,
    servers: Arc<HashMap<String, Arc<dyn McpServer>>>,
}

impl Sandbox {
    /// Create a new sandbox with the given execution limits and servers.
    ///
    /// Pre-fetches tool definitions from all servers to build a frozen
    /// snapshot for module resolution and discovery.
    ///
    /// # Errors
    ///
    /// Returns `CodemodeError` if the `QuickJS` runtime/context cannot be created
    /// or if the tool snapshot build fails.
    pub async fn new(
        limits: Option<&Limits>,
        servers: &[Arc<dyn McpServer>],
    ) -> Result<Self, CodemodeError> {
        Self::with_dirty_servers(limits, servers, &HashSet::new()).await
    }

    /// Create a new sandbox, refreshing dirty servers before building the snapshot.
    ///
    /// Servers whose IDs appear in `dirty_servers` **and** that support
    /// `tools/listChanged` will have `refresh_tools()` called before the tool
    /// snapshot is built (§8.1). The snapshot is then frozen for the lifetime
    /// of this sandbox (§8.2).
    ///
    /// # Errors
    ///
    /// Returns `CodemodeError` if refresh, runtime creation, or snapshot build fails.
    pub async fn with_dirty_servers(
        limits: Option<&Limits>,
        servers: &[Arc<dyn McpServer>],
        dirty_servers: &HashSet<String>,
    ) -> Result<Self, CodemodeError> {
        let runtime = AsyncRuntime::new().map_err(|e| CodemodeError::QuickJs(e.to_string()))?;

        // Configure memory limit on the runtime (timeout is deferred to execute)
        limits::configure_memory_limit(&runtime, limits).await;

        // §8.1: Refresh dirty servers before building the snapshot.
        // Only servers that both (a) support listChanged and (b) are in the
        // dirty set are refreshed.
        for server in servers {
            if server.supports_list_changed() && dirty_servers.contains(server.server_id()) {
                server.refresh_tools().await?;
            }
        }

        // Build tool snapshot from servers (§8.2: frozen for this invocation)
        let snapshot = if servers.is_empty() {
            Arc::new(ToolSnapshot::empty())
        } else {
            Arc::new(ToolSnapshot::build(servers).await?)
        };

        // Build server map keyed by normalized_id
        let server_map: HashMap<String, Arc<dyn McpServer>> = snapshot
            .servers
            .iter()
            .zip(servers.iter())
            .map(|(toolset, server)| (toolset.normalized_id.clone(), Arc::clone(server)))
            .collect();
        let server_map = Arc::new(server_map);

        // Register module resolver/loader for @codemode/* imports
        let resolver = CodemodeResolver::new(&snapshot);
        let loader = CodemodeLoader::new(&snapshot);
        runtime.set_loader(resolver, loader).await;

        let context = AsyncContext::full(&runtime)
            .await
            .map_err(|e| CodemodeError::QuickJs(e.to_string()))?;

        let state = Arc::new(Mutex::new(SandboxState::new(limits)));
        let timeout_ms = limits.and_then(|l| l.timeout_ms);

        Ok(Self {
            runtime,
            context,
            state,
            timeout_ms,
            snapshot,
            servers: server_map,
        })
    }

    /// Execute JavaScript code in the sandbox and return the structured response.
    ///
    /// Code is evaluated with ES module semantics (supporting `import`/`export`
    /// and top-level `await`) per spec §3.2.1. Script errors are captured as
    /// diagnostics rather than propagated (per spec §3.3.4).
    ///
    /// The result is determined by precedence: `export default` > `globalThis.__codemode_result__` > `null`.
    ///
    /// # Panics
    ///
    /// Panics if the internal sandbox state mutex is poisoned.
    pub async fn execute(&self, code: &str) -> RunResponse {
        // Install timeout handler at execution start (not construction) so the
        // deadline is measured from when code actually runs.
        limits::install_timeout_handler(&self.runtime, self.timeout_ms).await;

        // Reset the start time to execution start
        {
            let mut s = self.state.lock().expect("sandbox state lock");
            s.start_time = Instant::now();
        }

        let state = Arc::clone(&self.state);
        let snapshot = Arc::clone(&self.snapshot);
        let servers = Arc::clone(&self.servers);
        let code = code.to_string();

        // Shared slot to pass the result out of the async_with! block.
        // The Module is lifetime-bound to Ctx, so extraction must happen inside.
        let result_slot: Arc<Mutex<serde_json::Value>> =
            Arc::new(Mutex::new(serde_json::Value::Null));
        let result_slot_inner = Arc::clone(&result_slot);

        // Use async_with! which drives the QuickJS job queue (including microtask-based
        // setTimeout callbacks) while the inner future is Pending.
        async_with!(self.context => |ctx| {
            // Set up globals (console, __codemode_result__, polyfills, host bridge, strip banned globals)
            if let Err(e) = globals::setup_globals(&ctx, &state, &snapshot, &servers) {
                let msg = format!("Failed to set up sandbox globals: {e}");
                let code = if is_sandbox_limit_message(&msg) {
                    DiagnosticCode::SandboxLimit
                } else {
                    DiagnosticCode::UncaughtException
                };
                let mut s = state.lock().expect("sandbox state lock");
                s.push_error(code, msg);
                return;
            }

            // Declare and evaluate as an ES module (supports import/export and top-level await).
            // Using declare + eval instead of Module::evaluate so we retain the Module
            // and can inspect `export default`.
            let mut evaluated_module = None;

            match Module::declare(ctx.clone(), "<main>", &*code).catch(&ctx) {
                Ok(declared) => {
                    match declared.eval().catch(&ctx) {
                        Ok((module, promise)) => {
                            // Await the module evaluation promise (drives top-level await)
                            if let Err(err) = promise.into_future::<Value>().await.catch(&ctx) {
                                let has_mem_limit = {
                                    let s = state.lock().expect("sandbox state lock");
                                    s.has_memory_limit
                                };
                                let (diag_code, message) = classify_js_error(&err, has_mem_limit);
                                let mut s = state.lock().expect("sandbox state lock");
                                s.push_error(diag_code, message);
                            } else {
                                evaluated_module = Some(module);
                            }
                        }
                        Err(err) => {
                            let has_mem_limit = {
                                let s = state.lock().expect("sandbox state lock");
                                s.has_memory_limit
                            };
                            let (diag_code, message) = classify_js_error(&err, has_mem_limit);
                            let mut s = state.lock().expect("sandbox state lock");
                            s.push_error(diag_code, message);
                        }
                    }
                }
                Err(err) => {
                    let has_mem_limit = {
                        let s = state.lock().expect("sandbox state lock");
                        s.has_memory_limit
                    };
                    let (diag_code, message) = classify_js_error(&err, has_mem_limit);
                    let mut s = state.lock().expect("sandbox state lock");
                    s.push_error(diag_code, message);
                }
            }

            // Drain the microtask queue by awaiting a deferred promise.
            // This allows setTimeout callbacks (queued via Promise.resolve().then())
            // to fire before we extract results.
            if let Ok(drain) = ctx.eval::<Promise, _>("Promise.resolve().then(() => {})") {
                drain.into_future::<Value>().await.ok();
            }

            // Extract result inside async_with! because Module is lifetime-bound to Ctx.
            // Precedence: export default > globalThis.__codemode_result__ > null
            let result = extract_result(&ctx, evaluated_module.as_ref());
            let mut slot = result_slot_inner.lock().expect("result slot lock");
            *slot = result;
        })
        .await;

        let result = {
            let slot = result_slot.lock().expect("result slot lock");
            slot.clone()
        };

        // Build response from collected state
        let s = self.state.lock().expect("sandbox state lock");
        RunResponse {
            logs: s.logs.clone(),
            result,
            diagnostics: s.diagnostics.clone(),
            tool_trace: if s.tool_trace.is_empty() {
                None
            } else {
                Some(s.tool_trace.clone())
            },
        }
    }
}

/// Convert a JS `Value` to a `serde_json::Value` via `JSON.stringify`.
fn value_to_json<'js>(ctx: &Ctx<'js>, val: Value<'js>) -> serde_json::Value {
    if val.is_null() || val.is_undefined() {
        return serde_json::Value::Null;
    }

    match ctx.json_stringify(val) {
        Ok(Some(js_str)) => {
            let s = js_str.to_string();
            match s {
                Ok(rust_str) => serde_json::from_str(&rust_str).unwrap_or(serde_json::Value::Null),
                Err(_) => serde_json::Value::Null,
            }
        }
        _ => serde_json::Value::Null,
    }
}

/// Extract the result from execution.
///
/// Precedence: `export default` > `globalThis.__codemode_result__` > `null`.
fn extract_result<'js>(
    ctx: &Ctx<'js>,
    module: Option<&Module<'js, Evaluated>>,
) -> serde_json::Value {
    // 1. Check for `export default` (skip if undefined — means not set)
    if let Some(module) = module
        && let Ok(val) = module.get::<_, Value>("default")
        && !val.is_undefined()
    {
        return value_to_json(ctx, val);
    }

    // 2. Fall back to globalThis.__codemode_result__
    let globals = ctx.globals();
    let val: Value = match globals.get("__codemode_result__") {
        Ok(v) => v,
        Err(_) => return serde_json::Value::Null,
    };

    value_to_json(ctx, val)
}

/// Classify a JS error into a diagnostic code and message.
///
/// `has_memory_limit` indicates whether a memory limit was configured on
/// the runtime. When `true`, bare `throw null`/`throw undefined` values
/// are attributed to OOM (`QuickJS` throws `null` when it cannot allocate
/// an Error object). When `false`, they are treated as user-thrown values.
fn classify_js_error(err: &CaughtError<'_>, has_memory_limit: bool) -> (DiagnosticCode, String) {
    match err {
        CaughtError::Exception(exc) => {
            // Check the JS error's `name` property (e.g. "SyntaxError", "TypeError")
            let name: Option<String> = exc
                .as_object()
                .get::<_, Option<String>>("name")
                .ok()
                .flatten();
            let message = exc.message().unwrap_or_else(|| err.to_string());

            let code = match name.as_deref() {
                Some("SyntaxError") => DiagnosticCode::SyntaxError,
                Some("InternalError") => DiagnosticCode::SandboxLimit,
                _ if is_sandbox_limit_message(&message) => DiagnosticCode::SandboxLimit,
                _ if is_import_failure_message(&message) => DiagnosticCode::ImportFailure,
                _ => DiagnosticCode::UncaughtException,
            };

            let display = if let Some(name) = &name {
                format!("{name}: {message}")
            } else {
                message
            };
            (code, display)
        }
        CaughtError::Error(err) => {
            let message = err.to_string();
            let code = if is_sandbox_limit_message(&message) {
                DiagnosticCode::SandboxLimit
            } else if is_import_failure_message(&message) {
                DiagnosticCode::ImportFailure
            } else {
                DiagnosticCode::UncaughtException
            };
            (code, message)
        }
        CaughtError::Value(val) => {
            if val.is_null() || val.is_undefined() {
                // QuickJS throws null when it cannot allocate an Error object
                // (OOM). But user code can also `throw null` / `throw undefined`.
                // Only attribute to OOM when a memory limit is configured.
                if has_memory_limit {
                    return (DiagnosticCode::SandboxLimit, "out of memory".to_string());
                }
                return (
                    DiagnosticCode::UncaughtException,
                    "Thrown value: null".to_string(),
                );
            }
            let message = format!("Thrown value: {val:?}");
            (DiagnosticCode::UncaughtException, message)
        }
    }
}

/// Check if an error message indicates a failed module import.
///
/// rquickjs module resolution/loading failures produce `CaughtError::Error`
/// with messages like "Could not resolve module '...'" or "Could not load module '...'".
fn is_import_failure_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("could not resolve")
        || lower.contains("could not load")
        || lower.contains("error resolving module")
        || lower.contains("error loading module")
}

/// Check if an error message indicates a sandbox resource limit was hit.
fn is_sandbox_limit_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("interrupted")
        || lower.contains("out of memory")
        || lower.contains("stack overflow")
        || lower.contains("memory limit")
}

/// Provide a hint for common error types.
fn error_hint(code: DiagnosticCode) -> Option<String> {
    match code {
        DiagnosticCode::SyntaxError => {
            Some("Check your JavaScript syntax and ensure valid ES module code.".into())
        }
        DiagnosticCode::ImportFailure => Some(
            "Check that the module specifier is correct. Available modules: @codemode/discovery, @codemode/errors, @codemode/servers/<id>.".into(),
        ),
        DiagnosticCode::SandboxLimit => {
            Some("Reduce code complexity or increase the execution limits.".into())
        }
        DiagnosticCode::UncaughtException | DiagnosticCode::CapabilityUnavailable => None,
    }
}
