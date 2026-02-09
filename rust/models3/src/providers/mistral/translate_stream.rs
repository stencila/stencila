use crate::error::SdkResult;
use crate::http::sse::SseEvent;
use crate::providers::common::chat_completions::ChatCompletionsStreamState;
use crate::types::rate_limit::RateLimitInfo;
use crate::types::stream_event::StreamEvent;

use super::translate_error::ERROR_CONFIG;

/// Stateful translator for Mistral Chat Completions SSE events.
#[derive(Debug, Clone)]
pub struct MistralStreamState(ChatCompletionsStreamState);

impl Default for MistralStreamState {
    fn default() -> Self {
        Self(ChatCompletionsStreamState::new(
            "mistral",
            None,
            ERROR_CONFIG,
        ))
    }
}

impl MistralStreamState {
    #[must_use]
    pub fn with_rate_limit(rate_limit: Option<RateLimitInfo>) -> Self {
        Self(ChatCompletionsStreamState::new(
            "mistral",
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
    state: &mut MistralStreamState,
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
        ChatCompletionsStreamState::new("mistral", rate_limit, ERROR_CONFIG),
    )
}
