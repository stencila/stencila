//! Shared utilities for handler implementations.
//!
//! Functions in this module are used by multiple handlers (e.g.
//! codergen and shell) for runtime variable expansion, output
//! truncation, and building success outcomes with standard context
//! updates.

use indexmap::IndexMap;

use crate::context::{Context, ctx};
use crate::error::{AttractorError, AttractorResult};
use crate::graph::{AttrValue, Node, attr};
use crate::types::Outcome;

/// Maximum length for the truncated response stored in context updates.
const RESPONSE_TRUNCATION_LIMIT: usize = 200;

/// Truncate a string to the limit, appending `...` if truncated.
///
/// Finds the last char boundary at or before the limit to avoid
/// panicking on multi-byte UTF-8.
fn truncate_output(s: &str) -> String {
    if s.len() <= RESPONSE_TRUNCATION_LIMIT {
        s.to_string()
    } else {
        // Find the last char boundary at or before the byte limit.
        let boundary = s
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= RESPONSE_TRUNCATION_LIMIT)
            .last()
            .unwrap_or(0);
        let mut truncated = s[..boundary].to_string();
        truncated.push_str("...");
        truncated
    }
}

/// Build a success outcome that stores output text in standard context keys.
///
/// Sets `last_stage`, `last_output` (truncated), `last_output_full`,
/// and accumulates the node into `completed_stages`. Callers can insert
/// additional handler-specific keys into [`Outcome::context_updates`]
/// after this returns.
#[must_use]
pub fn build_output_outcome(node_id: &str, output: &str, context: &Context) -> Outcome {
    let mut outcome = Outcome::success();
    outcome.context_updates = IndexMap::new();
    outcome.context_updates.insert(
        ctx::LAST_STAGE.to_string(),
        serde_json::Value::String(node_id.to_string()),
    );
    outcome.context_updates.insert(
        ctx::LAST_OUTPUT.to_string(),
        serde_json::Value::String(truncate_output(output)),
    );
    outcome.context_updates.insert(
        ctx::LAST_OUTPUT_FULL.to_string(),
        serde_json::Value::String(output.to_string()),
    );

    let mut stages: Vec<serde_json::Value> = context
        .get(ctx::COMPLETED_STAGES)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    stages.push(serde_json::json!({"id": node_id, "status": "success"}));
    outcome.context_updates.insert(
        ctx::COMPLETED_STAGES.to_string(),
        serde_json::Value::Array(stages),
    );

    outcome
}

/// Resolve a node's optional `timeout` attribute as a duration.
///
/// Accepts both parsed DOT duration values (e.g. `timeout=250ms`) and quoted
/// duration strings (e.g. `timeout="15m"`). Invalid string values return a
/// pipeline error instead of silently disabling timeout enforcement.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidPipeline`] if a string timeout cannot be
/// parsed, or if the attribute uses an unsupported value type.
pub fn resolve_timeout_attr(node: &Node) -> AttractorResult<Option<std::time::Duration>> {
    let Some(value) = node.get_attr(attr::TIMEOUT) else {
        return Ok(None);
    };

    match value {
        AttrValue::Duration(duration) => Ok(Some(duration.inner())),
        AttrValue::String(spec) => crate::types::Duration::from_spec_str(spec)
            .map(|duration| Some(duration.inner()))
            .map_err(|error| AttractorError::InvalidPipeline {
                reason: format!("node '{}': invalid timeout attribute: {error}", node.id),
            }),
        other => Err(AttractorError::InvalidPipeline {
            reason: format!(
                "node '{}': timeout attribute has unsupported type '{}'; expected duration string",
                node.id,
                other.type_name()
            ),
        }),
    }
}
