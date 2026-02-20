//! Agent session and core agentic loop (spec 2.1, 2.5-2.8, 2.10, Appendix B).
//!
//! The [`Session`] struct manages the conversation lifecycle: accepting user
//! input, building LLM requests, executing tool calls, emitting events, and
//! handling errors. It uses [`Client::stream()`] for incremental token
//! delivery (spec 2.9), falling back to [`Client::complete()`] via the
//! default [`LlmClient::stream_complete()`] implementation when the client
//! does not support streaming.
//!
//! # Testing
//!
//! The [`LlmClient`] trait abstracts the LLM call for testability. Tests
//! inject a mock that returns predetermined responses. The default
//! [`stream_complete()`](LlmClient::stream_complete) implementation
//! delegates to [`complete()`](LlmClient::complete), so mocks only need
//! to implement `complete`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use stencila_models3::api::accumulator::StreamAccumulator;
use stencila_models3::error::SdkError;
use stencila_models3::retry::RetryPolicy;
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::stream_event::{StreamEvent, StreamEventType};
use stencila_models3::types::tool::{ToolCall, ToolChoice, ToolResult};

use crate::error::{AgentError, AgentResult};
use crate::events::{self, EventEmitter, EventReceiver};
use crate::execution::ExecutionEnvironment;
use crate::loop_detection;
use crate::profile::ProviderProfile;
use crate::registry::ToolOutput;
use crate::subagents::SubAgentManager;
use crate::truncation::{TruncationConfig, truncate_tool_output};
use crate::types::{
    AbortKind, AbortSignal, EventKind, SessionConfig, SessionState, Turn, now_timestamp,
};

// ---------------------------------------------------------------------------
// LlmClient trait
// ---------------------------------------------------------------------------

/// Abstraction over the LLM client for testability.
///
/// Production code uses [`Models3Client`] which wraps the real
/// [`stencila_models3::client::Client`]. Tests inject a mock.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a completion request to the LLM.
    async fn complete(&self, request: Request) -> Result<Response, SdkError>;

    /// Stream a completion request, calling `on_event` for each stream event.
    ///
    /// Returns the accumulated [`Response`] after the stream completes.
    /// The default implementation falls back to [`complete()`](Self::complete)
    /// and synthesizes a single text-delta event for the full response text,
    /// so mock clients only need to implement `complete`.
    ///
    /// Events are passed by value (owned) rather than by reference to avoid
    /// lifetime issues with the `async_trait` macro and HRTB closures.
    async fn stream_complete(
        &self,
        request: Request,
        on_event: &(dyn Fn(StreamEvent) + Send + Sync),
    ) -> Result<Response, SdkError> {
        let response = self.complete(request).await?;
        let text = response.text();
        if !text.is_empty() {
            on_event(StreamEvent::text_delta(&text));
        }
        Ok(response)
    }
}

/// Real implementation wrapping the models3 [`Client`](stencila_models3::client::Client).
pub struct Models3Client {
    client: stencila_models3::client::Client,
}

