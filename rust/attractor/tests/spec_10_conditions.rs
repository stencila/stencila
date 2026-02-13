use stencila_attractor::AttractorError;
use stencila_attractor::condition::{evaluate_condition, parse_condition};
use stencila_attractor::context::Context;
use stencila_attractor::types::{Outcome, StageStatus};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a success outcome and empty context.
fn success_ctx() -> (Outcome, Context) {
    (Outcome::success(), Context::new())
}

/// Create a success outcome with a preferred label.
fn success_with_label(label: &str) -> Outcome {
    let mut outcome = Outcome::success();
    outcome.preferred_label = label.to_string();
    outcome
}

/// Create a context with a single string key-value pair.
fn ctx_with(key: &str, value: &str) -> Context {
    let ctx = Context::new();
    ctx.set(key, serde_json::Value::String(value.to_string()));
    ctx
}

/// Assert that a condition string is rejected by `parse_condition` with an
/// `InvalidCondition` error whose message contains the given substring.
fn assert_parse_rejects(condition: &str, expected_substr: &str) -> TestResult {
    let result = parse_condition(condition);
    let err = result
        .err()
        .ok_or_else(|| format!("expected parse_condition({condition:?}) to fail"))?;
    assert!(
        matches!(err, AttractorError::InvalidCondition { .. }),
        "expected InvalidCondition for {condition:?}, got {err:?}"
    );
    assert!(
        err.to_string().contains(expected_substr),
        "error for {condition:?} should contain {expected_substr:?}, got: {err}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Empty / trivial conditions
// ---------------------------------------------------------------------------

#[test]
fn empty_condition_is_true() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("", &outcome, &ctx));
    Ok(())
}

#[test]
fn whitespace_only_condition_is_true() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("   ", &outcome, &ctx));
    Ok(())
}

#[test]
fn trailing_and_is_ignored() -> TestResult {
    // Trailing `&&` produces an empty clause which is skipped per §10.5
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("outcome=success&&", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Outcome matching
// ---------------------------------------------------------------------------

#[test]
fn outcome_equals_success() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("outcome=success", &outcome, &ctx));
    assert!(!evaluate_condition("outcome=fail", &outcome, &ctx));
    Ok(())
}

#[test]
fn outcome_equals_fail() -> TestResult {
    let outcome = Outcome::fail("oops");
    let ctx = Context::new();
    assert!(evaluate_condition("outcome=fail", &outcome, &ctx));
    assert!(!evaluate_condition("outcome=success", &outcome, &ctx));
    Ok(())
}

