use std::collections::HashSet;
use std::sync::Arc;

use crate::sandbox::Sandbox;
use crate::traits::McpServer;
use crate::types::{Diagnostic, DiagnosticCode, DiagnosticSeverity, RunRequest, RunResponse};

/// Execute JavaScript code in a fresh codemode sandbox.
///
/// This is the top-level entry point for `codemode.run` invocations.
/// It **always** returns a [`RunResponse`] — errors during sandbox creation,
/// tool refresh, or code execution are captured as diagnostics rather
/// than propagated (per spec §3.3.4).
///
/// # Capabilities checking (§3.2.3)
///
/// If `request.requested_capabilities` is set, each capability is checked
/// against the `capabilities()` advertised by the provided servers. A
/// warning diagnostic is emitted for any capability not matched by at
/// least one server.
///
/// # Dirty server refresh (§8.1)
///
/// Servers whose IDs appear in `dirty_servers` and that support
/// `tools/listChanged` are refreshed before the tool snapshot is built.
/// Pass an empty set when no servers have changed.
#[allow(clippy::implicit_hasher)]
pub async fn codemode_run(
    request: &RunRequest,
    servers: &[Arc<dyn McpServer>],
    dirty_servers: &HashSet<String>,
) -> RunResponse {
    let mut early_diagnostics = Vec::new();

    // §3.2.3: Check requestedCapabilities against available servers.
    if let Some(requested) = &request.requested_capabilities {
        check_capabilities(requested, servers, &mut early_diagnostics);
    }

    // Build sandbox (may fail — absorbed into diagnostics per §3.3.4).
    let sandbox =
        match Sandbox::with_dirty_servers(request.limits.as_ref(), servers, dirty_servers).await {
            Ok(sb) => sb,
            Err(e) => {
                early_diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode::UncaughtException,
                    message: format!("Failed to create sandbox: {e}"),
                    hint: Some("Check server connectivity and configuration.".into()),
                    path: None,
                    error_class: None,
                });
                return RunResponse {
                    diagnostics: early_diagnostics,
                    ..RunResponse::default()
                };
            }
        };

    // Execute the code.
    let mut response = sandbox.execute(&request.code).await;

    // Prepend any early diagnostics (capability warnings) before execution diagnostics.
    if !early_diagnostics.is_empty() {
        early_diagnostics.append(&mut response.diagnostics);
        response.diagnostics = early_diagnostics;
    }

    response
}

/// Check `requestedCapabilities` against server capabilities (§3.2.3).
///
/// Emits a warning diagnostic for each requested capability that is not
/// advertised by any of the provided servers.
fn check_capabilities(
    requested: &[String],
    servers: &[Arc<dyn McpServer>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let available: HashSet<String> = servers
        .iter()
        .filter_map(|s| s.capabilities())
        .flatten()
        .collect();

    for cap in requested {
        if !available.contains(cap) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: DiagnosticCode::CapabilityUnavailable,
                message: format!(
                    "Requested capability '{cap}' is not provided by any connected server"
                ),
                hint: Some(
                    "Check that the required server is connected and advertises this capability."
                        .into(),
                ),
                path: None,
                error_class: None,
            });
        }
    }
}
