use reqwest::header::HeaderMap;
use serde_json::Value;

use crate::error::SdkResult;
use crate::types::response::Response;

/// Translate a Chat Completions response into a unified response.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` if required fields are missing.
pub fn translate_response(raw_response: Value, headers: Option<&HeaderMap>) -> SdkResult<Response> {
    crate::providers::common::chat_completions::translate_response(
        raw_response,
        headers,
        "openai_chat_completions",
    )
}