#[test]
fn outcome_not_equals() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("outcome!=fail", &outcome, &ctx));
    assert!(!evaluate_condition("outcome!=success", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Preferred label
// ---------------------------------------------------------------------------

#[test]
fn preferred_label_equals() -> TestResult {
    let outcome = success_with_label("Fix");
    let ctx = Context::new();
    assert!(evaluate_condition("preferred_label=Fix", &outcome, &ctx));
    assert!(!evaluate_condition("preferred_label=Skip", &outcome, &ctx));
    Ok(())
}

#[test]
fn preferred_label_case_sensitive() -> TestResult {
    let outcome = success_with_label("Fix");
    let ctx = Context::new();
    assert!(!evaluate_condition("preferred_label=fix", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Context lookups
// ---------------------------------------------------------------------------

#[test]
fn context_dot_prefix_lookup() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("tests_passed", "true");
    assert!(evaluate_condition(
        "context.tests_passed=true",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn context_without_prefix_lookup() -> TestResult {
    // Unqualified keys also look up context values per §10.4
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("tests_passed", "true");
    assert!(evaluate_condition("tests_passed=true", &outcome, &ctx));
    Ok(())
}

#[test]
fn context_missing_key_is_empty_string() -> TestResult {
    let (outcome, ctx) = success_ctx();
    // Missing key compares as empty string — never equal to non-empty
    assert!(!evaluate_condition(
        "context.nonexistent=something",
        &outcome,
        &ctx
    ));
    // But it is equal to empty
    assert!(evaluate_condition("context.nonexistent=", &outcome, &ctx));
    Ok(())
}

#[test]
fn context_boolean_value_coercion() -> TestResult {
    // Boolean values in context should be coerced to string for comparison
    let (outcome, _) = success_ctx();
    let ctx = Context::new();
    ctx.set("flag", serde_json::Value::Bool(true));
    assert!(evaluate_condition("context.flag=true", &outcome, &ctx));
    assert!(!evaluate_condition("context.flag=false", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// AND conjunction
// ---------------------------------------------------------------------------

#[test]
fn and_both_true() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("tests_passed", "true");
    assert!(evaluate_condition(
        "outcome=success && context.tests_passed=true",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn and_first_false_short_circuits() -> TestResult {
    let outcome = Outcome::fail("oops");
    let ctx = ctx_with("tests_passed", "true");
    assert!(!evaluate_condition(
        "outcome=success && context.tests_passed=true",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn and_three_clauses() -> TestResult {
    let outcome = success_with_label("Deploy");
    let ctx = ctx_with("env", "prod");
    assert!(evaluate_condition(
        "outcome=success && preferred_label=Deploy && context.env=prod",
        &outcome,
        &ctx
    ));
    // Flip one clause
    assert!(!evaluate_condition(
        "outcome=success && preferred_label=Deploy && context.env=staging",
        &outcome,
        &ctx
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// Whitespace handling
// ---------------------------------------------------------------------------

#[test]
fn whitespace_around_operator() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("  outcome = success  ", &outcome, &ctx));
    assert!(evaluate_condition("outcome != fail", &outcome, &ctx));
    Ok(())
}

#[test]
fn whitespace_around_and() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("x", "1");
    assert!(evaluate_condition(
        "outcome=success  &&  context.x=1",
        &outcome,
        &ctx
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// §10.6 verbatim examples
// ---------------------------------------------------------------------------

#[test]
fn spec_example_route_on_success() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("outcome=success", &outcome, &ctx));
    let fail_outcome = Outcome::fail("oops");
    assert!(!evaluate_condition("outcome=success", &fail_outcome, &ctx));
    Ok(())
}

#[test]
fn spec_example_route_on_failure() -> TestResult {
    let outcome = Outcome::fail("oops");
    let ctx = Context::new();
    assert!(evaluate_condition("outcome=fail", &outcome, &ctx));
    Ok(())
}

#[test]
fn spec_example_success_and_context_flag() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("tests_passed", "true");
    assert!(evaluate_condition(
        "outcome=success && context.tests_passed=true",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn spec_example_context_not_equals() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("loop_state", "running");
    assert!(evaluate_condition(
        "context.loop_state!=exhausted",
        &outcome,
        &ctx
    ));
    // Now set to exhausted
    ctx.set(
        "loop_state",
        serde_json::Value::String("exhausted".to_string()),
    );
    assert!(!evaluate_condition(
        "context.loop_state!=exhausted",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn spec_example_preferred_label() -> TestResult {
    let outcome = success_with_label("Fix");
    let ctx = Context::new();
    assert!(evaluate_condition("preferred_label=Fix", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// All outcome statuses
// ---------------------------------------------------------------------------

#[test]
fn all_outcome_status_strings() -> TestResult {
    let ctx = Context::new();

    let cases = [
        (StageStatus::Success, "success"),
        (StageStatus::Fail, "fail"),
        (StageStatus::PartialSuccess, "partial_success"),
        (StageStatus::Retry, "retry"),
        (StageStatus::Skipped, "skipped"),
    ];
    for (status, expected_str) in cases {
        let outcome = Outcome {
            status,
            preferred_label: String::new(),
            suggested_next_ids: Vec::new(),
            context_updates: indexmap::IndexMap::new(),
            notes: String::new(),
            failure_reason: String::new(),
        };
        let cond = format!("outcome={expected_str}");
        assert!(
            evaluate_condition(&cond, &outcome, &ctx),
            "expected outcome={expected_str} to match for status {status:?}"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// parse_condition validation — valid inputs
// ---------------------------------------------------------------------------

#[test]
fn parse_condition_valid() -> TestResult {
    assert!(parse_condition("outcome=success").is_ok());
    assert!(parse_condition("outcome=success && context.x=1").is_ok());
    assert!(parse_condition("preferred_label=Fix").is_ok());
    assert!(parse_condition("").is_ok());
    assert!(parse_condition("context.loop_state!=exhausted").is_ok());
    Ok(())
}

#[test]
fn parse_condition_bare_key_accepted() -> TestResult {
    // Bare keys are accepted as a spec deviation (§10.5 pseudocode truthy branch)
    assert!(parse_condition("context.flag").is_ok());
    assert!(parse_condition("outcome=success && context.flag").is_ok());
    Ok(())
}

// ---------------------------------------------------------------------------
// parse_condition validation — rejected inputs
// ---------------------------------------------------------------------------

#[test]
fn parse_condition_rejects_double_equals() -> TestResult {
    assert_parse_rejects("outcome==success", "==")
}

#[test]
fn parse_condition_rejects_empty_key() -> TestResult {
    assert_parse_rejects("=success", "empty key")
}

#[test]
fn parse_condition_eq_value_with_equals_accepted() -> TestResult {
    // After split_once('='), everything right of the first `=` is the literal.
    // `outcome=success=extra` → key="outcome", value="success=extra".
    assert!(parse_condition("outcome=success=extra").is_ok());
    Ok(())
}

#[test]
fn parse_condition_neq_value_with_operators_accepted() -> TestResult {
    // After split_once("!="), everything right of the first `!=` is the literal.
    // Values containing `=` or `!=` are unusual but not structurally malformed
    // per §10.5 pseudocode (split with max=1).
    assert!(parse_condition("outcome!=success!=extra").is_ok());
    assert!(parse_condition("outcome!=a=b").is_ok());
    Ok(())
}

#[test]
fn parse_condition_rejects_invalid_bare_key() -> TestResult {
    // Hyphens are not valid in keys per grammar
    assert_parse_rejects("context.bad-key", "invalid key")
}

#[test]
fn parse_condition_rejects_bare_key_with_spaces() -> TestResult {
    // A bare key that looks like two words — the space makes it invalid
    // Note: "bad key" has a space which is not alphanumeric/underscore/dot
    assert_parse_rejects("bad key", "invalid key")
}

// ---------------------------------------------------------------------------
// Prefixed key precedence with empty value
// ---------------------------------------------------------------------------

#[test]
fn context_prefixed_key_empty_value_no_fallback() -> TestResult {
    // When context["context.flag"] exists but is "", resolve_key must return ""
    // and NOT fall back to the unprefixed context["flag"].
    let (outcome, _) = success_ctx();
    let ctx = Context::new();
    ctx.set("context.flag", serde_json::Value::String(String::new()));
    ctx.set("flag", serde_json::Value::String("true".to_string()));
    // Should resolve to "" (the prefixed key's value), not "true"
    assert!(!evaluate_condition("context.flag=true", &outcome, &ctx));
    assert!(evaluate_condition("context.flag=", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Bare-key truthy checks (spec deviation)
// ---------------------------------------------------------------------------

#[test]
fn bare_key_truthy_check() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("flag", "yes");
    // Bare key: truthy if resolved value is non-empty
    assert!(evaluate_condition("flag", &outcome, &ctx));
    assert!(evaluate_condition("context.flag", &outcome, &ctx));
    // Missing key is falsy
    assert!(!evaluate_condition("nonexistent", &outcome, &ctx));
    Ok(())
}

#[test]
fn bare_key_empty_string_is_falsy() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("flag", "");
    assert!(!evaluate_condition("flag", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Quoted string literals (spec deviation — quotes are stripped)
// ---------------------------------------------------------------------------

#[test]
fn quoted_literal_equals_unquoted() -> TestResult {
    let outcome = success_with_label("Fix");
    let ctx = Context::new();
    // Surrounding quotes are stripped: `"Fix"` compares as `Fix`
    assert!(evaluate_condition(
        "preferred_label=\"Fix\"",
        &outcome,
        &ctx
    ));
    Ok(())
}

#[test]
fn quoted_literal_neq() -> TestResult {
    let (outcome, ctx) = success_ctx();
    assert!(evaluate_condition("outcome!=\"fail\"", &outcome, &ctx));
    Ok(())
}

#[test]
fn unmatched_quotes_are_not_stripped() -> TestResult {
    // Only matching surrounding quotes are stripped; a lone leading quote stays
    let outcome = success_with_label("\"Fix");
    let ctx = Context::new();
    assert!(evaluate_condition("preferred_label=\"Fix", &outcome, &ctx));
    // But with proper surrounding quotes, it matches the bare value
    let outcome2 = success_with_label("Fix");
    assert!(!evaluate_condition(
        "preferred_label=\"Fix",
        &outcome2,
        &ctx
    ));
    Ok(())
}

#[test]
fn parse_condition_quoted_literal_valid() -> TestResult {
    assert!(parse_condition("outcome=\"success\"").is_ok());
    assert!(parse_condition("preferred_label=\"Fix\"").is_ok());
    assert!(parse_condition("context.x=\"hello world\"").is_ok());
    Ok(())
}

// ---------------------------------------------------------------------------
// Values containing operators (= and !=)
// ---------------------------------------------------------------------------

#[test]
fn eq_value_containing_equals() -> TestResult {
    // `context.formula=a=b` → key="context.formula", value="a=b"
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("formula", "a=b");
    assert!(evaluate_condition("context.formula=a=b", &outcome, &ctx));
    Ok(())
}

#[test]
fn neq_value_containing_equals() -> TestResult {
    // `context.x!=a=b` → key="context.x", value="a=b"
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("x", "other");
    assert!(evaluate_condition("context.x!=a=b", &outcome, &ctx));
    Ok(())
}

// ---------------------------------------------------------------------------
// Invalid key paths (dotted key validation)
// ---------------------------------------------------------------------------

#[test]
fn parse_condition_rejects_leading_dot() -> TestResult {
    assert_parse_rejects(".foo=bar", "invalid key path")
}

#[test]
fn parse_condition_rejects_trailing_dot() -> TestResult {
    assert_parse_rejects("foo.=bar", "invalid key path")
}

#[test]
fn parse_condition_rejects_consecutive_dots() -> TestResult {
    assert_parse_rejects("context..x=1", "invalid key path")
}

#[test]
fn parse_condition_rejects_bare_leading_dot() -> TestResult {
    assert_parse_rejects(".flag", "invalid key path")
}

#[test]
fn parse_condition_valid_dotted_key() -> TestResult {
    // Multi-segment dotted keys are valid
    assert!(parse_condition("context.a.b.c=1").is_ok());
    assert!(parse_condition("a.b=x").is_ok());
    Ok(())
}

// ---------------------------------------------------------------------------
// == in literal values (not as operator)
// ---------------------------------------------------------------------------

#[test]
fn double_equals_in_literal_accepted() -> TestResult {
    // `context.expr=a==b` should parse: key="context.expr", value="a==b"
    assert!(parse_condition("context.expr=a==b").is_ok());
    Ok(())
}

#[test]
fn double_equals_in_literal_evaluates() -> TestResult {
    let (outcome, _) = success_ctx();
    let ctx = ctx_with("expr", "a==b");
    assert!(evaluate_condition("context.expr=a==b", &outcome, &ctx));
    assert!(!evaluate_condition("context.expr=a!=b", &outcome, &ctx));
    Ok(())
}

#[test]
fn double_equals_operator_still_rejected() -> TestResult {
    // `outcome==success` is still rejected (== as operator)
    assert_parse_rejects("outcome==success", "==")
}

// ---------------------------------------------------------------------------
// Identifier segment validation (§10.2)
// ---------------------------------------------------------------------------

#[test]
fn parse_condition_rejects_digit_prefixed_key() -> TestResult {
    assert_parse_rejects("1abc=foo", "must start with a letter or underscore")
}

#[test]
fn parse_condition_rejects_digit_prefixed_segment() -> TestResult {
    assert_parse_rejects("context.9bad=foo", "must start with a letter or underscore")
}

#[test]
fn parse_condition_accepts_underscore_prefixed_key() -> TestResult {
    assert!(parse_condition("_private=true").is_ok());
    assert!(parse_condition("context._flag=1").is_ok());
    Ok(())
}

#[test]
fn parse_condition_accepts_key_with_digits_after_start() -> TestResult {
    assert!(parse_condition("node2=ok").is_ok());
    assert!(parse_condition("context.step3_result=done").is_ok());
    Ok(())
}
