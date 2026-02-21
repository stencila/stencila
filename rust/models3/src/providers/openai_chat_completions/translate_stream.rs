use crate::error::SdkResult;
use crate::http::sse::SseEvent;
use crate::providers::common::chat_completions::ChatCompletionsStreamState;
use crate::providers::common::translate_error::ErrorConfig;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::stream_event::StreamEvent;

static ERROR_CONFIG: ErrorConfig = ErrorConfig {
    provider_name: "openai_chat_completions",
    error_code_pointers: &["/error/code", "/error/type"],
    allow_numeric_codes: false,
    quota_keywords: &["quota", "insufficient_quota"],
    quota_codes: &["insufficient_quota", "quota_exceeded"],
};

/// Stateful translator for OpenAI-compatible Chat Completions SSE events.
///
/// This is a newtype wrapper around the shared [`ChatCompletionsStreamState`].
#[derive(Debug, Clone)]
pub struct OpenAIChatCompletionsStreamState(ChatCompletionsStreamState);

impl Default for OpenAIChatCompletionsStreamState {
    fn default() -> Self {
        Self(ChatCompletionsStreamState::new(
            "openai_chat_completions",
            None,
            ERROR_CONFIG,
        ))
    }
}

impl OpenAIChatCompletionsStreamState {
    #[must_use]
    pub fn with_rate_limit(rate_limit: Option<RateLimitInfo>) -> Self {
        Self(ChatCompletionsStreamState::new(
            "openai_chat_completions",
            rate_limit,
            ERROR_CONFIG,
        ))
    }
}

/// Translate a single SSE event into zero or more unified stream events.
///
/// # Errors
///
/// Returns `SdkError::Stream` when provider event payloads are malformed JSON.
pub fn translate_sse_event(
    event: &SseEvent,
    state: &mut OpenAIChatCompletionsStreamState,
) -> SdkResult<Vec<StreamEvent>> {
    crate::providers::common::chat_completions::translate_sse_event(event, &mut state.0)
}

/// Translate a full parsed SSE stream into unified stream events.
#[must_use]
pub fn translate_sse_stream<'a>(
    sse_stream: std::pin::Pin<Box<dyn futures::Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    rate_limit: Option<RateLimitInfo>,
) -> crate::provider::BoxStream<'a, SdkResult<StreamEvent>> {
    crate::providers::common::chat_completions::translate_sse_stream(
        sse_stream,
        ChatCompletionsStreamState::new("openai_chat_completions", rate_limit, ERROR_CONFIG),
    )
}
