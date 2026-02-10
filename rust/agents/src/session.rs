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

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use stencila_models3::api::accumulator::StreamAccumulator;
use stencila_models3::error::SdkError;
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
use crate::subagents::SubAgentManager;
use crate::truncation::{TruncationConfig, truncate_tool_output};
use crate::types::{SessionConfig, SessionState, Turn, now_timestamp};

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
// Abort types
// ---------------------------------------------------------------------------

/// Controller that creates abort signals and triggers cancellation.
///
/// Create with [`AbortController::new()`], pass [`signal()`](Self::signal)
/// to the session, and call [`abort()`](Self::abort) to cancel.
#[derive(Debug, Clone)]
pub struct AbortController {
    aborted: Arc<AtomicBool>,
}

impl AbortController {
    /// Create a new abort controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get an abort signal that can be polled.
    #[must_use]
    pub fn signal(&self) -> AbortSignal {
        AbortSignal {
            aborted: Arc::clone(&self.aborted),
        }
    }

    /// Signal abort. All signals derived from this controller will
    /// immediately report `is_aborted() == true`.
    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Release);
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

/// Polling signal for session cancellation.
#[derive(Debug, Clone)]
pub struct AbortSignal {
    aborted: Arc<AtomicBool>,
}

impl AbortSignal {
    /// Whether abort has been signaled.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Acquire)
    }

    /// Returns a future that resolves when abort is signaled.
    ///
    /// Polls the atomic flag every 10ms. Suitable for use with
    /// `tokio::select!` to cancel in-flight work.
    pub async fn cancelled(&self) {
        loop {
            if self.is_aborted() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// An agent session managing the conversation loop (spec 2.1).
///
/// Created via [`Session::new()`], driven by [`submit()`](Self::submit).
/// Events are delivered through the [`EventReceiver`] returned by the
/// constructor.
pub struct Session {
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
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("state", &self.state)
            .field("history_len", &self.history.len())
            .field("total_turns", &self.total_turns)
            .finish_non_exhaustive()
    }
}

impl Session {
    /// Create a new session.
    ///
    /// Returns the session and an [`EventReceiver`] for consuming events.
    /// The caller is responsible for building the system prompt (see
    /// [`build_system_prompt()`] helper).
    ///
    /// The `current_depth` parameter controls subagent nesting: 0 for a
    /// top-level session, incremented by 1 for each child.
    ///
    /// Emits a `SESSION_START` event immediately.
    pub fn new(
        mut profile: Box<dyn ProviderProfile>,
        execution_env: Arc<dyn ExecutionEnvironment>,
        client: Arc<dyn LlmClient>,
        config: SessionConfig,
        system_prompt: String,
        current_depth: u32,
    ) -> (Self, EventReceiver) {
        let (emitter, receiver) = events::channel();
        emitter.emit_session_start();

        let truncation_config = TruncationConfig {
            tool_output_limits: config.tool_output_limits.clone(),
            tool_line_limits: config.tool_line_limits.clone(),
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

        let subagent_manager = SubAgentManager::new(
            Arc::clone(&execution_env),
            Arc::clone(&client),
            current_depth,
            max_depth,
        );

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

    /// Transition the session to AwaitingInput state (spec 2.3).
    ///
    /// This is a host-driven API: after [`submit()`](Self::submit) returns,
    /// the host inspects the last assistant turn. If the model asked a
    /// question, the host calls this method. When the user answers, the host
    /// calls [`submit()`](Self::submit) again.
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

        // The loop runs until natural completion, a limit, or abort.
        // SDK errors propagate immediately (session → CLOSED).
        loop {
            // 2. Drain steering queue
            self.drain_steering();

            // 3. Check abort
            if self.is_aborted() {
                self.close();
                return Ok(());
            }

            // 3b. Check round limit
            if round_count >= self.config.max_tool_rounds_per_input {
                self.emit_turn_limit("max_tool_rounds_per_input", round_count);
                break;
            }

            // 3c. Check session turn limit
            if self.config.max_turns > 0 && self.total_turns >= self.config.max_turns {
                self.emit_turn_limit("max_turns", self.total_turns);
                break;
            }

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
            let partial_text = std::sync::Mutex::new(String::new());
            let stream_result: Option<Result<Response, SdkError>> = {
                let events_ref = &self.events;
                let partial_ref = &partial_text;
                let on_event = |event: StreamEvent| {
                    if event.event_type == StreamEventType::TextDelta
                        && let Some(ref delta) = event.delta
                    {
                        if let Ok(mut buf) = partial_ref.lock() {
                            buf.push_str(delta);
                        }
                        events_ref.emit_assistant_text_delta(delta);
                    }
                };

                let client = Arc::clone(&self.client);
                let use_streaming = self.profile.supports_streaming();

                let call = async {
                    if use_streaming {
                        client.stream_complete(request, &on_event).await
                    } else {
                        let response = client.complete(request).await?;
                        let text = response.text();
                        if !text.is_empty() {
                            on_event(StreamEvent::text_delta(&text));
                        }
                        Ok(response)
                    }
                };

                if let Some(ref signal) = self.abort_signal {
                    tokio::select! {
                        result = call => Some(result),
                        () = signal.cancelled() => None,
                    }
                } else {
                    Some(call.await)
                }
            };

            let response = match stream_result {
                None => {
                    // Abort: emit TEXT_END with any partial text received.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);
                    self.close();
                    return Ok(());
                }
                Some(Ok(r)) => r,
                Some(Err(e)) => {
                    // Error: emit TEXT_END with any partial text received.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);
                    return self.handle_sdk_error(e);
                }
            };

            // 6. Record assistant turn
            let text = response.text();
            let tool_calls = response.tool_calls();
            let reasoning = response.reasoning();
            let usage = response.usage.clone();
            let response_id = response.id.clone();

            self.events
                .emit_assistant_text_end(&text, reasoning.clone());

            self.history.push(Turn::Assistant {
                content: text,
                tool_calls: tool_calls.clone(),
                reasoning,
                usage,
                response_id: Some(response_id),
                timestamp: now_timestamp(),
            });
            self.total_turns += 1;

            // 7. Natural completion: no tool calls
            if tool_calls.is_empty() {
                break;
            }

            // 8. Execute tool calls (abort-aware per spec Graceful Shutdown)
            let results = match self.execute_tool_calls(&tool_calls).await {
                Some(r) => r,
                None => {
                    // Abort fired during tool execution — exit immediately
                    self.close();
                    return Ok(());
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
        // Reached on natural completion AND turn/round limits.

        // TODO(spec-ambiguity): The spec says follow-ups trigger "after the
        // current input is fully handled (model has produced a text-only
        // response)" (spec: 2.8, line 371), implying natural completion only.
        // However, the pseudocode (line 296) places the check after the loop
        // break, which is also reached on limits. We process follow-ups on
        // both paths because it is more useful — callers that queue follow-ups
        // expect them to run. (spec: 2.8)

        // Process follow-up queue.
        // Recursion depth is bounded by the number of queued follow-ups, which
        // is controlled by the caller (typically 0-2 items). Each level
        // consumes one entry from the queue, so the depth cannot grow.
        if let Some(followup) = self.followup_queue.pop_front() {
            return Box::pin(self.process_input(&followup)).await;
        }

        self.state = SessionState::Idle;
        Ok(())
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
                    ..
                } => {
                    if tool_calls.is_empty() {
                        messages.push(Message::assistant(content.as_str()));
                    } else {
                        let mut parts = Vec::new();
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
                    for result in results {
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
                break;
            }
            results.push(self.execute_single_tool(tc).await);
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

        futures::future::join_all(futs).await
    }

    /// Execute tool calls when subagent tools are present.
    ///
    /// Runs all calls sequentially, routing subagent tools through the
    /// [`SubAgentManager`] and regular tools through the normal executor.
    async fn execute_tools_with_subagents(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(tool_calls.len());
        for tc in tool_calls {
            if self.is_aborted() {
                break;
            }
            if SubAgentManager::is_subagent_tool(&tc.name) {
                results.push(self.execute_subagent_tool(tc).await);
            } else {
                results.push(self.execute_single_tool(tc).await);
            }
        }
        results
    }

    /// Execute a subagent tool call via the SubAgentManager.
    async fn execute_subagent_tool(&mut self, tool_call: &ToolCall) -> ToolResult {
        self.events
            .emit_tool_call_start(&tool_call.name, &tool_call.id);

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

    /// Execute a single tool call: emit events, run executor, truncate output.
    async fn execute_single_tool(&self, tool_call: &ToolCall) -> ToolResult {
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

    /// Emit a warning when conversation approaches the context window limit.
    ///
    /// Uses heuristic: 1 token ~ 4 characters. Warns at 80% of the profile's
    /// `context_window_size`. Informational only — no automatic compaction.
    ///
    // TODO(spec-ambiguity): The spec says `emit(WARNING, ...)` but the
    // EventKind enum has no WARNING variant. We use ERROR with
    // `"severity": "warning"` as a pragmatic alternative. (spec: 5.5)
    fn check_context_usage(&self) {
        let total_chars = self.estimate_history_chars();
        let approx_tokens = total_chars / 4;
        let context_size = self.profile.context_window_size();
        let threshold = (context_size as f64 * 0.8) as u64;

        if approx_tokens > threshold {
            let pct = if context_size > 0 {
                (approx_tokens as f64 / context_size as f64 * 100.0) as u32
            } else {
                100
            };
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
    /// All SDK errors reaching the session layer cause CLOSED state — retryable
    /// errors have already been retried by the SDK's retry layer.
    ///
    /// Context-length errors are emitted with `"severity": "warning"` per
    /// spec (the host may implement compaction), while other errors use
    /// plain ERROR events.
    fn handle_sdk_error(&mut self, error: SdkError) -> AgentResult<()> {
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

        self.state = SessionState::Closed;
        self.events.emit_session_end(self.state);
        Err(agent_error)
    }

    // -- Helpers --

    /// Check if the abort signal has been triggered.
    fn is_aborted(&self) -> bool {
        self.abort_signal.as_ref().is_some_and(|s| s.is_aborted())
    }

    /// Emit a TURN_LIMIT event with limit details.
    fn emit_turn_limit(&self, limit_type: &str, count: u32) {
        let mut data = serde_json::Map::new();
        data.insert("limit_type".into(), Value::String(limit_type.into()));
        data.insert("count".into(), Value::Number(count.into()));
        self.events.emit_turn_limit(data);
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
async fn execute_tool(
    tool_call: &ToolCall,
    registry: &crate::registry::ToolRegistry,
    env: &dyn ExecutionEnvironment,
    events: &EventEmitter,
    trunc_config: &TruncationConfig,
) -> ToolResult {
    events.emit_tool_call_start(&tool_call.name, &tool_call.id);

    // VALIDATE (spec 3.8 step 2) — before execute
    if let Err(e) = registry.validate_arguments(&tool_call.name, &tool_call.arguments) {
        let error_msg = e.to_string();
        events.emit_tool_call_end_error(&tool_call.id, &error_msg);
        return ToolResult {
            tool_call_id: tool_call.id.clone(),
            content: Value::String(error_msg),
            is_error: true,
        };
    }

    let result = match registry.get(&tool_call.name) {
        Some(tool) => tool.execute(tool_call.arguments.clone(), env).await,
        None => Err(AgentError::UnknownTool {
            name: tool_call.name.clone(),
        }),
    };

    match result {
        Ok(output) => {
            // Full output in event (spec 2.9: TOOL_CALL_END has untruncated output)
            events.emit_tool_call_end(&tool_call.id, &output);
            // Truncated version for LLM
            let truncated = truncate_tool_output(&output, &tool_call.name, trunc_config);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(truncated),
                is_error: false,
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            events.emit_tool_call_end_error(&tool_call.id, &error_msg);
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(error_msg),
                is_error: true,
            }
        }
    }
}

// Re-export for backward compatibility — the function lives in `prompts`
// where it belongs alongside the other prompt-building helpers.
pub use crate::prompts::build_system_prompt;
