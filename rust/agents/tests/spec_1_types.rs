//! Phase 1 tests: Core Types and Error Hierarchy
//!
//! Covers spec sections 2.1-2.4, 2.9, 3.8 (records), 4.1 (records), Appendix B.
//! Tests are grouped by module: error hierarchy, SessionConfig, SessionState,
//! Turn variants, ExecResult/DirEntry/GrepOptions, and EventKind/SessionEvent.

#![allow(clippy::result_large_err)]

use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::types::{
    DirEntry, EventKind, ExecResult, GrepOptions, ReasoningEffort, SessionConfig, SessionEvent,
    SessionState, Turn,
};
use stencila_models3::error::{ProviderDetails, SdkError};
use stencila_models3::types::tool::{ToolCall, ToolResult};
use stencila_models3::types::usage::Usage;

// ===========================================================================
// AgentError — Display
// ===========================================================================

#[test]
fn error_display_file_not_found() {
    let err = AgentError::FileNotFound {
        path: "/tmp/nope.rs".into(),
    };
    assert_eq!(err.to_string(), "file not found: /tmp/nope.rs");
}

#[test]
fn error_display_edit_conflict() {
    let err = AgentError::EditConflict {
        reason: "old_string not unique".into(),
    };
    assert_eq!(err.to_string(), "edit conflict: old_string not unique");
}

#[test]
fn error_display_shell_timeout() {
    let err = AgentError::ShellTimeout { timeout_ms: 10_000 };
    assert_eq!(err.to_string(), "shell timeout after 10000ms");
}

#[test]
fn error_display_shell_exit_error() {
    let err = AgentError::ShellExitError {
        code: 1,
        stderr: "not found".into(),
    };
    assert_eq!(err.to_string(), "shell exit code 1: not found");
}

#[test]
fn error_display_permission_denied() {
    let err = AgentError::PermissionDenied {
        path: "/etc/shadow".into(),
    };
    assert_eq!(err.to_string(), "permission denied: /etc/shadow");
}

#[test]
fn error_display_validation_error() {
    let err = AgentError::ValidationError {
        reason: "missing field".into(),
    };
    assert_eq!(err.to_string(), "validation error: missing field");
}

#[test]
fn error_display_unknown_tool() {
    let err = AgentError::UnknownTool {
        name: "foobar".into(),
    };
    assert_eq!(err.to_string(), "unknown tool: foobar");
}

#[test]
fn error_display_session_closed() {
    assert_eq!(AgentError::SessionClosed.to_string(), "session closed");
}

#[test]
fn error_display_turn_limit_exceeded() {
    let err = AgentError::TurnLimitExceeded {
        message: "200 rounds".into(),
    };
    assert_eq!(err.to_string(), "turn limit exceeded: 200 rounds");
}

#[test]
fn error_display_context_length_exceeded() {
    let err = AgentError::ContextLengthExceeded {
        message: "128k tokens".into(),
    };
    assert_eq!(err.to_string(), "context length exceeded: 128k tokens");
}

// ===========================================================================
// AgentError — From<SdkError>
// ===========================================================================

#[test]
fn error_from_sdk_error() {
    let sdk_err = SdkError::Authentication {
        message: "bad key".into(),
        details: ProviderDetails::default(),
    };
    let agent_err: AgentError = sdk_err.clone().into();
    assert_eq!(agent_err, AgentError::Sdk(sdk_err));
}

#[test]
fn error_sdk_wrapper_display() {
    let sdk_err = SdkError::Network {
        message: "timeout".into(),
    };
    let err = AgentError::Sdk(sdk_err);
    assert!(err.to_string().contains("timeout"));
}

// ===========================================================================
// AgentError — Classification helpers
// ===========================================================================

#[test]
fn error_is_tool_error_for_all_tool_variants() {
    let tool_errors: Vec<AgentError> = vec![
        AgentError::FileNotFound { path: "x".into() },
        AgentError::EditConflict { reason: "x".into() },
        AgentError::ShellTimeout { timeout_ms: 1 },
        AgentError::ShellExitError {
            code: 1,
            stderr: String::new(),
        },
        AgentError::PermissionDenied { path: "x".into() },
        AgentError::ValidationError { reason: "x".into() },
        AgentError::UnknownTool { name: "x".into() },
    ];
    for err in &tool_errors {
        assert!(err.is_tool_error(), "{err:?} should be a tool error");
        assert!(
            !err.is_session_error(),
            "{err:?} should NOT be a session error"
        );
    }
}

