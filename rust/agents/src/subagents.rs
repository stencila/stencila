//! Subagent spawning and lifecycle management (spec 7.1-7.4).
//!
//! A subagent is a child [`Session`] spawned by the parent to handle a scoped
//! task. The subagent runs its own agentic loop with its own conversation
//! history but shares the parent's execution environment (same filesystem).
//!
//! The four subagent tools (`spawn_agent`, `send_input`, `wait`,
//! `close_agent`) are registered automatically by [`Session::new()`] when the
//! session's depth allows spawning (`current_depth < max_subagent_depth`).
//! Tool calls are intercepted by the session layer and routed to the
//! [`SubAgentManager`] rather than through the regular tool executor path.
//!
//! # Async execution
//!
//! `spawn_agent` returns immediately while the child session runs in a
//! `tokio::spawn` task. The parent communicates with the child via channels:
//! a oneshot for the initial result, and an mpsc for subsequent commands
//! (`send_input`, `close`). This enables true parallel subagent execution
//! (spec 7.4).

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;
use tokio::sync::{mpsc, oneshot};

use crate::error::{AgentError, AgentResult};
use crate::execution::ExecutionEnvironment;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::session::{AbortController, AbortSignal, LlmClient, Session};
use crate::types::SessionConfig;

// ---------------------------------------------------------------------------
// Tool name constants
// ---------------------------------------------------------------------------

/// Tool name for spawning a subagent.
pub const TOOL_SPAWN_AGENT: &str = "spawn_agent";
/// Tool name for sending input to a running subagent.
pub const TOOL_SEND_INPUT: &str = "send_input";
/// Tool name for waiting on a subagent to complete.
pub const TOOL_WAIT: &str = "wait";
/// Tool name for terminating a subagent.
pub const TOOL_CLOSE_AGENT: &str = "close_agent";

/// All subagent tool names.
const SUBAGENT_TOOL_NAMES: &[&str] = &[
    TOOL_SPAWN_AGENT,
    TOOL_SEND_INPUT,
    TOOL_WAIT,
    TOOL_CLOSE_AGENT,
];

// ---------------------------------------------------------------------------
// Argument parsing helpers
// ---------------------------------------------------------------------------

/// Extract a required string argument, returning a validation error if absent.
fn require_str<'a>(args: &'a Value, key: &str) -> AgentResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AgentError::ValidationError {
            reason: format!("missing required string parameter: {key}"),
        })
}

/// Extract an optional string argument.
fn optional_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

/// Extract an optional u64 argument.
fn optional_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}

// ---------------------------------------------------------------------------
// Subagent types (spec 7.3)
// ---------------------------------------------------------------------------

/// Status of a subagent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubAgentStatus {
    /// The subagent is actively processing or idle.
    Running,
    /// The subagent completed naturally.
    Completed,
    /// The subagent encountered an unrecoverable error.
    Failed,
}

/// Result returned when a subagent finishes (spec 7.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubAgentResult {
    /// Final text output from the subagent's last assistant turn.
    pub output: String,
    /// Whether the subagent completed successfully.
    pub success: bool,
    /// Number of LLM turns the subagent used.
    pub turns_used: u32,
}

// ---------------------------------------------------------------------------
// Channel types for parent ↔ spawned-task communication
// ---------------------------------------------------------------------------

/// Command sent from the parent to a running subagent task.
enum AgentCommand {
    /// Send a follow-up message to the subagent.
    SendInput {
        message: String,
        reply_tx: oneshot::Sender<AgentStepResult>,
    },
    /// Request graceful shutdown.
    Close,
}

/// Outcome of a single agent step (initial task or send_input).
#[derive(Debug, Clone)]
enum AgentStepResult {
    Completed(SubAgentResult),
    Failed(SubAgentResult),
}

// ---------------------------------------------------------------------------
// SubAgentHandle
// ---------------------------------------------------------------------------

