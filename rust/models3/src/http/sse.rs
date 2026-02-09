//! Server-Sent Events (SSE) parser.
//!
//! Transforms a `Stream<Result<Bytes, reqwest::Error>>` into a stream of
//! [`SseEvent`] values, handling all five SSE line types per the spec
//! (Section 7.7):
//!
//! - `event:` — event type
//! - `data:` — payload (may span multiple `data:` lines, joined by `\n`)
//! - `retry:` — reconnection interval (milliseconds)
//! - Comment lines (starting with `:`)
//! - Blank lines — event boundary (dispatches the accumulated event)
//!
//! The parser also detects the `[DONE]` sentinel used by OpenAI-compatible
//! endpoints to signal end of stream.
//!
//! # Why custom instead of a crate
//!
//! We evaluated `eventsource-stream` (v0.2.3, ~271K downloads/month) and
//! `reqwest-eventsource` (v0.6.0, ~935K downloads/month). Both are solid
//! crates, but neither handles the `[DONE]` sentinel that OpenAI-compatible
//! endpoints use to signal end-of-stream — we would need a filter wrapper
//! on top. The SSE spec itself is frozen, so the parser logic is unlikely
//! to need upstream fixes. Our implementation is ~100 lines of parser code,
//! adds zero extra dependencies, and is tailored for the LLM streaming use
//! case (native `[DONE]` handling, no `id:` field which LLM APIs don't use,
//! tested against both Anthropic-style and OpenAI-style event sequences).

use std::pin::Pin;

use futures::stream::Stream;

use crate::error::{SdkError, SdkResult};

/// A parsed SSE event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    /// The event type from `event:` lines. Defaults to `"message"` per the
    /// SSE specification when no `event:` line precedes the data.
    pub event_type: String,

    /// The event payload from `data:` lines. Multiple `data:` lines within
    /// a single event are joined with `\n`.
    pub data: String,

    /// Reconnection interval in milliseconds from `retry:` lines, if present.
    pub retry: Option<u64>,
}

/// Sentinel value that signals the end of an SSE stream (OpenAI convention).
pub const DONE_SENTINEL: &str = "[DONE]";

/// Parse a byte stream into SSE events.
///
/// The returned stream yields one [`SseEvent`] per blank-line-delimited
/// event block. Events whose data is exactly `[DONE]` are **not** yielded;
/// the stream simply ends.
#[must_use]
pub fn parse_sse(
    byte_stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
) -> Pin<Box<dyn Stream<Item = SdkResult<SseEvent>> + Send>> {
    Box::pin(SseParser::new(byte_stream))
}

/// Internal parser state.
struct SseParser {
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    /// Leftover bytes from the previous chunk that didn't end on a line boundary.
    buffer: String,
    /// Accumulated event type for the current event block.
    event_type: Option<String>,
    /// Accumulated data lines for the current event block.
    data_lines: Vec<String>,
    /// Reconnection interval from the most recent `retry:` line.
    retry: Option<u64>,
    /// Whether we've seen at least one `data:` line in the current block.
    has_data: bool,
    /// Stream is done (received `[DONE]` or inner stream ended).
    done: bool,
}

impl SseParser {
    fn new(
        inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    ) -> Self {
        Self {
            inner,
            buffer: String::new(),
            event_type: None,
            data_lines: Vec::new(),
            retry: None,
            has_data: false,
            done: false,
        }
    }

    /// Reset the per-event accumulator. Called after dispatching an event.
    fn reset_event(&mut self) {
        self.event_type = None;
        self.data_lines.clear();
        self.has_data = false;
        // Note: `retry` is intentionally NOT reset — the spec says the
        // reconnection interval persists across events until changed.
    }

    /// Process a single line according to SSE rules.
    /// Returns `Some(SseEvent)` if a complete event should be dispatched.
    fn process_line(&mut self, line: &str) -> Option<SseEvent> {
        // Blank line = event boundary
        if line.is_empty() {
            if self.has_data {
                let data = self.data_lines.join("\n");

                // Check for [DONE] sentinel
                if data.trim() == DONE_SENTINEL {
                    self.done = true;
                    return None;
                }

                let event = SseEvent {
                    event_type: self
                        .event_type
                        .take()
                        .unwrap_or_else(|| "message".to_string()),
                    data,
                    retry: self.retry,
                };
                self.reset_event();
                return Some(event);
            }
            // Blank line with no data — just reset
            self.reset_event();
            return None;
        }

        // Comment line (starts with `:`)
        if line.starts_with(':') {
            return None;
        }

        // Field line: split on first `:`
        let (field, value) = if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let mut value = &line[colon_pos + 1..];
            // Strip single leading space after colon per SSE spec
            if value.starts_with(' ') {
                value = &value[1..];
            }
            (field, value)
        } else {
            // Line with no colon — field is the entire line, value is empty
            (line, "")
        };

        match field {
            "event" => {
                self.event_type = Some(value.to_string());
            }
            "data" => {
                self.data_lines.push(value.to_string());
                self.has_data = true;
            }
            "retry" => {
                if let Ok(ms) = value.trim().parse::<u64>() {
                    self.retry = Some(ms);
                }
                // Invalid retry values are silently ignored per SSE spec
            }
            _ => {
                // Unknown fields are ignored per SSE spec
            }
        }

        None
    }
}

impl Stream for SseParser {
    type Item = SdkResult<SseEvent>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.done {
            return std::task::Poll::Ready(None);
        }