impl Models3Client {
    /// Wrap an existing models3 client.
    pub fn new(client: stencila_models3::client::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl LlmClient for Models3Client {
    async fn complete(&self, request: Request) -> Result<Response, SdkError> {
        self.client.complete(request).await
    }

    async fn stream_complete(
        &self,
        request: Request,
        on_event: &(dyn Fn(StreamEvent) + Send + Sync),
    ) -> Result<Response, SdkError> {
        // Try streaming first; fall back to complete() only if the provider
        // signals that streaming is unsupported (Configuration / InvalidRequest).
        // Auth, rate-limit, network, and other errors must propagate.
        match self.client.stream(request.clone()).await {
            Ok(mut stream) => {
                let mut accumulator = StreamAccumulator::new();
                while let Some(result) = stream.next().await {
                    let event = result?;
                    accumulator.process(&event);
                    on_event(event);
                }
                Ok(accumulator.response())
            }
            Err(
                SdkError::Configuration { .. }
                | SdkError::InvalidRequest { .. }
                | SdkError::NotFound { .. },
            ) => {
                // Runtime safety net: streaming unsupported by provider/model.
                let response = self.client.complete(request).await?;
                let text = response.text();
                if !text.is_empty() {
                    on_event(StreamEvent::text_delta(&text));
                }
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// ImageAttachment
// ---------------------------------------------------------------------------

/// Image data attached to a tool result for multimodal providers.
struct ImageAttachment {
    data: Vec<u8>,
    media_type: String,
}

// ---------------------------------------------------------------------------
// API Session
// ---------------------------------------------------------------------------

/// An agent session managing the conversation loop (spec 2.1).
///
/// Created via [`ApiSession::new()`], driven by [`submit()`](Self::submit).
/// Events are delivered through the [`EventReceiver`] returned by the
/// constructor.
pub struct ApiSession {
    config: SessionConfig,
    state: SessionState,
    history: Vec<Turn>,
    profile: Box<dyn ProviderProfile>,
    execution_env: Arc<dyn ExecutionEnvironment>,
    client: Arc<dyn LlmClient>,
    events: EventEmitter,
    steering_queue: VecDeque<String>,
    followup_queue: VecDeque<String>,
    system_prompt: String,
    abort_signal: Option<AbortSignal>,
    /// Count of LLM request/response cycles across the entire session.
    ///
    /// Incremented once per assistant response (not per history entry).
    /// The spec's `count_turns(session)` (line 234) is ambiguous — it could
    /// mean total history entries or LLM cycles. We use LLM cycles because
    /// it provides more useful limiting behavior: a single tool-using turn
    /// counts as 1 turn regardless of how many tool results are generated.
    total_turns: u32,
    truncation_config: TruncationConfig,
    /// Bounded sliding window of tool-call signatures for loop detection.
    tool_call_signatures: VecDeque<String>,
    /// Manager for child agent sessions (spec 7.1).
    subagent_manager: SubAgentManager,
    /// Image attachments from tool results, keyed by tool_call_id.
    /// Provider-generated tool_call_ids are UUIDs, so collisions are
    /// not a practical concern.
    image_attachments: HashMap<String, ImageAttachment>,

    /// MCP connection pool for server lifecycle and subagent sharing.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    mcp_pool: Option<Arc<stencila_mcp::ConnectionPool>>,

    /// Whether this session owns the MCP pool and should shut it down on close.
    /// Only the top-level session that created the pool owns it; child sessions
    /// share the pool via `Arc` but must not call `start_shutdown()`.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    owns_mcp_pool: bool,
}

impl std::fmt::Debug for ApiSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("state", &self.state)
            .field("history_len", &self.history.len())
            .field("total_turns", &self.total_turns)
            .finish_non_exhaustive()
    }
}

impl ApiSession {
    /// Create a new session.
    ///
    /// Returns the session and an [`EventReceiver`] for consuming events.
    /// The caller is responsible for building the system prompt (see
    /// [`build_system_prompt()`] helper) and passing the resulting
    /// [`McpContext`] so the session can manage pool lifecycle and
    /// propagate MCP/codemode capabilities to subagents.
    ///
    /// The `current_depth` parameter controls subagent nesting: 0 for a
    /// top-level session, incremented by 1 for each child. Top-level
    /// sessions (depth 0) own the MCP pool and shut it down on close;
    /// child sessions share the pool without owning it.
    ///
    /// Emits a `SESSION_START` event immediately.
    pub fn new(
        mut profile: Box<dyn ProviderProfile>,
        execution_env: Arc<dyn ExecutionEnvironment>,
        client: Arc<dyn LlmClient>,
        config: SessionConfig,
        system_prompt: String,
        current_depth: u32,
        mcp_context: Option<crate::prompts::McpContext>,
    ) -> (Self, EventReceiver) {
        let (emitter, receiver) = events::channel();
        emitter.emit_session_start();

        let truncation_config = TruncationConfig {
            tool_output_limits: config.tool_output_limits.clone(),
            tool_line_limits: config.tool_line_limits.clone(),
        };

        // Commit instructions layer (between skills/MCP and user instructions)
        let system_prompt = if let Some(ref ci) = config.commit_instructions {
            format!("{system_prompt}\n\n{ci}")
        } else {
            system_prompt
        };

        // Layer 5: append user instruction override if present (spec 6.1)
        let system_prompt = if let Some(ref user_instr) = config.user_instructions {
            format!("{system_prompt}\n\n{user_instr}")
        } else {
            system_prompt
        };

        let max_depth = config.max_subagent_depth;

        // Auto-register subagent tools when this session is allowed to spawn
        // subagents (depth < max_depth). Errors are logged but non-fatal.
        if current_depth < max_depth
            && let Err(e) = profile.register_subagent_tools()
        {
            tracing::warn!("failed to register subagent tools: {e}");
        }

        #[allow(unused_mut)]
        let mut subagent_manager = SubAgentManager::new(
            Arc::clone(&execution_env),
            Arc::clone(&client),
            current_depth,
            config.clone(),
        );

        // Apply MCP context: store pool for lifecycle management and
        // propagate to subagent manager for child session sharing.
        #[cfg(any(feature = "mcp", feature = "codemode"))]
        let (mcp_pool, owns_mcp_pool) = if let Some(ctx) = mcp_context {
            subagent_manager.set_mcp_pool(std::sync::Arc::clone(&ctx.pool));

            #[cfg(feature = "codemode")]
            if let Some(ref tracker) = ctx.dirty_tracker {
                subagent_manager.set_dirty_tracker(std::sync::Arc::clone(tracker));
            }

            let owns = current_depth == 0;
            (Some(ctx.pool), owns)
        } else {
            (None, false)
        };

        // Suppress unused-variable when no MCP features are active.
        #[cfg(not(any(feature = "mcp", feature = "codemode")))]
        let _ = mcp_context;

        let session = Self {
            config,
            state: SessionState::Idle,
            history: Vec::new(),
            profile,
            execution_env,
            client,
            events: emitter,
            steering_queue: VecDeque::new(),
            followup_queue: VecDeque::new(),
            system_prompt,
            abort_signal: None,
            total_turns: 0,
            truncation_config,
            tool_call_signatures: VecDeque::new(),
            subagent_manager,
            image_attachments: HashMap::new(),
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            mcp_pool,
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            owns_mcp_pool,
        };

        (session, receiver)
    }

    // -- Public API --

    /// Submit user input and run the agentic loop until natural completion,
    /// a limit is hit, or an error occurs.
    ///
    /// # State transitions
    ///
    /// - IDLE → PROCESSING → IDLE (natural completion or turn limit)
    /// - IDLE → PROCESSING → AWAITING_INPUT (model asked a question, auto-detected)
    /// - AWAITING_INPUT → PROCESSING → IDLE (user answered a question)
    /// - IDLE → PROCESSING → CLOSED (unrecoverable error or abort)
    ///
    /// # Errors
    ///
    /// Returns `Err(SessionClosed)` if the session is already closed.
    /// Returns `Err(Sdk(..))` for unrecoverable LLM errors.
    pub async fn submit(&mut self, input: &str) -> AgentResult<()> {
        if self.state == SessionState::Closed {
            return Err(AgentError::SessionClosed);
        }

        self.state = SessionState::Processing;
        self.process_input(input).await
    }

    /// Manually transition the session to AwaitingInput state (spec 2.3).
    ///
    /// By default, the session auto-detects questions and transitions to
    /// `AwaitingInput` automatically (see
    /// [`SessionConfig::auto_detect_awaiting_input`]). This method is a
    /// manual override for hosts that disable auto-detection or need to
    /// force the transition based on their own heuristics.
    ///
    /// # Errors
    ///
    /// Returns `Err(InvalidState)` if the session is not in the Idle state.
    pub fn set_awaiting_input(&mut self) -> AgentResult<()> {
        if self.state != SessionState::Idle {
            return Err(AgentError::InvalidState {
                expected: "Idle".into(),
                actual: format!("{:?}", self.state),
            });
        }
        self.state = SessionState::AwaitingInput;
        Ok(())
    }

    /// Queue a steering message to be injected after the current tool round.
    ///
    /// If the session is idle, the message is delivered on the next
    /// [`submit()`](Self::submit) call.
    pub fn steer(&mut self, message: impl Into<String>) {
        self.steering_queue.push_back(message.into());
    }

    /// Queue a follow-up message to be processed after the current input
    /// fully completes (natural completion or limit).
    pub fn follow_up(&mut self, message: impl Into<String>) {
        self.followup_queue.push_back(message.into());
    }

    /// Close the session. Emits `SESSION_END` and transitions to CLOSED.
    // TODO(spec-ambiguity): The spec pseudocode (line 301) emits SESSION_END
    // on every loop completion (transitioning to IDLE), but the event
    // definition (line 413) says "session closed". We only emit on close/
    // error/abort to avoid noisy events on every IDLE transition. (spec: 2.9)
    pub fn close(&mut self) {
        if self.state != SessionState::Closed {
            // Clean up active subagents before closing (spec graceful shutdown, line 1431)
            self.subagent_manager.close_all();

            // Shut down MCP connection pool (only if this session owns it)
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            if self.owns_mcp_pool
                && let Some(ref pool) = self.mcp_pool
            {
                pool.start_shutdown();
            }

            self.state = SessionState::Closed;
            self.events.emit_session_end(self.state);
        }
    }

    /// Set an abort signal for cancellation.
    pub fn set_abort_signal(&mut self, signal: AbortSignal) {
        self.abort_signal = Some(signal);
    }

    // -- Getters --

    /// Current session state.
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Full conversation history.
    #[must_use]
    pub fn history(&self) -> &[Turn] {
        &self.history
    }

    /// Session configuration (read-only).
    #[must_use]
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Session configuration (mutable, e.g. to change reasoning_effort).
    pub fn config_mut(&mut self) -> &mut SessionConfig {
        &mut self.config
    }

    /// The session ID from the event emitter.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.events.session_id()
    }

    /// Total number of LLM request/response cycles in this session.
    #[must_use]
    pub fn total_turns(&self) -> u32 {
        self.total_turns
    }

    /// The MCP connection pool, if MCP/codemode is active.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    #[must_use]
    pub fn mcp_pool(&self) -> Option<&Arc<stencila_mcp::ConnectionPool>> {
        self.mcp_pool.as_ref()
    }

    // -- Core loop (spec 2.5) --

    /// Run the agentic loop for a single user input.
    ///
    /// Uses a single post-loop exit path so that follow-up processing and
    /// context-usage checks are always reached regardless of exit reason
    /// (natural completion, turn limit, or abort).
    async fn process_input(&mut self, input: &str) -> AgentResult<()> {
        // 1. Record user turn
        self.history.push(Turn::user(input));
        self.events.emit_user_input(input);

        let mut round_count: u32 = 0;
        // Set to true only when the loop exits via natural completion
        // (text-only assistant response). Prevents question detection from
        // matching stale assistant turns when exiting via limits.
        let mut natural_completion = false;
        // Track whether we have already attempted context compaction for
        // the current LLM call. Prevents infinite retry loops — we compact
        // at most once per call.
        let mut compaction_attempted = false;

        // The loop runs until natural completion, a limit, or abort.
        // SDK errors propagate immediately (session → CLOSED).
        loop {
            // 2. Check abort
            match self.abort_kind() {
                AbortKind::Hard => {
                    self.close();
                    return Ok(());
                }
                AbortKind::Soft => break,
                AbortKind::Active => {}
            }

            // 2b. Check round limit
            if self.config.max_tool_rounds_per_input > 0
                && round_count >= self.config.max_tool_rounds_per_input
            {
                self.emit_turn_limit("max_tool_rounds_per_input", round_count);
                break;
            }

            // 2c. Check session turn limit
            if self.config.max_turns > 0 && self.total_turns >= self.config.max_turns {
                self.emit_turn_limit("max_turns", self.total_turns);
                break;
            }

            // 3. Drain steering queue (after limit checks so messages are not
            //    consumed without a subsequent LLM call to deliver them)
            self.drain_steering();

            // 4. Context-usage warning (spec 5.5)
            self.check_context_usage();

            // 5. Build and send LLM request (streaming, abort-aware)
            let request = self.build_request();
            self.events.emit_assistant_text_start();

            // Stream the LLM response, emitting text deltas incrementally.
            // When the profile does not support streaming, fall back to
            // complete() with a single synthesized delta (avoids a wasted
            // stream() round-trip).  The block isolates borrows of
            // self.events / self.abort_signal so that self.close() /
            // self.handle_sdk_error() can take &mut self after the block
            // ends.
            //
            // `partial_text` accumulates streamed deltas so that abort and
            // error paths can emit TEXT_END with the text received so far
            // (instead of an empty string that contradicts earlier deltas).
            //
            // Retryable errors (network, timeout, server, rate-limit) are
            // retried with exponential backoff. Once any content has been
            // delivered to the user (text deltas *or* reasoning events),
            // retries are skipped because the partial output cannot be
            // un-delivered.
            let retry_policy = RetryPolicy::default();
            let partial_text = std::sync::Mutex::new(String::new());
            let content_emitted = std::sync::atomic::AtomicBool::new(false);
            let stream_result: Option<Result<Response, SdkError>> = {
                let events_ref = &self.events;
                let partial_ref = &partial_text;
                let emitted_ref = &content_emitted;
                let on_event = |event: StreamEvent| match event.event_type {
                    StreamEventType::TextDelta => {
                        if let Some(ref delta) = event.delta {
                            emitted_ref.store(true, std::sync::atomic::Ordering::Relaxed);
                            if let Ok(mut buf) = partial_ref.lock() {
                                buf.push_str(delta);
                            }
                            events_ref.emit_assistant_text_delta(delta);
                        }
                    }
                    StreamEventType::ReasoningStart => {
                        emitted_ref.store(true, std::sync::atomic::Ordering::Relaxed);
                        events_ref.emit_assistant_reasoning_start();
                    }
                    StreamEventType::ReasoningDelta => {
                        if let Some(ref delta) = event.reasoning_delta {
                            events_ref.emit_assistant_reasoning_delta(delta);
                        }
                    }
                    StreamEventType::ReasoningEnd => {
                        events_ref.emit_assistant_reasoning_end();
                    }
                    _ => {}
                };

                let client = Arc::clone(&self.client);
                let use_streaming = self.profile.supports_streaming();

                let call_with_retry = async {
                    let mut attempt: u32 = 0;
                    loop {
                        let result = if use_streaming {
                            client.stream_complete(request.clone(), &on_event).await
                        } else {
                            match client.complete(request.clone()).await {
                                Ok(response) => {
                                    let text = response.text();
                                    if !text.is_empty() {
                                        on_event(StreamEvent::text_delta(&text));
                                    }
                                    Ok(response)
                                }
                                Err(e) => Err(e),
                            }
                        };

                        match result {
                            Ok(response) => return Ok(response),
                            Err(error) => {
                                // Don't retry if any content (text or
                                // reasoning) has been delivered to the user.
                                let delivered =
                                    emitted_ref.load(std::sync::atomic::Ordering::Relaxed);

                                if !delivered
                                    && let Some(delay) = retry_policy.resolve_delay(&error, attempt)
                                {
                                    let label = retry_error_label(&error);
                                    tracing::warn!(
                                        attempt,
                                        delay_secs = delay,
                                        error = %error,
                                        "retryable LLM error, backing off"
                                    );
                                    events_ref.emit_info(
                                        "LLM_RETRY",
                                        format!(
                                            "{label}, retrying ({}/{})...",
                                            attempt + 1,
                                            retry_policy.max_retries
                                        ),
                                    );
                                    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                                    attempt += 1;
                                    continue;
                                }

                                return Err(error);
                            }
                        }
                    }
                };

                if let Some(ref signal) = self.abort_signal {
                    tokio::select! {
                        result = call_with_retry => Some(result),
                        () = signal.cancelled() => None,
                    }
                } else {
                    Some(call_with_retry.await)
                }
            };

            let response = match stream_result {
                None => {
                    // Abort during streaming: emit TEXT_END with partial text.
                    // Partial text is NOT recorded in history.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);
                    match self.abort_kind() {
                        AbortKind::Hard => {
                            self.close();
                            return Ok(());
                        }
                        _ => break,
                    }
                }
                Some(Ok(r)) => r,
                Some(Err(e)) => {
                    // Error: emit TEXT_END with any partial text received.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);

                    // On context-length errors, attempt automatic compaction
                    // once before giving up. This drops thinking blocks and
                    // summarises older tool results to free context space.
                    if matches!(e, SdkError::ContextLength { .. })
                        && !compaction_attempted
                        && self.compact_history()
                    {
                        compaction_attempted = true;
                        let mut data = serde_json::Map::new();
                        data.insert("severity".into(), Value::String("warning".into()));
                        data.insert(
                            "message".into(),
                            Value::String(
                                "Context length exceeded — compacted history and retrying".into(),
                            ),
                        );
                        self.events.emit(EventKind::Error, data);
                        continue;
                    }

                    return self.handle_sdk_error(e);
                }
            };

            // Successful LLM call — reset the compaction flag so that
            // later rounds (which may accumulate new content) can compact
            // again if needed.
            compaction_attempted = false;

            // 6. Record assistant turn
            let text = response.text();
            let tool_calls = response.tool_calls();
            let reasoning = response.reasoning();
            let thinking_parts: Vec<ContentPart> = response
                .message
                .content
                .iter()
                .filter(|p| {
                    matches!(
                        p,
                        ContentPart::Thinking { .. } | ContentPart::RedactedThinking { .. }
                    )
                })
                .cloned()
                .collect();
            let usage = response.usage.clone();
            let response_id = response.id.clone();

            self.events
                .emit_assistant_text_end(&text, reasoning.clone());

            self.history.push(Turn::Assistant {
                content: text,
                tool_calls: tool_calls.clone(),
                reasoning,
                thinking_parts,
                usage,
                response_id: Some(response_id),
                timestamp: now_timestamp(),
            });
            self.total_turns += 1;

            // 7. Natural completion: no tool calls
            if tool_calls.is_empty() {
                natural_completion = true;
                break;
            }

            // 8. Execute tool calls (abort-aware per spec Graceful Shutdown)
            let results = match self.execute_tool_calls(&tool_calls).await {
                Some(r) => r,
                None => {
                    // Abort fired during tool execution (tokio::select!
                    // cancelled the work future). Fabricate [Aborted]
                    // results for all tool calls so history stays
                    // well-formed — the assistant turn with tool_calls
                    // was already recorded above.
                    match self.abort_kind() {
                        AbortKind::Hard => {
                            self.close();
                            return Ok(());
                        }
                        _ => {
                            let aborted_results = tool_calls
                                .iter()
                                .map(|tc| aborted_tool_result(&tc.id))
                                .collect();
                            self.history.push(Turn::tool_results(aborted_results));
                            break;
                        }
                    }
                }
            };
            self.history.push(Turn::tool_results(results));

            round_count += 1;

            // 9. Loop detection (spec 2.10)
            if self.config.enable_loop_detection {
                self.check_loop_detection();
            }
        }

        // -- Single post-loop exit path --
        // Reached on natural completion, turn/round limits, AND soft abort.

        let soft_aborted = self.abort_kind() == AbortKind::Soft;

        // TODO(spec-ambiguity): The spec says follow-ups trigger "after the
        // current input is fully handled (model has produced a text-only
        // response)" (spec: 2.8, line 371), implying natural completion only.
        // However, the pseudocode (line 296) places the check after the loop
        // break, which is also reached on limits. We process follow-ups on
        // both paths because it is more useful — callers that queue follow-ups
        // expect them to run. (spec: 2.8)
        //
        // Soft abort skips follow-ups: the user cancelled this exchange,
        // so queued follow-ups should wait for the next explicit submit.

        // Process follow-up queue.
        // Recursion depth is bounded by the number of queued follow-ups, which
        // is controlled by the caller (typically 0-2 items). Each level
        // consumes one entry from the queue, so the depth cannot grow.
        if !soft_aborted && let Some(followup) = self.followup_queue.pop_front() {
            return Box::pin(self.process_input(&followup)).await;
        }

        // Auto-detect AwaitingInput (spec 2.3): only on natural completion
        // (text-only response). Limit-triggered exits and soft aborts must
        // not inspect stale assistant turns from earlier in the history.
        self.state = if soft_aborted {
            // Reset the signal so the next submit() starts clean.
            if let Some(ref signal) = self.abort_signal {
                signal.reset_soft();
            }
            SessionState::Idle
        } else if natural_completion
            && self.config.auto_detect_awaiting_input
            && self.looks_like_question()
        {
            SessionState::AwaitingInput
        } else {
            SessionState::Idle
        };
        Ok(())
    }

    /// Check whether the last assistant turn looks like a question to the user.
    ///
    /// Returns `true` when the most recent assistant turn has no tool calls and
    /// the last line of the text matches a question-like pattern: a trailing
    /// `?`, a solicitation phrase ("let me know", "would you like", etc.), or
    /// an interrogative word prefix **combined with** a trailing `?`.
    ///
    /// Interrogative words alone (e.g. "What follows is...") are not enough
    /// because they frequently begin declarative sentences.
    fn looks_like_question(&self) -> bool {
        let last = self
            .history
            .iter()
            .rev()
            .find(|t| matches!(t, Turn::Assistant { .. }));

        match last {
            Some(Turn::Assistant {
                content,
                tool_calls,
                ..
            }) => {
                // Only consider text-only responses (natural completion).
                if !tool_calls.is_empty() {
                    return false;
                }

                let trimmed = content.trim();
                if trimmed.is_empty() {
                    return false;
                }

                // Check for trailing question mark — always sufficient.
                if trimmed.ends_with('?') {
                    return true;
                }

                // Check the last line for solicitation phrases that reliably
                // indicate the model is waiting for user input, even without
                // a trailing `?`.
                let last_line = trimmed.lines().next_back().unwrap_or("").trim();
                let lower = last_line.to_lowercase();

                // Solicitation phrases: almost always request user action.
                let solicitation_prefixes = [
                    "would you",
                    "shall i",
                    "do you",
                    "should i",
                    "let me know",
                    "please confirm",
                    "please let me know",
                ];

                solicitation_prefixes
                    .iter()
                    .any(|prefix| lower.starts_with(prefix))
            }
            _ => false,
        }
    }

    // -- Request building --

    /// Build the LLM request from current state.
    fn build_request(&self) -> Request {
        let mut messages = vec![Message::system(&self.system_prompt)];
        messages.extend(self.convert_history_to_messages());

        let mut request = Request::new(self.profile.model(), messages);
        request.tools = Some(self.profile.tools());
        request.tool_choice = Some(ToolChoice::Auto);
        request.reasoning_effort = self
            .config
            .reasoning_effort
            .as_ref()
            .map(|r| r.as_str().to_string());
        request.provider = Some(self.profile.id().to_string());

        if let Some(opts) = self.profile.provider_options() {
            request.provider_options = Some(opts);
        }

        request
    }

    /// Convert the conversation history into models3 Messages.
    fn convert_history_to_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        for turn in &self.history {
            match turn {
                Turn::User { content, .. } => {
                    messages.push(Message::user(content.as_str()));
                }
                Turn::Steering { content, .. } => {
                    // Steering messages are user-role per spec 2.6
                    messages.push(Message::user(content.as_str()));
                }
                Turn::System { content, .. } => {
                    messages.push(Message::system(content.as_str()));
                }
                Turn::Assistant {
                    content,
                    tool_calls,
                    thinking_parts,
                    ..
                } => {
                    if thinking_parts.is_empty() && tool_calls.is_empty() {
                        messages.push(Message::assistant(content.as_str()));
                    } else {
                        let mut parts = Vec::new();
                        // Thinking blocks must precede text/tool_call content
                        // (required by Anthropic for extended thinking).
                        parts.extend(thinking_parts.iter().cloned());
                        if !content.is_empty() {
                            parts.push(ContentPart::text(content.as_str()));
                        }
                        for tc in tool_calls {
                            parts.push(ContentPart::tool_call(
                                &tc.id,
                                &tc.name,
                                tc.arguments.clone(),
                            ));
                        }
                        messages.push(Message::new(Role::Assistant, parts));
                    }
                }
                Turn::ToolResults { results, .. } => {
                    let include_images = self.profile.id() == "anthropic";
                    for result in results {
                        if include_images
                            && let Some(att) = self.image_attachments.get(&result.tool_call_id)
                        {
                            let mut msg = Message::new(
                                Role::Tool,
                                vec![
                                    ContentPart::tool_result(
                                        &result.tool_call_id,
                                        result.content.clone(),
                                        result.is_error,
                                    ),
                                    ContentPart::image_data(att.data.clone(), &att.media_type),
                                ],
                            );
                            msg.tool_call_id = Some(result.tool_call_id.clone());
                            messages.push(msg);
                            continue;
                        }
                        messages.push(Message::tool_result(
                            &result.tool_call_id,
                            result.content.clone(),
                            result.is_error,
                        ));
                    }
                }
            }
        }

        messages
    }

    // -- Steering --

    /// Drain the steering queue, appending turns and emitting events.
    fn drain_steering(&mut self) {
        while let Some(msg) = self.steering_queue.pop_front() {
            self.events.emit_steering_injected(&msg);
            self.history.push(Turn::steering(&msg));
        }
    }

    // -- Tool execution --

    /// Execute all tool calls from an assistant response.
    ///
    /// Returns `None` if the abort signal fires during execution (spec
    /// Graceful Shutdown Sequence). Parallel execution when the profile
    /// supports it, sequential otherwise.
    ///
    /// Subagent tool calls (`spawn_agent`, `send_input`, `wait`,
    /// `close_agent`) are intercepted and routed to the [`SubAgentManager`]
    /// rather than through the regular tool executor path (spec 7.1).
    async fn execute_tool_calls(&mut self, tool_calls: &[ToolCall]) -> Option<Vec<ToolResult>> {
        let abort = self.abort_signal.clone();

        let work = async {
            // Separate subagent calls from regular calls.
            // Subagent calls must run sequentially through &mut self because
            // SubAgentManager needs mutable access.
            let has_subagent = tool_calls
                .iter()
                .any(|tc| SubAgentManager::is_subagent_tool(&tc.name));

            if has_subagent {
                // When any subagent tool is present, run all calls sequentially
                // to avoid borrow conflicts with the subagent manager.
                self.execute_tools_with_subagents(tool_calls).await
            } else if self.profile.supports_parallel_tool_calls() && tool_calls.len() > 1 {
                self.execute_tools_parallel(tool_calls).await
            } else {
                self.execute_tools_sequential(tool_calls).await
            }
        };

        match abort {
            Some(signal) => {
                tokio::select! {
                    results = work => Some(results),
                    () = signal.cancelled() => None,
                }
            }
            None => Some(work.await),
        }
    }

    async fn execute_tools_sequential(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(tool_calls.len());
        for tc in tool_calls {
            if self.is_aborted() {
                // Backfill remaining tool calls with [Aborted] so the
                // result count matches tool_calls.len() (1:1 mapping).
                for remaining in &tool_calls[results.len()..] {
                    results.push(aborted_tool_result(&remaining.id));
                }
                break;
            }
            let (result, attachment) = self.execute_single_tool(tc).await;
            self.store_attachment_if_supported(attachment);
            results.push(result);
        }
        results
    }

    async fn execute_tools_parallel(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        // Collect references needed by all futures
        let env = &*self.execution_env;
        let registry = self.profile.tool_registry();
        let events = &self.events;
        let trunc_config = &self.truncation_config;

        let futs: Vec<_> = tool_calls
            .iter()
            .map(|tc| execute_tool(tc, registry, env, events, trunc_config))
            .collect();

        let pairs = futures::future::join_all(futs).await;
        let mut results = Vec::with_capacity(pairs.len());
        for (result, attachment) in pairs {
            self.store_attachment_if_supported(attachment);
            results.push(result);
        }
        results
    }

    /// Execute tool calls when subagent tools are present.
    ///
    /// Runs all calls sequentially, routing subagent tools through the
    /// [`SubAgentManager`] and regular tools through the normal executor.
    async fn execute_tools_with_subagents(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(tool_calls.len());
        for tc in tool_calls {
            if self.is_aborted() {
                // Backfill remaining tool calls with [Aborted] so the
                // result count matches tool_calls.len() (1:1 mapping).
                for remaining in &tool_calls[results.len()..] {
                    results.push(aborted_tool_result(&remaining.id));
                }
                break;
            }
            if SubAgentManager::is_subagent_tool(&tc.name) {
                results.push(self.execute_subagent_tool(tc).await);
            } else {
                let (result, attachment) = self.execute_single_tool(tc).await;
                self.store_attachment_if_supported(attachment);
                results.push(result);
            }
        }
        results
    }

    /// Execute a subagent tool call via the SubAgentManager.
    async fn execute_subagent_tool(&mut self, tool_call: &ToolCall) -> ToolResult {
        self.events
            .emit_tool_call_start(&tool_call.name, &tool_call.id, &tool_call.arguments);

        let result = self
            .subagent_manager
            .execute(&tool_call.name, tool_call.arguments.clone(), &*self.profile)
            .await;

        match result {
            Ok(output) => {
                self.events.emit_tool_call_end(&tool_call.id, &output);
                let truncated =
                    truncate_tool_output(&output, &tool_call.name, &self.truncation_config);
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(truncated),
                    is_error: false,
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.events
                    .emit_tool_call_end_error(&tool_call.id, &error_msg);
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(error_msg),
                    is_error: true,
                }
            }
        }
    }

    /// Store an image attachment only when the provider supports images in
    /// tool results. Avoids accumulating dead weight for providers that
    /// receive only the text fallback (OpenAI, Gemini).
    fn store_attachment_if_supported(&mut self, attachment: Option<(String, ImageAttachment)>) {
        if let Some((id, img)) = attachment
            && self.profile.id() == "anthropic"
        {
            self.image_attachments.insert(id, img);
        }
    }

    /// Execute a single tool call: emit events, run executor, truncate output.
    ///
    /// Returns `(ToolResult, Option<(tool_call_id, ImageAttachment)>)`.
    async fn execute_single_tool(
        &self,
        tool_call: &ToolCall,
    ) -> (ToolResult, Option<(String, ImageAttachment)>) {
        execute_tool(
            tool_call,
            self.profile.tool_registry(),
            &*self.execution_env,
            &self.events,
            &self.truncation_config,
        )
        .await
    }

    // -- Context usage (spec 5.5) --

    /// Emit context usage information and warn when approaching the limit.
    ///
    /// Uses heuristic: 1 token ~ 4 characters. Always emits a
    /// `ContextUsage` event with the current percentage. Additionally emits
    /// a warning `Error` event at 80% of the profile's
    /// `context_window_size`. Informational only — no automatic compaction.
    fn check_context_usage(&self) {
        let total_chars = self.estimate_history_chars();
        let approx_tokens = total_chars / 4;
        let context_size = self.profile.context_window_size();
        let pct = if context_size > 0 {
            (approx_tokens as f64 / context_size as f64 * 100.0) as u32
        } else {
            0
        };

        // Always emit context usage info
        let mut usage_data = serde_json::Map::new();
        usage_data.insert("percent".into(), Value::Number(pct.into()));
        usage_data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
        usage_data.insert(
            "context_window_size".into(),
            Value::Number(context_size.into()),
        );
        self.events
            .emit(crate::types::EventKind::ContextUsage, usage_data);

        // Warn at 80%
        let threshold = (context_size as f64 * 0.8) as u64;
        if approx_tokens > threshold {
            let mut data = serde_json::Map::new();
            data.insert("severity".into(), Value::String("warning".into()));
            data.insert(
                "message".into(),
                Value::String(format!("Context usage at ~{pct}% of context window")),
            );
            data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
            data.insert(
                "context_window_size".into(),
                Value::Number(context_size.into()),
            );
            self.events.emit(crate::types::EventKind::Error, data);
        }
    }

    /// Estimate total character count across history and system prompt.
    fn estimate_history_chars(&self) -> u64 {
        let mut chars: u64 = self.system_prompt.len() as u64;
        for turn in &self.history {
            match turn {
                Turn::User { content, .. }
                | Turn::Steering { content, .. }
                | Turn::System { content, .. } => {
                    chars += content.len() as u64;
                }
                Turn::Assistant {
                    content,
                    tool_calls,
                    reasoning,
                    thinking_parts,
                    ..
                } => {
                    chars += content.len() as u64;
                    for tc in tool_calls {
                        chars += tc.name.len() as u64;
                        chars += tc.arguments.to_string().len() as u64;
                    }
                    if let Some(r) = reasoning {
                        chars += r.len() as u64;
                    }
                    for part in thinking_parts {
                        if let ContentPart::Thinking { thinking }
                        | ContentPart::RedactedThinking { thinking } = part
                        {
                            chars += thinking.text.len() as u64;
                            if let Some(ref sig) = thinking.signature {
                                chars += sig.len() as u64;
                            }
                        }
                    }
                }
                Turn::ToolResults { results, .. } => {
                    for r in results {
                        chars += r.content.to_string().len() as u64;
                    }
                }
            }
        }
        chars
    }

    /// Attempt to compact the conversation history to reduce context size.
    ///
    /// Returns `true` if the history was actually modified (some content was
    /// removed), `false` if there is nothing left to compact.
    ///
    /// Strategy (all three phases are applied, tracked by a `modified` flag):
    /// 1. Strip thinking/reasoning blocks from all assistant turns.
    /// 2. Summarise tool results in older turns (keep the last 2 exchanges
    ///    intact so the model retains recent context).
    /// 3. Drop the oldest turns from the middle of the conversation, keeping
    ///    the first user message for task context and the last few turns for
    ///    recency. The tail boundary is adjusted to avoid orphaned
    ///    `ToolResults` turns whose matching `Assistant(tool_calls)` was
    ///    dropped.
    fn compact_history(&mut self) -> bool {
        let before = self.estimate_history_chars();
        let mut modified = false;

        // Phase 1: strip thinking/reasoning from ALL assistant turns.
        for turn in self.history.iter_mut() {
            if let Turn::Assistant {
                reasoning,
                thinking_parts,
                ..
            } = turn
            {
                if reasoning.is_some() {
                    *reasoning = None;
                    modified = true;
                }
                if !thinking_parts.is_empty() {
                    thinking_parts.clear();
                    modified = true;
                }
            }
        }

        // Phase 2: summarise tool results in older turns and remove
        // associated image attachments.
        // Keep the last 4 history entries intact (roughly the last
        // assistant + tool_results pair).
        let len = self.history.len();
        let preserve_tail = 4.min(len);
        let compactable = len.saturating_sub(preserve_tail);
        for turn in self.history[..compactable].iter_mut() {
            if let Turn::ToolResults { results, .. } = turn {
                for r in results.iter_mut() {
                    // Always strip image attachments from older turns —
                    // image bytes can be megabytes while the textual
                    // content is short.
                    if self.image_attachments.remove(&r.tool_call_id).is_some() {
                        modified = true;
                    }
                    let s = r.content.to_string();
                    if s.len() > 200 {
                        r.content = Value::String(format!(
                            "[Output compacted — {} chars removed to free context space]",
                            s.len()
                        ));
                        modified = true;
                    }
                }
            }
        }

        // Phase 3: if the history is very long, drop middle exchanges.
        // Keep the first turn (user task) and the last turns, removing
        // everything in between if there are more than 10 turns total.
        //
        // The tail boundary is adjusted backwards to avoid starting with
        // an orphaned ToolResults turn (whose matching Assistant was
        // dropped). This keeps tool-call/tool-result pairs intact so
        // providers don't reject the request.
        if self.history.len() > 10 {
            let keep_head = 1;
            let total = self.history.len();
            let mut tail_start = total.saturating_sub(preserve_tail.max(4));

            // Walk forward from the candidate boundary until we find a
            // turn that is safe to start with (anything other than
            // ToolResults, which would be orphaned).
            while tail_start < total {
                if matches!(self.history[tail_start], Turn::ToolResults { .. }) {
                    tail_start += 1;
                } else {
                    break;
                }
            }

            let keep_tail = total - tail_start;
            if keep_head + keep_tail < total {
                let removed = total - keep_head - keep_tail;
                // Remove image attachments for tool results in dropped turns.
                for turn in &self.history[keep_head..tail_start] {
                    if let Turn::ToolResults { results, .. } = turn {
                        for r in results {
                            self.image_attachments.remove(&r.tool_call_id);
                        }
                    }
                }
                let summary = Turn::system(format!(
                    "[Context compacted: {removed} earlier turns were removed to fit within \
                     the model's context window. The original user request and recent \
                     conversation are preserved.]"
                ));
                let mut new_history = Vec::with_capacity(keep_head + 1 + keep_tail);
                new_history.extend(self.history.drain(..keep_head));
                new_history.push(summary);
                let remaining = self.history.len();
                new_history.extend(self.history.drain(remaining - keep_tail..));
                self.history = new_history;
                modified = true;
            }
        }

        let after = self.estimate_history_chars();
        tracing::debug!(
            before_chars = before,
            after_chars = after,
            modified,
            "context compaction complete"
        );
        modified
    }

    // -- Loop detection --

    /// Check for repeating tool-call patterns and inject steering if found.
    ///
    /// Uses a bounded sliding window (capped at `loop_detection_window`) to
    /// avoid unbounded growth.
    fn check_loop_detection(&mut self) {
        let window = self.config.loop_detection_window as usize;

        // Collect signatures from the latest assistant turn's tool calls
        if let Some(Turn::Assistant { tool_calls, .. }) = self
            .history
            .iter()
            .rev()
            .find(|t| matches!(t, Turn::Assistant { .. }))
        {
            for tc in tool_calls {
                self.tool_call_signatures
                    .push_back(loop_detection::tool_call_signature(tc));
                // Keep window bounded
                while self.tool_call_signatures.len() > window {
                    self.tool_call_signatures.pop_front();
                }
            }
        }

        let sigs: Vec<String> = self.tool_call_signatures.iter().cloned().collect();
        if let Some(message) = loop_detection::detect_loop(&sigs, window) {
            self.events.emit_loop_detection(&message);
            self.history.push(Turn::steering(&message));
        }
    }

    // -- Error handling --

    /// Handle an SDK error from an LLM call (spec Appendix B).
    ///
    /// Non-retryable errors (authentication, invalid request, etc.) close the
    /// session — these indicate a persistent problem that the user cannot fix
    /// by simply retrying.
    ///
    /// Retryable errors (network, timeout, server, rate-limit) have already
    /// been retried with exponential backoff by the session-level retry loop.
    /// If they still fail, the session transitions back to IDLE rather than
    /// CLOSED, allowing the user to try again without losing conversation
    /// history.
    ///
    /// Context-length errors are emitted with `"severity": "warning"` per
    /// spec (the host may implement compaction), while other errors use
    /// plain ERROR events.
    fn handle_sdk_error(&mut self, error: SdkError) -> AgentResult<()> {
        let is_retryable = error.is_retryable();
        let is_context_length = matches!(error, SdkError::ContextLength { .. });
        let agent_error = AgentError::from(error);

        if is_context_length {
            // Emit with severity:warning per spec — host can implement compaction
            let mut data = serde_json::Map::new();
            data.insert("severity".into(), Value::String("warning".into()));
            data.insert("code".into(), Value::String(agent_error.code().into()));
            data.insert("message".into(), Value::String(agent_error.to_string()));
            self.events.emit(crate::types::EventKind::Error, data);
        } else {
            self.events
                .emit_error(agent_error.code(), agent_error.to_string());
        }

        if is_retryable {
            // Retryable errors: keep the session open so the user can retry.
            // Transition back to IDLE rather than CLOSED.
            self.state = SessionState::Idle;
            Err(agent_error)
        } else {
            self.close();
            Err(agent_error)
        }
    }

    // -- Helpers --

    /// Read the current abort kind with a single atomic load.
    ///
    /// Returns [`AbortKind::Active`] when no signal is attached.
    fn abort_kind(&self) -> AbortKind {
        self.abort_signal
            .as_ref()
            .map_or(AbortKind::Active, AbortSignal::kind)
    }

    /// Check if any abort (soft or hard) has been triggered.
    fn is_aborted(&self) -> bool {
        self.abort_kind() != AbortKind::Active
    }

    /// Emit a TURN_LIMIT event with limit details.
    fn emit_turn_limit(&self, limit_type: &str, count: u32) {
        let mut data = serde_json::Map::new();
        data.insert("limit_type".into(), Value::String(limit_type.into()));
        data.insert("count".into(), Value::Number(count.into()));
        self.events.emit_turn_limit(data);
    }
}

impl Drop for ApiSession {
    fn drop(&mut self) {
        self.close();
    }
}

/// Human-readable label for a retryable error, used in the `LLM_RETRY` info
/// event so the user sees an accurate description of what went wrong.
fn retry_error_label(error: &SdkError) -> &'static str {
    match error {
        SdkError::RateLimit { .. } => "Rate limited",
        SdkError::Server { .. } => "Server error",
        SdkError::RequestTimeout { .. } => "Request timeout",
        SdkError::Network { .. } => "Network error",
        SdkError::Stream { .. } => "Stream error",
        // Fallback — should not happen since only retryable errors reach here.
        _ => "Transient error",
    }
}

