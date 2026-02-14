use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use futures::{Stream, StreamExt};

use crate::error::{SdkError, SdkResult};
use crate::retry::{self, RetryPolicy};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::response_format::ResponseFormat;
use crate::types::stream_event::{StreamEvent, StreamEventType};
use crate::types::tool::ToolChoice;
use crate::types::usage::Usage;

use super::accumulator::StreamAccumulator;
use super::cancel::AbortSignal;
use super::options::{RequestTemplate, impl_common_builders, resolve_client};
use super::tools::{
    build_messages, execute_all_tools, has_active_tools, has_passive_tool_calls,
    should_execute_tools, tool_definitions, tool_result_messages, validate_prompt_messages,
};
use super::types::{StepResult, StopCondition, Tool};

/// Shared stop-condition function type (Arc-wrapped for `Send + Sync`).
type ArcStopCondition = Arc<dyn Fn(&[StepResult]) -> bool + Send + Sync>;

/// Options for [`stream_generate()`].
///
/// Mirrors [`GenerateOptions`](super::generate::GenerateOptions) for streaming.
pub struct StreamOptions<'a> {
    pub model: String,
    pub prompt: Option<String>,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub tools: Vec<Tool>,
    pub tool_choice: Option<ToolChoice>,
    pub max_tool_rounds: u32,
    pub stop_when: Option<StopCondition>,
    pub response_format: Option<ResponseFormat>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub stop_sequences: Option<Vec<String>>,
    pub reasoning_effort: Option<String>,
    pub provider: Option<String>,
    pub provider_options: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub max_retries: u32,
    pub timeout: Option<crate::types::timeout::Timeout>,
    pub abort_signal: Option<AbortSignal>,
    pub client: Option<&'a crate::client::Client>,
}

impl StreamOptions<'_> {
    #[must_use]
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: None,
            messages: Vec::new(),
            system: None,
            tools: Vec::new(),
            tool_choice: None,
            max_tool_rounds: 1,
            stop_when: None,
            response_format: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop_sequences: None,
            reasoning_effort: None,
            provider: None,
            provider_options: None,
            max_retries: 2,
            timeout: None,
            abort_signal: None,
            client: None,
        }
    }

    #[must_use]
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    #[must_use]
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    #[must_use]
    pub fn max_tool_rounds(mut self, rounds: u32) -> Self {
        self.max_tool_rounds = rounds;
        self
    }

    #[must_use]
    pub fn stop_when(mut self, condition: StopCondition) -> Self {
        self.stop_when = Some(condition);
        self
    }

    #[must_use]
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    #[must_use]
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    #[must_use]
    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }
}

impl_common_builders!(StreamOptions<'a>);

impl StreamOptions<'_> {
    /// Build a [`RequestTemplate`] from these options.
    fn request_template(&self) -> RequestTemplate {
        RequestTemplate {
            model: self.model.clone(),
            provider: self.provider.clone(),
            tool_choice: self.tool_choice.clone(),
            response_format: self.response_format.clone(),
            temperature: self.temperature,
            top_p: self.top_p,
            max_tokens: self.max_tokens,
            stop_sequences: self.stop_sequences.clone(),
            reasoning_effort: self.reasoning_effort.clone(),
            provider_options: self.provider_options.clone(),
            timeout: self.timeout,
        }
    }
}

/// Shared mutable state for a streaming generation.
///
/// Uses `std::sync::Mutex` (not `tokio::sync::Mutex`) because
/// `StreamAccumulator::process()` is synchronous and locks are brief.
struct StreamShared {
    accumulator: StreamAccumulator,
    steps: Vec<StepResult>,
    total_usage: Usage,
    finished: bool,
    final_response: Option<Response>,
}

/// Lock the shared state, recovering from poisoned mutex.
fn lock_shared(shared: &Mutex<StreamShared>) -> MutexGuard<'_, StreamShared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The result of a streaming generation.
///
/// Provides both incremental consumption (via `next_event()`) and
/// convenience methods to get the accumulated state.
pub struct StreamResult<'a> {
    inner: Pin<Box<dyn Stream<Item = SdkResult<StreamEvent>> + Send + 'a>>,
    shared: Arc<Mutex<StreamShared>>,
    events: Vec<StreamEvent>,
}

