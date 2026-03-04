//! Tool guard framework: a friction layer between agent tool calls and tool
//! execution. Dispatches to shell, file, and web evaluators based on tool name.
//!
//! See `tool-guards-spec.md` for the full specification.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

pub mod audit;
pub mod file;
pub mod paths;
pub mod shell;
pub mod web;

// ---------------------------------------------------------------------------
// TrustLevel
// ---------------------------------------------------------------------------

/// Trust level controlling how strictly an agent's operations are guarded.
///
/// - `Low`: shell is default-deny; file and web rules apply strictest verdicts.
/// - `Medium` (default): default-allow with destructive behavior blocking.
/// - `High`: default-allow with relaxed blocking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustLevel {
    Low,
    Medium,
    High,
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::Medium
    }
}

impl TrustLevel {
    /// Parse from the schema's `trust_level` string field.
    ///
    /// Returns `Medium` for `None` or unrecognized values.
    pub fn from_schema(value: Option<&str>) -> Self {
        match value {
            Some("low") => Self::Low,
            Some("high") => Self::High,
            _ => Self::Medium,
        }
    }
}

// ---------------------------------------------------------------------------
// GuardVerdict
// ---------------------------------------------------------------------------

/// Result of evaluating a tool call against guard rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardVerdict {
    Allow,
    Warn {
        reason: &'static str,
        suggestion: &'static str,
        rule_id: &'static str,
    },
    Deny {
        reason: &'static str,
        suggestion: &'static str,
        rule_id: &'static str,
    },
}

/// Return the strictest of two verdicts (Deny > Warn > Allow).
///
/// When both have the same severity, the first (`a`) wins — this preserves
/// table-order / registration-order tie-breaking used by all sub-guards.
pub(crate) fn strictest_verdict(a: GuardVerdict, b: GuardVerdict) -> GuardVerdict {
    match (&a, &b) {
        (GuardVerdict::Deny { .. }, _) => a,
        (_, GuardVerdict::Deny { .. }) => b,
        (GuardVerdict::Warn { .. }, _) => a,
        (_, GuardVerdict::Warn { .. }) => b,
        _ => a,
    }
}

// ---------------------------------------------------------------------------
// GuardContext
// ---------------------------------------------------------------------------

/// Per-session context for guard evaluation and audit attribution.
#[derive(Debug, Clone)]
pub struct GuardContext {
    pub session_id: Arc<str>,
    pub agent_name: Arc<str>,
}

impl GuardContext {
    /// Create a new guard context.
    pub fn new(session_id: impl Into<Arc<str>>, agent_name: impl Into<Arc<str>>) -> Self {
        Self {
            session_id: session_id.into(),
            agent_name: agent_name.into(),
        }
    }

