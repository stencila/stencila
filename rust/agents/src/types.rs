use serde::{Deserialize, Serialize};
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

    /// Whether loop detection is enabled.
    #[serde(default = "default_true")]
    pub enable_loop_detection: bool,

    /// Number of consecutive identical tool calls before a loop warning.
    #[serde(default = "default_loop_detection_window")]
    pub loop_detection_window: u32,

    /// Maximum nesting level for subagents.
    #[serde(default = "default_max_subagent_depth")]
    pub max_subagent_depth: u32,

    /// User instruction override text (spec layer 5, appended last to system prompt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_instructions: Option<String>,
}

fn default_max_tool_rounds() -> u32 {
    200
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
            user_instructions: None,
        }
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
