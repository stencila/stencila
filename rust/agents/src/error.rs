use std::fmt;

use stencila_models3::error::SdkError;

/// The result type for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// Agent-level error hierarchy.
///
/// Separates tool-level errors (recoverable — sent back to the LLM as error
/// results so the model can adapt) from session-level errors (affect the
/// session lifecycle).
///
/// All match arms in classification methods are exhaustive (no wildcards)
/// so that adding a new variant forces a compile-time decision.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AgentError {
    // --- Tool-level errors (Appendix B) ---
    /// File does not exist at the given path.
    #[error("file not found: {path}")]
    FileNotFound { path: String },

    /// `old_string` was not found or not unique in the target file.
    #[error("edit conflict: {reason}")]
    EditConflict { reason: String },

    /// Shell command exceeded its timeout.
    #[error("shell timeout after {timeout_ms}ms")]
    ShellTimeout { timeout_ms: u64 },

    /// Shell command exited with a nonzero code.
    #[error("shell exit code {code}: {stderr}")]
    ShellExitError { code: i32, stderr: String },

    /// Operation not permitted (e.g., write to protected path).
    #[error("permission denied: {path}")]
    PermissionDenied { path: String },

    /// Tool arguments failed JSON schema validation.
    #[error("validation error: {reason}")]
    ValidationError { reason: String },

    /// Model called a tool that is not in the registry.
    #[error("unknown tool: {name}")]
    UnknownTool { name: String },

    /// General I/O error during tool execution (filesystem, process, etc.).
    #[error("i/o error: {message}")]
    Io { message: String },

    /// An MCP server error during tool execution.
    #[error("mcp error: {message}")]
    Mcp { message: String },

    // --- Session-level errors (Appendix B) ---
    /// The session has been closed and cannot accept further input.
    #[error("session closed")]
    SessionClosed,

    /// The session is not in the expected state for the requested operation.
    #[error("invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    /// Turn or tool-round limit has been exceeded.
    #[error("turn limit exceeded: {message}")]
    TurnLimitExceeded { message: String },

    /// The conversation exceeded the model's context window.
    #[error("context length exceeded: {message}")]
    ContextLengthExceeded { message: String },

    // --- Wrapper ---
    /// An error from the underlying LLM SDK.
    #[error("sdk error: {0}")]
    Sdk(SdkError),
}

impl AgentError {
    /// Whether this error is a tool-level error that should be sent back
    /// to the LLM as an error result (giving the model a chance to recover).
    ///
    /// All arms are listed explicitly — no wildcards — so adding a new
    /// variant produces a compile error until classified.
    #[must_use]
    pub fn is_tool_error(&self) -> bool {
        match self {
            Self::FileNotFound { .. }
            | Self::EditConflict { .. }
            | Self::ShellTimeout { .. }
            | Self::ShellExitError { .. }
            | Self::PermissionDenied { .. }
            | Self::ValidationError { .. }
            | Self::UnknownTool { .. }
            | Self::Io { .. }
            | Self::Mcp { .. } => true,

            Self::SessionClosed
            | Self::InvalidState { .. }
            | Self::TurnLimitExceeded { .. }
            | Self::ContextLengthExceeded { .. }
            | Self::Sdk(_) => false,
        }
    }

    /// Whether this error is a session-level error that affects the
    /// session lifecycle.
    ///
    /// Per spec 2.8 and Appendix B, session-level errors include:
    /// - Agent-native variants: SessionClosed, TurnLimitExceeded,
    ///   ContextLengthExceeded.
    /// - Non-retryable SDK errors: Authentication, AccessDenied,
    ///   ContextLength, QuotaExceeded, ContentFilter, InvalidRequest,
    ///   Configuration, Abort, etc.
    ///
    /// Delegates to [`SdkError::is_retryable()`] as the single source of
    /// truth — any non-retryable SDK error is session-level. Retryable
    /// SDK errors (RateLimit, Server, Network, etc.) are already handled
    /// by the SDK's retry layer; if they still reach the agent, Phase 8
    /// loop code treats them as unrecoverable too, but `is_session_error()`
    /// returns `false` so the loop can distinguish and log differently.
    ///
    /// All arms are listed explicitly — no wildcards — so adding a new
    /// variant produces a compile error until classified.
    #[must_use]
    pub fn is_session_error(&self) -> bool {
        match self {
            Self::SessionClosed
            | Self::InvalidState { .. }
            | Self::TurnLimitExceeded { .. }
            | Self::ContextLengthExceeded { .. } => true,

            Self::Sdk(sdk_err) => !sdk_err.is_retryable(),

            Self::FileNotFound { .. }
            | Self::EditConflict { .. }
            | Self::ShellTimeout { .. }
            | Self::ShellExitError { .. }
            | Self::PermissionDenied { .. }
            | Self::ValidationError { .. }
            | Self::UnknownTool { .. }
            | Self::Io { .. }
            | Self::Mcp { .. } => false,
        }
    }

    /// A short error code suitable for logging or event payloads.
    ///
    /// All arms are listed explicitly — no wildcards — so adding a new
    /// variant produces a compile error until classified.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "FILE_NOT_FOUND",
            Self::EditConflict { .. } => "EDIT_CONFLICT",
            Self::ShellTimeout { .. } => "SHELL_TIMEOUT",
            Self::ShellExitError { .. } => "SHELL_EXIT_ERROR",
            Self::PermissionDenied { .. } => "PERMISSION_DENIED",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::UnknownTool { .. } => "UNKNOWN_TOOL",
            Self::Io { .. } => "IO_ERROR",
            Self::Mcp { .. } => "MCP_ERROR",
            Self::SessionClosed => "SESSION_CLOSED",
            Self::InvalidState { .. } => "INVALID_STATE",
            Self::TurnLimitExceeded { .. } => "TURN_LIMIT_EXCEEDED",
            Self::ContextLengthExceeded { .. } => "CONTEXT_LENGTH_EXCEEDED",
            Self::Sdk(_) => "SDK_ERROR",
        }
    }
}

impl AgentError {
    /// Convert an `io::Error` into the most specific `AgentError` variant,
    /// using `path` for context in the error message.
    pub fn from_io(err: std::io::Error, path: &std::path::Path) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound {
                path: path.display().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                path: path.display().to_string(),
            },
            _ => Self::Io {
                message: format!("{}: {err}", path.display()),
            },
        }
    }
}

impl From<SdkError> for AgentError {
    fn from(err: SdkError) -> Self {
        Self::Sdk(err)
    }
}

#[cfg(any(feature = "mcp", feature = "codemode"))]
impl From<stencila_mcp::McpError> for AgentError {
    fn from(err: stencila_mcp::McpError) -> Self {
        Self::Mcp {
            message: err.to_string(),
        }
    }
}

/// Serialize `AgentError` as a JSON object with `code` and `message` fields.
impl serde::Serialize for AgentError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AgentError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &fmt::format(format_args!("{self}")))?;
        s.end()
    }
}