    /// Fallback context used when a guard is present but no explicit context
    /// was provided. Ensures enforcement is never silently skipped due to
    /// missing attribution metadata.
    pub(crate) fn fallback() -> Self {
        Self {
            session_id: Arc::from("unknown"),
            agent_name: Arc::from("unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// ToolGuard
// ---------------------------------------------------------------------------

/// Unified guard policy object that dispatches to per-domain evaluators.
///
/// Shared via `Arc` across parent and child sessions. The workspace root is
/// fixed at construction and cannot be widened by child agents.
pub struct ToolGuard {
    trust_level: TrustLevel,
    #[allow(dead_code)]
    workspace_root: PathBuf,
    shell_guard: shell::ShellToolGuard,
    file_guard: file::FileToolGuard,
    web_guard: web::WebToolGuard,
    audit_tx: Option<audit::AuditSender>,
}

impl std::fmt::Debug for ToolGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolGuard")
            .field("trust_level", &self.trust_level)
            .field("workspace_root", &self.workspace_root)
            .finish()
    }
}

const SHELL_TOOL: &str = "shell";

const FILE_TOOLS: &[&str] = &[
    "read_file",
    "read_many_files",
    "write_file",
    "edit_file",
    "apply_patch",
    "grep",
];

const WEB_TOOL: &str = "web_fetch";

impl ToolGuard {
    /// Construct a new guard policy.
    ///
    /// `allowed_domains` and `disallowed_domains` configure the web guard.
    pub fn new(
        trust_level: TrustLevel,
        workspace_root: PathBuf,
        allowed_domains: Option<Vec<String>>,
        disallowed_domains: Option<Vec<String>>,
    ) -> Self {
        let file_guard = file::FileToolGuard::new(workspace_root.clone());
        let web_guard = web::WebToolGuard::new(allowed_domains, disallowed_domains);

        let audit_tx = audit::spawn_audit_writer(&workspace_root);

        Self {
            trust_level,
            workspace_root,
            shell_guard: shell::ShellToolGuard,
            file_guard,
            web_guard,
            audit_tx,
        }
    }

    /// Evaluate a tool call against guard rules.
    ///
    /// Dispatches to the appropriate sub-guard based on `tool_name`.
    /// Returns the strictest verdict across all evaluated segments.
    /// Non-`Allow` verdicts are recorded to the audit database.
    pub fn evaluate(
        &self,
        _context: &GuardContext,
        tool_name: &str,
        args: &Value,
        working_dir: &std::path::Path,
    ) -> GuardVerdict {
        let verdict = self.evaluate_inner(tool_name, args, working_dir);

        if !matches!(verdict, GuardVerdict::Allow) {
            self.emit_audit_event(_context, tool_name, args, working_dir, &verdict);
        }

        verdict
    }

    fn evaluate_inner(
        &self,
        tool_name: &str,
        args: &Value,
        working_dir: &std::path::Path,
    ) -> GuardVerdict {
        if tool_name == SHELL_TOOL {
            if let Some(command) = args.get("command").and_then(|v| v.as_str()) {
                return self.shell_guard.evaluate(command, self.trust_level);
            }
        }

        if FILE_TOOLS.contains(&tool_name) {
            return self
                .file_guard
                .evaluate(tool_name, args, working_dir, self.trust_level);
        }

        if tool_name == WEB_TOOL {
            if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
                return self.web_guard.evaluate(url, self.trust_level);
            }
        }

        GuardVerdict::Allow
    }

    fn emit_audit_event(
        &self,
        context: &GuardContext,
        tool_name: &str,
        args: &Value,
        working_dir: &std::path::Path,
        verdict: &GuardVerdict,
    ) {
        let audit_tx = match &self.audit_tx {
            Some(tx) => tx,
            None => return,
        };

        let (verdict_str, rule_id, reason, suggestion) = match verdict {
            GuardVerdict::Deny {
                rule_id,
                reason,
                suggestion,
            } => ("Deny", *rule_id, *reason, *suggestion),
            GuardVerdict::Warn {
                rule_id,
                reason,
                suggestion,
            } => ("Warn", *rule_id, *reason, *suggestion),
            GuardVerdict::Allow => return,
        };

        let (input, matched_segment) =
            self.extract_audit_fields(tool_name, args, working_dir, verdict);

        audit_tx.send(audit::AuditEvent {
            session_id: context.session_id.to_string(),
            agent_name: context.agent_name.to_string(),
            trust_level: format!("{:?}", self.trust_level).to_lowercase(),
            tool_name: tool_name.to_string(),
            input,
            matched_segment,
            verdict: verdict_str,
            rule_id,
            reason,
            suggestion,
        });
    }

    /// Extract `(input, matched_segment)` for an audit event.
    ///
    /// For file tools, delegates to the file guard's `audit_segment` method
    /// which re-evaluates each path to find the first normalized path that
    /// produced the decisive verdict (spec §9.1).
    fn extract_audit_fields(
        &self,
        tool_name: &str,
        args: &Value,
        working_dir: &std::path::Path,
        verdict: &GuardVerdict,
    ) -> (String, String) {
        match tool_name {
            "shell" => {
                let command = args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<missing>");
                (command.to_string(), command.to_string())
            }

            "web_fetch" => {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<missing>");
                (url.to_string(), url.to_string())
            }

            // File tools: matched_segment comes from the file guard which
            // knows about normalization and multi-target decisive path logic.
            // Input serialization varies by tool per spec §9.1.
            _ if FILE_TOOLS.contains(&tool_name) => {
                let segment = self.file_guard.audit_segment(
                    tool_name,
                    args,
                    working_dir,
                    self.trust_level,
                    verdict,
                );
                let input = match tool_name {
                    "read_many_files" => match args.get("paths").and_then(|v| v.as_array()) {
                        Some(arr) => {
                            let strs: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                            serde_json::to_string(&strs).unwrap_or_default()
                        }
                        None => args.to_string(),
                    },
                    "apply_patch" => args
                        .get("patch")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    "grep" => segment.clone(),
                    // read_file, write_file, edit_file
                    _ => args
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>")
                        .to_string(),
                };
                (input, segment)
            }

            _ => {
                let fallback = args.to_string();
                (fallback.clone(), fallback)
            }
        }
    }

    /// The trust level this guard was constructed with.
    pub fn trust_level(&self) -> TrustLevel {
        self.trust_level
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_level_default_is_medium() {
        assert_eq!(TrustLevel::default(), TrustLevel::Medium);
    }

    #[test]
    fn trust_level_from_schema() {
        assert_eq!(TrustLevel::from_schema(None), TrustLevel::Medium);
        assert_eq!(TrustLevel::from_schema(Some("low")), TrustLevel::Low);
        assert_eq!(TrustLevel::from_schema(Some("medium")), TrustLevel::Medium);
        assert_eq!(TrustLevel::from_schema(Some("high")), TrustLevel::High);
        assert_eq!(TrustLevel::from_schema(Some("bogus")), TrustLevel::Medium);
    }

    #[test]
    fn evaluate_returns_allow_for_unknown_tool() {
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/tmp"), None, None);
        let ctx = GuardContext::new("test-session", "test-agent");
        let verdict = guard.evaluate(
            &ctx,
            "unknown_tool",
            &serde_json::json!({}),
            std::path::Path::new("/tmp"),
        );
        assert_eq!(verdict, GuardVerdict::Allow);
    }

    #[test]
    fn evaluate_dispatches_shell_tool() {
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/tmp"), None, None);
        let ctx = GuardContext::new("test-session", "test-agent");
        let verdict = guard.evaluate(
            &ctx,
            "shell",
            &serde_json::json!({"command": "rm -rf /"}),
            std::path::Path::new("/tmp"),
        );
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));
    }

