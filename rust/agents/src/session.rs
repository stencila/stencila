//! Unified agent session type that wraps both API and CLI sessions.
//!
//! [`AgentSession`] is an enum with two variants — [`Api`](AgentSession::Api)
//! for standard model-API sessions and [`Cli`](AgentSession::Cli) for CLI-tool
//! backed sessions. Common operations (`submit`, `close`, `set_abort_signal`,
//! etc.) are delegated through match arms so callers don't need to distinguish
//! between the two.

use crate::api_session::ApiSession;
use crate::cli_providers::CliSession;
use crate::error::AgentResult;
use crate::types::{AbortSignal, SessionConfig, SessionState, Turn};

/// Unified agent session that routes to either an API or CLI backend.
///
/// Returned by [`create_session`](crate::convenience::create_session). Callers
/// can use the delegated methods without knowing which backend is active, or
/// match on the variant when they need backend-specific features (e.g.
/// [`Session::steer`], [`Session::mcp_pool`]).
pub enum AgentSession {
    /// Standard API-based session using the models3 HTTP client.
    Api(ApiSession),
    /// CLI-tool backed session (Claude CLI, Codex CLI, Gemini CLI).
    Cli(CliSession),
}

impl AgentSession {
    /// Submit user input to the session.
    ///
    /// Delegates to [`Session::submit`] or [`CliSession::submit`].
    pub async fn submit(&mut self, input: &str) -> AgentResult<()> {
        match self {
            Self::Api(s) => s.submit(input).await,
            Self::Cli(s) => s.submit(input).await,
        }
    }

    /// Close the session, emitting `SESSION_END`.
    pub fn close(&mut self) {
        match self {
            Self::Api(s) => s.close(),
            Self::Cli(s) => s.close(),
        }
    }

    /// Set an abort signal for cancellation.
    pub fn set_abort_signal(&mut self, signal: AbortSignal) {
        match self {
            Self::Api(s) => s.set_abort_signal(signal),
            Self::Cli(s) => s.set_abort_signal(signal),
        }
    }

    /// Current session state.
    #[must_use]
    pub fn state(&self) -> SessionState {
        match self {
            Self::Api(s) => s.state(),
            Self::Cli(s) => s.state(),
        }
    }

    /// Full conversation history.
    #[must_use]
    pub fn history(&self) -> &[Turn] {
        match self {
            Self::Api(s) => s.history(),
            Self::Cli(s) => s.history(),
        }
    }

    /// Session configuration.
    #[must_use]
    pub fn config(&self) -> &SessionConfig {
        match self {
            Self::Api(s) => s.config(),
            Self::Cli(s) => s.config(),
        }
    }

    /// The session ID from the event emitter.
    #[must_use]
    pub fn session_id(&self) -> &str {
        match self {
            Self::Api(s) => s.session_id(),
            Self::Cli(s) => s.session_id(),
        }
    }

    /// Total number of submit calls completed.
    #[must_use]
    pub fn total_turns(&self) -> u32 {
        match self {
            Self::Api(s) => s.total_turns(),
            Self::Cli(s) => s.total_turns(),
        }
    }
}

impl std::fmt::Debug for AgentSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(s) => f.debug_tuple("AgentSession::Api").field(s).finish(),
            Self::Cli(s) => f.debug_tuple("AgentSession::Cli").field(s).finish(),
        }
    }
}

impl Drop for AgentSession {
    fn drop(&mut self) {
        self.close();
    }
}
