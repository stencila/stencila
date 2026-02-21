use crate::error::SdkResult;
use crate::providers::common::chat_completions::{
    ChatCompletionsConfig, NullContentHandling, TranslatedChatCompletionsRequest,
};
use crate::types::request::Request;

static CONFIG: ChatCompletionsConfig = ChatCompletionsConfig {
    provider_name: "ollama",
    option_namespaces: &["ollama", "openai_compatible"],
    builtin_tools_guard_namespaces: &["openai", "ollama", "openai_compatible"],
    null_content_handling: NullContentHandling::ExplicitNull,
};

/// Translate a unified request into an Ollama Chat Completions request.
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
