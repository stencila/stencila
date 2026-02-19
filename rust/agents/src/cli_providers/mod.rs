//! CLI-based providers that delegate agentic interaction to external CLI tools.
//!
//! Each CLI tool (Claude, Codex, Gemini) is a full agent with its own tool
//! execution loop. The agents crate delegates the entire interaction to the
//! subprocess rather than running its own tool loop, observing and streaming
//! events back through the standard [`EventEmitter`].
//!
//! # Provider naming
//!
//! CLI providers use the `-cli` suffix: `claude-cli`, `codex-cli`, `gemini-cli`.

pub mod claude;
pub mod codex;
pub mod gemini;

use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::{BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};

use crate::error::{AgentError, AgentResult};
use crate::events::{self, EventEmitter, EventReceiver};
use crate::types::{AbortKind, AbortSignal, SessionConfig, SessionState, Turn};

// ---------------------------------------------------------------------------
// CliProvider trait
// ---------------------------------------------------------------------------

/// Trait for CLI tool adapters.
///
/// Each CLI tool (Claude, Codex, Gemini) implements this to handle
/// subprocess spawning, output parsing, and event mapping.
#[async_trait]
pub trait CliProvider: Send + std::fmt::Debug {
    /// Provider identifier (e.g. "claude-cli").
    fn id(&self) -> &str;

    /// Submit user input, spawning or communicating with the CLI subprocess.
    ///
    /// The provider emits events through the provided [`EventEmitter`] and
    /// checks the optional [`AbortSignal`] for cancellation.
    async fn submit(
        &mut self,
        input: &str,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()>;

    /// Close the CLI subprocess and clean up resources.
    fn close(&mut self);

    /// Whether this provider supports resuming sessions across submit calls.
    fn supports_resume(&self) -> bool;

    /// The CLI tool's session ID, if one has been established.
    fn session_id(&self) -> Option<&str>;
}

// ---------------------------------------------------------------------------
// CliProviderConfig
// ---------------------------------------------------------------------------

/// Configuration passed to CLI providers, derived from agent definition.
#[derive(Debug, Clone)]
pub struct CliProviderConfig {
    /// Model identifier to pass to the CLI tool.
    pub model: Option<String>,

    /// System prompt / instructions to append.
    pub instructions: Option<String>,

    /// Maximum turns for the CLI tool (if supported).
    pub max_turns: Option<u32>,

    /// Working directory for the subprocess.
    pub working_dir: Option<String>,
}

impl CliProviderConfig {
    /// Build from a [`SessionConfig`] and agent metadata.
    ///
    /// If `model` is a catalog alias (e.g. "sonnet", "opus"), it is resolved
    /// to the canonical model ID before being stored so that CLI tools receive
    /// the identifier they expect. If the model name is not found in the
    /// catalog, the original string is passed through so that the CLI tool
    /// can attempt to use it directly.
    #[must_use]
    pub fn from_session_config(config: &SessionConfig, model: Option<&str>) -> Self {
        Self {
            model: model.map(resolve_model_alias),
            instructions: config.user_instructions.clone(),
            max_turns: if config.max_turns > 0 {
                Some(config.max_turns)
            } else {
                None
            },
            working_dir: None,
        }
    }
}

/// Resolve a model name through the catalog, returning the canonical ID.
///
/// If the name is found in the catalog, returns the canonical ID.
/// Otherwise, returns the original string unchanged so that CLI tools
/// can attempt to use it directly (e.g. for model IDs not yet in the
/// catalog).
fn resolve_model_alias(model: &str) -> String {
    stencila_models3::catalog::get_model_info(model)
        .ok()
        .flatten()
        .map_or_else(|| model.to_string(), |info| info.id)
}

// ---------------------------------------------------------------------------
// CliSession
// ---------------------------------------------------------------------------

/// A session backed by an external CLI tool.
///
/// Wraps a [`CliProvider`] and manages event emission, history recording,
/// abort signal forwarding, and turn counting. The CLI tool handles its
/// own tool execution loop — this session observes and relays events.
pub struct CliSession {
    provider: Box<dyn CliProvider>,
    config: SessionConfig,
    state: SessionState,
    history: Vec<Turn>,
    events: EventEmitter,
    abort_signal: Option<AbortSignal>,
    total_turns: u32,
    session_id: String,
}

impl std::fmt::Debug for CliSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliSession")
            .field("provider", &self.provider.id())
            .field("state", &self.state)
            .field("total_turns", &self.total_turns)
            .finish_non_exhaustive()
    }
}