    #[test]
    fn evaluate_dispatches_web_fetch() {
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/tmp"), None, None);
        let ctx = GuardContext::new("test-session", "test-agent");

        // Metadata endpoint should be denied
        let verdict = guard.evaluate(
            &ctx,
            "web_fetch",
            &serde_json::json!({"url": "http://169.254.169.254/latest/meta-data/iam/security-credentials/"}),
            std::path::Path::new("/tmp"),
        );
        assert!(matches!(
            verdict,
            GuardVerdict::Deny {
                rule_id: "web.credential_url",
                ..
            }
        ));

        // Normal HTTPS URL should be allowed
        let verdict = guard.evaluate(
            &ctx,
            "web_fetch",
            &serde_json::json!({"url": "https://example.com/"}),
            std::path::Path::new("/tmp"),
        );
        assert_eq!(verdict, GuardVerdict::Allow);

        // HTTP URL should warn at medium
        let verdict = guard.evaluate(
            &ctx,
            "web_fetch",
            &serde_json::json!({"url": "http://example.com/"}),
            std::path::Path::new("/tmp"),
        );
        assert!(matches!(
            verdict,
            GuardVerdict::Warn {
                rule_id: "web.non_https",
                ..
            }
        ));
    }

    #[test]
    fn evaluate_dispatches_web_fetch_with_domain_lists() {
        // With allowlist
        let guard = ToolGuard::new(
            TrustLevel::Medium,
            PathBuf::from("/tmp"),
            Some(vec!["docs.rs".to_string()]),
            None,
        );
        let ctx = GuardContext::new("test-session", "test-agent");

        let verdict = guard.evaluate(
            &ctx,
            "web_fetch",
            &serde_json::json!({"url": "https://evil.com/"}),
            std::path::Path::new("/tmp"),
        );
        assert!(matches!(
            verdict,
            GuardVerdict::Deny {
                rule_id: "web.domain_allowlist",
                ..
            }
        ));

        let verdict = guard.evaluate(
            &ctx,
            "web_fetch",
            &serde_json::json!({"url": "https://docs.rs/"}),
            std::path::Path::new("/tmp"),
        );
        assert_eq!(verdict, GuardVerdict::Allow);
    }

