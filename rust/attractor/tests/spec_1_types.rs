use std::collections::HashSet;

use indexmap::IndexMap;
use serde_json::json;

use stencila_attractor::checkpoint::Checkpoint;
use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorError;
use stencila_attractor::types::{Duration, FidelityMode, Outcome, ReasoningEffort, StageStatus};

// ---------------------------------------------------------------------------
// Error tests
// ---------------------------------------------------------------------------

#[test]
fn error_display_retryable() -> Result<(), Box<dyn std::error::Error>> {
    let err = AttractorError::RateLimited {
        message: "too many requests".into(),
    };
    let display = err.to_string();
    assert!(
        display.contains("too many requests"),
        "expected display to contain payload, got: {display}"
    );
    Ok(())
}

#[test]
fn error_is_retryable() -> Result<(), Box<dyn std::error::Error>> {
    let retryable = [
        AttractorError::RateLimited {
            message: String::new(),
        },
        AttractorError::NetworkTimeout {
            message: String::new(),
        },
        AttractorError::TemporaryUnavailable {
            message: String::new(),
        },
    ];
    for err in &retryable {
        assert!(err.is_retryable(), "{err:?} should be retryable");
        assert!(!err.is_terminal(), "{err:?} should not be terminal");
        assert!(!err.is_pipeline(), "{err:?} should not be pipeline");
    }

    // Io is retryable (transient I/O failure)
    assert!(
        AttractorError::Io {
            message: String::new()
        }
        .is_retryable()
    );

    // Non-retryable samples
    assert!(!AttractorError::NoStartNode.is_retryable());
    assert!(
        !AttractorError::InvalidPrompt {
            message: String::new()
        }
        .is_retryable()
    );
    assert!(
        !AttractorError::Json {
            message: String::new()
        }
        .is_retryable()
    );
    Ok(())
}

#[test]
fn error_is_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let terminal = [
        AttractorError::InvalidPrompt {
            message: String::new(),
        },
        AttractorError::MissingContext { key: String::new() },
        AttractorError::AuthenticationFailed {
            message: String::new(),
        },
        AttractorError::HandlerFailed {
            node_id: String::new(),
            reason: String::new(),
        },
        AttractorError::Json {
            message: String::new(),
        },
    ];
    for err in &terminal {
        assert!(err.is_terminal(), "{err:?} should be terminal");
        assert!(!err.is_retryable(), "{err:?} should not be retryable");
        assert!(!err.is_pipeline(), "{err:?} should not be pipeline");
    }
    Ok(())
}

#[test]
fn error_is_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = [
        AttractorError::NoStartNode,
        AttractorError::NoExitNode,
        AttractorError::UnreachableNode {
            node_id: String::new(),
        },
        AttractorError::InvalidCondition {
            condition: String::new(),
            reason: String::new(),
        },
        AttractorError::NodeNotFound {
            node_id: String::new(),
        },
        AttractorError::InvalidPipeline {
            reason: String::new(),
        },
    ];
    for err in &pipeline {
        assert!(err.is_pipeline(), "{err:?} should be pipeline");
        assert!(!err.is_retryable(), "{err:?} should not be retryable");
        assert!(!err.is_terminal(), "{err:?} should not be terminal");
    }
    Ok(())
}

#[test]
fn error_codes_unique() -> Result<(), Box<dyn std::error::Error>> {
    let all_errors: Vec<AttractorError> = vec![
        AttractorError::RateLimited {
            message: String::new(),
        },
        AttractorError::NetworkTimeout {
            message: String::new(),
        },
        AttractorError::TemporaryUnavailable {
            message: String::new(),
        },
        AttractorError::InvalidPrompt {
            message: String::new(),
        },
        AttractorError::MissingContext { key: String::new() },
        AttractorError::AuthenticationFailed {
            message: String::new(),
        },
        AttractorError::HandlerFailed {
            node_id: String::new(),
            reason: String::new(),
        },
        AttractorError::NoStartNode,
        AttractorError::NoExitNode,
        AttractorError::UnreachableNode {
            node_id: String::new(),
        },
        AttractorError::InvalidCondition {
            condition: String::new(),
            reason: String::new(),
        },
        AttractorError::NodeNotFound {
            node_id: String::new(),
        },
        AttractorError::InvalidPipeline {
            reason: String::new(),
        },
        AttractorError::Io {
            message: String::new(),
        },
        AttractorError::Json {
            message: String::new(),
        },
    ];

    let codes: HashSet<&str> = all_errors.iter().map(AttractorError::code).collect();
    assert_eq!(
        codes.len(),
        all_errors.len(),
        "error codes must be unique; got {codes:?}"
    );
    Ok(())
}

