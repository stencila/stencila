//! Condition expression language for edge routing (§10).
//!
//! Evaluates boolean expressions that gate edge eligibility during pipeline
//! traversal. The grammar is deliberately simple:
//!
//! ```text
//! ConditionExpr ::= Clause ( '&&' Clause )*
//! Clause        ::= Key '=' Literal | Key '!=' Literal | Key
//! Key           ::= 'outcome' | 'preferred_label' | 'context.' Path | Path
//! ```
//!
//! All clauses are AND-combined. Missing context keys compare as empty strings.
//!
//! **Spec deviation — bare keys:** Bare-key clauses (without `=` or `!=`) are
//! supported as truthy checks, following the §10.5 pseudocode branch. The §10.2
//! grammar does not include this form, but the pseudocode explicitly handles it.
//! Both the evaluator and validator accept bare keys for consistency.
//!
//! **Spec deviation — unquoted literals:** Condition literals are compared as
//! plain text without requiring DOT-style `"..."` quoting. Surrounding double
//! quotes on a literal are stripped so that `preferred_label="Fix"` and
//! `preferred_label=Fix` are equivalent. All §10.6 examples use unquoted
//! literals.

use crate::context::Context;
use crate::error::{AttractorError, AttractorResult};
use crate::types::Outcome;

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

/// Construct an [`AttractorError::InvalidCondition`].
fn condition_error(full_condition: &str, reason: impl Into<String>) -> AttractorError {
    AttractorError::InvalidCondition {
        condition: full_condition.to_string(),
        reason: reason.into(),
    }
}

// ---------------------------------------------------------------------------
// Shared clause parser
// ---------------------------------------------------------------------------

/// A parsed condition clause.
#[derive(Debug, PartialEq, Eq)]
enum ParsedClause<'a> {
    /// `key = value`
    Eq { key: &'a str, value: &'a str },
    /// `key != value`
    Neq { key: &'a str, value: &'a str },
    /// Bare key (truthy check, spec deviation)
    Bare { key: &'a str },
}

/// Parse a single clause string into a [`ParsedClause`].
///
/// Splits on `!=` first (since it contains `=`), then on `=`. If neither
/// operator is found, treats the clause as a bare key.
///
/// Surrounding double quotes on literal values are stripped so that both
/// `preferred_label="Fix"` and `preferred_label=Fix` work identically.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidCondition`] if the clause is structurally
/// malformed (e.g. `==` operator, empty key, invalid key path).
fn parse_clause<'a>(clause: &'a str, full_condition: &str) -> AttractorResult<ParsedClause<'a>> {
    // Try `!=` first (since it contains `=`)
    if let Some((key, value)) = clause.split_once("!=") {
        let key = key.trim();
        let value = strip_quotes(value.trim());
        validate_key(key, clause, full_condition)?;
        return Ok(ParsedClause::Neq { key, value });
    }

    // Try `=` (split-at-first semantics per §10.5)
    if let Some((key, value)) = clause.split_once('=') {
        let key = key.trim();
        let raw_value = value.trim();
        // Reject `==` as an operator: `split_once('=')` on `key==val` yields
        // value `=val`. A leading `=` means the user wrote `==` (not in grammar).
        // Literals like `key=a==b` are fine — value is `a==b`, no leading `=`.
        if raw_value.starts_with('=') {
            return Err(condition_error(
                full_condition,
                format!("clause `{clause}` uses `==` (use `=` instead)"),
            ));
        }
        let value = strip_quotes(raw_value);
        validate_key(key, clause, full_condition)?;
        return Ok(ParsedClause::Eq { key, value });
    }

    // Bare key (truthy check — spec deviation)
    let key = clause.trim();
    validate_key(key, clause, full_condition)?;
    Ok(ParsedClause::Bare { key })
}

/// Strip surrounding double quotes from a string, if present.
fn strip_quotes(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
}