/// A handle to a running subagent (spec 7.3).
pub struct SubAgentHandle {
    /// Unique identifier for this subagent.
    pub id: String,
    /// Current status.
    status: SubAgentStatus,
    /// Channel for sending commands to the spawned task.
    command_tx: mpsc::Sender<AgentCommand>,
    /// Oneshot receiver for the initial task result. `None` after consumed.
    initial_result_rx: Option<oneshot::Receiver<AgentStepResult>>,
    /// Cached result from the most recent completed step.
    cached_result: Option<SubAgentResult>,
    /// Controller to signal abort to the child session.
    abort_controller: AbortController,
    /// Join handle for the spawned task. `None` after consumed by close.
    join_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SubAgentHandle {
    /// Apply a step result to this handle, updating status and cached result.
    fn apply_step_result(&mut self, step_result: &AgentStepResult) {
        match step_result {
            AgentStepResult::Completed(result) => {
                self.status = SubAgentStatus::Completed;
                self.cached_result = Some(result.clone());
            }
            AgentStepResult::Failed(result) => {
                self.status = SubAgentStatus::Failed;
                self.cached_result = Some(result.clone());
            }
        }
    }
}

impl std::fmt::Debug for SubAgentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubAgentHandle")
            .field("id", &self.id)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Spawned task function
// ---------------------------------------------------------------------------

/// Async function that owns the child Session and runs inside `tokio::spawn`.
///
/// 1. Runs the initial task via `session.submit(task)`.
/// 2. Sends the result on `initial_result_tx`.
/// 3. Enters a command loop, processing `SendInput` and `Close` commands.
/// 4. Exits when the command channel closes, a `Close` is received, or
///    the abort signal fires.
async fn run_agent_task(
    mut session: Session,
    task: String,
    abort_signal: AbortSignal,
    initial_result_tx: oneshot::Sender<AgentStepResult>,
    mut command_rx: mpsc::Receiver<AgentCommand>,
) {
    // Run initial task. submit() returns Ok(()) both on natural completion
    // and on abort (after calling session.close()), so we check session state
    // to distinguish: Closed means abort/error → Failed.
    let initial_step = session.submit(&task).await;
    let step_result = match initial_step {
        Ok(()) if session.state() != crate::types::SessionState::Closed => {
            AgentStepResult::Completed(extract_result_from_session(&session))
        }
        _ => AgentStepResult::Failed(extract_result_from_session(&session)),
    };

    // Send initial result (ignore error if parent dropped receiver)
    let _ = initial_result_tx.send(step_result);

    // Command loop
    loop {
        tokio::select! {
            cmd = command_rx.recv() => {
                match cmd {
                    Some(AgentCommand::SendInput { message, reply_tx }) => {
                        let result = session.submit(&message).await;
                        let step = match result {
                            Ok(()) if session.state() != crate::types::SessionState::Closed => {
                                AgentStepResult::Completed(extract_result_from_session(&session))
                            }
                            _ => AgentStepResult::Failed(extract_result_from_session(&session)),
                        };
                        let _ = reply_tx.send(step);
                    }
                    Some(AgentCommand::Close) | None => {
                        session.close();
                        return;
                    }
                }
            }
            () = abort_signal.cancelled() => {
                session.close();
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SubAgentManager
// ---------------------------------------------------------------------------

/// Manages the lifecycle of subagents spawned by a parent session (spec 7.1).
///
/// Holds references to the parent's execution environment and LLM client so
/// that child sessions can be created on demand. Depth limiting prevents
/// recursive spawning beyond `max_subagent_depth`.
pub struct SubAgentManager {
    agents: HashMap<String, SubAgentHandle>,
    execution_env: Arc<dyn ExecutionEnvironment>,
    client: Arc<dyn LlmClient>,
    /// Current nesting depth (0 = top-level session).
    current_depth: u32,
    /// Maximum allowed depth from config.
    max_depth: u32,
    /// Counter for generating unique agent IDs.
    next_id: u32,
}

impl std::fmt::Debug for SubAgentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubAgentManager")
            .field("agent_count", &self.agents.len())
            .field("current_depth", &self.current_depth)
            .field("max_depth", &self.max_depth)
            .finish_non_exhaustive()
    }
}

impl SubAgentManager {
    /// Create a new subagent manager.
    pub fn new(
        execution_env: Arc<dyn ExecutionEnvironment>,
        client: Arc<dyn LlmClient>,
        current_depth: u32,
        max_depth: u32,
    ) -> Self {
        Self {
            agents: HashMap::new(),
            execution_env,
            client,
            current_depth,
            max_depth,
            next_id: 0,
        }
    }

    /// Whether a tool name is a subagent tool that should be intercepted.
    pub fn is_subagent_tool(name: &str) -> bool {
        SUBAGENT_TOOL_NAMES.contains(&name)
    }

    /// Execute a subagent tool call.
    ///
    /// Routes to the appropriate handler based on tool name.
    ///
    /// Uses `Box::pin` to break the recursive async cycle:
    /// `execute → spawn_agent → session.submit → execute_tool_calls →
    /// execute_subagent_tool → execute`.
    pub fn execute<'a>(
        &'a mut self,
        tool_name: &'a str,
        args: Value,
        parent_profile: &'a dyn ProviderProfile,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<String>> + Send + 'a>> {
        Box::pin(async move {
            match tool_name {
                TOOL_SPAWN_AGENT => self.spawn_agent(args, parent_profile).await,
                TOOL_SEND_INPUT => self.send_input(args).await,
                TOOL_WAIT => self.wait_agent(args).await,
                TOOL_CLOSE_AGENT => self.close_agent(args).await,
                _ => Err(AgentError::UnknownTool {
                    name: tool_name.to_string(),
                }),
            }
        })
    }

    /// Spawn a new subagent (spec 7.2).
    ///
    /// Creates a child session and launches it in a `tokio::spawn` task.
    /// Returns immediately with `{"status": "running"}` while the child
    /// runs asynchronously. Use `wait` to retrieve the result.
    async fn spawn_agent(
        &mut self,
        args: Value,
        parent_profile: &dyn ProviderProfile,
    ) -> AgentResult<String> {
        // Check depth limit
        if self.current_depth >= self.max_depth {
            return Err(AgentError::ValidationError {
                reason: format!(
                    "maximum subagent depth ({}) exceeded — cannot spawn sub-sub-agents",
                    self.max_depth
                ),
            });
        }

        let task = require_str(&args, "task")?.to_string();
        let working_dir = optional_str(&args, "working_dir").map(str::to_string);
        let model_override = optional_str(&args, "model").map(str::to_string);
        let max_turns =
            optional_u64(&args, "max_turns").map_or(50, |v| u32::try_from(v).unwrap_or(u32::MAX));

        // Build child config
        let child_config = SessionConfig {
            max_turns,
            max_subagent_depth: self.max_depth,
            ..SessionConfig::default()
        };

        // Use parent's profile type to create child profile, with optional model override.
        // Subagent tools are registered by Session::new() based on depth.
        let child_model = model_override.as_deref();
        let child_profile = create_child_profile(
            parent_profile,
            child_model,
            child_config.max_command_timeout_ms,
        )?;

        // Build system prompt for child, including working_dir scope if specified
        let mut system_prompt =
            crate::prompts::build_system_prompt(&*child_profile, &*self.execution_env).await?;
        if let Some(ref dir) = working_dir {
            system_prompt.push_str(&format!(
                "\n\nYou are scoped to the subdirectory: {dir}\n\
                 Focus your work within this directory."
            ));
        }

        // Create the child session. The receiver is dropped immediately —
        // the emitter silently discards events when no receiver exists,
        // avoiding unbounded memory growth from unconsumed child events.
        let (mut session, _receiver) = Session::new(
            child_profile,
            Arc::clone(&self.execution_env),
            Arc::clone(&self.client),
            child_config,
            system_prompt,
            self.current_depth + 1,
        );

        // Set up abort controller for the child
        let abort_controller = AbortController::new();
        session.set_abort_signal(abort_controller.signal());

        // Create channels
        let (command_tx, command_rx) = mpsc::channel(16);
        let (initial_result_tx, initial_result_rx) = oneshot::channel();

        // Spawn the task
        let abort_signal = abort_controller.signal();
        let join_handle = tokio::spawn(run_agent_task(
            session,
            task,
            abort_signal,
            initial_result_tx,
            command_rx,
        ));

        // Generate agent ID
        self.next_id += 1;
        let agent_id = format!("agent-{}", self.next_id);

        let handle = SubAgentHandle {
            id: agent_id.clone(),
            status: SubAgentStatus::Running,
            command_tx,
            initial_result_rx: Some(initial_result_rx),
            cached_result: None,
            abort_controller,
            join_handle: Some(join_handle),
        };

        self.agents.insert(agent_id.clone(), handle);

        // Return immediately — child is running in background
        Ok(serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": "running",
        }))
        .unwrap_or_default())
    }

