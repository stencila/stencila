use serde::ser::SerializeMap;

/// Errors that can occur during attractor pipeline execution.
///
/// Organized into three categories per Appendix D of the specification:
/// - **Retryable**: transient failures that may succeed on retry
/// - **Terminal**: permanent failures that cannot be retried
/// - **Pipeline**: structural problems with the pipeline definition
///
/// Additionally, wrapper variants for standard library errors.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AttractorError {
    // -- Retryable --
    /// Rate limit exceeded by a provider.
    #[error("rate limited: {message}")]
    RateLimited { message: String },

    /// Network request timed out.
    #[error("network timeout: {message}")]
    NetworkTimeout { message: String },

    /// Service temporarily unavailable.
    #[error("temporarily unavailable: {message}")]
    TemporaryUnavailable { message: String },

    // -- Terminal --
    /// The prompt supplied to a stage handler is invalid.
    #[error("invalid prompt: {message}")]
    InvalidPrompt { message: String },

    /// A required context key is missing.
    #[error("missing context key: {key}")]
    MissingContext { key: String },

    /// Authentication with a provider failed.
    #[error("authentication failed: {message}")]
    AuthenticationFailed { message: String },

    /// A stage handler returned an error.
    #[error("handler failed for node {node_id}: {reason}")]
    HandlerFailed { node_id: String, reason: String },

    // -- Pipeline --
    /// The pipeline graph has no start node.
    #[error("pipeline has no start node")]
    NoStartNode,

    /// The pipeline graph has no exit node.
    #[error("pipeline has no exit node")]
    NoExitNode,

    /// A node in the pipeline is unreachable from the start node.
    #[error("unreachable node: {node_id}")]
    UnreachableNode { node_id: String },

    /// A transition condition is invalid.
    #[error("invalid condition `{condition}`: {reason}")]
    InvalidCondition { condition: String, reason: String },

    /// A referenced node does not exist in the pipeline.
    #[error("node not found: {node_id}")]
    NodeNotFound { node_id: String },

    /// The pipeline definition is invalid.
    #[error("invalid pipeline: {reason}")]
    InvalidPipeline { reason: String },

    // -- Wrappers --
    /// An I/O error occurred.
    #[error("io error: {message}")]
    Io { message: String },

    /// A JSON serialization/deserialization error occurred.
    #[error("json error: {message}")]
    Json { message: String },
}

impl AttractorError {
    /// Whether the error is retryable (transient).
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. }
            | Self::NetworkTimeout { .. }
            | Self::TemporaryUnavailable { .. }
            | Self::Io { .. } => true,

            Self::InvalidPrompt { .. }
            | Self::MissingContext { .. }
            | Self::AuthenticationFailed { .. }
            | Self::HandlerFailed { .. }
            | Self::Json { .. }
            | Self::NoStartNode
            | Self::NoExitNode
            | Self::UnreachableNode { .. }
            | Self::InvalidCondition { .. }
            | Self::NodeNotFound { .. }
            | Self::InvalidPipeline { .. } => false,
        }
    }

    /// Whether the error is terminal (permanent).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::InvalidPrompt { .. }
            | Self::MissingContext { .. }
            | Self::AuthenticationFailed { .. }
            | Self::HandlerFailed { .. }
            | Self::Json { .. } => true,

            Self::RateLimited { .. }
            | Self::NetworkTimeout { .. }
            | Self::TemporaryUnavailable { .. }
            | Self::Io { .. }
            | Self::NoStartNode
            | Self::NoExitNode
            | Self::UnreachableNode { .. }
            | Self::InvalidCondition { .. }
            | Self::NodeNotFound { .. }
            | Self::InvalidPipeline { .. } => false,
        }
    }

    /// Whether the error is a pipeline structural error.
    #[must_use]
    pub fn is_pipeline(&self) -> bool {
        match self {
            Self::NoStartNode
            | Self::NoExitNode
            | Self::UnreachableNode { .. }
            | Self::InvalidCondition { .. }
            | Self::NodeNotFound { .. }
            | Self::InvalidPipeline { .. } => true,

            Self::RateLimited { .. }
            | Self::NetworkTimeout { .. }
            | Self::TemporaryUnavailable { .. }
            | Self::InvalidPrompt { .. }
            | Self::MissingContext { .. }
            | Self::AuthenticationFailed { .. }
            | Self::HandlerFailed { .. }
            | Self::Io { .. }
            | Self::Json { .. } => false,
        }
    }

    /// A unique error code string for this variant.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::NetworkTimeout { .. } => "NETWORK_TIMEOUT",
            Self::TemporaryUnavailable { .. } => "TEMPORARY_UNAVAILABLE",
            Self::InvalidPrompt { .. } => "INVALID_PROMPT",
            Self::MissingContext { .. } => "MISSING_CONTEXT",
            Self::AuthenticationFailed { .. } => "AUTHENTICATION_FAILED",
            Self::HandlerFailed { .. } => "HANDLER_FAILED",
            Self::NoStartNode => "NO_START_NODE",
            Self::NoExitNode => "NO_EXIT_NODE",
            Self::UnreachableNode { .. } => "UNREACHABLE_NODE",
            Self::InvalidCondition { .. } => "INVALID_CONDITION",
            Self::NodeNotFound { .. } => "NODE_NOT_FOUND",
            Self::InvalidPipeline { .. } => "INVALID_PIPELINE",
            Self::Io { .. } => "IO_ERROR",
            Self::Json { .. } => "JSON_ERROR",
        }
    }
}

impl From<std::io::Error> for AttractorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for AttractorError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json {
            message: err.to_string(),
        }
    }
}

impl serde::Serialize for AttractorError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("code", self.code())?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}

/// A `Result` type alias using [`AttractorError`].
pub type AttractorResult<T> = Result<T, AttractorError>;
