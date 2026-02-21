use crate::error::SdkError;
use crate::providers::common::translate_error::{ErrorConfig, translate_error as common_translate};

static CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "openai",
    error_code_pointers: &["/error/code", "/error/type"],
    allow_numeric_codes: false,
    quota_keywords: &["quota", "insufficient_quota"],
    quota_codes: &["insufficient_quota", "quota_exceeded"],
};

/// Apply OpenAI-specific error enrichment and message/code-based classification.
///
/// Adds a hint when the `ChatGPT` backend codex endpoint rejects a model,
/// directing the user to set `OPENAI_API_KEY` for the standard API.
#[must_use]
pub fn translate_error(error: SdkError) -> SdkError {
    let error = add_codex_model_hint(error);
    common_translate(error, &CONFIG)
}

/// When the `ChatGPT` backend rejects a model ("not supported when using Codex"),
/// append guidance so the user knows to set `OPENAI_API_KEY`.
fn add_codex_model_hint(error: SdkError) -> SdkError {
    if let SdkError::InvalidRequest { message, details } = &error
        && message.contains("not supported when using Codex")
    {
        return SdkError::InvalidRequest {
            message: format!(
                "{message}. To use this model, set OPENAI_API_KEY \
                     (the ChatGPT backend only supports newer models like gpt-5)"
            ),
            details: details.clone(),
        };
    }
    error
}
