use crate::error::SdkResult;
use crate::providers::common::chat_completions::{
    ChatCompletionsConfig, NullContentHandling, TranslatedChatCompletionsRequest,
};
use crate::types::request::Request;

static CONFIG: ChatCompletionsConfig = ChatCompletionsConfig {
    provider_name: "deepseek",
    option_namespaces: &["deepseek", "openai_compatible"],
    builtin_tools_guard_namespaces: &["openai", "deepseek", "openai_compatible"],
    null_content_handling: NullContentHandling::ExplicitNull,
};

/// Translate a unified request into a DeepSeek Chat Completions request.
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
