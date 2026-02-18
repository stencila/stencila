use serde::{Deserialize, Serialize};
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::tool::ToolCall;
use stencila_models3::types::tool::ToolResult;
use stencila_models3::types::usage::Usage;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the current UTC time as an ISO 8601 string.
#[must_use]
pub fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// ReasoningEffort (spec 2.2)
// ---------------------------------------------------------------------------

/// Reasoning effort level for the model.
///
/// Spec 2.2/2.7 defines valid values as `"low"`, `"medium"`, `"high"`, or null.
///
/// **Intentional extension:** The `Custom` variant accepts arbitrary strings
/// for forward-compatibility with provider-specific effort levels that may be
/// added after this spec was written. Note that providers may reject unknown
/// values at the API boundary — callers should prefer the named variants for
/// portable behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    /// A provider-specific effort level not covered by spec 2.2/2.7.
    ///
    /// Providers may reject unknown values. Prefer the named variants
    /// for portable behavior.
    #[serde(untagged)]
    Custom(String),
}

impl ReasoningEffort {
    /// Return the string representation for use in LLM requests.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Session Configuration (spec 2.2)
// ---------------------------------------------------------------------------

/// Configuration for an agent session.
///
/// All fields carry the spec-mandated defaults (Section 2.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum total turns (0 = unlimited).
    #[serde(default)]
    pub max_turns: u32,

    /// Maximum tool rounds per user input.
    #[serde(default = "default_max_tool_rounds")]
    pub max_tool_rounds_per_input: u32,

    /// Default command timeout in milliseconds (10 seconds).
    #[serde(default = "default_command_timeout_ms")]
    pub default_command_timeout_ms: u64,

    /// Maximum command timeout in milliseconds (10 minutes).
    #[serde(default = "default_max_command_timeout_ms")]
    pub max_command_timeout_ms: u64,

    /// Reasoning effort level: Low, Medium, High, or None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Per-tool character output limits (overrides defaults from spec 5.2).
    #[serde(default)]
    pub tool_output_limits: std::collections::HashMap<String, usize>,

    /// Per-tool line output limits (overrides defaults from spec 5.2).
    #[serde(default)]
    pub tool_line_limits: std::collections::HashMap<String, usize>,

    /// Whether to discover and enable workspace skills.
    ///
    /// When enabled and the `skills` feature is active, the system prompt
    /// includes compact skill metadata and a `use_skill` tool is registered
    /// for on-demand full-content loading.
    #[serde(default = "default_true")]
    pub enable_skills: bool,

    /// Whether to register MCP tools directly in the agent's tool registry.
    ///
    /// When enabled and the `mcp` feature is active, each MCP tool is
    /// registered as an individual tool that the LLM can call directly.
    /// Simple for LLMs but can overwhelm with many tools.
    #[serde(default)]
    pub enable_mcp: bool,

    /// Whether to register a single `codemode` tool for MCP orchestration.
    ///
    /// When enabled and the `codemode` feature is active, a single `codemode`
    /// tool is registered that lets the LLM write JavaScript to orchestrate
    /// MCP calls in a sandboxed environment. TypeScript declarations are
    /// included in the system prompt.
    ///
    /// Defaults to `false` — callers must explicitly opt in to MCP
    /// discovery and server connections.
    #[serde(default)]
    pub enable_codemode: bool,

    /// Whether loop detection is enabled.
    #[serde(default = "default_true")]
    pub enable_loop_detection: bool,

    /// Number of consecutive identical tool calls before a loop warning.
    #[serde(default = "default_loop_detection_window")]
    pub loop_detection_window: u32,

    /// Maximum nesting level for subagents.
    #[serde(default = "default_max_subagent_depth")]
    pub max_subagent_depth: u32,

    /// Commit instructions appended to the system prompt.
    ///
    /// Populated by [`crate::convenience::create_session()`] with instructions
    /// to set the git committer identity. Inherited by subagents via
    /// [`for_child()`](Self::for_child).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_instructions: Option<String>,

    /// User instruction override text (spec layer 5, appended last to system prompt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_instructions: Option<String>,

    /// Whether to auto-detect when the model is asking the user a question
    /// and transition to `AwaitingInput` instead of `Idle` (spec 2.3).
    #[serde(default = "default_true")]
    pub auto_detect_awaiting_input: bool,
}

fn default_max_tool_rounds() -> u32 {
    0
}
fn default_command_timeout_ms() -> u64 {
    10_000
}
fn default_max_command_timeout_ms() -> u64 {
    600_000
}
fn default_true() -> bool {
    true
}
fn default_loop_detection_window() -> u32 {
    10
}
fn default_max_subagent_depth() -> u32 {
    1
}

