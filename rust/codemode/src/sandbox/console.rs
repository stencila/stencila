use std::sync::{Arc, Mutex};

use rquickjs::{Ctx, function::Func};

use crate::types::{LogEntry, LogLevel};

use super::SandboxState;

/// JavaScript source that defines the `console` object.
///
/// The JS console serializes each argument (using `String(value)` for
/// primitives, `JSON.stringify` for objects, with `[Unserializable Object]`
/// fallback) and calls the injected `__codemode_record_log` Rust function.
const CONSOLE_JS: &str = r#"
(function() {
    const _record = globalThis.__codemode_record_log;
    delete globalThis.__codemode_record_log;

    function serialize(value) {
        if (value === null) return 'null';
        if (value === undefined) return 'undefined';
        if (typeof value === 'object' || typeof value === 'function') {
            try { return JSON.stringify(value); }
            catch (e) { return '[Unserializable Object]'; }
        }
        return String(value);
    }

    function makeLogger(level) {
        return function(...args) {
            _record(level, args.map(serialize).join(' '));
        };
    }

    globalThis.console = Object.freeze({
        log: makeLogger('log'),
        debug: makeLogger('debug'),
        warn: makeLogger('warn'),
        error: makeLogger('error'),
    });
})();
"#;

/// Inject the console capture system into the sandbox.
///
/// This injects a Rust function `__codemode_record_log` that captures
/// log entries into `SandboxState`, then evaluates `CONSOLE_JS` which
/// builds the `console` object using that function.
pub(super) fn inject_console(
    ctx: &Ctx<'_>,
    state: &Arc<Mutex<SandboxState>>,
) -> Result<(), rquickjs::Error> {
    let state = Arc::clone(state);
    let record_log = Func::from(move |level: String, message: String| {
        let mut s = state.lock().expect("sandbox state lock");

        // Check byte limit
        let bytes = message.len() as u64;
        if let Some(max) = s.max_log_bytes {
            if s.log_truncated {
                return;
            }
            if s.log_bytes + bytes > max {
                s.log_truncated = true;
                let time_ms = s.elapsed_ms();
                s.logs.push(LogEntry {
                    level: LogLevel::Warn,
                    message: format!("Log output truncated at {max} byte limit"),
                    time_ms,
                });
                return;
            }
        }

        s.log_bytes += bytes;
        let time_ms = s.elapsed_ms();
        let log_level = match level.as_str() {
            "debug" => LogLevel::Debug,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Log,
        };
        s.logs.push(LogEntry {
            level: log_level,
            message,
            time_ms,
        });
    });

    ctx.globals().set("__codemode_record_log", record_log)?;

    // Evaluate the console wrapper JS
    ctx.eval::<(), _>(CONSOLE_JS)?;

    Ok(())
}