    #[test]
    fn evaluate_dispatches_file_tools() {
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/workspace"), None, None);
        let ctx = GuardContext::new("test-session", "test-agent");

        // read_file to system path should be denied
        let verdict = guard.evaluate(
            &ctx,
            "read_file",
            &serde_json::json!({"file_path": "/proc/self/environ"}),
            std::path::Path::new("/workspace"),
        );
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));

        // write_file to system path should be denied
        let verdict = guard.evaluate(
            &ctx,
            "write_file",
            &serde_json::json!({"file_path": "/etc/passwd", "content": "bad"}),
            std::path::Path::new("/workspace"),
        );
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));

        // write_file to workspace path should be allowed
        let verdict = guard.evaluate(
            &ctx,
            "write_file",
            &serde_json::json!({"file_path": "/workspace/src/main.rs", "content": "ok"}),
            std::path::Path::new("/workspace"),
        );
        assert_eq!(verdict, GuardVerdict::Allow);
    }

    // -----------------------------------------------------------------------
    // Audit field extraction tests (fixes for review findings #1–#3)
    // -----------------------------------------------------------------------

    #[test]
    fn audit_read_many_files_decisive_path_is_offending_not_first() {
        // Finding #2: matched_segment must be the first path that produced the
        // strictest verdict, not simply the first path in the array.
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/workspace"), None, None);
        let args = serde_json::json!({
            "paths": ["/workspace/ok.rs", "/proc/self/environ"]
        });
        let verdict =
            guard.evaluate_inner("read_many_files", &args, std::path::Path::new("/workspace"));
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));

        let (input, segment) = guard.extract_audit_fields(
            "read_many_files",
            &args,
            std::path::Path::new("/workspace"),
            &verdict,
        );
        // input should be a JSON array of the raw paths
        assert!(input.contains("/workspace/ok.rs"));
        assert!(input.contains("/proc/self/environ"));
        // matched_segment must be the path that triggered Deny, not the first one
        assert_eq!(segment, "/proc/self/environ");
    }

    #[test]
    fn audit_apply_patch_uses_stencila_format_not_unified_diff() {
        // Finding #1: Stencila patches use `*** Delete File: path` format.
        // The audit must parse this correctly, not return "Delete" as the path.
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/workspace"), None, None);
        let patch = "\
*** Delete File: /etc/shadow
*** Delete File: /workspace/a.rs
*** Delete File: /workspace/b.rs
*** Delete File: /workspace/c.rs
*** Delete File: /workspace/d.rs
*** Delete File: /workspace/e.rs";
        let args = serde_json::json!({ "patch": patch });
        let verdict =
            guard.evaluate_inner("apply_patch", &args, std::path::Path::new("/workspace"));
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));

        let (_input, segment) = guard.extract_audit_fields(
            "apply_patch",
            &args,
            std::path::Path::new("/workspace"),
            &verdict,
        );
        // The decisive path is /etc/shadow (system write), not "Delete"
        assert_eq!(segment, "/etc/shadow");
    }

    #[test]
    fn audit_apply_patch_delete_many_reports_count() {
        // When the decisive rule is apply_patch_delete_many, segment is <delete_count:N>
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/workspace"), None, None);
        let patch = "\
*** Delete File: /workspace/a.rs
*** Delete File: /workspace/b.rs
*** Delete File: /workspace/c.rs
*** Delete File: /workspace/d.rs
*** Delete File: /workspace/e.rs";
        let args = serde_json::json!({ "patch": patch });
        let verdict =
            guard.evaluate_inner("apply_patch", &args, std::path::Path::new("/workspace"));
        assert!(matches!(
            verdict,
            GuardVerdict::Warn {
                rule_id: "file.apply_patch_delete_many",
                ..
            }
        ));

        let (_input, segment) = guard.extract_audit_fields(
            "apply_patch",
            &args,
            std::path::Path::new("/workspace"),
            &verdict,
        );
        assert_eq!(segment, "<delete_count:5>");
    }

    #[test]
    fn audit_file_path_is_normalized() {
        // Finding #3: audit matched_segment must be normalized, not raw.
        let guard = ToolGuard::new(TrustLevel::Medium, PathBuf::from("/workspace"), None, None);
        // Use a relative path with `..` that resolves outside workspace
        let args = serde_json::json!({ "file_path": "../../other-project/data.db" });
        let verdict = guard.evaluate_inner(
            "read_file",
            &args,
            std::path::Path::new("/workspace/subdir"),
        );
        // At medium trust, outside-workspace read is Warn
        assert!(matches!(
            verdict,
            GuardVerdict::Warn {
                rule_id: "file.outside_workspace_read",
                ..
            }
        ));

        let (_input, segment) = guard.extract_audit_fields(
            "read_file",
            &args,
            std::path::Path::new("/workspace/subdir"),
            &verdict,
        );
        // Segment should be normalized absolute path, not raw relative
        assert_eq!(segment, "/other-project/data.db");
    }

    #[test]
    fn audit_grep_missing_path_uses_working_dir_normalized() {
        // Finding #3: grep with no path should use working_dir for audit
        let guard = ToolGuard::new(TrustLevel::Low, PathBuf::from("/workspace"), None, None);
        // grep with no `path` field, working_dir is outside workspace
        let args = serde_json::json!({ "pattern": "secret" });
        let verdict = guard.evaluate_inner("grep", &args, std::path::Path::new("/other"));
        // At low trust, outside-workspace read is Deny
        assert!(matches!(verdict, GuardVerdict::Deny { .. }));

        let (input, segment) =
            guard.extract_audit_fields("grep", &args, std::path::Path::new("/other"), &verdict);
        // Both input and segment should be the resolved working_dir path
        assert_eq!(input, "/other");
        assert_eq!(segment, "/other");
    }
}