impl CliSession {
    /// Create a new CLI session with the given provider.
    ///
    /// Returns the session and an [`EventReceiver`] for consuming events.
    /// Emits a `SESSION_START` event immediately.
    pub fn new(provider: Box<dyn CliProvider>, config: SessionConfig) -> (Self, EventReceiver) {
        let (emitter, receiver) = events::channel();
        let session_id = emitter.session_id().to_string();
        emitter.emit_session_start();

        let session = Self {
            provider,
            config,
            state: SessionState::Idle,
            history: Vec::new(),
            events: emitter,
            abort_signal: None,
            total_turns: 0,
            session_id,
        };

        (session, receiver)
    }

    /// Submit user input to the CLI provider.
    ///
    /// # State transitions
    ///
    /// - IDLE -> PROCESSING -> IDLE (natural completion)
    /// - IDLE -> PROCESSING -> CLOSED (error or abort)
    ///
    /// # Errors
    ///
    /// Returns `Err(SessionClosed)` if the session is already closed.
    pub async fn submit(&mut self, input: &str) -> AgentResult<()> {
        if self.state == SessionState::Closed {
            return Err(AgentError::SessionClosed);
        }

        // Enforce session turn limit (mirrors ApiSession behaviour).
        if self.config.max_turns > 0 && self.total_turns >= self.config.max_turns {
            return Err(AgentError::TurnLimitExceeded {
                message: format!(
                    "max_turns ({}) reached after {} submit(s)",
                    self.config.max_turns, self.total_turns,
                ),
            });
        }

        // Reset a previous soft abort so the provider sees a clean signal.
        if let Some(ref signal) = self.abort_signal {
            signal.reset_soft();
        }

        self.state = SessionState::Processing;
        self.history.push(Turn::user(input));
        self.events.emit_user_input(input);

        let result = self
            .provider
            .submit(input, &self.events, self.abort_signal.as_ref())
            .await;

        match result {
            Ok(()) => {
                self.total_turns += 1;
                // Check if aborted during processing
                match self.abort_kind() {
                    AbortKind::Hard => self.close(),
                    AbortKind::Soft => {
                        // Reset so the next submit starts clean.
                        if let Some(ref signal) = self.abort_signal {
                            signal.reset_soft();
                        }
                        self.state = SessionState::Idle;
                    }
                    AbortKind::Active => {
                        self.state = SessionState::Idle;
                    }
                }
                Ok(())
            }
            Err(e) => {
                self.events.emit_error(e.code(), e.to_string());
                self.close();
                Err(e)
            }
        }
    }

    /// Close the session. Emits `SESSION_END` and transitions to CLOSED.
    pub fn close(&mut self) {
        if self.state != SessionState::Closed {
            self.provider.close();
            self.state = SessionState::Closed;
            self.events.emit_session_end(self.state);
        }
    }

    /// Set an abort signal for cancellation.
    pub fn set_abort_signal(&mut self, signal: AbortSignal) {
        self.abort_signal = Some(signal);
    }

    /// Current session state.
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Full conversation history (observational).
    #[must_use]
    pub fn history(&self) -> &[Turn] {
        &self.history
    }

    /// Session configuration.
    #[must_use]
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// The session ID from the event emitter.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Total number of submit calls completed.
    #[must_use]
    pub fn total_turns(&self) -> u32 {
        self.total_turns
    }