#[test]
fn error_is_session_error_for_agent_native_variants() {
    let session_errors: Vec<AgentError> = vec![
        AgentError::SessionClosed,
        AgentError::TurnLimitExceeded {
            message: "x".into(),
        },
        AgentError::ContextLengthExceeded {
            message: "x".into(),
        },
    ];
    for err in &session_errors {
        assert!(err.is_session_error(), "{err:?} should be a session error");
        assert!(!err.is_tool_error(), "{err:?} should NOT be a tool error");
    }
}

/// All non-retryable SDK errors are session-level per spec 2.8:
/// "authentication error, context overflow, or other non-retryable error"
/// → session transitions to CLOSED.
#[test]
fn error_is_session_error_for_all_non_retryable_sdk_variants() {
    let pd = ProviderDetails::default();
    let non_retryable: Vec<SdkError> = vec![
        SdkError::Authentication {
            message: "bad key".into(),
            details: pd.clone(),
        },
        SdkError::AccessDenied {
            message: "forbidden".into(),
            details: pd.clone(),
        },
        SdkError::NotFound {
            message: "no model".into(),
            details: pd.clone(),
        },
        SdkError::InvalidRequest {
            message: "bad params".into(),
            details: pd.clone(),
        },
        SdkError::ContextLength {
            message: "too long".into(),
            details: pd.clone(),
        },
        SdkError::QuotaExceeded {
            message: "billing".into(),
            details: pd.clone(),
        },
        SdkError::ContentFilter {
            message: "blocked".into(),
            details: pd,
        },
        SdkError::Configuration {
            message: "bad config".into(),
        },
        SdkError::Abort {
            message: "cancelled".into(),
        },
        SdkError::InvalidToolCall {
            message: "bad call".into(),
        },
        SdkError::NoObjectGenerated {
            message: "no json".into(),
        },
    ];
    for sdk_err in &non_retryable {
        assert!(
            !sdk_err.is_retryable(),
            "precondition: {sdk_err:?} should be non-retryable"
        );
        let err = AgentError::Sdk(sdk_err.clone());
        assert!(
            err.is_session_error(),
            "Sdk({sdk_err:?}) should be session-level (non-retryable)"
        );
        assert!(
            !err.is_tool_error(),
            "Sdk({sdk_err:?}) should NOT be a tool error"
        );
    }
}

/// Retryable SDK errors are NOT session-level — they are normally handled
/// by the SDK's retry layer before reaching the agent.
#[test]
fn error_is_not_session_error_for_retryable_sdk_variants() {
    let pd = ProviderDetails::default();
    let retryable: Vec<SdkError> = vec![
        SdkError::RateLimit {
            message: "429".into(),
            details: pd.clone(),
        },
        SdkError::Server {
            message: "500".into(),
            details: pd,
        },
        SdkError::RequestTimeout {
            message: "timed out".into(),
        },
        SdkError::Network {
            message: "dns".into(),
        },
        SdkError::Stream {
            message: "reset".into(),
        },
    ];
    for sdk_err in &retryable {
        assert!(
            sdk_err.is_retryable(),
            "precondition: {sdk_err:?} should be retryable"
        );
        let err = AgentError::Sdk(sdk_err.clone());
        assert!(
            !err.is_session_error(),
            "Sdk({sdk_err:?}) should NOT be session-level (retryable)"
        );
    }
}

// ===========================================================================
// AgentError — code() and serialization
// ===========================================================================

#[test]
fn error_code_values() {
    assert_eq!(
        AgentError::FileNotFound { path: "x".into() }.code(),
        "FILE_NOT_FOUND"
    );
    assert_eq!(
        AgentError::EditConflict { reason: "x".into() }.code(),
        "EDIT_CONFLICT"
    );
    assert_eq!(
        AgentError::ShellTimeout { timeout_ms: 1 }.code(),
        "SHELL_TIMEOUT"
    );
    assert_eq!(
        AgentError::ShellExitError {
            code: 1,
            stderr: String::new()
        }
        .code(),
        "SHELL_EXIT_ERROR"
    );
    assert_eq!(AgentError::SessionClosed.code(), "SESSION_CLOSED");
    assert_eq!(
        AgentError::Sdk(SdkError::Network {
            message: "x".into()
        })
        .code(),
        "SDK_ERROR"
    );
}

