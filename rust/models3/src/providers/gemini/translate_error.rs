use crate::error::SdkError;
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};

static CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "gemini",
    error_code_pointers: &["/error/status", "/error/code"],
    allow_numeric_codes: true,
    quota_keywords: &["quota", "resource_exhausted"],
    quota_codes: &["RESOURCE_EXHAUSTED", "quota_exceeded"],
};

/// Apply Gemini-specific error enrichment and message/code-based classification.
#[must_use]
pub fn translate_error(error: SdkError) -> SdkError {
    common_translate(error, &CONFIG)
}