impl SessionConfig {
    /// Derive a child configuration for a subagent session.
    ///
    /// Inherits the parent's behavioral settings (timeouts, limits, loop
    /// detection, skills) while allowing per-child overrides for `max_turns`
    /// and `max_subagent_depth`. Fields that are session-specific
    /// (`user_instructions`, `reasoning_effort`) are not inherited.
    ///
    /// Using this method instead of `SessionConfig { ..Default::default() }`
    /// ensures newly added fields are inherited correctly without manual
    /// updates in every call site.
    #[must_use]
    pub fn for_child(&self, max_turns: u32, max_subagent_depth: u32) -> Self {
        Self {
            max_turns,
            max_subagent_depth,
            // Inherit parent's behavioral settings
            max_tool_rounds_per_input: self.max_tool_rounds_per_input,
            default_command_timeout_ms: self.default_command_timeout_ms,
            max_command_timeout_ms: self.max_command_timeout_ms,
            tool_output_limits: self.tool_output_limits.clone(),
            tool_line_limits: self.tool_line_limits.clone(),
            enable_loop_detection: self.enable_loop_detection,
            loop_detection_window: self.loop_detection_window,
            auto_detect_awaiting_input: self.auto_detect_awaiting_input,
            enable_skills: self.enable_skills,
            enable_mcp: self.enable_mcp,
            enable_codemode: self.enable_codemode,
            // Inherited: subagents use the same commit trailer
            commit_instructions: self.commit_instructions.clone(),
            // Session-specific: not inherited
            reasoning_effort: None,
            user_instructions: None,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_turns: 0,
            max_tool_rounds_per_input: default_max_tool_rounds(),
            default_command_timeout_ms: default_command_timeout_ms(),
            max_command_timeout_ms: default_max_command_timeout_ms(),
            reasoning_effort: None,
            tool_output_limits: std::collections::HashMap::new(),
            tool_line_limits: std::collections::HashMap::new(),
            enable_loop_detection: true,
            loop_detection_window: default_loop_detection_window(),
            max_subagent_depth: default_max_subagent_depth(),
            commit_instructions: None,
            user_instructions: None,
            auto_detect_awaiting_input: true,
            enable_skills: true,
            enable_mcp: false,
            enable_codemode: false,
        }
    }
}

impl SessionConfig {
    /// Build a [`SessionConfig`] from an agent definition.
    ///
    /// Maps the agent's schema fields (reasoning effort, max turns, tool
    /// timeout, etc.) onto session config values and reads the AGENT.md
    /// body as `user_instructions`.
    ///
    /// # Errors
    ///
    /// Returns an error if the AGENT.md file cannot be read.
    pub async fn from_agent(agent: &crate::agent_def::AgentInstance) -> eyre::Result<Self> {
        let user_instructions = agent.instructions().await?;

        let mut config = SessionConfig {
            user_instructions,
            ..Default::default()
        };

        if let Some(effort) = &agent.reasoning_effort {
            config.reasoning_effort = Some(match effort.as_str() {
                "low" => ReasoningEffort::Low,
                "medium" => ReasoningEffort::Medium,
                "high" => ReasoningEffort::High,
                other => ReasoningEffort::Custom(other.to_string()),
            });
        }

        if let Some(val) = agent.options.max_turns
            && val >= 0
        {
            config.max_turns = val as u32;
        }

        if let Some(val) = agent.options.max_tool_rounds
            && val >= 0
        {
            config.max_tool_rounds_per_input = val as u32;
        }

        if let Some(val) = agent.options.tool_timeout
            && val > 0
        {
            // Agent specifies timeout in seconds, config uses milliseconds
            config.default_command_timeout_ms = (val as u64).saturating_mul(1000);
        }

        if let Some(val) = agent.options.max_subagent_depth
            && val >= 0
        {
            config.max_subagent_depth = val as u32;
        }

        if let Some(val) = agent.options.enable_mcp {
            config.enable_mcp = val;
        }

        if let Some(val) = agent.options.enable_codemode {
            config.enable_codemode = val;
        }

        Ok(config)
    }
}

// ---------------------------------------------------------------------------
// Session State (spec 2.3)
// ---------------------------------------------------------------------------

/// Lifecycle state of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    /// Waiting for user input.
    Idle,
    /// Running the agentic loop.
    Processing,
    /// Model asked the user a question.
    AwaitingInput,
    /// Session terminated (normal or error).
    Closed,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---------------------------------------------------------------------------
// Turn Types (spec 2.4)
// ---------------------------------------------------------------------------

