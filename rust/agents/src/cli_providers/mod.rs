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
use crate::tool_guard::TrustLevel;
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

    /// Whether a submit error is likely transient/recoverable and worth one
    /// automatic retry.
    fn should_retry_submit_error(&self, _error: &AgentError) -> bool {
        false
    }

    /// Reset provider state after a failed submit before retrying.
    fn reset_after_submit_error(&mut self) {}

    /// Close the CLI subprocess and clean up resources.
    fn close(&mut self);

    /// Whether this provider supports resuming sessions across submit calls.
    fn supports_resume(&self) -> bool;

    /// The CLI tool's session ID, if one has been established.
    fn session_id(&self) -> Option<&str>;

    /// Return the provider's resume state as a JSON value, or `None` if
    /// the provider has no resumable state (e.g. no session established yet,
    /// or the provider does not support resumption).
    fn resume_state(&self) -> Option<serde_json::Value> {
        None
    }

    /// Restore resume state previously obtained from [`resume_state()`].
    ///
    /// The default implementation is a no-op. Providers that support
    /// resumption override this to re-import the session/conversation ID.
    fn set_resume_state(&mut self, _state: serde_json::Value) -> AgentResult<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CliProviderConfig
// ---------------------------------------------------------------------------

/// Configuration passed to CLI providers, derived from agent definition.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct CliProviderConfig {
    /// Model identifier to pass to the CLI tool.
    pub model: Option<String>,

    /// System prompt / instructions to append.
    pub instructions: Option<String>,

    /// Tool names to pass to the CLI tool, if supported.
    ///
    /// These are provider-specific names, not necessarily Stencila tool names.
    /// An empty list means disable all CLI-agent tools when the provider has a
    /// flag for that behavior.
    pub allowed_tools: Option<Vec<String>>,

    /// Permission mode to pass to the CLI tool, if supported.
    pub permission_mode: Option<String>,

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
    ///
    /// **Note:** `instructions` is set to `None` for the generic CLI path. CLI
    /// tools are full agents with their own system prompts and tool registries;
    /// provider-specific constructors can opt in to lightweight prompt or
    /// policy hints when the CLI exposes suitable flags.
    #[must_use]
    pub fn from_session_config(config: &SessionConfig, model: Option<&str>) -> Self {
        Self {
            model: model.map(resolve_model_alias),
            instructions: None,
            allowed_tools: None,
            permission_mode: None,
            max_turns: if config.max_turns > 0 {
                Some(config.max_turns)
            } else {
                None
            },
            working_dir: None,
        }
    }

    /// Build Claude CLI configuration from a Stencila session configuration.
    ///
    /// Claude Code exposes first-class flags for appending Stencila's agent
    /// instructions and approximating Stencila's tool/trust policy. Claude
    /// still owns tool execution internally; these flags constrain Claude Code
    /// rather than applying Stencila's API-session [`ToolGuard`](crate::tool_guard::ToolGuard).
    #[must_use]
    pub fn from_session_config_for_claude(
        config: &SessionConfig,
        model: Option<&str>,
        trust_level: TrustLevel,
    ) -> Self {
        let mut cli_config = Self::from_session_config(config, model);

        cli_config.instructions = claude_instructions(config);
        cli_config.allowed_tools = config
            .allowed_tools
            .as_ref()
            .map(|tools| map_stencila_tools_to_claude_tools(tools, trust_level));
        cli_config.permission_mode = Some(
            match trust_level {
                TrustLevel::Low => "plan",
                TrustLevel::Medium => "default",
                TrustLevel::High => "acceptEdits",
            }
            .to_string(),
        );

        cli_config
    }
}

/// Build the prompt fragment appended to Claude Code's default system prompt.
fn claude_instructions(config: &SessionConfig) -> Option<String> {
    let mut layers = Vec::new();

    if let Some(instructions) = config
        .commit_instructions
        .as_deref()
        .filter(|instructions| !instructions.trim().is_empty())
    {
        layers.push(instructions);
    }

    if let Some(instructions) = config
        .user_instructions
        .as_deref()
        .filter(|instructions| !instructions.trim().is_empty())
    {
        layers.push(instructions);
    }

    (!layers.is_empty()).then(|| layers.join("\n\n"))
}