#[test]
fn error_serialize_json() -> AgentResult<()> {
    let err = AgentError::FileNotFound {
        path: "/tmp/nope".into(),
    };
    let json = serde_json::to_value(&err).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(json["code"], "FILE_NOT_FOUND");
    assert_eq!(json["message"], "file not found: /tmp/nope");
    Ok(())
}

// ===========================================================================
// SessionConfig (spec 2.2)
// ===========================================================================

#[test]
fn session_config_defaults_match_spec() {
    let config = SessionConfig::default();
    assert_eq!(config.max_turns, 0, "0 = unlimited");
    assert_eq!(config.max_tool_rounds_per_input, 200);
    assert_eq!(config.default_command_timeout_ms, 10_000);
    assert_eq!(config.max_command_timeout_ms, 600_000);
    assert_eq!(config.reasoning_effort, None);
    assert!(config.tool_output_limits.is_empty());
    assert!(config.enable_loop_detection);
    assert_eq!(config.loop_detection_window, 10);
    assert_eq!(config.max_subagent_depth, 1);
}

#[test]
fn session_config_serde_roundtrip() -> AgentResult<()> {
    let config = SessionConfig::default();
    let json = serde_json::to_string(&config).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: SessionConfig =
        serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(config, back);
    Ok(())
}

#[test]
fn session_config_from_partial_json() -> AgentResult<()> {
    let json = r#"{"max_turns": 50}"#;
    let config: SessionConfig =
        serde_json::from_str(json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(config.max_turns, 50);
    // All other fields should have spec defaults
    assert_eq!(config.max_tool_rounds_per_input, 200);
    assert_eq!(config.default_command_timeout_ms, 10_000);
    assert!(config.enable_loop_detection);
    Ok(())
}

#[test]
fn session_config_custom_tool_limits() -> AgentResult<()> {
    let json = r#"{"tool_output_limits": {"read_file": 100000}}"#;
    let config: SessionConfig =
        serde_json::from_str(json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(config.tool_output_limits.get("read_file"), Some(&100_000));
    Ok(())
}

#[test]
fn session_config_with_reasoning_effort() -> AgentResult<()> {
    let json = r#"{"reasoning_effort": "high"}"#;
    let config: SessionConfig =
        serde_json::from_str(json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(config.reasoning_effort, Some(ReasoningEffort::High));
    Ok(())
}

#[test]
fn session_config_invalid_json_rejected() {
    let result = serde_json::from_str::<SessionConfig>("not json");
    assert!(result.is_err());
}

#[test]
fn session_config_wrong_type_rejected() {
    let result = serde_json::from_str::<SessionConfig>(r#"{"max_turns": "fifty"}"#);
    assert!(result.is_err());
}

// ===========================================================================
// ReasoningEffort (spec 2.2)
// ===========================================================================

#[test]
fn reasoning_effort_known_values() -> AgentResult<()> {
    for (json_str, expected) in [
        ("\"low\"", ReasoningEffort::Low),
        ("\"medium\"", ReasoningEffort::Medium),
        ("\"high\"", ReasoningEffort::High),
    ] {
        let parsed: ReasoningEffort =
            serde_json::from_str(json_str).map_err(|e| AgentError::ValidationError {
                reason: e.to_string(),
            })?;
        assert_eq!(parsed, expected);
    }
    Ok(())
}

#[test]
fn reasoning_effort_custom_value() -> AgentResult<()> {
    let parsed: ReasoningEffort =
        serde_json::from_str("\"turbo\"").map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(parsed, ReasoningEffort::Custom("turbo".into()));
    assert_eq!(parsed.as_str(), "turbo");
    Ok(())
}

#[test]
fn reasoning_effort_as_str() {
    assert_eq!(ReasoningEffort::Low.as_str(), "low");
    assert_eq!(ReasoningEffort::Medium.as_str(), "medium");
    assert_eq!(ReasoningEffort::High.as_str(), "high");
    assert_eq!(ReasoningEffort::Custom("turbo".into()).as_str(), "turbo");
}

#[test]
fn reasoning_effort_display() {
    assert_eq!(format!("{}", ReasoningEffort::High), "high");
    assert_eq!(
        format!("{}", ReasoningEffort::Custom("turbo".into())),
        "turbo"
    );
}

#[test]
fn reasoning_effort_roundtrip() -> AgentResult<()> {
    for effort in [
        ReasoningEffort::Low,
        ReasoningEffort::Medium,
        ReasoningEffort::High,
    ] {
        let json = serde_json::to_string(&effort).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
        let back: ReasoningEffort =
            serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
                reason: e.to_string(),
            })?;
        assert_eq!(effort, back);
    }
    Ok(())
}

// ===========================================================================
// SessionState (spec 2.3)
// ===========================================================================

#[test]
fn session_state_default_is_idle() {
    assert_eq!(SessionState::default(), SessionState::Idle);
}

#[test]
fn session_state_all_variants_exist() {
    let _idle = SessionState::Idle;
    let _processing = SessionState::Processing;
    let _awaiting = SessionState::AwaitingInput;
    let _closed = SessionState::Closed;
}

#[test]
fn session_state_equality() {
    assert_eq!(SessionState::Idle, SessionState::Idle);
    assert_ne!(SessionState::Idle, SessionState::Processing);
}

#[test]
fn session_state_serde_roundtrip() -> AgentResult<()> {
    for state in [
        SessionState::Idle,
        SessionState::Processing,
        SessionState::AwaitingInput,
        SessionState::Closed,
    ] {
        let json = serde_json::to_string(&state).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
        let back: SessionState =
            serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
                reason: e.to_string(),
            })?;
        assert_eq!(state, back, "roundtrip failed for {state:?}");
    }
    Ok(())
}

