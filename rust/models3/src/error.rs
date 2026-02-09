use serde::{Deserialize, Serialize};

/// Convenience alias for `Result<T, SdkError>`.
pub type SdkResult<T> = Result<T, SdkError>;

/// The unified error type for the SDK.
///
/// All errors from providers, network, configuration, and SDK logic
/// are represented as variants of this enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum SdkError {
    // --- Provider errors ---
    /// 401: Invalid API key or expired token.
    #[error("authentication error: {message}")]
    Authentication {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 403: Insufficient permissions.
    #[error("access denied: {message}")]
    AccessDenied {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 404: Model or endpoint not found.
    #[error("not found: {message}")]
    NotFound {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 400/422: Malformed request or invalid parameters.
    #[error("invalid request: {message}")]
    InvalidRequest {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 429: Rate limit exceeded.
    #[error("rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 500-504: Provider internal error.
    #[error("server error: {message}")]
    Server {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// Response blocked by safety/content filter.
    #[error("content filter: {message}")]
    ContentFilter {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// 413: Input + output exceeds context window.
    #[error("context length exceeded: {message}")]
    ContextLength {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    /// Billing or usage quota exhausted.
    #[error("quota exceeded: {message}")]
    QuotaExceeded {
        message: String,
        #[serde(flatten)]
        details: ProviderDetails,
    },

    // --- SDK errors ---
    /// Request or stream timeout.
    #[error("request timeout: {message}")]
    RequestTimeout { message: String },

    /// Request cancelled via abort signal.
    #[error("request aborted: {message}")]
    Abort { message: String },

    /// Network-level failure.
    #[error("network error: {message}")]
    Network { message: String },

    /// Error during stream consumption.
    #[error("stream error: {message}")]
    Stream { message: String },

    /// Tool call arguments failed validation.
    #[error("invalid tool call: {message}")]
    InvalidToolCall { message: String },

    /// Structured output parsing/validation failed.
    #[error("no object generated: {message}")]
    NoObjectGenerated { message: String },

    /// SDK misconfiguration (missing provider, bad config, etc.).
    #[error("configuration error: {message}")]
    Configuration { message: String },
}

/// Additional details for provider-originated errors.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProviderDetails {
    /// Which provider returned the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// HTTP status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// Provider-specific error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Whether this error is safe to retry.
    ///
    /// Per spec, every provider error carries this as a data field.
    /// The authoritative source is `SdkError::is_retryable()`, but this
    /// field ensures the property survives serialization round-trips.
    #[serde(default)]
    pub retryable: bool,
    /// Seconds to wait before retrying (from Retry-After header).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<f64>,
    /// Raw error response body from provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

impl SdkError {
    /// Whether this error is safe to retry.
    ///
    /// Each variant is listed explicitly so that adding a new variant
    /// forces a compile-time decision about its retry behavior.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn is_retryable(&self) -> bool {
        match self {
            // Retryable: transient errors
            Self::RateLimit { .. } => true,
            Self::Server { .. } => true,
            Self::RequestTimeout { .. } => true,
            Self::Network { .. } => true,
            Self::Stream { .. } => true,

            // Not retryable: client mistakes or deterministic failures
            Self::Authentication { .. } => false,
            Self::AccessDenied { .. } => false,
            Self::NotFound { .. } => false,
            Self::InvalidRequest { .. } => false,
            Self::ContextLength { .. } => false,
            Self::QuotaExceeded { .. } => false,
            Self::ContentFilter { .. } => false,
            Self::Configuration { .. } => false,
            Self::Abort { .. } => false,
            Self::InvalidToolCall { .. } => false,
            Self::NoObjectGenerated { .. } => false,
        }
    }

    /// Extract the HTTP status code, if this is a provider error.
    #[must_use]
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Authentication { details, .. }
            | Self::AccessDenied { details, .. }
            | Self::NotFound { details, .. }
            | Self::InvalidRequest { details, .. }
            | Self::RateLimit { details, .. }
            | Self::Server { details, .. }
            | Self::ContentFilter { details, .. }
            | Self::ContextLength { details, .. }
            | Self::QuotaExceeded { details, .. } => details.status_code,
            _ => None,
        }
    }

    /// Extract the retry-after delay in seconds, if present.
    #[must_use]
    pub fn retry_after(&self) -> Option<f64> {
        match self {
            Self::RateLimit { details, .. } | Self::Server { details, .. } => details.retry_after,
            _ => None,
        }
    }

    /// Classify an HTTP status code into the appropriate error variant.
    ///
    /// # Errors
    ///
    /// This is a constructor — it always returns an `SdkError` variant
    /// (not wrapped in `Result`).
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_status_code(
        status: u16,
        message: impl Into<String>,
        provider: Option<String>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    ) -> Self {
        let message = message.into();
        let make_details = |retryable: bool| ProviderDetails {
            provider: provider.clone(),
            status_code: Some(status),
            error_code: error_code.clone(),
            retryable,
            retry_after,
            raw: raw.clone(),
        };

        match status {
            400 | 422 => Self::InvalidRequest {
                message,
                details: make_details(false),
            },
            401 => Self::Authentication {
                message,
                details: make_details(false),
            },
            403 => Self::AccessDenied {
                message,
                details: make_details(false),
            },
            404 => Self::NotFound {
                message,
                details: make_details(false),
            },
            408 => Self::RequestTimeout { message },
            413 => Self::ContextLength {
                message,
                details: make_details(false),
            },
            429 => Self::RateLimit {
                message,
                details: make_details(true),
            },
            // 500-504 and unknown status codes default to retryable per spec §6.3
            _ => Self::Server {
                message,
                details: make_details(true),
            },
        }
    }

    /// Attempt to classify an error from its message text when the status
    /// code is ambiguous or unavailable.
    #[must_use]
    pub fn classify_from_message(message: &str) -> Option<ErrorClassification> {
        let lower = message.to_lowercase();
        if lower.contains("not found") || lower.contains("does not exist") {
            Some(ErrorClassification::NotFound)
        } else if lower.contains("unauthorized") || lower.contains("invalid key") {
            Some(ErrorClassification::Authentication)
        } else if lower.contains("context length") || lower.contains("too many tokens") {
            Some(ErrorClassification::ContextLength)
        } else if lower.contains("content filter") || lower.contains("safety") {
            Some(ErrorClassification::ContentFilter)
        } else {
            None
        }
    }
}

/// Classification hints derived from error message text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClassification {
    NotFound,
    Authentication,
    ContextLength,
    ContentFilter,
}