    /// Wait for a subagent to complete and return its result (spec 7.2).
    ///
    /// If the agent has already completed and the result is cached, returns
    /// immediately. Otherwise, awaits the initial result from the spawned task.
    async fn wait_agent(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?.to_string();

        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!("unknown agent_id: {agent_id}"),
            })?;

        // Return cached result if available
        if let Some(ref result) = agent.cached_result {
            return Ok(format_wait_result(&agent_id, agent.status, result));
        }

        // Await the initial result
        let rx = agent
            .initial_result_rx
            .take()
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!(
                    "agent {agent_id} initial result already consumed and no cached result"
                ),
            })?;

        let step_result = rx.await.map_err(|_| AgentError::Io {
            message: format!("agent {agent_id} task ended without sending result"),
        })?;

        agent.apply_step_result(&step_result);

        let result = agent
            .cached_result
            .as_ref()
            .expect("just set cached_result above");
        Ok(format_wait_result(&agent_id, agent.status, result))
    }

    /// Send a message to a subagent (spec 7.2).
    ///
    /// If the initial task has not yet completed, waits for it first.
    /// Then sends a follow-up message via the command channel.
    async fn send_input(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?.to_string();
        let message = require_str(&args, "message")?.to_string();

        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!("unknown agent_id: {agent_id}"),
            })?;

        // Consume initial result if still pending
        if let Some(rx) = agent.initial_result_rx.take() {
            let step_result = rx.await.map_err(|_| AgentError::Io {
                message: format!("agent {agent_id} task ended without sending result"),
            })?;
            agent.apply_step_result(&step_result);
        }

        // Check agent didn't fail
        if agent.status == SubAgentStatus::Failed {
            return Err(AgentError::ValidationError {
                reason: format!("agent {agent_id} has failed and cannot accept input"),
            });
        }

        // Send the command and wait for reply
        let (reply_tx, reply_rx) = oneshot::channel();
        agent
            .command_tx
            .send(AgentCommand::SendInput { message, reply_tx })
            .await
            .map_err(|_| AgentError::Io {
                message: format!("agent {agent_id} task is no longer running"),
            })?;

        let step_result = reply_rx.await.map_err(|_| AgentError::Io {
            message: format!("agent {agent_id} task ended without sending reply"),
        })?;

        agent.apply_step_result(&step_result);

        let error_msg = match step_result {
            AgentStepResult::Completed(_) => None,
            AgentStepResult::Failed(_) => Some("agent failed during send_input"),
        };
        let result = agent
            .cached_result
            .as_ref()
            .expect("just set cached_result above");
        Ok(format_result_json(&agent_id, result, error_msg))
    }

    /// Close all active subagents (spec graceful shutdown, line 1431).
    ///
    /// Called by [`Session::close()`] which is synchronous, so this method
    /// cannot await child task completion. It signals abort on each agent
    /// (children will terminate on their next poll point) and, if a Tokio
    /// runtime is available, spawns a background cleanup task to await the
    /// join handles with a timeout. Without a runtime the handles are simply
    /// dropped — the abort signal alone is sufficient for termination.
    ///
    /// Because this returns before children have fully exited, child tasks
    /// may briefly outlive the parent's CLOSED transition.
    pub fn close_all(&mut self) {
        let mut join_handles = Vec::new();
        for mut handle in self.agents.drain().map(|(_, h)| h) {
            handle.abort_controller.abort();
            // Best-effort close command (task may already be shutting down)
            let _ = handle.command_tx.try_send(AgentCommand::Close);
            if let Some(jh) = handle.join_handle.take() {
                join_handles.push(jh);
            }
        }

        // Spawn a cleanup task that awaits all join handles with a timeout.
        // Use try_current() to avoid panicking when called outside a runtime
        // (e.g. from Drop or synchronous test teardown).
        if !join_handles.is_empty()
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            handle.spawn(async move {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    futures::future::join_all(join_handles),
                )
                .await;
            });
        }
        // Without a runtime: handles are dropped, abort signal ensures
        // children terminate on their next poll.
    }

    /// Terminate a subagent and remove it from the managed set (spec 7.2).
    ///
    /// The handle is removed from the map immediately to prevent a second
    /// `close_agent` call from racing on the same agent. The child task is
    /// then signalled via abort + close command, and we join with a 5-second
    /// timeout. If the join times out the task is orphaned (the abort signal
    /// will still cause it to exit on its next poll point) and the response
    /// reports `"closed": false` so the caller knows termination was not
    /// confirmed. Task panics are also detected and reported as failures.
    async fn close_agent(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?.to_string();

        let handle = self
            .agents
            .remove(&agent_id)
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!("unknown agent_id: {agent_id}"),
            })?;

        // Destructure to avoid partial-move issues with drop(command_tx)
        let SubAgentHandle {
            abort_controller,
            command_tx,
            mut initial_result_rx,
            mut status,
            join_handle,
            ..
        } = handle;

        // Signal abort
        abort_controller.abort();
        // Best-effort close command
        let _ = command_tx.try_send(AgentCommand::Close);
        // Drop the command sender so the task's recv() returns None
        drop(command_tx);

        // Wait for the task to finish with a timeout.
        // Three outcomes: confirmed exit, panic, or timeout.
        let task_confirmed_exited = if let Some(jh) = join_handle {
            match tokio::time::timeout(std::time::Duration::from_secs(5), jh).await {
                Ok(Ok(())) => true,  // task exited normally
                Ok(Err(_)) => {
                    // Task panicked — mark failed, but it *has* exited
                    status = SubAgentStatus::Failed;
                    true
                }
                Err(_) => false, // timeout — task may still be alive
            }
        } else {
            true // no join handle means task already finished
        };

        // Resolve actual status AFTER the task has exited. At this point the
        // oneshot sender has either sent a value or been dropped, so try_recv
        // is deterministic — no race with in-flight completion.
        if let Some(mut rx) = initial_result_rx.take()
            && let Ok(step_result) = rx.try_recv()
        {
            match &step_result {
                AgentStepResult::Completed(_) => status = SubAgentStatus::Completed,
                AgentStepResult::Failed(_) => status = SubAgentStatus::Failed,
            }
        } else if status == SubAgentStatus::Running {
            // No result received and status never updated — mark as failed.
            // This covers: join timeout, task panic, or sender dropped.
            status = SubAgentStatus::Failed;
        }

        let final_status = status;

        Ok(serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": format!("{final_status:?}").to_lowercase(),
            "closed": task_confirmed_exited,
        }))
        .unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// Tool definitions (spec 7.2)