#[test]
fn session_state_serializes_screaming_snake() -> AgentResult<()> {
    let json = serde_json::to_string(&SessionState::AwaitingInput).map_err(|e| {
        AgentError::ValidationError {
            reason: e.to_string(),
        }
    })?;
    assert_eq!(json, "\"AWAITING_INPUT\"");
    Ok(())
}

#[test]
fn session_state_invalid_string_rejected() {
    let result = serde_json::from_str::<SessionState>("\"INVALID\"");
    assert!(result.is_err());
}

// ===========================================================================
// Turn (spec 2.4)
// ===========================================================================

#[test]
fn turn_user_construction() {
    let turn = Turn::user("hello");
    assert!(matches!(&turn, Turn::User { content, .. } if content == "hello"));
}

#[test]
fn turn_user_has_valid_timestamp() {
    let turn = Turn::user("test");
    let ts = turn.timestamp();
    assert!(!ts.is_empty(), "timestamp must not be empty");
    // Basic ISO 8601 check: contains 'T' and timezone info
    assert!(ts.contains('T'), "timestamp should be ISO 8601: {ts}");
}

#[test]
fn turn_assistant_construction() {
    let turn = Turn::assistant("response text");
    assert!(
        matches!(&turn, Turn::Assistant { content, tool_calls, reasoning, .. }
            if content == "response text"
            && tool_calls.is_empty()
            && reasoning.is_none()
        )
    );
}

#[test]
fn turn_tool_results_construction() {
    let results = vec![ToolResult {
        tool_call_id: "tc_1".into(),
        content: serde_json::Value::String("ok".into()),
        is_error: false,
    }];
    let turn = Turn::tool_results(results);
    assert!(matches!(&turn, Turn::ToolResults { results, .. } if results.len() == 1));
}

#[test]
fn turn_system_construction() {
    let turn = Turn::system("you are a helpful agent");
    assert!(matches!(&turn, Turn::System { content, .. } if content == "you are a helpful agent"));
}

#[test]
fn turn_steering_construction() {
    let turn = Turn::steering("try a different approach");
    assert!(
        matches!(&turn, Turn::Steering { content, .. } if content == "try a different approach")
    );
}

