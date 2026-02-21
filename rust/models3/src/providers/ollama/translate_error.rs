use crate::error::SdkError;
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};

pub(crate) static ERROR_CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "ollama",
    error_code_pointers: &["/error/code", "/error/type"],
    allow_numeric_codes: false,
    quota_keywords: &["quota", "insufficient_quota"],
    quota_codes: &["insufficient_quota", "quota_exceeded"],
};

/// Apply Ollama error enrichment and classification.
#[must_use]
pub fn translate_error(error: SdkError) -> SdkError {
    common_translate(error, &ERROR_CONFIG)
}