/// Map Stencila and Claude tool names to Claude Code built-in tool names.
///
/// Existing agent allowlists may already contain Claude-native names such as
/// `Read` or constrained patterns such as `Bash(python:*)`; these are preserved
/// directly. Unknown tools are omitted because passing Stencila-specific names
/// to Claude's `--tools` flag can fail startup. If every requested tool is
/// unknown, the returned list is empty, causing Claude tools to be disabled
/// rather than silently falling back to all tools.
///
/// Stencila's `shell` tool is only translated to unrestricted Claude `Bash` at
/// high trust. API-backed Stencila sessions apply command-level shell guarding,
/// but in Claude CLI sessions Claude owns the internal tool loop and Stencila
/// cannot apply that guard to individual commands. Agents that need narrower
/// shell permissions should specify Claude-native patterns such as
/// `Bash(python:*)` explicitly.
fn map_stencila_tools_to_claude_tools(tools: &[String], trust_level: TrustLevel) -> Vec<String> {
    let mut claude_tools = Vec::new();

    for tool in tools {
        let tool = tool.trim();
        if tool.is_empty() {
            continue;
        }

        let mapped = match tool {
            "read_file" | "read_many_files" => Some("Read".to_string()),
            "write_file" => Some("Write".to_string()),
            "edit_file" | "apply_patch" => Some("Edit".to_string()),
            "shell" if matches!(trust_level, TrustLevel::High) => Some("Bash".to_string()),
            "shell" => None,
            "grep" => Some("Grep".to_string()),
            "glob" => Some("Glob".to_string()),
            "list_dir" => Some("LS".to_string()),
            "web_fetch" => Some("WebFetch".to_string()),
            tool if is_claude_tool_spec(tool) => Some(tool.to_string()),
            _ => None,
        };

        if let Some(tool) = mapped
            && !claude_tools.iter().any(|existing| existing == &tool)
        {
            claude_tools.push(tool);
        }
    }

    claude_tools
}