/// The collected result after consuming all stream events.
///
/// This is the "easy path" equivalent of consuming the entire stream.
#[derive(Debug)]
pub struct CollectedStreamResult {
    /// The fully-accumulated response.
    pub response: Response,
    /// All events that were yielded.
    pub events: Vec<StreamEvent>,
    /// Step results for multi-step tool loops.
    pub steps: Vec<StepResult>,
    /// Aggregated usage across all steps.
    pub total_usage: Usage,
}

impl<'a> StreamResult<'a> {
    /// Get the next event from the stream.
    ///
    /// Returns `None` when the stream is exhausted.
    pub async fn next_event(&mut self) -> Option<SdkResult<StreamEvent>> {
        let item = self.inner.next().await;
        if let Some(Ok(ref event)) = item {
            let mut shared = lock_shared(&self.shared);
            shared.accumulator.process(event);
            self.events.push(event.clone());
        }
        item
    }

    /// Get the accumulated response so far (mid-stream snapshot).
    ///
    /// This reflects all events consumed via `next_event()` up to this point.
    #[must_use]
    pub fn partial_response(&self) -> Response {
        let shared = lock_shared(&self.shared);
        shared.accumulator.response()
    }

    /// Get the final response after the stream has completed.
    ///
    /// Returns `None` if the stream hasn't finished yet.
    #[must_use]
    pub fn response(&self) -> Option<Response> {
        let shared = lock_shared(&self.shared);
        shared.final_response.clone()
    }

    /// Consume the stream, collecting all events and returning the final result.
    ///
    /// This is the "easy path" for callers that want the old buffered behavior.
    ///
    /// # Errors
    ///
    /// Returns any error encountered during streaming.
    pub async fn collect(mut self) -> SdkResult<CollectedStreamResult> {
        while let Some(item) = self.next_event().await {
            item?;
        }

        let shared = lock_shared(&self.shared);

        let response = shared
            .final_response
            .clone()
            .unwrap_or_else(|| shared.accumulator.response());

        Ok(CollectedStreamResult {
            response,
            events: self.events,
            steps: shared.steps.clone(),
            total_usage: shared.total_usage.clone(),
        })
    }

    /// Return a stream that yields only text deltas.
    ///
    /// Errors from the underlying stream (e.g. abort, timeout, connection
    /// failures) are surfaced as `Err` items rather than silently ending
    /// the stream.
    #[must_use]
    pub fn text_stream(self) -> Pin<Box<dyn Stream<Item = SdkResult<String>> + Send + 'a>> {
        Box::pin(futures::stream::unfold(self, |mut s| async move {
            loop {
                match s.next_event().await {
                    Some(Ok(event)) => {
                        if event.event_type == StreamEventType::TextDelta
                            && let Some(delta) = event.delta
                        {
                            return Some((Ok(delta), s));
                        }
                        // Non-text event — keep consuming
                    }
                    Some(Err(e)) => return Some((Err(e), s)),
                    None => return None,
                }
            }
        }))
    }
}

