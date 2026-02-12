use std::fmt;

/// Errors that can occur in the MCP crate.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// A configuration file could not be read or parsed.
    #[error("config error: {0}")]
    Config(String),

    /// The transport layer encountered an error.
    #[error("transport error for server `{server_id}`: {message}")]
    Transport { server_id: String, message: String },

    /// The MCP protocol exchange was invalid or unexpected.
    #[error("protocol error for server `{server_id}`: {message}")]
    Protocol { server_id: String, message: String },

    /// The requested server was not found in the pool or config.
    #[error("server `{server_id}` not found")]
    ServerNotFound { server_id: String },

    /// A connection attempt to a server failed.
    #[error("connection failed for server `{server_id}`: {message}")]
    ConnectionFailed { server_id: String, message: String },

    /// The MCP server rejected the call due to invalid or missing credentials.
    #[error("authentication error for server `{server_id}`: {message}")]
    Authentication { server_id: String, message: String },

    /// A request to a server timed out.
    #[error("server `{server_id}` timed out after {timeout}")]
    Timeout {
        server_id: String,
        timeout: PrettyDuration,
    },

    /// An I/O error occurred (e.g. reading a config file).
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization/deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// A TOML deserialization error.
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// A [`std::time::Duration`] wrapper that displays in a human-friendly format.
#[derive(Debug, Clone, Copy)]
pub struct PrettyDuration(pub std::time::Duration);

impl fmt::Display for PrettyDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();
        let millis = self.0.subsec_millis();
        if secs > 0 {
            write!(f, "{secs}.{millis:03}s")
        } else {
            write!(f, "{millis}ms")
        }
    }
}

/// Convenience type alias for results in this crate.
pub type McpResult<T> = Result<T, McpError>;