// ---------------------------------------------------------------------------

/// Tool definition for `spawn_agent`.
pub fn spawn_agent_definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_SPAWN_AGENT.into(),
        description: "Spawn a subagent to handle a scoped task autonomously.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Natural language task description for the subagent."
                },
                "working_dir": {
                    "type": "string",
                    "description": "Optional subdirectory to scope the agent to."
                },
                "model": {
                    "type": "string",
                    "description": "Optional model override (default: parent's model)."
                },
                "max_turns": {
                    "type": "integer",
                    "description": "Turn limit for the subagent (default: 50)."
                }
            },
            "required": ["task"]
        }),
        strict: false,
    }
}

/// Tool definition for `send_input`.
pub fn send_input_definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_SEND_INPUT.into(),
        description: "Send a message to a running subagent.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "ID of the subagent to send input to."
                },
                "message": {
                    "type": "string",
                    "description": "Message to send to the subagent."
                }
            },
            "required": ["agent_id", "message"]
        }),
        strict: false,
    }
}

/// Tool definition for `wait`.
pub fn wait_definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_WAIT.into(),
        description: "Wait for a subagent to complete and return its result.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "ID of the subagent to wait for."
                }
            },
            "required": ["agent_id"]
        }),
        strict: false,
    }
}

/// Tool definition for `close_agent`.
pub fn close_agent_definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_CLOSE_AGENT.into(),
        description: "Terminate a subagent.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "ID of the subagent to terminate."
                }
            },
            "required": ["agent_id"]
        }),
        strict: false,
    }
}