/// High-level streaming generation with automatic tool execution loop.
///
/// Returns a [`StreamResult`] that can be consumed incrementally via
/// `next_event()`, or collected in one shot via `collect()`.
///
/// Events are yielded truly incrementally — each `TextDelta` arrives as
/// soon as the provider produces it, without buffering the entire step.
///
/// # Timeout enforcement
///
/// The `per_step` timeout (spec §4.7) is currently applied to each provider
/// stream connection setup. It does not currently bound inter-event read
/// time once the stream is established. The `total` timeout is not enforced
/// within the stream itself because the stream is lazy — callers can wrap
/// `collect()` or the `next_event()` loop with `tokio::time::timeout` if a
/// total bound is needed.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if both `prompt` and `messages` are
/// provided, or if no client is available.
pub async fn stream_generate(opts: StreamOptions<'_>) -> SdkResult<StreamResult<'_>> {
    validate_prompt_messages(opts.prompt.as_deref(), &opts.messages)?;

    // Eagerly check abort signal before setting up the stream
    if let Some(ref signal) = opts.abort_signal {
        signal.check()?;
    }

    let client = resolve_client(opts.client)?;

    let conversation = build_messages(
        opts.prompt.as_deref(),
        &opts.messages,
        opts.system.as_deref(),
    );

    let tool_defs = if opts.tools.is_empty() {
        None
    } else {
        Some(tool_definitions(&opts.tools))
    };

    let has_active = has_active_tools(&opts.tools);
    let retry_policy = RetryPolicy {
        max_retries: opts.max_retries,
        ..RetryPolicy::default()
    };

    let shared = Arc::new(Mutex::new(StreamShared {
        accumulator: StreamAccumulator::new(),
        steps: Vec::new(),
        total_usage: Usage::default(),
        finished: false,
        final_response: None,
    }));

    let shared_for_stream = shared.clone();

    let per_step_timeout = opts.timeout.and_then(|t| t.per_step);
    let template = opts.request_template();
    let max_tool_rounds = opts.max_tool_rounds;

    let tools = Arc::new(opts.tools);
    let stop_when: Option<ArcStopCondition> =
        opts.stop_when.map(|f| Arc::from(f) as ArcStopCondition);

    // Build the stream using `try_unfold`. Each unfold step yields exactly
    // one event. The state machine has two phases:
    //
    //   `NeedConnect` — must open a new provider stream (start or after tool loop).
    //   `Streaming`   — yielding events one-by-one from an open provider stream.
    //
    // When the provider stream ends with tool calls, the state transitions
    // through tool execution, emits a `StepFinish` event, then returns to
    // `NeedConnect` for the next round.
    let state = UnfoldState {
        phase: Phase::NeedConnect,
        conversation,
        round_num: 0,
        client,
        tool_defs,
        has_active,
        retry_policy,
        template,
        max_tool_rounds,
        per_step_timeout,
        tools,
        stop_when,
        abort_signal: opts.abort_signal,
        shared: shared_for_stream,
        step_accumulator: StreamAccumulator::new(),
        pending_step_finish: None,
    };

    let stream = futures::stream::try_unfold(state, unfold_step);

    Ok(StreamResult {
        inner: Box::pin(stream),
        shared,
        events: Vec::new(),
    })
}

/// The phases of the streaming state machine.
enum Phase<'a> {
    /// Need to open a new provider stream (initial connect or after tool loop).
    NeedConnect,
    /// Actively yielding events from an open provider stream.
    Streaming(Pin<Box<dyn Stream<Item = SdkResult<StreamEvent>> + Send + 'a>>),
    /// Stream has ended (after error event or final step).
    Done,
}

/// State held across unfold iterations.
struct UnfoldState<'a> {
    phase: Phase<'a>,
    conversation: Vec<Message>,
    round_num: u32,
    client: &'a crate::client::Client,
    tool_defs: Option<Vec<crate::types::tool::ToolDefinition>>,
    has_active: bool,
    retry_policy: RetryPolicy,
    template: RequestTemplate,
    max_tool_rounds: u32,
    per_step_timeout: Option<f64>,
    tools: Arc<Vec<Tool>>,
    stop_when: Option<ArcStopCondition>,
    abort_signal: Option<AbortSignal>,
    shared: Arc<Mutex<StreamShared>>,
    /// Per-step accumulator to detect tool calls when the provider stream ends.
    step_accumulator: StreamAccumulator,
    /// A `StepFinish` event to yield after finishing tool execution (before
    /// transitioning to the next round's `NeedConnect`).
    pending_step_finish: Option<StreamEvent>,
}