    /// Read the current abort kind.
    fn abort_kind(&self) -> AbortKind {
        self.abort_signal
            .as_ref()
            .map_or(AbortKind::Active, AbortSignal::kind)
    }
}

impl Drop for CliSession {
    fn drop(&mut self) {
        self.close();
    }
}

// ---------------------------------------------------------------------------
// Shared child-process helpers
// ---------------------------------------------------------------------------

/// Kill a child process (if running) and reap it.
///
/// Used by providers that spawn a new subprocess per `submit` call. After
/// this returns, the process is guaranteed to have exited.
async fn kill_child(child: &mut Option<Child>) {
    if let Some(c) = child {
        let _ = c.start_kill();
        let _ = c.wait().await;
    }
}

/// Wait for a child process to complete and check its exit status.
///
/// On non-zero exit, `error_detail` is used as the error message when
/// non-empty.  When empty the function falls back to reading whatever
/// remains on the child's stderr handle.
async fn wait_for_child(
    child: &mut Option<Child>,
    provider_name: &str,
    error_detail: &str,
) -> AgentResult<()> {
    let Some(c) = child else {
        return Ok(());
    };

    let status = c.wait().await.map_err(|e| AgentError::CliProcessFailed {
        code: -1,
        stderr: format!("Failed to wait for {provider_name} process: {e}"),
    })?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        let stderr = if !error_detail.is_empty() {
            error_detail.to_string()
        } else if let Some(ref mut stderr_handle) = c.stderr {
            let mut buf = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(stderr_handle, &mut buf).await;
            buf
        } else {
            String::new()
        };
        return Err(AgentError::CliProcessFailed { code, stderr });
    }

    Ok(())
}

/// Best-effort kill a child process (without waiting) and clear the handle.
///
/// Used by [`CliProvider::close`] implementations. Unlike [`kill_child`]
/// this does **not** reap the process — it fires `start_kill` and drops the
/// handle, which is appropriate for synchronous `close()` methods.
fn close_child(child: &mut Option<Child>) {
    if let Some(c) = child {
        let _ = c.start_kill();
    }
    *child = None;
}

/// Abort-aware line reader for CLI subprocess stdout.
///
/// Reads lines from `stdout_lines`, checking `abort` between iterations.
/// For each non-empty line, `on_line` is called.  Returns `true` if the
/// loop was interrupted by an abort signal (and kills/reaps the child),
/// `false` on natural EOF.
///
/// This is the shared skeleton used by Claude, Gemini, and Codex providers.
async fn read_lines_until_eof_or_abort(
    lines: &mut Lines<BufReader<ChildStdout>>,
    child: &mut Option<Child>,
    abort: Option<&AbortSignal>,
    provider_name: &str,
    mut on_line: impl FnMut(String),
) -> AgentResult<bool> {
    loop {
        if let Some(signal) = abort
            && signal.is_aborted()
        {
            kill_child(child).await;
            return Ok(true);
        }

        let line = tokio::select! {
            line = lines.next_line() => line,
            () = async {
                if let Some(signal) = abort {
                    signal.cancelled().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                kill_child(child).await;
                return Ok(true);
            }
        };

        match line {
            Ok(Some(line)) => {
                if !line.trim().is_empty() {
                    on_line(line);
                }
            }
            Ok(None) => return Ok(false), // EOF
            Err(e) => {
                return Err(AgentError::CliParseError {
                    message: format!("Failed to read {provider_name} output: {e}"),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CLI detection helpers
// ---------------------------------------------------------------------------

/// Check whether a CLI binary is available on PATH.
pub fn is_cli_available(binary: &str) -> bool {
    which::which(binary).is_ok()
}

/// Get the version string of a CLI binary, if available.
pub async fn cli_version(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Ensure a CLI binary is available, returning a descriptive error if not.
pub fn require_cli(binary: &str) -> AgentResult<()> {
    if is_cli_available(binary) {
        Ok(())
    } else {
        Err(AgentError::CliNotFound {
            binary: binary.to_string(),
        })
    }
}