/// All four subagent tool definitions.
pub fn subagent_definitions() -> Vec<ToolDefinition> {
    vec![
        spawn_agent_definition(),
        send_input_definition(),
        wait_definition(),
        close_agent_definition(),
    ]
}

// ---------------------------------------------------------------------------
// Tool registration
// ---------------------------------------------------------------------------

/// Register the four subagent tools into a tool registry.
///
/// The executors are no-ops — subagent tool calls are intercepted by the
/// session layer and routed to [`SubAgentManager`] directly. The registry
/// entries exist so that tool definitions are included in LLM requests.
pub fn register_subagent_tools(registry: &mut ToolRegistry) -> AgentResult<()> {
    for def in subagent_definitions() {
        registry.register(RegisteredTool::new(def, noop_executor()))?;
    }
    Ok(())
}

/// No-op executor for subagent tools.
///
/// Subagent tool calls are intercepted by the session before reaching the
/// registry executor. This exists only to satisfy the `RegisteredTool`
/// constructor requirement.
fn noop_executor() -> crate::registry::ToolExecutorFn {
    Box::new(|_args, _env| {
        Box::pin(async {
            Err(AgentError::Io {
                message: "subagent tool should be intercepted by session layer".into(),
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a wait result as JSON.
fn format_wait_result(agent_id: &str, status: SubAgentStatus, result: &SubAgentResult) -> String {
    serde_json::to_string(&json!({
        "agent_id": agent_id,
        "status": format!("{status:?}").to_lowercase(),
        "output": result.output,
        "success": result.success,
        "turns_used": result.turns_used,
    }))
    .unwrap_or_default()
}

/// Format a result JSON for send_input responses.
///
/// On success, includes the agent output, success flag and turns used.
/// On failure, includes the error message.
fn format_result_json(agent_id: &str, result: &SubAgentResult, error: Option<&str>) -> String {
    if let Some(e) = error {
        serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": "failed",
            "error": e,
        }))
        .unwrap_or_default()
    } else {
        serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": "completed",
            "output": result.output,
            "success": result.success,
            "turns_used": result.turns_used,
        }))
        .unwrap_or_default()
    }
}

/// Extract the result from a session (called inside the spawned task).
fn extract_result_from_session(session: &Session) -> SubAgentResult {
    let output = session
        .history()
        .iter()
        .rev()
        .find_map(|turn| match turn {
            crate::types::Turn::Assistant { content, .. } if !content.is_empty() => {
                Some(content.clone())
            }
            _ => None,
        })
        .unwrap_or_default();

    SubAgentResult {
        output,
        success: session.state() != crate::types::SessionState::Closed,
        turns_used: session.total_turns(),
    }
}

/// Create a child profile matching the parent's provider type.
///
/// If `model_override` is `Some`, the child uses that model instead of the
/// parent's. Subagent tool registration is handled by [`Session::new()`]
/// based on the child's depth — this function only creates the base profile.
///
/// Custom tools from the parent are **not** copied into the child. The child
/// only has the base tools for its provider, plus subagent tools if depth
/// permits. Copying tool definitions without their executors would advertise
/// tools that always fail at execution time.
fn create_child_profile(
    parent_profile: &dyn ProviderProfile,
    model_override: Option<&str>,
    max_command_timeout_ms: u64,
) -> AgentResult<Box<dyn ProviderProfile>> {
    let model = model_override.unwrap_or(parent_profile.model());
    let provider_id = parent_profile.id();

    let profile: Box<dyn ProviderProfile> = match provider_id {
        "openai" => Box::new(crate::profiles::OpenAiProfile::new(
            model,
            max_command_timeout_ms,
        )?),
        "anthropic" => Box::new(crate::profiles::AnthropicProfile::new(
            model,
            max_command_timeout_ms,
        )?),
        "gemini" => Box::new(crate::profiles::GeminiProfile::new(
            model,
            max_command_timeout_ms,
        )?),
        _ => {
            tracing::warn!(
                provider_id,
                "unknown provider for subagent — falling back to Anthropic profile"
            );
            Box::new(crate::profiles::AnthropicProfile::new(
                model,
                max_command_timeout_ms,
            )?)
        }
    };

    Ok(profile)
}
