//! Tool guard framework: a friction layer between agent tool calls and tool
//! execution. Dispatches to shell, file, and web evaluators based on tool name.
//!
//! See `tool-guards-spec.md` for the full specification.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

pub mod audit;
pub mod file_guard;
pub mod paths;
pub mod shell;
pub mod web_guard;

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

// ---------------------------------------------------------------------------
// GuardContext
// ---------------------------------------------------------------------------

/// Per-session context for guard evaluation and audit attribution.
#[derive(Debug, Clone)]
pub struct GuardContext {
    pub session_id: Arc<str>,
    pub agent_name: Arc<str>,
}

// ---------------------------------------------------------------------------
// ToolGuard
// ---------------------------------------------------------------------------

/// Unified guard policy object that dispatches to per-domain evaluators.
///
/// Shared via `Arc` across parent and child sessions. The workspace root is
/// fixed at construction and cannot be widened by child agents.
pub struct ToolGuard {
    #[allow(dead_code)]
    trust_level: TrustLevel,
    #[allow(dead_code)]
    workspace_root: PathBuf,
    // Sub-guards will be added in later phases:
    // shell_guard: ShellToolGuard,
    // file_guard: FileToolGuard,
    // web_guard: WebToolGuard,
    // audit_tx: Option<mpsc::Sender<AuditEvent>>,
}

impl std::fmt::Debug for ToolGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolGuard")
            .field("trust_level", &self.trust_level)
            .field("workspace_root", &self.workspace_root)
            .finish()
    }
}

impl ToolGuard {
    /// Construct a new guard policy.
    ///
    /// `trust_level` defaults to `Medium` when `None` is passed.
    /// `allowed_domains` and `disallowed_domains` configure the web guard.
    #[cfg(feature = "tool-guard")]
    pub fn new(
        trust_level: TrustLevel,
        workspace_root: PathBuf,
        _allowed_domains: Option<Vec<String>>,
        _disallowed_domains: Option<Vec<String>>,
    ) -> Self {
        Self {
            trust_level,
            workspace_root,
        }
    }

    /// No-op constructor when the `tool-guard` feature is disabled.
    #[cfg(not(feature = "tool-guard"))]
    pub fn new(
        trust_level: TrustLevel,
        workspace_root: PathBuf,
        _allowed_domains: Option<Vec<String>>,
        _disallowed_domains: Option<Vec<String>>,
    ) -> Self {
        Self {
            trust_level,
            workspace_root,
        }
    }

    /// Evaluate a tool call against guard rules.
    ///
    /// Dispatches to the appropriate sub-guard based on `tool_name`.
    /// Returns the strictest verdict across all evaluated segments.
    ///
    /// When the `tool-guard` feature is disabled, always returns `Allow`.
    pub fn evaluate(
        &self,
        _context: &GuardContext,
        _tool_name: &str,
        _args: &Value,
        _working_dir: &std::path::Path,
    ) -> GuardVerdict {
        #[cfg(not(feature = "tool-guard"))]
        {
            return GuardVerdict::Allow;
        }

        #[cfg(feature = "tool-guard")]
        {
            // Dispatch stub — returns Allow for now.
            // Phases 1–4 will wire shell/file/web evaluators here.
            GuardVerdict::Allow
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
        let ctx = GuardContext {
            session_id: Arc::from("test-session"),
            agent_name: Arc::from("test-agent"),
        };
        let verdict = guard.evaluate(
            &ctx,
            "unknown_tool",
            &serde_json::json!({}),
            std::path::Path::new("/tmp"),
        );
        assert_eq!(verdict, GuardVerdict::Allow);
    }
}
