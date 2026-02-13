//! Validation and linting for pipeline graphs (§7).
//!
//! Provides a diagnostic model, 13 built-in lint rules, and public
//! [`validate`] / [`validate_or_raise`] entry points. Custom lint rules
//! can be registered alongside the built-ins.

pub mod rules;

use crate::error::{AttractorError, AttractorResult};
use crate::graph::Graph;

/// The severity of a validation diagnostic (§7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The pipeline will not execute correctly.
    Error,
    /// The pipeline may behave unexpectedly.
    Warning,
    /// Informational note.
    Info,
}

/// A single validation diagnostic produced by a lint rule (§7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The rule name that produced this diagnostic.
    pub rule: String,
    /// The severity level.
    pub severity: Severity,
    /// A human-readable message.
    pub message: String,
    /// The node ID this diagnostic relates to, if any.
    pub node_id: Option<String>,
    /// The edge (from, to) this diagnostic relates to, if any.
    pub edge: Option<(String, String)>,
    /// A suggested fix, if available.
    pub fix: Option<String>,
}

/// A lint rule that validates a pipeline graph (§7.4).
pub trait LintRule: Send + Sync {
    /// A short identifier for this rule.
    fn name(&self) -> &str;

    /// Run this rule against the graph and return any diagnostics.
    fn apply(&self, graph: &Graph) -> Vec<Diagnostic>;
}

/// Validate a pipeline graph against all built-in rules plus any extra rules.
///
/// Returns all diagnostics found (both errors and warnings).
pub fn validate(graph: &Graph, extra_rules: &[&dyn LintRule]) -> Vec<Diagnostic> {
    let builtins = rules::builtin_rules();
    let mut diagnostics = Vec::new();

    for rule in &builtins {
        diagnostics.extend(rule.apply(graph));
    }
    for rule in extra_rules {
        diagnostics.extend(rule.apply(graph));
    }

    diagnostics
}

/// Validate a pipeline graph and return an error if any ERROR-level diagnostics exist.
///
/// Returns the full list of diagnostics (including warnings) on success.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidPipeline`] if any ERROR-level diagnostic is found.
pub fn validate_or_raise(
    graph: &Graph,
    extra_rules: &[&dyn LintRule],
) -> AttractorResult<Vec<Diagnostic>> {
    let diagnostics = validate(graph, extra_rules);

    let errors: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    if !errors.is_empty() {
        let messages: Vec<String> = errors
            .iter()
            .map(|d| {
                if let Some(ref node_id) = d.node_id {
                    format!("[{}] {} (node: {})", d.rule, d.message, node_id)
                } else if let Some(ref edge) = d.edge {
                    format!(
                        "[{}] {} (edge: {} -> {})",
                        d.rule, d.message, edge.0, edge.1
                    )
                } else {
                    format!("[{}] {}", d.rule, d.message)
                }
            })
            .collect();

        return Err(AttractorError::InvalidPipeline {
            reason: format!(
                "validation failed with {} error(s):\n  {}",
                errors.len(),
                messages.join("\n  ")
            ),
        });
    }

    Ok(diagnostics)
}
