//! Spec Section 6 conformance tests.
//!
//! Target areas:
//! - Status code and provider error body classification
//! - Retryable vs non-retryable behavior
//! - Error message classification

use stencila_models3::error::{ErrorClassification, ProviderDetails, SdkError};

// ── is_retryable() ────────────────────────────────────────────────────

#[test]
fn retryable_errors() {
    let retryable_errors = vec![
        SdkError::RateLimit {
            message: "too many requests".into(),
            details: ProviderDetails::default(),
        },
        SdkError::Server {
            message: "internal error".into(),
            details: ProviderDetails::default(),
        },
        SdkError::RequestTimeout {
            message: "timed out".into(),
        },
        SdkError::Network {
            message: "connection reset".into(),
        },
        SdkError::Stream {
            message: "stream broken".into(),
        },
    ];

    for err in retryable_errors {
        assert!(err.is_retryable(), "{err} should be retryable");
    }
}

#[test]
fn non_retryable_errors() {
    let non_retryable_errors = vec![
        SdkError::Authentication {
            message: "invalid key".into(),
            details: ProviderDetails::default(),
        },
        SdkError::AccessDenied {
            message: "forbidden".into(),
            details: ProviderDetails::default(),
        },
        SdkError::NotFound {
            message: "model not found".into(),
            details: ProviderDetails::default(),
        },
        SdkError::InvalidRequest {
            message: "bad param".into(),
            details: ProviderDetails::default(),
        },
        SdkError::ContextLength {
            message: "too long".into(),
            details: ProviderDetails::default(),
        },
        SdkError::QuotaExceeded {
            message: "quota used".into(),
            details: ProviderDetails::default(),
        },
        SdkError::ContentFilter {
            message: "blocked".into(),
            details: ProviderDetails::default(),
        },
        SdkError::Configuration {
            message: "bad config".into(),
        },
        SdkError::Abort {
            message: "cancelled".into(),
        },
        SdkError::InvalidToolCall {
            message: "bad args".into(),
        },
        SdkError::NoObjectGenerated {
            message: "parse fail".into(),
        },
    ];

    for err in non_retryable_errors {
        assert!(!err.is_retryable(), "{err} should NOT be retryable");
    }
}

// ── from_status_code() ────────────────────────────────────────────────

#[test]
fn status_code_mapping() {
    let cases = vec![
        (400, false, "InvalidRequest"),
        (401, false, "Authentication"),
        (403, false, "AccessDenied"),
        (404, false, "NotFound"),
        (408, true, "RequestTimeout"),
        (413, false, "ContextLength"),
        (422, false, "InvalidRequest"),
        (429, true, "RateLimit"),
        (500, true, "Server"),
        (502, true, "Server"),
        (503, true, "Server"),
        (504, true, "Server"),
    ];

    for (status, expected_retryable, label) in cases {
        let err =
            SdkError::from_status_code(status, "test", Some("provider".into()), None, None, None);
        assert_eq!(
            err.is_retryable(),
            expected_retryable,
            "status {status} ({label}): retryable mismatch"
        );
    }
}

#[test]
fn status_code_preserved_in_error() {
    let err =
        SdkError::from_status_code(429, "rate limited", Some("openai".into()), None, None, None);
    assert_eq!(err.status_code(), Some(429));
}

#[test]
fn retry_after_preserved() {
    let err = SdkError::from_status_code(
        429,
        "rate limited",
        Some("openai".into()),
        None,
        Some(30.0),
        None,
    );
    assert_eq!(err.retry_after(), Some(30.0));
}

#[test]
fn retry_after_none_for_non_rate_limit() {
    let err = SdkError::from_status_code(401, "auth error", None, None, None, None);
    assert_eq!(err.retry_after(), None);
}

// ── Unknown status codes default to retryable (spec §6.3) ─────────────

#[test]
fn unknown_status_code_is_retryable() {
    let err = SdkError::from_status_code(599, "unknown", Some("test".into()), None, None, None);
    assert!(err.is_retryable(), "unknown status should be retryable");
}

#[test]
fn unknown_status_retryable_metadata_agrees() -> Result<(), Box<dyn std::error::Error>> {
    let err = SdkError::from_status_code(599, "unknown", Some("test".into()), None, None, None);
    let json = serde_json::to_string(&err)?;
    assert!(
        json.contains("\"retryable\":true"),
        "retryable field must be true for unknown status: {json}"
    );
    Ok(())
}

// ── error_code preservation ───────────────────────────────────────────

#[test]
fn error_code_preserved_in_details() -> Result<(), Box<dyn std::error::Error>> {
    let err = SdkError::from_status_code(
        429,
        "rate limited",
        Some("anthropic".into()),
        Some("rate_limit_exceeded".into()),
        None,
        None,
    );
    if let SdkError::RateLimit { details, .. } = &err {
        assert_eq!(details.error_code.as_deref(), Some("rate_limit_exceeded"));
    } else {
        return Err(format!("expected RateLimit variant, got {err:?}").into());
    }
    Ok(())
}

// ── classify_from_message() ───────────────────────────────────────────

#[test]
fn classify_not_found() {
    assert_eq!(
        SdkError::classify_from_message("model not found"),
        Some(ErrorClassification::NotFound)
    );
    assert_eq!(
        SdkError::classify_from_message("resource does not exist"),
        Some(ErrorClassification::NotFound)
    );
}

#[test]
fn classify_authentication() {
    assert_eq!(
        SdkError::classify_from_message("Unauthorized access"),
        Some(ErrorClassification::Authentication)
    );
    assert_eq!(
        SdkError::classify_from_message("invalid key provided"),
        Some(ErrorClassification::Authentication)
    );
}

#[test]
fn classify_context_length() {
    assert_eq!(
        SdkError::classify_from_message("context length exceeded"),
        Some(ErrorClassification::ContextLength)
    );
    assert_eq!(
        SdkError::classify_from_message("too many tokens in request"),
        Some(ErrorClassification::ContextLength)
    );
}

#[test]
fn classify_content_filter() {
    assert_eq!(
        SdkError::classify_from_message("content filter triggered"),
        Some(ErrorClassification::ContentFilter)
    );
    assert_eq!(
        SdkError::classify_from_message("blocked for safety reasons"),
        Some(ErrorClassification::ContentFilter)
    );
}

#[test]
fn classify_unknown() {
    assert_eq!(
        SdkError::classify_from_message("something random happened"),
        None
    );
}

// ── Error Display ─────────────────────────────────────────────────────

#[test]
fn error_display_format() {
    let err = SdkError::Authentication {
        message: "invalid API key".into(),
        details: ProviderDetails {
            provider: Some("anthropic".into()),
            status_code: Some(401),
            ..ProviderDetails::default()
        },
    };
    let display = format!("{err}");
    assert!(display.contains("invalid API key"));
}

// ── Serde round-trip ──────────────────────────────────────────────────

#[test]
fn error_serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let err = SdkError::RateLimit {
        message: "too fast".into(),
        details: ProviderDetails {
            provider: Some("openai".into()),
            status_code: Some(429),
            retry_after: Some(5.0),
            ..ProviderDetails::default()
        },
    };
    let json = serde_json::to_string(&err)?;
    let back: SdkError = serde_json::from_str(&json)?;
    assert_eq!(back, err);
    Ok(())
}
