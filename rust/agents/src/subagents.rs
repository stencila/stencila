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
//! # Known limitation: synchronous execution
//!
//! The current implementation runs subagent sessions synchronously —
//! `spawn_agent` blocks until the child session completes before returning
//! a result. This means `wait` is effectively a no-op (the agent is always
//! already finished) and true parallel exploration (spec 7.4) is not yet
//! supported. A future iteration will use `tokio::spawn` to run child
//! sessions in background tasks, enabling concurrent subagent execution.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::events;
use crate::execution::ExecutionEnvironment;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::session::{LlmClient, Session};
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

/// A handle to a running subagent (spec 7.3).
pub struct SubAgentHandle {
    /// Unique identifier for this subagent.
    pub id: String,
    /// The subagent's independent session.
    session: Session,
    /// Current status.
    status: SubAgentStatus,
    /// Event receiver (kept alive so events are not lost).
    _receiver: events::EventReceiver,
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
    /// Currently runs the child session to completion synchronously (see
    /// module-level known limitation docs). Returns the full result
    /// including output text and turns used.
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

        let task = require_str(&args, "task")?;
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
        let child_profile = create_child_profile(parent_profile, child_model)?;

        // Build system prompt for child, including working_dir scope if specified
        let mut system_prompt =
            crate::prompts::build_system_prompt(&*child_profile, &*self.execution_env).await?;
        if let Some(ref dir) = working_dir {
            system_prompt.push_str(&format!(
                "\n\nYou are scoped to the subdirectory: {dir}\n\
                 Focus your work within this directory."
            ));
        }

        // Create the child session
        let (session, receiver) = Session::new(
            child_profile,
            Arc::clone(&self.execution_env),
            Arc::clone(&self.client),
            child_config,
            system_prompt,
            self.current_depth + 1,
        );

        // Generate agent ID
        self.next_id += 1;
        let agent_id = format!("agent-{}", self.next_id);

        let handle = SubAgentHandle {
            id: agent_id.clone(),
            session,
            status: SubAgentStatus::Running,
            _receiver: receiver,
        };

        self.agents.insert(agent_id.clone(), handle);

        // Submit the task to the child session
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or(AgentError::SessionClosed)?;
        let submit_result = agent.session.submit(task).await;