/// Whether a tool allowlist entry is already a Claude Code tool spec.
fn is_claude_tool_spec(tool: &str) -> bool {
    const CLAUDE_TOOLS: &[&str] = &[
        "Bash",
        "BashOutput",
        "Edit",
        "ExitPlanMode",
        "Glob",
        "Grep",
        "KillBash",
        "LS",
        "MultiEdit",
        "NotebookEdit",
        "Read",
        "Task",
        "TodoWrite",
        "WebFetch",
        "WebSearch",
        "Write",
    ];

    CLAUDE_TOOLS.iter().any(|name| {
        tool == *name
            || tool
                .strip_prefix(name)
                .is_some_and(|suffix| suffix.starts_with('(') && suffix.ends_with(')'))
    })
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
    agent_name: String,
    /// Timestamp captured at session construction for checkpoint records.
    created_at: String,
    workflow_attribution: Option<crate::store::WorkflowAttribution>,
    persistence_store: Option<std::sync::Arc<crate::store::AgentSessionStore>>,
    persistence_mode: Option<crate::store::SessionPersistence>,
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
            agent_name: String::new(),
            created_at: crate::types::now_timestamp(),
            workflow_attribution: None,
            persistence_store: None,
            persistence_mode: None,
        };

        (session, receiver)
    }

    /// Set the agent name recorded in checkpoint persistence records.
    pub fn set_agent_name(&mut self, name: impl Into<String>) {
        self.agent_name = name.into();
    }

    /// Set workflow attribution metadata for checkpoint persistence records.
    pub fn set_workflow_attribution(&mut self, attribution: crate::store::WorkflowAttribution) {
        self.workflow_attribution = Some(attribution);
    }

    /// Wire checkpoint persistence into this session.
    ///
    /// Immediately inserts a session record into the store (creation checkpoint).
    /// Errors during the creation checkpoint are logged but swallowed regardless
    /// of the persistence policy.
    pub fn set_persistence(
        &mut self,
        store: std::sync::Arc<crate::store::AgentSessionStore>,
        persistence: crate::store::SessionPersistence,
    ) {
        self.persistence_store = Some(store);
        self.persistence_mode = Some(persistence);
        if let Err(e) = self.checkpoint() {
            tracing::warn!("creation checkpoint failed (swallowed): {e}");
        }
    }

    /// Like [`set_persistence`](Self::set_persistence), but returns the creation-checkpoint result.
    pub fn set_persistence_checked(
        &mut self,
        store: std::sync::Arc<crate::store::AgentSessionStore>,
        persistence: crate::store::SessionPersistence,
    ) -> crate::error::AgentResult<()> {
        self.persistence_store = Some(store);
        self.persistence_mode = Some(persistence);
        self.checkpoint()
    }

    /// Write the current session state to the persistence store.
    fn checkpoint(&self) -> AgentResult<()> {
        let Some(ref store) = self.persistence_store else {
            return Ok(());
        };
        if !crate::store::should_persist(self.persistence_mode.as_ref()) {
            return Ok(());
        }

        let resumability =
            if self.provider.supports_resume() && self.provider.session_id().is_some() {
                crate::store::Resumability::Full
            } else {
                crate::store::Resumability::None
            };

        let (workflow_run_id, workflow_thread_id, workflow_node_id) =
            crate::store::workflow_fields(self.workflow_attribution.as_ref());

        let record = crate::store::SessionRecord {
            session_id: self.session_id.clone(),
            backend_kind: "cli".to_string(),
            agent_name: self.agent_name.clone(),
            provider_name: self.provider.id().to_string(),
            model_name: String::new(),
            state: self.state,
            total_turns: i64::from(self.total_turns),
            resumability,
            created_at: self.created_at.clone(),
            updated_at: crate::types::now_timestamp(),
            workflow_run_id,
            workflow_thread_id,
            workflow_node_id,
            provider_resume_state: self.provider.session_id().map(|id| id.to_string()),
            config_snapshot: None,
            system_prompt: None,
            lease_holder: None,
            lease_expires_at: None,
        };

        crate::store::write_checkpoint(store, &record, &self.history)
    }

    /// Handle the result of a checkpoint call according to the persistence policy.
    fn handle_checkpoint_result(&self, result: AgentResult<()>) -> AgentResult<()> {
        crate::store::handle_checkpoint_result(self.persistence_mode.as_ref(), result)
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

        let first_result = self
            .provider
            .submit(input, &self.events, self.abort_signal.as_ref())
            .await;

        let result = match first_result {
            Ok(()) => Ok(()),
            Err(first_error)
                if self.abort_kind() == AbortKind::Active
                    && self.provider.should_retry_submit_error(&first_error) =>
            {
                self.events.emit_info(
                    "CLI_RECOVERY_RETRY",
                    format!("Retrying CLI submit after error: {first_error}"),
                );
                self.provider.reset_after_submit_error();
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                self.provider
                    .submit(input, &self.events, self.abort_signal.as_ref())
                    .await
            }
            Err(error) => Err(error),
        };

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
                let cp = self.checkpoint();
                self.handle_checkpoint_result(cp)
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
            if let Err(e) = self.checkpoint() {
                tracing::warn!("close checkpoint failed (swallowed): {e}");
            }
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

    /// The session's event emitter.
    #[must_use]
    pub fn events(&self) -> &EventEmitter {
        &self.events
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_config_includes_instructions_tools_and_permission_mode() {
        let config = SessionConfig {
            commit_instructions: Some("Use conventional commits".to_string()),
            user_instructions: Some("Review code carefully".to_string()),
            allowed_tools: Some(vec![
                "read_file".to_string(),
                "apply_patch".to_string(),
                "read_file".to_string(),
            ]),
            ..Default::default()
        };

        let cli_config = CliProviderConfig::from_session_config_for_claude(
            &config,
            Some("sonnet"),
            TrustLevel::Low,
        );

        assert_eq!(cli_config.permission_mode.as_deref(), Some("plan"));
        assert_eq!(
            cli_config.allowed_tools,
            Some(vec!["Read".into(), "Edit".into()])
        );

        let instructions = cli_config.instructions.as_deref().unwrap_or_default();
        assert!(instructions.contains("Use conventional commits"));
        assert!(instructions.contains("Review code carefully"));
    }

    #[test]
    fn claude_config_omits_empty_instruction_layers() {
        let config = SessionConfig {
            commit_instructions: Some("  \n".to_string()),
            user_instructions: Some("Review code carefully".to_string()),
            ..Default::default()
        };

        assert_eq!(
            claude_instructions(&config).as_deref(),
            Some("Review code carefully")
        );
    }

    #[test]
    fn claude_tool_mapping_preserves_native_tools_and_patterns() {
        let tools = vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
            "Bash(python:*)".to_string(),
            "Read".to_string(),
        ];

        assert_eq!(
            map_stencila_tools_to_claude_tools(&tools, TrustLevel::Medium),
            vec![
                "Read".to_string(),
                "Write".to_string(),
                "Bash".to_string(),
                "Bash(python:*)".to_string(),
            ]
        );
    }

    #[test]
    fn claude_tool_mapping_only_maps_stencila_shell_at_high_trust() {
        let tools = vec!["shell".to_string()];

        assert!(map_stencila_tools_to_claude_tools(&tools, TrustLevel::Low).is_empty());
        assert!(map_stencila_tools_to_claude_tools(&tools, TrustLevel::Medium).is_empty());
        assert_eq!(
            map_stencila_tools_to_claude_tools(&tools, TrustLevel::High),
            vec!["Bash".to_string()]
        );
    }

    #[test]
    fn claude_tool_mapping_returns_empty_for_empty_or_unknown_allowlist() {
        assert!(map_stencila_tools_to_claude_tools(&[], TrustLevel::Medium).is_empty());
        assert!(
            map_stencila_tools_to_claude_tools(&["delegate".to_string()], TrustLevel::Medium)
                .is_empty()
        );
    }
}