/// Create a synthetic `[Aborted]` tool result for a tool call that was
/// cancelled before it could complete.
fn aborted_tool_result(tool_call_id: &str) -> ToolResult {
    ToolResult {
        tool_call_id: tool_call_id.to_string(),
        content: serde_json::Value::String("[Aborted]".into()),
        is_error: true,
    }
}

// ---------------------------------------------------------------------------
// Free function for tool execution (enables parallel use without &mut self)
// ---------------------------------------------------------------------------

/// Execute a single tool call. Factored out as a free function so it can be
/// called in parallel without borrowing `&mut Session`.
///
/// Performs schema validation (spec 3.8 step 2) between tool lookup and
/// execution. Invalid arguments produce an `is_error: true` result without
/// invoking the executor.
///
/// Returns `(ToolResult, Option<(tool_call_id, ImageAttachment)>)`. The
/// second element is `Some` when the tool produced image data that should
/// be attached to the tool result message for multimodal providers.
async fn execute_tool(
    tool_call: &ToolCall,
    registry: &crate::registry::ToolRegistry,
    env: &dyn ExecutionEnvironment,
    events: &EventEmitter,
    trunc_config: &TruncationConfig,
) -> (ToolResult, Option<(String, ImageAttachment)>) {
    events.emit_tool_call_start(&tool_call.name, &tool_call.id, &tool_call.arguments);

    // Check for argument parse errors (garbled/non-JSON arguments from the model)
    if let Some(ref err) = tool_call.parse_error {
        let error_msg = format!(
            "invalid JSON arguments for tool '{}': {err}. The arguments must be a valid JSON object.",
            tool_call.name
        );
        events.emit_tool_call_end_error(&tool_call.id, &error_msg);
        return (
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(error_msg),
                is_error: true,
            },
            None,
        );
    }

    // VALIDATE (spec 3.8 step 2) — before execute
    if let Err(e) = registry.validate_arguments(&tool_call.name, &tool_call.arguments) {
        let error_msg = e.to_string();
        events.emit_tool_call_end_error(&tool_call.id, &error_msg);
        return (
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(error_msg),
                is_error: true,
            },
            None,
        );
    }

    let result = match registry.get(&tool_call.name) {
        Some(tool) => tool.execute(tool_call.arguments.clone(), env).await,
        None => Err(AgentError::UnknownTool {
            name: tool_call.name.clone(),
        }),
    };

    match result {
        Ok(output) => {
            let text = output.as_text();
            // Full output in event (spec 2.9: TOOL_CALL_END has untruncated output)
            events.emit_tool_call_end(&tool_call.id, text);
            // Truncated version for LLM
            let truncated = truncate_tool_output(text, &tool_call.name, trunc_config);

            let attachment = match output {
                ToolOutput::ImageWithText {
                    data, media_type, ..
                } => Some((tool_call.id.clone(), ImageAttachment { data, media_type })),
                ToolOutput::Text(_) => None,
            };

            (
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(truncated),
                    is_error: false,
                },
                attachment,
            )
        }
        Err(e) => {
            let error_msg = e.to_string();
            events.emit_tool_call_end_error(&tool_call.id, &error_msg);
            (
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(error_msg),
                    is_error: true,
                },
                None,
            )
        }
    }
}

// Re-export for backward compatibility — the function lives in `prompts`
// where it belongs alongside the other prompt-building helpers.
pub use crate::prompts::build_system_prompt;