        match submit_result {
            Ok(()) => {
                agent.status = SubAgentStatus::Completed;
                Ok(build_result_json(&agent_id, agent, None))
            }
            Err(e) => {
                agent.status = SubAgentStatus::Failed;
                Ok(build_result_json(&agent_id, agent, Some(&e)))
            }
        }
    }

    /// Send a message to a running subagent (spec 7.2).
    ///
    /// # Synchronous-model deviation
    ///
    /// The spec says "running subagent" but in the current synchronous model
    /// agents are always `Completed` after spawn (never truly "running").
    /// We therefore accept any non-`Failed` agent so that the LLM can send
    /// follow-up messages to a completed agent. Once async spawn is
    /// implemented, this should be tightened to `Running` only.
    async fn send_input(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?;
        let message = require_str(&args, "message")?;

        let agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!("unknown agent_id: {agent_id}"),
            })?;

        if agent.status == SubAgentStatus::Failed {
            return Err(AgentError::ValidationError {
                reason: format!("agent {agent_id} has failed and cannot accept input"),
            });
        }

        // Re-submit to the child session (it should be in Idle state)
        agent.status = SubAgentStatus::Running;
        let submit_result = agent.session.submit(message).await;

        match submit_result {
            Ok(()) => {
                agent.status = SubAgentStatus::Completed;
                Ok(build_result_json(agent_id, agent, None))
            }
            Err(e) => {
                agent.status = SubAgentStatus::Failed;
                Ok(build_result_json(agent_id, agent, Some(&e)))
            }
        }
    }

    /// Wait for a subagent to complete and return its result (spec 7.2).
    ///
    /// Since `spawn_agent` and `send_input` already run the session to
    /// completion synchronously, `wait` simply returns the current result.
    async fn wait_agent(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?;

        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| AgentError::ValidationError {
                reason: format!("unknown agent_id: {agent_id}"),
            })?;

        let result = extract_result(agent);
        Ok(serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": format!("{:?}", agent.status).to_lowercase(),
            "output": result.output,
            "success": result.success,
            "turns_used": result.turns_used,
        }))
        .unwrap_or_default())
    }

    /// Close all active subagents (spec graceful shutdown, line 1431).
    ///
    /// Called by [`Session::close()`] to ensure child sessions are terminated
    /// before the parent session transitions to CLOSED.
    pub fn close_all(&mut self) {
        for handle in self.agents.values_mut() {
            handle.session.close();
        }
        self.agents.clear();
    }

    /// Terminate a subagent and remove it from the managed set (spec 7.2).
    async fn close_agent(&mut self, args: Value) -> AgentResult<String> {
        let agent_id = require_str(&args, "agent_id")?.to_string();

        let mut handle =
            self.agents
                .remove(&agent_id)
                .ok_or_else(|| AgentError::ValidationError {
                    reason: format!("unknown agent_id: {agent_id}"),
                })?;

        handle.session.close();
        let final_status = handle.status;

        Ok(serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": format!("{final_status:?}").to_lowercase(),
            "closed": true,
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

/// Build the JSON response string for a spawn/send_input result.
///
/// On success, includes the agent output, success flag and turns used.
/// On failure, includes the error message. Both cases include agent_id
/// and status.
fn build_result_json(
    agent_id: &str,
    handle: &SubAgentHandle,
    error: Option<&AgentError>,
) -> String {
    if let Some(e) = error {
        serde_json::to_string(&json!({
            "agent_id": agent_id,
            "status": "failed",
            "error": e.to_string(),
        }))
        .unwrap_or_default()
    } else {
        let result = extract_result(handle);
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

/// Extract the result from a subagent handle.
fn extract_result(handle: &SubAgentHandle) -> SubAgentResult {
    let output = handle
        .session
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
        success: handle.status == SubAgentStatus::Completed,
        turns_used: handle.session.total_turns(),
    }
}

/// Create a child profile matching the parent's provider type.
///
/// If `model_override` is `Some`, the child uses that model instead of the
/// parent's. Subagent tool registration is handled by [`Session::new()`]
/// based on the child's depth — this function only creates the base profile.
///
/// Any custom tools registered on the parent profile (beyond the base set)
/// are copied into the child as passive definitions so the LLM sees the
/// same tool surface. Tool executors are not copied — custom tools on the
/// child will be routed through the child's own registry.
fn create_child_profile(
    parent_profile: &dyn ProviderProfile,
    model_override: Option<&str>,
) -> AgentResult<Box<dyn ProviderProfile>> {
    let model = model_override.unwrap_or(parent_profile.model());
    let provider_id = parent_profile.id();

    let mut profile: Box<dyn ProviderProfile> = match provider_id {
        "openai" => Box::new(crate::profiles::OpenAiProfile::new(model)?),
        "anthropic" => Box::new(crate::profiles::AnthropicProfile::new(model)?),
        "gemini" => Box::new(crate::profiles::GeminiProfile::new(model)?),
        _ => {
            tracing::warn!(
                provider_id,
                "unknown provider for subagent — falling back to Anthropic profile"
            );
            Box::new(crate::profiles::AnthropicProfile::new(model)?)
        }
    };

    // Copy parent's custom tool definitions that aren't in the child's base set.
    // Uses no-op executors since tool calls route through the child's own session.
    let child_names: Vec<String> = profile
        .tool_registry()
        .names()
        .iter()
        .map(|n| n.to_string())
        .collect();
    for def in parent_profile.tool_registry().definitions() {
        if !child_names.iter().any(|n| n == &def.name)
            && !SUBAGENT_TOOL_NAMES.contains(&def.name.as_str())
        {
            profile
                .tool_registry_mut()
                .register(RegisteredTool::new(def, noop_executor()))?;
        }
    }

    Ok(profile)
}
