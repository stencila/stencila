use std::collections::VecDeque;
use std::pin::Pin;

use futures::{Stream, StreamExt, stream};

use crate::error::SdkResult;
use crate::http::sse::SseEvent;
use crate::provider::BoxStream;
use crate::types::stream_event::StreamEvent;

/// Trait implemented by each provider's stream state to translate SSE events
/// into unified stream events.
pub(crate) trait SseStreamState: Send {
    /// Translate a single parsed SSE event into zero or more unified stream events.
    fn translate_event(&mut self, event: &SseEvent) -> SdkResult<Vec<StreamEvent>>;

    /// Called when the SSE stream ends (no more events).
    ///
    /// Returns events to emit before terminating (e.g. `TextEnd`, `Finish`),
    /// or an empty `Vec` to terminate immediately.
    ///
    /// This will be called at most once. After it returns an empty `Vec`,
    /// the stream terminates.
    fn on_stream_end(&mut self) -> Vec<StreamEvent>;
}

/// Generic SSE stream translator.
///
/// Wraps a raw SSE event stream with a stateful translator that converts
/// provider-specific events into unified `StreamEvent`s.
#[must_use]
pub(crate) fn translate_sse_stream<'a, S: SseStreamState + 'a>(
    sse_stream: Pin<Box<dyn Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
    state: S,
) -> BoxStream<'a, SdkResult<StreamEvent>> {
    struct Translator<'a, S> {
        stream: Pin<Box<dyn Stream<Item = SdkResult<SseEvent>> + Send + 'a>>,
        state: S,
        queue: VecDeque<StreamEvent>,
        ended: bool,
    }

    let translator = Translator {
        stream: sse_stream,
        state,
        queue: VecDeque::new(),
        ended: false,
    };

    Box::pin(stream::try_unfold(
        translator,
        |mut translator| async move {
            loop {
                if let Some(next) = translator.queue.pop_front() {
                    return Ok(Some((next, translator)));
                }

                if translator.ended {
                    return Ok(None);
                }

                match translator.stream.next().await {
                    Some(Ok(sse_event)) => {
                        let translated = translator.state.translate_event(&sse_event)?;
                        translator.queue.extend(translated);
                    }
                    Some(Err(err)) => return Err(err),
                    None => {
                        let end_events = translator.state.on_stream_end();
                        if end_events.is_empty() {
                            return Ok(None);
                        }
                        translator.ended = true;
                        translator.queue.extend(end_events);
                    }
                }
            }
        },
    ))
}
