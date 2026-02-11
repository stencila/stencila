mod console;
mod globals;
mod limits;
mod polyfills;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Promise, Value, async_with,
};

use crate::error::CodemodeError;
use crate::types::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, Limits, LogEntry, RunResponse, ToolTraceEntry,
};

/// Shared mutable state for the sandbox execution.
///
/// Accessed by the console capture, tool call tracking, and limit enforcement.
#[allow(dead_code)] // tool_call_count and max_tool_calls used in later phases
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
        }
    }

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
}

impl Sandbox {
    /// Create a new sandbox with the given execution limits.
    pub async fn new(limits: Option<&Limits>) -> Result<Self, CodemodeError> {
        let runtime = AsyncRuntime::new().map_err(|e| CodemodeError::QuickJs(e.to_string()))?;

        // Configure memory limit on the runtime (timeout is deferred to execute)
        limits::configure_memory_limit(&runtime, limits).await;

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
        })
    }

    /// Execute JavaScript code in the sandbox and return the structured response.
    ///
    /// This always returns a `RunResponse` — script errors are captured as
    /// diagnostics rather than propagated (per spec §3.3.4).
    ///
    /// Uses `async_with!` to drive the QuickJS job queue (including setTimeout
    /// callbacks backed by `Promised` futures).
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
        let code = code.to_string();

        // Use async_with! which drives the QuickJS job queue (including microtask-based
        // setTimeout callbacks) while the inner future is Pending.
        async_with!(self.context => |ctx| {
            // Set up globals (console, __codemode_result__, polyfills, strip banned globals)
            if let Err(e) = globals::setup_globals(&ctx, &state) {
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

            // Evaluate the code
            match ctx.eval::<Value, _>(&*code).catch(&ctx) {
                Ok(_) => {}
                Err(err) => {
                    let (diag_code, message) = classify_js_error(&err);
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
        })
        .await;

        // Extract __codemode_result__ (after pending jobs have completed)
        let result = self.context.with(|ctx| extract_result(&ctx)).await;

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

/// Extract `globalThis.__codemode_result__` and convert to a JSON value.
fn extract_result(ctx: &Ctx<'_>) -> serde_json::Value {
    let globals = ctx.globals();
    let val: Value = match globals.get("__codemode_result__") {
        Ok(v) => v,
        Err(_) => return serde_json::Value::Null,
    };

    if val.is_null() || val.is_undefined() {
        return serde_json::Value::Null;
    }

    // Use JSON.stringify to convert the JS value to a JSON string,
    // then parse it into serde_json::Value
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

/// Classify a JS error into a diagnostic code and message.
fn classify_js_error(err: &CaughtError<'_>) -> (DiagnosticCode, String) {
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
            } else {
                DiagnosticCode::UncaughtException
            };
            (code, message)
        }
        CaughtError::Value(val) => {
            let message = format!("Thrown value: {val:?}");
            (DiagnosticCode::UncaughtException, message)
        }
    }
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
        DiagnosticCode::SandboxLimit => {
            Some("Reduce code complexity or increase the execution limits.".into())
        }
        _ => None,
    }
}
