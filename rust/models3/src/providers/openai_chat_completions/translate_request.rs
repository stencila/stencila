use crate::error::SdkResult;
use crate::providers::common::chat_completions::{ChatCompletionsConfig, NullContentHandling};
use crate::types::request::Request;

pub use crate::providers::common::chat_completions::TranslatedChatCompletionsRequest;

/// Backwards-compatible alias.
pub type TranslatedRequest = TranslatedChatCompletionsRequest;

static CONFIG: ChatCompletionsConfig = ChatCompletionsConfig {
    provider_name: "openai_chat_completions",
    option_namespaces: &["openai_chat_completions", "openai_compatible"],
    builtin_tools_guard_namespaces: &["openai", "openai_chat_completions", "openai_compatible"],
    null_content_handling: NullContentHandling::ExplicitNull,
};

/// Translate a unified request into an OpenAI-compatible Chat Completions request.
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` for unsupported content or invalid
/// provider options.
pub fn translate_request(
    request: &Request,
    stream: bool,
) -> SdkResult<TranslatedChatCompletionsRequest> {
    crate::providers::common::chat_completions::translate_request(request, stream, &CONFIG)
}