#[test]
fn error_serialize_json() -> Result<(), Box<dyn std::error::Error>> {
    let err = AttractorError::NodeNotFound {
        node_id: "stage_3".into(),
    };
    let json_str = serde_json::to_string(&err)?;
    let v: serde_json::Value = serde_json::from_str(&json_str)?;
    assert_eq!(v["code"], "NODE_NOT_FOUND");
    assert!(v["message"].as_str().is_some_and(|m| m.contains("stage_3")));
    Ok(())
}

// ---------------------------------------------------------------------------
// Duration tests
// ---------------------------------------------------------------------------

#[test]
fn duration_parse_all_units() -> Result<(), Box<dyn std::error::Error>> {
    let cases: [(&str, u64); 5] = [
        ("250ms", 250),
        ("900s", 900_000),
        ("15m", 900_000),
        ("2h", 7_200_000),
        ("1d", 86_400_000),
    ];
    for (input, expected_ms) in cases {
        let d = Duration::from_spec_str(input)?;
        assert_eq!(
            d.inner().as_millis(),
            u128::from(expected_ms),
            "failed for {input}"
        );
    }
    Ok(())
}

#[test]
fn duration_display_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    // Only test values canonical for their unit (900s = 15m, so use 7s instead)
    let inputs = ["250ms", "7s", "15m", "2h", "1d"];
    for input in inputs {
        let d = Duration::from_spec_str(input)?;
        let displayed = d.to_string();
        assert_eq!(displayed, input, "roundtrip failed for {input}");
    }
    Ok(())
}

#[test]
fn duration_parse_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let invalids = ["abc", "15x", "", "-5s"];
    for input in invalids {
        assert!(
            Duration::from_spec_str(input).is_err(),
            "expected error for {input:?}"
        );
    }
    Ok(())
}

#[test]
fn duration_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let d = Duration::from_spec_str("15m")?;
    let json = serde_json::to_string(&d)?;
    assert_eq!(json, "\"15m\"");
    let d2: Duration = serde_json::from_str(&json)?;
    assert_eq!(d, d2);
    Ok(())
}

// ---------------------------------------------------------------------------
// StageStatus tests
// ---------------------------------------------------------------------------

#[test]
fn stage_status_serde_lowercase() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(serde_json::to_string(&StageStatus::Success)?, "\"success\"");
    assert_eq!(
        serde_json::to_string(&StageStatus::PartialSuccess)?,
        "\"partial_success\""
    );
    assert_eq!(serde_json::to_string(&StageStatus::Fail)?, "\"fail\"");
    Ok(())
}

#[test]
fn stage_status_is_success() -> Result<(), Box<dyn std::error::Error>> {
    assert!(StageStatus::Success.is_success());
    assert!(StageStatus::PartialSuccess.is_success());
    assert!(!StageStatus::Fail.is_success());
    assert!(!StageStatus::Retry.is_success());
    assert!(!StageStatus::Skipped.is_success());
    Ok(())
}

