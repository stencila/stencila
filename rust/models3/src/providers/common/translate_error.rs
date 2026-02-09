use crate::error::{ErrorClassification, ProviderDetails, SdkError};

/// Configuration for provider-specific error translation.
///
/// Each provider creates a `const` instance of this struct and passes
/// it to [`translate_error`] to get shared enrichment + refinement
/// with provider-specific details.
pub(crate) struct ErrorConfig {
    /// Provider name set on `details.provider` if missing.
    pub provider_name: &'static str,
    /// JSON pointers to try for extracting an error code from the raw
    /// response, tried in order until one succeeds.
    pub error_code_pointers: &'static [&'static str],
    /// If true, also accept numeric values at the error code pointers
    /// (Gemini uses numeric `/error/code`).
    pub allow_numeric_codes: bool,
    /// Lowercase keywords in the error message that indicate quota exhaustion.
    pub quota_keywords: &'static [&'static str],
    /// Error code values that indicate quota exhaustion.
    pub quota_codes: &'static [&'static str],
}

/// Translate an error using the shared enrichment and refinement pipeline.
///
/// 1. Sets `details.provider` if missing.
/// 2. Extracts `details.error_code` from the raw response body.
/// 3. Refines `RateLimit` → `QuotaExceeded` when quota keywords/codes match.
/// 4. Refines `InvalidRequest`/`Server` via message-based classification.
#[must_use]
pub(crate) fn translate_error(error: SdkError, config: &ErrorConfig) -> SdkError {
    match error {
        SdkError::Authentication {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::Authentication { message, details }
        }
        SdkError::AccessDenied {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::AccessDenied { message, details }
        }
        SdkError::NotFound {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::NotFound { message, details }
        }
        SdkError::InvalidRequest {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            refine_provider_error(message, details, FallbackVariant::InvalidRequest, config)
        }
        SdkError::RateLimit {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            if is_quota_exceeded(&message, &details, config) {
                SdkError::QuotaExceeded { message, details }
            } else {
                SdkError::RateLimit { message, details }
            }
        }
        SdkError::Server {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            refine_provider_error(message, details, FallbackVariant::Server, config)
        }
        SdkError::ContentFilter {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::ContentFilter { message, details }
        }
        SdkError::ContextLength {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::ContextLength { message, details }
        }
        SdkError::QuotaExceeded {
            message,
            mut details,
        } => {
            enrich_details(&mut details, config);
            SdkError::QuotaExceeded { message, details }
        }
        other => other,
    }
}

fn refine_provider_error(
    message: String,
    details: ProviderDetails,
    fallback: FallbackVariant,
    config: &ErrorConfig,
) -> SdkError {
    if is_quota_exceeded(&message, &details, config) {
        return SdkError::QuotaExceeded { message, details };
    }

    match SdkError::classify_from_message(&message) {
        Some(ErrorClassification::NotFound) => SdkError::NotFound { message, details },
        Some(ErrorClassification::Authentication) => SdkError::Authentication { message, details },
        Some(ErrorClassification::ContextLength) => SdkError::ContextLength { message, details },
        Some(ErrorClassification::ContentFilter) => SdkError::ContentFilter { message, details },
        None => match fallback {
            FallbackVariant::InvalidRequest => SdkError::InvalidRequest { message, details },
            FallbackVariant::Server => SdkError::Server { message, details },
        },
    }
}

#[derive(Debug, Clone, Copy)]
enum FallbackVariant {
    InvalidRequest,
    Server,
}

fn enrich_details(details: &mut ProviderDetails, config: &ErrorConfig) {
    if details.provider.is_none() {
        details.provider = Some(config.provider_name.to_string());
    }

    if details.error_code.is_none()
        && let Some(raw) = details.raw.as_ref()
    {
        for &pointer in config.error_code_pointers {
            if let Some(v) = raw.pointer(pointer) {
                if let Some(s) = v.as_str() {
                    details.error_code = Some(s.to_string());
                    break;
                }
                if config.allow_numeric_codes
                    && let Some(n) = v.as_u64()
                {
                    details.error_code = Some(n.to_string());
                    break;
                }
            }
        }
    }
}

fn is_quota_exceeded(message: &str, details: &ProviderDetails, config: &ErrorConfig) -> bool {
    let lower = message.to_lowercase();
    for &kw in config.quota_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    if let Some(code) = details.error_code.as_deref() {
        for &qc in config.quota_codes {
            if code == qc {
                return true;
            }
        }
    }

    false
}