        loop {
            // First, try to process lines already in the buffer
            while let Some(newline_pos) = this.buffer.find('\n') {
                let line = this.buffer[..newline_pos]
                    .trim_end_matches('\r')
                    .to_string();
                this.buffer = this.buffer[newline_pos + 1..].to_string();

                if let Some(event) = this.process_line(&line) {
                    return std::task::Poll::Ready(Some(Ok(event)));
                }

                if this.done {
                    return std::task::Poll::Ready(None);
                }
            }

            // Need more data from the inner stream
            match this.inner.as_mut().poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    this.buffer.push_str(&chunk);
                    // Loop back to process new lines
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(SdkError::Network {
                        message: e.to_string(),
                    })));
                }
                std::task::Poll::Ready(None) => {
                    // Inner stream ended — process any remaining buffer
                    // as a final line, then dispatch if we have data.
                    let remaining = std::mem::take(&mut this.buffer);
                    if !remaining.is_empty() {
                        this.process_line(&remaining);
                    }
                    if this.has_data {
                        let data = this.data_lines.join("\n");
                        if data.trim() == DONE_SENTINEL {
                            return std::task::Poll::Ready(None);
                        }
                        let event = SseEvent {
                            event_type: this
                                .event_type
                                .take()
                                .unwrap_or_else(|| "message".to_string()),
                            data,
                            retry: this.retry,
                        };
                        this.reset_event();
                        this.done = true;
                        return std::task::Poll::Ready(Some(Ok(event)));
                    }
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{StreamExt, stream};

    /// Helper: make a byte stream from a string.
    fn bytes_stream(
        input: &str,
    ) -> Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>> {
        let bytes = bytes::Bytes::from(input.to_string());
        Box::pin(stream::once(async move { Ok(bytes) }))
    }

    /// Helper: make a byte stream from multiple chunks.
    fn chunked_bytes_stream(
        chunks: Vec<&str>,
    ) -> Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>> {
        let items: Vec<Result<bytes::Bytes, reqwest::Error>> = chunks
            .into_iter()
            .map(|s| Ok(bytes::Bytes::from(s.to_string())))
            .collect();
        Box::pin(stream::iter(items))
    }

    /// Collect all events from an SSE stream.
    async fn collect_events(input: &str) -> Vec<SseEvent> {
        let stream = parse_sse(bytes_stream(input));
        stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect()
    }

    #[tokio::test]
    async fn simple_data_event() -> SdkResult<()> {
        let events = collect_events("data: hello world\n\n").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "message");
        assert_eq!(events[0].data, "hello world");
        Ok(())
    }

    #[tokio::test]
    async fn event_with_type() -> SdkResult<()> {
        let events =
            collect_events("event: content_block_delta\ndata: {\"text\": \"hi\"}\n\n").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "content_block_delta");
        assert_eq!(events[0].data, "{\"text\": \"hi\"}");
        Ok(())
    }

    #[tokio::test]
    async fn multi_line_data() -> SdkResult<()> {
        let events = collect_events("data: line one\ndata: line two\ndata: line three\n\n").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line one\nline two\nline three");
        Ok(())
    }

    #[tokio::test]
    async fn multiple_events() -> SdkResult<()> {
        let input = "data: first\n\ndata: second\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "first");
        assert_eq!(events[1].data, "second");
        Ok(())
    }

    #[tokio::test]
    async fn comment_lines_ignored() -> SdkResult<()> {
        let input = ": this is a comment\ndata: hello\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn retry_line_parsed() -> SdkResult<()> {
        let input = "retry: 5000\ndata: hello\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].retry, Some(5000));
        Ok(())
    }

    #[tokio::test]
    async fn retry_persists_across_events() -> SdkResult<()> {
        let input = "retry: 3000\ndata: first\n\ndata: second\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].retry, Some(3000));
        assert_eq!(events[1].retry, Some(3000));
        Ok(())
    }

    #[tokio::test]
    async fn invalid_retry_ignored() -> SdkResult<()> {
        let input = "retry: not-a-number\ndata: hello\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].retry, None);
        Ok(())
    }

    #[tokio::test]
    async fn done_sentinel_ends_stream() -> SdkResult<()> {
        let input = "data: hello\n\ndata: [DONE]\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn empty_data_field() -> SdkResult<()> {
        let input = "data:\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "");
        Ok(())
    }

    #[tokio::test]
    async fn data_without_space_after_colon() -> SdkResult<()> {
        let input = "data:no-space\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "no-space");
        Ok(())
    }

    #[tokio::test]
    async fn blank_lines_without_data_ignored() -> SdkResult<()> {
        let input = "\n\ndata: hello\n\n\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn crlf_line_endings() -> SdkResult<()> {
        let input = "data: hello\r\n\r\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn chunked_delivery() -> SdkResult<()> {
        // Event split across multiple chunks
        let stream = chunked_bytes_stream(vec!["data: hel", "lo\n\n"]);
        let events: Vec<SseEvent> = parse_sse(stream)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn anthropic_style_events() -> SdkResult<()> {
        let input = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\"}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        );
        let events = collect_events(input).await;
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, "message_start");
        assert_eq!(events[1].event_type, "content_block_delta");
        assert_eq!(events[2].event_type, "message_stop");
        Ok(())
    }

    #[tokio::test]
    async fn openai_style_events() -> SdkResult<()> {
        let input = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" there\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let events = collect_events(input).await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "message");
        assert_eq!(events[1].event_type, "message");
        Ok(())
    }

    #[tokio::test]
    async fn unknown_fields_ignored() -> SdkResult<()> {
        let input = "id: 123\ndata: hello\nunknown: value\n\n";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn event_at_end_of_stream_without_trailing_newline() -> SdkResult<()> {
        // Some providers don't send a trailing blank line for the last event
        let input = "data: final";
        let events = collect_events(input).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "final");
        Ok(())
    }
}