#[test]
fn stage_status_roundtrip_all() -> Result<(), Box<dyn std::error::Error>> {
    let all = [
        StageStatus::Success,
        StageStatus::Fail,
        StageStatus::PartialSuccess,
        StageStatus::Retry,
        StageStatus::Skipped,
    ];
    for status in all {
        let json = serde_json::to_string(&status)?;
        let back: StageStatus = serde_json::from_str(&json)?;
        assert_eq!(status, back);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Outcome tests
// ---------------------------------------------------------------------------

#[test]
fn outcome_constructor_success() -> Result<(), Box<dyn std::error::Error>> {
    let o = Outcome::success();
    assert_eq!(o.status, StageStatus::Success);
    assert!(o.preferred_label.is_empty());
    assert!(o.failure_reason.is_empty());
    assert!(o.context_updates.is_empty());
    Ok(())
}

#[test]
fn outcome_constructor_fail() -> Result<(), Box<dyn std::error::Error>> {
    let o = Outcome::fail("something broke");
    assert_eq!(o.status, StageStatus::Fail);
    assert_eq!(o.failure_reason, "something broke");
    Ok(())
}

#[test]
fn outcome_serde_appendix_c() -> Result<(), Box<dyn std::error::Error>> {
    let o = Outcome::success();
    let v: serde_json::Value = serde_json::to_value(&o)?;

    // Must use "outcome" not "status"
    assert!(v.get("outcome").is_some(), "expected 'outcome' key in JSON");
    assert!(
        v.get("status").is_none(),
        "must not have 'status' key in JSON"
    );

    // Empty optional fields should be omitted
    assert!(
        v.get("preferred_next_label").is_none(),
        "empty preferred_label should be skipped"
    );
    assert!(
        v.get("failure_reason").is_none(),
        "empty failure_reason should be skipped"
    );
    Ok(())
}

#[test]
fn outcome_preferred_label_alias() -> Result<(), Box<dyn std::error::Error>> {
    // "preferred_next_label" (canonical)
    let json1 = r#"{"outcome":"success","preferred_next_label":"edge_a"}"#;
    let o1: Outcome = serde_json::from_str(json1)?;
    assert_eq!(o1.preferred_label, "edge_a");

    // "preferred_label" (alias)
    let json2 = r#"{"outcome":"success","preferred_label":"edge_b"}"#;
    let o2: Outcome = serde_json::from_str(json2)?;
    assert_eq!(o2.preferred_label, "edge_b");
    Ok(())
}

// ---------------------------------------------------------------------------
// FidelityMode tests
// ---------------------------------------------------------------------------

#[test]
fn fidelity_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let all = [
        FidelityMode::Full,
        FidelityMode::Truncate,
        FidelityMode::Compact,
        FidelityMode::SummaryLow,
        FidelityMode::SummaryMedium,
        FidelityMode::SummaryHigh,
    ];
    for mode in all {
        let json = serde_json::to_string(&mode)?;
        let back: FidelityMode = serde_json::from_str(&json)?;
        assert_eq!(mode, back, "roundtrip failed for {mode:?}");
    }

    // Verify the colon variants serialize correctly
    assert_eq!(
        serde_json::to_string(&FidelityMode::SummaryLow)?,
        "\"summary:low\""
    );
    Ok(())
}

#[test]
fn fidelity_default_is_compact() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(FidelityMode::default(), FidelityMode::Compact);
    Ok(())
}

// ---------------------------------------------------------------------------
// ReasoningEffort tests
// ---------------------------------------------------------------------------

#[test]
fn reasoning_effort_serde_and_default() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(ReasoningEffort::default(), ReasoningEffort::High);
    assert_eq!(ReasoningEffort::High.as_str(), "high");

    let all = [
        ReasoningEffort::Low,
        ReasoningEffort::Medium,
        ReasoningEffort::High,
    ];
    for effort in all {
        let json = serde_json::to_string(&effort)?;
        let back: ReasoningEffort = serde_json::from_str(&json)?;
        assert_eq!(effort, back);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Context tests
// ---------------------------------------------------------------------------

#[test]
fn context_set_get_scalar() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("name", json!("Alice"));
    ctx.set("age", json!(30));
    ctx.set("active", json!(true));

    assert_eq!(ctx.get("name"), Some(json!("Alice")));
    assert_eq!(ctx.get("age"), Some(json!(30)));
    assert_eq!(ctx.get("active"), Some(json!(true)));
    assert_eq!(ctx.get("missing"), None);
    Ok(())
}

#[test]
fn context_set_get_structured() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("tags", json!(["a", "b", "c"]));
    ctx.set("meta", json!({"key": "val"}));

    assert_eq!(ctx.get("tags"), Some(json!(["a", "b", "c"])));
    assert_eq!(ctx.get("meta"), Some(json!({"key": "val"})));
    Ok(())
}

