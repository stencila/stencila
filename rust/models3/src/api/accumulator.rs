use std::collections::HashMap;

use crate::types::content::ContentPart;
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::role::Role;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolCall;
use crate::types::usage::Usage;
use crate::types::warning::Warning;

/// Collects stream events into a complete [`Response`].
///
/// This bridges streaming and blocking modes: any code that works with
/// a `Response` can also work with streamed output by accumulating first.
///
/// ```text
/// let mut acc = StreamAccumulator::new();
/// while let Some(event) = stream.next().await {
///     acc.process(&event?);
/// }
/// let response = acc.response();
/// ```
///
/// # Limitations
///
/// TODO: The accumulator assumes a single text segment per step.
/// Concurrent or interleaved text/reasoning segments are not yet
/// supported (spec §4.4). Multiple text deltas are concatenated into
/// a single string rather than tracked as separate content parts.
#[derive(Debug, Default)]
pub struct StreamAccumulator {
    text: String,
    reasoning: String,
    tool_calls: Vec<ToolCall>,
    /// In-progress tool calls being built from deltas, keyed by tool call ID.
    /// Multiple calls can be in-flight simultaneously when providers emit
    /// interleaved deltas (e.g. OpenAI Chat Completions).
    pending_tool_calls: HashMap<String, InProgressToolCall>,
    finish_reason: Option<FinishReason>,
    usage: Option<Usage>,
    warnings: Vec<Warning>,
    response_id: Option<String>,
    model: Option<String>,
    provider: Option<String>,
}

/// A tool call being assembled from start/delta/end events.
#[derive(Debug, Default)]
struct InProgressToolCall {
    id: String,
    name: String,
    arguments_buf: String,
}

impl StreamAccumulator {
    /// Create a new empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single stream event, updating internal state.
    #[allow(clippy::too_many_lines)]
    pub fn process(&mut self, event: &StreamEvent) {
        match &event.event_type {
            StreamEventType::StreamStart => {
                if let Some(ref w) = event.warnings {
                    self.warnings.extend(w.iter().cloned());
                }
            }
            StreamEventType::TextDelta => {
                if let Some(ref delta) = event.delta {
                    self.text.push_str(delta);
                }
            }
            StreamEventType::ReasoningDelta => {
                if let Some(ref delta) = event.reasoning_delta {
                    self.reasoning.push_str(delta);
                }
            }
            StreamEventType::ToolCallStart => {
                if let Some(ref tc) = event.tool_call {
                    self.pending_tool_calls.insert(
                        tc.id.clone(),
                        InProgressToolCall {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            arguments_buf: String::new(),
                        },
                    );
                }
            }
            StreamEventType::ToolCallDelta => {
                if let Some(ref tc) = event.tool_call {
                    // Look up by ID so interleaved deltas go to the right call
                    if let Some(current) = self.pending_tool_calls.get_mut(&tc.id)
                        && let Some(ref raw) = tc.raw_arguments
                    {
                        current.arguments_buf.push_str(raw);
                    }
                }
            }
            StreamEventType::ToolCallEnd => {
                // Remove the matching pending call by ID. If the end event has an
                // empty/missing ID, fall back to the sole pending call (preserving
                // pre-interleaving-fix behavior for providers that omit the ID).
                let call_id = event
                    .tool_call
                    .as_ref()
                    .map(|tc| tc.id.clone())
                    .filter(|id| !id.is_empty());
                let current = call_id
                    .and_then(|id| self.pending_tool_calls.remove(&id))
                    .or_else(|| {
                        if self.pending_tool_calls.len() == 1 {
                            let key = self.pending_tool_calls.keys().next()?.clone();
                            self.pending_tool_calls.remove(&key)
                        } else {
                            None
                        }
                    });
                if let Some(current) = current {
                    if current.arguments_buf.is_empty() {
                        // No deltas received (Gemini pattern: Start + End with full args).
                        // Fall back to the arguments carried on the end event itself.
                        if let Some(ref tc) = event.tool_call {
                            self.tool_calls.push(ToolCall {
                                id: if tc.id.is_empty() {
                                    current.id
                                } else {
                                    tc.id.clone()
                                },
                                name: if tc.name.is_empty() {
                                    current.name
                                } else {
                                    tc.name.clone()
                                },
                                arguments: tc.arguments.clone(),
                                raw_arguments: tc.raw_arguments.clone(),
                                parse_error: tc.parse_error.clone(),
                            });
                        } else {
                            // Degenerate: no event data at all — emit empty-object tool call
                            self.tool_calls.push(ToolCall {
                                id: current.id,
                                name: current.name,
                                arguments: serde_json::Value::Object(serde_json::Map::new()),
                                raw_arguments: None,
                                parse_error: None,
                            });
                        }
                    } else {
                        // Normal delta pattern: parse accumulated argument buffer
                        let (arguments, parse_error) =
                            ToolCall::parse_arguments(&current.arguments_buf);
                        let raw_arguments = Some(current.arguments_buf);
                        self.tool_calls.push(ToolCall {
                            id: current.id,
                            name: current.name,
                            arguments,
                            raw_arguments,
                            parse_error,
                        });
                    }
                }
            }
            StreamEventType::Finish => {
                if let Some(ref fr) = event.finish_reason {
                    self.finish_reason = Some(fr.clone());
                }
                if let Some(ref u) = event.usage {
                    self.usage = Some(u.clone());
                }
                // If the event carries a fully-built response, capture metadata
                if let Some(ref resp) = event.response {
                    self.response_id = Some(resp.id.clone());
                    self.model = Some(resp.model.clone());
                    self.provider = Some(resp.provider.clone());
                }
            }
            // Other events are ignored by the accumulator
            _ => {}
        }
    }

    /// Build the accumulated [`Response`].
    ///
    /// Call after all events have been processed.
    #[must_use]
    pub fn response(&self) -> Response {
        let mut content: Vec<ContentPart> = Vec::new();

        // Add reasoning if present
        if !self.reasoning.is_empty() {
            content.push(ContentPart::Thinking {
                thinking: crate::types::content::ThinkingData {
                    text: self.reasoning.clone(),
                    signature: None,
                    redacted: false,
                },
            });
        }

        // Add text if present
        if !self.text.is_empty() {
            content.push(ContentPart::text(&self.text));
        }

        // Add tool calls
        for tc in &self.tool_calls {
            content.push(ContentPart::tool_call(
                &tc.id,
                &tc.name,
                tc.arguments.clone(),
            ));
        }

        Response {
            id: self.response_id.clone().unwrap_or_default(),
            model: self.model.clone().unwrap_or_default(),
            provider: self.provider.clone().unwrap_or_default(),
            message: Message::new(Role::Assistant, content),
            finish_reason: self
                .finish_reason
                .clone()
                .unwrap_or_else(|| FinishReason::new(Reason::Other, None)),
            usage: self.usage.clone().unwrap_or_default(),
            raw: None,
            warnings: if self.warnings.is_empty() {
                None
            } else {
                Some(self.warnings.clone())
            },
            rate_limit: None,
        }
    }

    /// The accumulated text so far.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The accumulated tool calls so far.
    #[must_use]
    pub fn tool_calls(&self) -> &[ToolCall] {
        &self.tool_calls
    }
}
