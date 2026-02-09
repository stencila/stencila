use crate::error::SdkError;
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};

static CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "openai_chat_completions",
    error_code_pointers: &["/error/code", "/error/type"],
    allow_numeric_codes: false,
    quota_keywords: &["quota", "insufficient_quota"],
    quota_codes: &["insufficient_quota", "quota_exceeded"],
};

/// Apply OpenAI-compatible error enrichment and classification.
#[must_use]
pub fn translate_error(error: SdkError) -> SdkError {
    common_translate(error, &CONFIG)
}