/// Validate that a key is non-empty, uses only valid characters, and follows
/// the `Identifier ('.' Identifier)*` path structure per §10.2.
fn validate_key(key: &str, clause: &str, full_condition: &str) -> AttractorResult<()> {
    if key.is_empty() {
        return Err(condition_error(
            full_condition,
            format!("clause `{clause}` has an empty key"),
        ));
    }

    // Character class: alphanumeric, underscores, and dots only
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(condition_error(
            full_condition,
            format!("clause `{clause}` has an invalid key `{key}`"),
        ));
    }

    // Path structure: no leading/trailing dots, no consecutive dots
    if key.starts_with('.') || key.ends_with('.') || key.contains("..") {
        return Err(condition_error(
            full_condition,
            format!("clause `{clause}` has an invalid key path `{key}`"),
        ));
    }

    // Each segment must be a valid Identifier: [A-Za-z_][A-Za-z0-9_]* per §10.2
    for segment in key.split('.') {
        let first = segment.bytes().next().unwrap_or(b'0');
        if !first.is_ascii_alphabetic() && first != b'_' {
            return Err(condition_error(
                full_condition,
                format!(
                    "clause `{clause}` has an invalid key segment `{segment}` \
                     (must start with a letter or underscore)"
                ),
            ));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Condition-level iteration (shared between evaluate and validate)
// ---------------------------------------------------------------------------

/// Iterate over non-empty, trimmed clauses in a condition string.
fn for_each_clause(
    condition: &str,
    mut f: impl FnMut(&str) -> AttractorResult<bool>,
) -> AttractorResult<bool> {
    let condition = condition.trim();
    if condition.is_empty() {
        return Ok(true);
    }

    for clause in condition.split("&&") {
        let clause = clause.trim();
        if clause.is_empty() {
            continue;
        }
        if !f(clause)? {
            return Ok(false);
        }
    }
    Ok(true)
}

// ---------------------------------------------------------------------------
// Public API: evaluate
// ---------------------------------------------------------------------------

/// Evaluate a condition expression against an outcome and context.
///
/// Returns `true` if the condition is satisfied (all clauses pass).
/// An empty or whitespace-only condition is always `true` (§10.5).
#[must_use]
pub fn evaluate_condition(condition: &str, outcome: &Outcome, context: &Context) -> bool {
    // parse_clause can only fail on structurally invalid input. At runtime,
    // treat parse failures as false (condition not satisfied) rather than
    // panicking, since conditions come from user-authored DOT files.
    for_each_clause(condition, |clause| {
        let Ok(parsed) = parse_clause(clause, condition) else {
            return Ok(false);
        };
        Ok(evaluate_parsed_clause(&parsed, outcome, context))
    })
    .unwrap_or(false)
}

/// Evaluate a single parsed clause.
fn evaluate_parsed_clause(parsed: &ParsedClause<'_>, outcome: &Outcome, context: &Context) -> bool {
    match parsed {
        ParsedClause::Eq { key, value } => resolve_key(key, outcome, context) == *value,
        ParsedClause::Neq { key, value } => resolve_key(key, outcome, context) != *value,
        ParsedClause::Bare { key } => !resolve_key(key, outcome, context).is_empty(),
    }
}

// ---------------------------------------------------------------------------
// Public API: validate
// ---------------------------------------------------------------------------

/// Validate a condition expression for syntactic correctness.
///
/// Accepts both `Key Op Literal` clauses (per §10.2 grammar) and bare-key
/// clauses (per §10.5 pseudocode truthy branch). See the module-level doc
/// comment on bare-key support as a spec deviation.
///
/// This is used at validation/lint time (§7) to catch malformed conditions
/// before pipeline execution.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidCondition`] if any clause is malformed
/// (empty key, `==` operator, or invalid key path).
pub fn parse_condition(condition: &str) -> AttractorResult<()> {
    for_each_clause(condition, |clause| {
        parse_clause(clause, condition.trim())?;
        Ok(true)
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Key resolution
// ---------------------------------------------------------------------------

/// Coerce a [`serde_json::Value`] to a string, matching [`Context::get_string`]
/// semantics (§5.1).
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Resolve a key to its string value per §10.4.
///
/// Resolution order:
/// 1. `"outcome"` → outcome status as string
/// 2. `"preferred_label"` → outcome preferred label
/// 3. `"context."` prefix → context lookup by presence (with prefix first,
///    then without). Fallback uses key *presence* (`get` returns `Some`), not
///    string emptiness, per §10.4 pseudocode.
/// 4. Unqualified key → direct context lookup
///
/// Missing keys resolve to an empty string.
fn resolve_key(key: &str, outcome: &Outcome, context: &Context) -> String {
    if key == "outcome" {
        return outcome.status.as_str().to_string();
    }

    if key == "preferred_label" {
        return outcome.preferred_label.clone();
    }

    if let Some(path) = key.strip_prefix("context.") {
        // Try with the full key first (including "context." prefix).
        // Use presence check (get → Some), not string emptiness, per §10.4.
        if let Some(value) = context.get(key) {
            return value_to_string(&value);
        }
        // Then try without the prefix
        return context.get_string(path);
    }

    // Direct context lookup for unqualified keys
    context.get_string(key)
}