/// A single step of the `try_unfold` state machine.
///
/// Each invocation yields exactly one event:
/// - In `NeedConnect` phase: opens a provider stream, transitions to `Streaming`,
///   then falls through to yield the first event.
/// - In `Streaming` phase: yields the next event from the provider stream.
///   When the provider stream ends, determines whether to loop (tool calls)
///   or finish.
#[allow(clippy::too_many_lines)]
async fn unfold_step(
    mut state: UnfoldState<'_>,
) -> SdkResult<Option<(StreamEvent, UnfoldState<'_>)>> {
    // If we have a pending StepFinish event from a completed tool round,
    // yield it before connecting for the next round.
    if let Some(event) = state.pending_step_finish.take() {
        return Ok(Some((event, state)));
    }

    // If we need to connect, open a new provider stream.
    if matches!(state.phase, Phase::NeedConnect) {
        // Check abort before connecting
        if let Some(ref signal) = state.abort_signal {
            signal.check()?;
        }

        let request = state
            .template
            .to_request(&state.conversation, state.tool_defs.as_ref());
        let connect_future = retry::retry(
            &state.retry_policy,
            || state.client.stream(request.clone()),
            None,
            None,
        );

        // Apply per-step timeout if configured (spec §4.7).
        let event_stream = match state.per_step_timeout {
            Some(secs) => tokio::time::timeout(Duration::from_secs_f64(secs), connect_future)
                .await
                .map_err(|_| SdkError::RequestTimeout {
                    message: format!("per-step timeout of {secs}s exceeded"),
                })?,
            None => connect_future.await,
        }?;

        state.step_accumulator = StreamAccumulator::new();
        state.phase = Phase::Streaming(Box::pin(event_stream));
    }

    // We are now in the Streaming phase. Yield the next event.
    let Phase::Streaming(ref mut inner_stream) = state.phase else {
        // Should be unreachable after the NeedConnect block above
        return Ok(None);
    };

    // Poll the next event, with abort signal racing if configured.
    let next_item = match state.abort_signal {
        Some(ref signal) => {
            tokio::select! {
                biased;
                () = signal.notified() => {
                    return Err(SdkError::Abort {
                        message: "operation was aborted".into(),
                    });
                }
                item = inner_stream.next() => item,
            }
        }
        None => inner_stream.next().await,
    };

    match next_item {
        Some(Ok(event)) => {
            // Feed to per-step accumulator so we can inspect tool calls when done.
            state.step_accumulator.process(&event);
            Ok(Some((event, state)))
        }
        Some(Err(e)) => {
            // Provider stream error after partial delivery — per spec §6.6,
            // no retry. Emit an error event and end the stream cleanly.
            // Snapshot whatever was accumulated so `response()` returns
            // the partial result instead of `None`.
            let error_event = StreamEvent::error(e);
            let mut shared = lock_shared(&state.shared);
            shared.finished = true;
            shared.final_response = Some(state.step_accumulator.response());
            drop(shared);
            state.phase = Phase::Done;
            Ok(Some((error_event, state)))
        }
        None => {
            // Provider stream ended. Build the step result and decide what's next.
            handle_step_end(state).await
        }
    }
}

/// Handle the end of a provider stream step.
///
/// Builds a `StepResult`, executes tools if needed, and either:
/// - Transitions to `NeedConnect` for the next round (tool loop continues), or
/// - Marks the stream as done.
async fn handle_step_end(
    mut state: UnfoldState<'_>,
) -> SdkResult<Option<(StreamEvent, UnfoldState<'_>)>> {
    let response = state.step_accumulator.response();
    let tool_calls = response.tool_calls();

    let should_exec = state.has_active
        && should_execute_tools(&tool_calls, &response.finish_reason)
        && state.round_num < state.max_tool_rounds;

    let has_passive = has_passive_tool_calls(&state.tools, &tool_calls);

    let tool_results = if should_exec {
        execute_all_tools(&state.tools, &tool_calls, state.abort_signal.as_ref()).await
    } else {
        Vec::new()
    };

    let step = StepResult::new(&response, tool_calls, tool_results.clone());

    let mut shared = lock_shared(&state.shared);
    shared.total_usage = shared.total_usage.clone() + response.usage.clone();
    shared.steps.push(step);

    let should_continue = should_exec
        && !tool_results.is_empty()
        && !has_passive
        && !state.stop_when.as_ref().is_some_and(|f| f(&shared.steps));

    // Must drop the lock before transitioning state
    drop(shared);

    if should_continue {
        // Prepare next round
        let (assistant_msg, result_msgs) = tool_result_messages(&response, &tool_results);
        state.conversation.push(assistant_msg);
        state.conversation.extend(result_msgs);
        state.round_num += 1;

        // Emit a StepFinish event, then transition to NeedConnect
        let step_finish = StreamEvent::step_finish(
            FinishReason::new(Reason::ToolCalls, None),
            response.usage.clone(),
        );
        state.phase = Phase::NeedConnect;
        Ok(Some((step_finish, state)))
    } else {
        // Done — record final response and signal stream end.
        let mut shared = lock_shared(&state.shared);
        shared.finished = true;
        shared.final_response = Some(response);
        Ok(None)
    }
}
