use std::sync::{Arc, Mutex};

use rquickjs::Ctx;

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

/// Set up all sandbox globals.
///
/// This configures the sandbox environment in the correct order:
/// 1. Inject `__codemode_result__` (initially null)
/// 2. Inject console capture
/// 3. Inject polyfills (URL, URLSearchParams, TextEncoder, TextDecoder)
/// 4. Inject setTimeout/clearTimeout
/// 5. Strip banned globals (eval, Function constructor)
pub(super) fn setup_globals(
    ctx: &Ctx<'_>,
    state: &Arc<Mutex<SandboxState>>,
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

    // 4. Inject setTimeout/clearTimeout
    ctx.eval::<(), _>(SET_TIMEOUT_JS)?;

    // 5. Strip banned globals
    ctx.eval::<(), _>(STRIP_GLOBALS_JS)?;

    Ok(())
}
