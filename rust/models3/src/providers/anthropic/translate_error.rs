use crate::error::SdkError;
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};

static CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "anthropic",
    error_code_pointers: &["/error/type"],
    allow_numeric_codes: false,
    quota_keywords: &["billing", "quota"],
    quota_codes: &["billing_error", "quota_exceeded"],
};

/// Apply Anthropic-specific error enrichment and message/code-based classification.
#[must_use]
pub fn translate_error(error: SdkError) -> SdkError {
    common_translate(error, &CONFIG)
}