#[test]
fn turn_timestamp_accessor() {
    let turn = Turn::user("hi");
    let ts = turn.timestamp();
    assert!(!ts.is_empty());
}

#[test]
fn turn_serde_roundtrip_user() -> AgentResult<()> {
    let turn = Turn::user("test input");
    let json = serde_json::to_string(&turn).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: Turn = serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(turn, back);
    Ok(())
}

#[test]
fn turn_serde_roundtrip_assistant_with_tool_calls() -> AgentResult<()> {
    let turn = Turn::Assistant {
        content: "Let me read that file.".into(),
        tool_calls: vec![ToolCall {
            id: "tc_42".into(),
            name: "read_file".into(),
            arguments: serde_json::json!({"file_path": "/tmp/test.rs"}),
            raw_arguments: None,
            parse_error: None,
        }],
        reasoning: Some("thinking...".into()),
        usage: Usage {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            ..Usage::default()
        },
        response_id: Some("resp_123".into()),
        timestamp: "2025-06-15T12:00:00+00:00".into(),
    };
    let json = serde_json::to_string(&turn).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: Turn = serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(turn, back);
    Ok(())
}

#[test]
fn turn_tagged_serialization() -> AgentResult<()> {
    let turn = Turn::user("hi");
    let val = serde_json::to_value(&turn).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(val["type"], "user");
    assert_eq!(val["content"], "hi");

    let steering = Turn::steering("steer");
    let val = serde_json::to_value(&steering).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(val["type"], "steering");
    Ok(())
}

#[test]
fn turn_deser_missing_timestamp_rejected() {
    // Timestamp is now required — missing it should fail deserialization.
    let json = r#"{"type": "user", "content": "hi"}"#;
    let result = serde_json::from_str::<Turn>(json);
    assert!(
        result.is_err(),
        "missing required timestamp field should fail"
    );
}

#[test]
fn turn_deser_with_explicit_timestamp() -> AgentResult<()> {
    let json = r#"{"type": "user", "content": "hi", "timestamp": "2025-01-01T00:00:00Z"}"#;
    let turn: Turn = serde_json::from_str(json).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(turn.timestamp(), "2025-01-01T00:00:00Z");
    Ok(())
}

// ===========================================================================
// ExecResult (spec 4.1)
// ===========================================================================

#[test]
fn exec_result_construction() {
    let result = ExecResult {
        stdout: "hello\n".into(),
        stderr: String::new(),
        exit_code: 0,
        timed_out: false,
        duration_ms: 42,
    };
    assert_eq!(result.exit_code, 0);
    assert!(!result.timed_out);
}

#[test]
fn exec_result_serde_roundtrip() -> AgentResult<()> {
    let result = ExecResult {
        stdout: "output".into(),
        stderr: "warn".into(),
        exit_code: 1,
        timed_out: true,
        duration_ms: 10_000,
    };
    let json = serde_json::to_string(&result).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: ExecResult =
        serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(result, back);
    Ok(())
}

#[test]
fn exec_result_missing_field_rejected() {
    let json = r#"{"stdout": "ok", "exit_code": 0}"#;
    let result = serde_json::from_str::<ExecResult>(json);
    assert!(result.is_err(), "missing required fields should fail");
}

// ===========================================================================
// DirEntry (spec 4.1)
// ===========================================================================

#[test]
fn dir_entry_file() {
    let entry = DirEntry {
        name: "main.rs".into(),
        is_dir: false,
        size: Some(1234),
    };
    assert!(!entry.is_dir);
    assert_eq!(entry.size, Some(1234));
}

#[test]
fn dir_entry_directory() {
    let entry = DirEntry {
        name: "src".into(),
        is_dir: true,
        size: None,
    };
    assert!(entry.is_dir);
    assert_eq!(entry.size, None);
}

#[test]
fn dir_entry_serde_roundtrip() -> AgentResult<()> {
    let entry = DirEntry {
        name: "test.txt".into(),
        is_dir: false,
        size: Some(42),
    };
    let json = serde_json::to_string(&entry).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: DirEntry = serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    assert_eq!(entry, back);
    Ok(())
}

// ===========================================================================
// GrepOptions (spec 3.3)
// ===========================================================================