#[test]
fn context_get_string_coercion() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("num", json!(42));
    ctx.set("flag", json!(true));
    ctx.set("text", json!("hello"));
    ctx.set("arr", json!([1, 2]));
    ctx.set("obj", json!({"a": 1}));
    ctx.set("null_val", json!(null));

    assert_eq!(ctx.get_string("num"), "42");
    assert_eq!(ctx.get_string("flag"), "true");
    assert_eq!(ctx.get_string("text"), "hello");
    assert_eq!(ctx.get_string("arr"), "[1,2]");
    assert_eq!(ctx.get_string("obj"), "{\"a\":1}");
    assert_eq!(ctx.get_string("null_val"), "null");
    assert_eq!(ctx.get_string("missing"), "");
    Ok(())
}

#[test]
fn context_apply_updates() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("a", json!(1));

    let mut updates = IndexMap::new();
    updates.insert("a".to_string(), json!(2)); // overwrite
    updates.insert("b".to_string(), json!("new")); // new key
    ctx.apply_updates(&updates);

    assert_eq!(ctx.get("a"), Some(json!(2)));
    assert_eq!(ctx.get("b"), Some(json!("new")));
    Ok(())
}

#[test]
fn context_snapshot_independent() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("x", json!(1));

    let snap = ctx.snapshot();
    ctx.set("x", json!(999));

    assert_eq!(snap.get("x"), Some(&json!(1)));
    assert_eq!(ctx.get("x"), Some(json!(999)));
    Ok(())
}

#[test]
fn context_deep_clone_isolation() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("k", json!("original"));
    ctx.append_log("log1");

    let clone = ctx.deep_clone();
    ctx.set("k", json!("modified"));
    ctx.append_log("log2");

    assert_eq!(clone.get("k"), Some(json!("original")));
    assert_eq!(clone.logs().len(), 1);
    assert_eq!(ctx.logs().len(), 2);
    Ok(())
}

#[test]
fn context_append_log_and_logs() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.append_log("first");
    ctx.append_log("second");
    ctx.append_log("third");

    let logs = ctx.logs();
    assert_eq!(logs, vec!["first", "second", "third"]);
    Ok(())
}

// ---------------------------------------------------------------------------
// Checkpoint tests
// ---------------------------------------------------------------------------

#[test]
fn checkpoint_from_context() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("key", json!("value"));
    ctx.append_log("entry1");

    let mut retries = IndexMap::new();
    retries.insert("node_a".to_string(), 2u32);

    let cp = Checkpoint::from_context(
        &ctx,
        "node_b",
        vec!["node_a".to_string()],
        IndexMap::new(),
        retries.clone(),
    );

    assert_eq!(cp.current_node, "node_b");
    assert_eq!(cp.completed_nodes, vec!["node_a"]);
    assert_eq!(cp.node_retries, retries);
    assert_eq!(cp.context_values.get("key"), Some(&json!("value")));
    assert_eq!(cp.logs, vec!["entry1"]);
    assert!(!cp.timestamp.is_empty());
    Ok(())
}

#[test]
fn checkpoint_save_load_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("data", json!(42));

    let cp = Checkpoint::from_context(
        &ctx,
        "stage_2",
        vec!["stage_1".into()],
        IndexMap::new(),
        IndexMap::new(),
    );

    let dir = tempfile::tempdir()?;
    let path = dir.path().join("checkpoint.json");
    cp.save(&path)?;

    let loaded = Checkpoint::load(&path)?;
    assert_eq!(cp, loaded);
    Ok(())
}

#[test]
fn checkpoint_json_key_context() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new();
    ctx.set("x", json!(1));

    let cp = Checkpoint::from_context(&ctx, "n", vec![], IndexMap::new(), IndexMap::new());
    let v: serde_json::Value = serde_json::to_value(&cp)?;

    // JSON key should be "context", not "context_values"
    assert!(v.get("context").is_some(), "expected 'context' key in JSON");
    assert!(
        v.get("context_values").is_none(),
        "must not have 'context_values' key in JSON"
    );
    Ok(())
}
