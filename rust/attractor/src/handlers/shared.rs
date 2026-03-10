//! Shared utilities for handler implementations.
//!
//! Functions in this module are used by multiple handlers (e.g.
//! codergen and shell) for runtime variable expansion, output
//! truncation, and building success outcomes with standard context
//! updates.

use indexmap::IndexMap;

use crate::context::Context;
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

/// Expand runtime variables in an input string from context values.
///
/// Expands variables in two phases:
///
/// 1. **Built-in aliases** — `$last_output`, `$last_stage`, and
///    `$last.outcome` are replaced first via simple string substitution.
///    These map to context keys that differ from the variable name
///    (e.g. `$last_output` → `last_output_full`, `$last_outcome` →
///    `outcome`), so they must be handled before the generic expansion.
/// 2. **Context variables** — any remaining `$KEY` references (where KEY
///    starts with a letter or underscore and may contain letters, digits,
///    underscores, and dots) are resolved against the pipeline context.
///
/// Both phases run at execution time so each stage sees the outputs of
/// previously completed stages. The parse-time `$goal` expansion
/// (in [`VariableExpansionTransform`]) runs earlier, so `$goal` is
/// never present at this point.
pub(super) fn expand_runtime_variables(input: &str, context: &Context) -> String {
    let mut result = input.to_string();

    // Phase 1: built-in aliases
    if result.contains("$last_stage") {
        let value = context.get_string("last_stage");
        result = result.replace("$last_stage", &value);
    }

    if result.contains("$last_outcome") {
        let value = context.get_string("outcome");
        result = result.replace("$last_outcome", &value);
    }

    if result.contains("$last_output") {
        let value = context.get_string("last_output_full");
        result = result.replace("$last_output", &value);
    }

    // Phase 2: generic context variable expansion ($KEY → context value)
    if result.contains('$') {
        result = expand_context_variables(&result, context);
    }

    result
}

/// Replace all remaining `$KEY` references with the corresponding context value.
///
/// A variable reference starts with `$` followed by a letter or
/// underscore, then any combination of letters, digits, underscores,
/// and dots (e.g. `$human.feedback`, `$step_1.result`). The matched
/// key is looked up directly in the pipeline context.
///
/// A `$` not followed by a valid identifier start character (letter or
/// underscore) passes through literally, so `$50` or `$` at end-of-string
/// are preserved.
///
/// Missing keys resolve to an empty string, consistent with
/// [`Context::get_string`] behavior.
fn expand_context_variables(input: &str, context: &Context) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = rest.find('$') {
        result.push_str(&rest[..pos]);
        let after_dollar = &rest[pos + 1..]; // skip "$"

        // The key must start with a letter or underscore
        let starts_ident = after_dollar
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');

        if !starts_ident {
            // Not a variable reference — emit the "$" literally
            result.push('$');
            rest = after_dollar;
            continue;
        }

        // Consume the key: letters, digits, underscores, dots
        let key_len = after_dollar
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .unwrap_or(after_dollar.len());

        let key = &after_dollar[..key_len];
        result.push_str(&context.get_string(key));
        rest = &after_dollar[key_len..];
    }

    result.push_str(rest);
    result
}

/// Build a success outcome that stores output text in standard context keys.
///
/// Sets `last_stage`, `last_output` (truncated), `last_output_full`,
/// and accumulates the node into `completed_stages`. Callers can insert
/// additional handler-specific keys into [`Outcome::context_updates`]
/// after this returns.
pub(super) fn build_output_outcome(node_id: &str, output: &str, context: &Context) -> Outcome {
    let mut outcome = Outcome::success();
    outcome.context_updates = IndexMap::new();
    outcome.context_updates.insert(
        "last_stage".to_string(),
        serde_json::Value::String(node_id.to_string()),
    );
    outcome.context_updates.insert(
        "last_output".to_string(),
        serde_json::Value::String(truncate_output(output)),
    );
    outcome.context_updates.insert(
        "last_output_full".to_string(),
        serde_json::Value::String(output.to_string()),
    );

    let mut stages: Vec<serde_json::Value> = context
        .get("completed_stages")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    stages.push(serde_json::json!({"id": node_id, "status": "success"}));
    outcome.context_updates.insert(
        "completed_stages".to_string(),
        serde_json::Value::Array(stages),
    );

    outcome
}