#[test]
fn grep_options_defaults() {
    let opts = GrepOptions::default();
    assert_eq!(opts.glob_filter, None);
    assert!(!opts.case_insensitive);
    assert_eq!(opts.max_results, 100);
}

#[test]
fn grep_options_serde_roundtrip() -> AgentResult<()> {
    let opts = GrepOptions {
        glob_filter: Some("*.rs".into()),
        case_insensitive: true,
        max_results: 50,
    };
    let json = serde_json::to_string(&opts).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: GrepOptions =
        serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(opts, back);
    Ok(())
}

// ===========================================================================
// EventKind (spec 2.9)
// ===========================================================================

#[test]
fn event_kind_all_variants_exist() {
    let kinds = [
        EventKind::SessionStart,
        EventKind::SessionEnd,
        EventKind::UserInput,
        EventKind::AssistantTextStart,
        EventKind::AssistantTextDelta,
        EventKind::AssistantTextEnd,
        EventKind::ToolCallStart,
        EventKind::ToolCallOutputDelta,
        EventKind::ToolCallEnd,
        EventKind::SteeringInjected,
        EventKind::TurnLimit,
        EventKind::LoopDetection,
        EventKind::Error,
    ];
    assert_eq!(kinds.len(), 13, "spec defines 13 event kinds");
}

#[test]
fn event_kind_serde_screaming_snake() -> AgentResult<()> {
    let json = serde_json::to_string(&EventKind::ToolCallStart).map_err(|e| {
        AgentError::ValidationError {
            reason: e.to_string(),
        }
    })?;
    assert_eq!(json, "\"TOOL_CALL_START\"");

    let json = serde_json::to_string(&EventKind::AssistantTextDelta).map_err(|e| {
        AgentError::ValidationError {
            reason: e.to_string(),
        }
    })?;
    assert_eq!(json, "\"ASSISTANT_TEXT_DELTA\"");
    Ok(())
}

#[test]
fn event_kind_equality() {
    assert_eq!(EventKind::Error, EventKind::Error);
    assert_ne!(EventKind::Error, EventKind::SessionEnd);
}

#[test]
fn event_kind_invalid_string_rejected() {
    let result = serde_json::from_str::<EventKind>("\"NOT_A_KIND\"");
    assert!(result.is_err());
}

// ===========================================================================
// SessionEvent (spec 2.9)
// ===========================================================================

#[test]
fn session_event_construction() {
    let mut data = serde_json::Map::new();
    data.insert("text".into(), serde_json::Value::String("hello".into()));

    let event = SessionEvent {
        kind: EventKind::AssistantTextEnd,
        timestamp: "2025-01-01T00:00:00Z".into(),
        session_id: "session_123".into(),
        data,
    };
    assert_eq!(event.kind, EventKind::AssistantTextEnd);
    assert_eq!(event.session_id, "session_123");
    assert_eq!(event.data["text"], "hello");
}

#[test]
fn session_event_serde_roundtrip() -> AgentResult<()> {
    let event = SessionEvent {
        kind: EventKind::ToolCallEnd,
        timestamp: "2025-06-15T12:00:00Z".into(),
        session_id: "session_abc".into(),
        data: serde_json::Map::new(),
    };
    let json = serde_json::to_string(&event).map_err(|e| AgentError::ValidationError {
        reason: e.to_string(),
    })?;
    let back: SessionEvent =
        serde_json::from_str(&json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert_eq!(event, back);
    Ok(())
}

#[test]
fn session_event_empty_data() -> AgentResult<()> {
    let json = r#"{
        "kind": "SESSION_START",
        "timestamp": "2025-01-01T00:00:00Z",
        "session_id": "s1"
    }"#;
    let event: SessionEvent =
        serde_json::from_str(json).map_err(|e| AgentError::ValidationError {
            reason: e.to_string(),
        })?;
    assert!(event.data.is_empty());
    Ok(())
}

#[test]
fn session_event_missing_required_fields_rejected() {
    let json = r#"{"kind": "SESSION_START"}"#;
    let result = serde_json::from_str::<SessionEvent>(json);
    assert!(
        result.is_err(),
        "missing timestamp and session_id should fail"
    );
}