/// A single entry in the conversation history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Turn {
    /// User submitted input.
    User { content: String, timestamp: String },

    /// Model produced text and/or tool calls.
    Assistant {
        content: String,
        #[serde(default)]
        tool_calls: Vec<ToolCall>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        thinking_parts: Vec<ContentPart>,
        #[serde(default)]
        usage: Usage,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_id: Option<String>,
        timestamp: String,
    },

    /// Results of tool executions.
    ToolResults {
        results: Vec<ToolResult>,
        timestamp: String,
    },

    /// System message.
    System { content: String, timestamp: String },

    /// Steering message injected between tool rounds.
    Steering { content: String, timestamp: String },
}

impl Turn {
    /// Create a user turn with the current timestamp.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: content.into(),
            timestamp: now_timestamp(),
        }
    }

    /// Create an assistant turn with just text and the current timestamp.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant {
            content: content.into(),
            tool_calls: Vec::new(),
            reasoning: None,
            thinking_parts: Vec::new(),
            usage: Usage::default(),
            response_id: None,
            timestamp: now_timestamp(),
        }
    }

    /// Create a tool results turn with the current timestamp.
    #[must_use]
    pub fn tool_results(results: Vec<ToolResult>) -> Self {
        Self::ToolResults {
            results,
            timestamp: now_timestamp(),
        }
    }

    /// Create a system turn with the current timestamp.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
            timestamp: now_timestamp(),
        }
    }

    /// Create a steering turn with the current timestamp.
    #[must_use]
    pub fn steering(content: impl Into<String>) -> Self {
        Self::Steering {
            content: content.into(),
            timestamp: now_timestamp(),
        }
    }

    /// Return the timestamp of this turn.
    #[must_use]
    pub fn timestamp(&self) -> &str {
        match self {
            Self::User { timestamp, .. }
            | Self::Assistant { timestamp, .. }
            | Self::ToolResults { timestamp, .. }
            | Self::System { timestamp, .. }
            | Self::Steering { timestamp, .. } => timestamp,
        }
    }
}

// ---------------------------------------------------------------------------
// ExecResult (spec 4.1)
// ---------------------------------------------------------------------------

/// Result of executing a shell command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecResult {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Whether the command was killed due to timeout.
    pub timed_out: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// DirEntry (spec 4.1)
// ---------------------------------------------------------------------------

/// An entry in a directory listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirEntry {
    /// File or directory name.
    pub name: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (None for directories or if unavailable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

// ---------------------------------------------------------------------------
// GrepOptions (spec 3.3, grep tool parameters)
// ---------------------------------------------------------------------------

/// Options for the grep search operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepOptions {
    /// File pattern filter (e.g., "*.py").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glob_filter: Option<String>,
    /// Case-insensitive matching.
    #[serde(default)]
    pub case_insensitive: bool,
    /// Maximum number of results (default: 100).
    #[serde(default = "default_max_grep_results")]
    pub max_results: u32,
}

fn default_max_grep_results() -> u32 {
    100
}

impl Default for GrepOptions {
    fn default() -> Self {
        Self {
            glob_filter: None,
            case_insensitive: false,
            max_results: default_max_grep_results(),
        }
    }
}

// ---------------------------------------------------------------------------
// Event System (spec 2.9)
// ---------------------------------------------------------------------------

/// The kind of event emitted by the agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventKind {
    /// Session created.
    SessionStart,
    /// Session closed (includes final state).
    SessionEnd,
    /// User submitted input.
    UserInput,
    /// Model began generating text.
    AssistantTextStart,
    /// Incremental text token.
    AssistantTextDelta,
    /// Model finished text (includes full text).
    AssistantTextEnd,
    /// Model began reasoning/thinking.
    AssistantReasoningStart,
    /// Incremental reasoning/thinking token.
    AssistantReasoningDelta,
    /// Model finished reasoning/thinking.
    AssistantReasoningEnd,
    /// Tool execution began.
    ToolCallStart,
    /// Incremental tool output (for streaming tools).
    ToolCallOutputDelta,
    /// Tool execution finished (includes FULL untruncated output).
    ToolCallEnd,
    /// A steering message was added to history.
    SteeringInjected,
    /// A turn limit was hit.
    TurnLimit,
    /// A loop pattern was detected.
    LoopDetection,
    /// Context usage update (approximate percentage of context window used).
    ContextUsage,
    /// An error occurred.
    Error,
}

/// A typed event emitted by the agent session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEvent {
    /// The kind of event.
    pub kind: EventKind,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// The session that emitted this event.
    pub session_id: String,
    /// Event-specific data.
    #[serde(default)]
    pub data: serde_json::Map<String, serde_json::Value>,
}
