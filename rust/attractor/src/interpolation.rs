//! Runtime interpolation helpers for pipeline and workflow handlers.

use crate::context::{Context, ctx};

/// Expand runtime variables in an input string from context values.
///
/// Expands variables in two phases:
///
/// 1. **Built-in aliases** — `$last_output`, `$last_stage`, and
///    `$last_outcome` are replaced first via simple string substitution.
///    These map to context keys that differ from the variable name
///    (e.g. `$last_output` → `last_output_full`, `$last_outcome` →
///    `outcome`), so they must be handled before the generic expansion.
/// 2. **Context variables** — any remaining `$KEY` references (where KEY
///    starts with a letter or underscore and may contain letters, digits,
///    underscores, and dots) are resolved against the pipeline context.
///
/// Both phases run at execution time so each stage sees the outputs of
/// previously completed stages. The parse-time `$goal` expansion
/// runs earlier, so `$goal` is normally never present at this point.
///
/// Unknown variables are left unchanged so shell variables like `$COUNT`
/// continue to work when they are not workflow-context keys.
#[must_use]
pub fn expand_runtime_variables(input: &str, context: &Context) -> String {
    let mut result = input.to_string();

    if result.contains("$last_stage") && context.get(ctx::LAST_STAGE).is_some() {
        result = result.replace("$last_stage", &context.get_string(ctx::LAST_STAGE));
    }

    if result.contains("$last_outcome") && context.get(ctx::OUTCOME).is_some() {
        result = result.replace("$last_outcome", &context.get_string(ctx::OUTCOME));
    }

    if result.contains("$last_output") && context.get(ctx::LAST_OUTPUT_FULL).is_some() {
        result = result.replace("$last_output", &context.get_string(ctx::LAST_OUTPUT_FULL));
    }

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
/// Missing keys are left unchanged.
fn expand_context_variables(input: &str, context: &Context) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = rest.find('$') {
        result.push_str(&rest[..pos]);
        let after_dollar = &rest[pos + 1..];

        let starts_ident = after_dollar
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');

        if !starts_ident {
            result.push('$');
            rest = after_dollar;
            continue;
        }

        let key_len = after_dollar
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .unwrap_or(after_dollar.len());

        let key = &after_dollar[..key_len];
        if context.get(key).is_some() {
            result.push_str(&context.get_string(key));
        } else {
            result.push('$');
            result.push_str(key);
        }
        rest = &after_dollar[key_len..];
    }

    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_known_variables() {
        let context = Context::new();
        context.set(ctx::LAST_OUTPUT_FULL, serde_json::json!("child goal"));
        context.set("human.feedback", serde_json::json!("Add tests"));

        assert_eq!(
            expand_runtime_variables("Use this: $last_output. $human.feedback", &context),
            "Use this: child goal. Add tests"
        );
    }

    #[test]
    fn leaves_unknown_variables_unchanged() {
        let context = Context::new();

        assert_eq!(
            expand_runtime_variables("COUNT=$COUNT; test $COUNT -ge 2", &context),
            "COUNT=$COUNT; test $COUNT -ge 2"
        );
        assert_eq!(
            expand_runtime_variables("Previous stage: $last_stage", &context),
            "Previous stage: $last_stage"
        );
    }
}
