mod cancellation;
mod candidates;
mod helpers;
mod key_handling;
mod polling;
mod sessions;
mod submission;
mod workflows;

use crossterm::event::{Event, MouseEventKind};
use ratatui::style::Color;
use strum::Display;
use tokio::{sync::mpsc, task::JoinHandle};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    agent::{AgentHandle, AgentProgress, ResponseSegment, RunningAgentExchange},
    autocomplete::{
        AgentsState, CancelState, CommandsState, FilesState, HistoryState, MentionsState,
        ResponsesState, WorkflowsState, agents::AgentDefinitionInfo,
        workflows::WorkflowDefinitionInfo,
    },
    cli_commands::CliCommandNode,
    config::AppConfig,
    history::InputHistory,
    input::InputState,
    shell::RunningShellCommand,
    workflow::{PendingInterview, WorkflowRunHandle},
};

/// The current input mode of the TUI.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Default)]
#[strum(serialize_all = "lowercase")]
pub enum AppMode {
    #[default]
    Agent,
    Shell,
    Workflow,
}

/// The kind of exchange, determining sidebar color.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeKind {
    Agent,
    Shell,
    Workflow,
}

impl ExchangeKind {
    /// Sidebar color for this kind.
    pub fn color(self) -> Color {
        match self {
            Self::Agent => Color::Blue,
            Self::Shell => Color::Yellow,
            Self::Workflow => Color::Rgb(0, 180, 160),
        }
    }
}

impl From<AppMode> for ExchangeKind {
    fn from(mode: AppMode) -> Self {
        match mode {
            AppMode::Agent => Self::Agent,
            AppMode::Shell => Self::Shell,
            AppMode::Workflow => Self::Workflow,
        }
    }
}

/// The status of an exchange.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExchangeStatus {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Fixed palette of colors for agent names, cycled on first encounter.
const AGENT_COLORS: [Color; 8] = [
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Green,
    Color::LightYellow,
    Color::LightRed,
    Color::LightBlue,
    Color::LightMagenta,
];

/// Registry that assigns a stable color to each unique agent name.
///
/// Colors are drawn from [`AGENT_COLORS`] and assigned lazily on first
/// encounter, cycling when more names than palette entries are seen.
pub struct AgentColorRegistry {
    map: HashMap<String, Color>,
    next_index: usize,
}

impl AgentColorRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_index: 0,
        }
    }

    /// Get (or assign) a color for the given agent name (case-insensitive).
    pub fn color_for(&mut self, name: &str) -> Color {
        let key = name.to_ascii_lowercase();
        if let Some(&c) = self.map.get(&key) {
            return c;
        }
        let c = AGENT_COLORS[self.next_index % AGENT_COLORS.len()];
        self.next_index += 1;
        self.map.insert(key, c);
        c
    }

    /// Lookup without assigning (for read-only contexts).
    pub fn get(&self, name: &str) -> Option<Color> {
        self.map.get(&name.to_ascii_lowercase()).copied()
    }
}

/// An agent session with its own model, instructions, and running state.
pub struct AgentSession {
    /// Name of the agent.
    pub name: String,

    /// The agent definition this session was created from, if any.
    pub definition: Option<AgentDefinitionInfo>,

    /// Agent handle for submitting chat messages.
    /// Created lazily on first chat submit.
    agent: Option<AgentHandle>,

    /// Agent exchanges currently running in the background.
    /// Each entry is `(message_index, running_exchange)`.
    pub running_agent_exchanges: Vec<(usize, RunningAgentExchange)>,

    /// Approximate context usage percentage (0–100+), updated from agent events.
    pub context_usage_percent: u32,
}

impl AgentSession {
    /// Create a new agent session with the given name.
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            agent: None,
            definition: None,
            running_agent_exchanges: Vec::new(),
            context_usage_percent: 0,
        }
    }
}

/// State for an active workflow (workflow mode).
pub struct ActiveWorkflow {
    /// The workflow definition info from the picker.
    pub info: WorkflowDefinitionInfo,

    /// Current state of the workflow.
    pub state: ActiveWorkflowState,

    /// Running workflow task handle, set once the goal is submitted.
    pub run_handle: Option<WorkflowRunHandle>,

    /// Interview question pending user response, if any.
    pub pending_interview: Option<PendingInterview>,

    /// Message index of the current stage exchange (for linking prompt → response).
    pub current_exchange_msg_index: Option<usize>,

    /// Shared progress state for the current stage's agent session events.
    pub current_stage_progress: Option<Arc<Mutex<AgentProgress>>>,

    /// Message index of the workflow-level status message (Minimal mode).
    pub workflow_status_msg_index: Option<usize>,

    /// Message index of the current stage-level status message (Simple mode).
    pub stage_status_msg_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ActiveWorkflowState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// A message displayed in the messages area.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// The initial welcome message.
    Welcome,

    /// The site preview is ready
    SitePreviewReady { url: String },

    /// A system/informational message (mode transitions, slash command output, etc.).
    System { content: String },

    /// A request/response exchange.
    Exchange {
        kind: ExchangeKind,
        status: ExchangeStatus,
        request: String,
        response: Option<String>,

        /// Structured response segments for rendering (agent exchanges only).
        /// When present, the renderer uses these for styled tool-call and
        /// warning annotations. When `None`, `response` is rendered as plain text.
        response_segments: Option<Vec<ResponseSegment>>,

        /// Shell exit code (only meaningful for Shell kind).
        exit_code: Option<i32>,

        /// Index of the agent session that owns this exchange.
        /// `None` for shell exchanges.
        agent_index: Option<usize>,

        /// Agent name for workflow exchanges (not backed by a session).
        agent_name: Option<String>,
    },

    /// Singleton message for a workflow's status used for [`WorkflowVerbosity::Minimal`].
    WorkflowStatus {
        /// Current state of workflow.
        state: WorkflowStatusState,

        /// Display label (e.g. "Workflow my-workflow").
        label: String,

        /// Detail text (e.g outcome or failure reason).
        detail: Option<String>,
    },

    /// A workflow progress message used for [`WorkflowVerbosity::Simple`].
    WorkflowProgress {
        // The kind of progress
        kind: WorkflowProgressKind,

        /// Display label (e.g. "Workflow my-workflow").
        label: String,

        /// Detail text.
        detail: Option<String>,
    },
}

/// The state of a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatusState {
    /// Workflow is running
    Running,
    /// Workflow has completed
    Completed,
    /// Workflow has failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

/// The kind of progress in a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowProgressKind {
    /// Workflow started
    Started,
    /// Stage is running
    Running,
    /// Stage or workflow completed
    Completed,
    /// Stage or workflow failed
    Failed,
    /// Stage retrying.
    Retrying,
    /// Workflow was cancelled
    Cancelled,
}

/// Discover agent definitions, returning an empty vec if no runtime is available.
fn discover_agents() -> Vec<stencila_agents::agent_def::AgentInstance> {
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        return Vec::new();
    };
    // block_in_place panics on current_thread runtime; catch that gracefully.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            handle.block_on(stencila_agents::agent_def::discover(
                &std::env::current_dir().unwrap_or_default(),
            ))
        })
    }))
    .unwrap_or_default()
}

/// Parsed result of an `#agent-name prompt` mention in the input.
struct AgentMention {
    /// Name of the agent to target.
    agent_name: String,
    /// Optional prompt to send (trimmed, `&` suffix removed).
    prompt: Option<String>,
    /// Whether to switch back to the original session after sending.
    switch_back: bool,
}

/// Top-level application state.
///
/// All mutable state lives here. The render function takes `&App` (immutable)
/// while event handlers take `&mut App`.
pub struct App {
    /// App config
    config: AppConfig,

    /// Whether the app should exit.
    pub should_quit: bool,

    /// Current input mode (chat or shell).
    pub mode: AppMode,

    /// Chat messages displayed in the message area.
    pub messages: Vec<AppMessage>,

    /// Current input buffer.
    pub input: InputState,
    /// Command history with navigation.
    pub input_history: InputHistory,

    /// Commands autocomplete popup state.
    pub commands_state: CommandsState,
    /// Files autocomplete popup state.
    pub files_state: FilesState,
    /// History autocomplete popup state.
    pub history_state: HistoryState,
    /// Responses autocomplete popup state.
    pub responses_state: ResponsesState,
    /// Cancel picker popup state.
    pub cancel_state: CancelState,
    /// Agent picker popup state.
    pub agents_state: AgentsState,
    /// Workflow picker popup state.
    pub workflows_state: WorkflowsState,
    /// Agent mention autocomplete popup state (triggered by `#`).
    pub mentions_state: MentionsState,

    /// Stored paste contents keyed by paste number. Large pastes are inserted
    /// as `[Paste #N: preview…]` tokens in the input buffer; the full text is
    /// kept here and expanded at submit time (same pattern as response refs).
    pub pastes: std::collections::HashMap<usize, String>,
    /// Counter for generating paste numbers.
    paste_counter: usize,

    /// Vertical scroll offset for the input area. Persisted across frames so
    /// the viewport only scrolls when the cursor moves beyond the visible edges.
    pub input_scroll: u16,

    /// Ghost suggestion suffix (the part beyond what's typed, insertable text only).
    /// Shown as dim inline text for fish-shell-style history completion.
    pub ghost_suggestion: Option<String>,
    /// Whether the ghost suggestion was truncated from a multiline history entry.
    /// When true, the UI appends a dim "…" indicator after the ghost text.
    pub ghost_is_truncated: bool,
    /// Offset for cycling ghost suggestions via Up/Down arrows.
    /// 0 = most recent prefix match (default), incremented by Up, decremented by Down.
    ghost_nav_offset: usize,

    /// Agent sessions. Index 0 is the default session.
    pub sessions: Vec<AgentSession>,
    /// Index of the currently active agent session.
    pub active_session: usize,
    /// Shell commands currently running in the background.
    /// Each entry is `(message_index, running_command)` linking to the exchange in `messages`.
    pub running_shell_commands: Vec<(usize, RunningShellCommand)>,

    /// The currently active workflow (if in workflow mode).
    pub active_workflow: Option<ActiveWorkflow>,

    /// Tick counter for pulsating sidebar animation on running exchanges.
    pub tick_count: u32,

    /// Receiver for tracing log messages captured by the TUI logging layer.
    log_receiver: mpsc::UnboundedReceiver<String>,

    /// Whether the message view is pinned to the bottom (auto-follows new content).
    /// When `true`, the view always shows the latest messages.
    /// When `false`, `scroll_offset` holds the absolute top-line position.
    pub scroll_pinned: bool,
    /// Scroll offset for the message area (absolute top-line position when unpinned).
    pub scroll_offset: u16,
    /// Total lines rendered in the last frame's message area (set by `ui::render`).
    pub total_message_lines: u16,
    /// Visible height of the message area in the last frame (set by `ui::render`).
    pub visible_message_height: u16,

    /// Background site preview handle, if site config exists.
    pub site_preview: Option<crate::site_preview::SitePreviewHandle>,

    /// Background upgrade check handle, consumed once resolved.
    upgrade_handle: Option<JoinHandle<Option<String>>>,
    /// Set when a newer version is available (from the background upgrade check).
    pub upgrade_available: Option<String>,
    /// Message index of a running `/upgrade` shell command, if any.
    /// Used to clear `upgrade_available` when the upgrade succeeds.
    upgrade_msg_index: Option<usize>,

    /// Cache for markdown rendering of response text segments.
    pub md_render_cache: crate::ui::markdown::MdRenderCache,

    /// Registry mapping agent names to stable colors.
    pub color_registry: AgentColorRegistry,

    /// CLI command tree for slash command passthrough and autocomplete.
    pub cli_tree: Option<Arc<Vec<CliCommandNode>>>,
}

impl App {
    /// Create a new App with a welcome banner.
    ///
    pub fn new(
        log_receiver: mpsc::UnboundedReceiver<String>,
        upgrade_handle: Option<JoinHandle<Option<String>>>,
        cli_tree: Option<Arc<Vec<CliCommandNode>>>,
    ) -> Self {
        let default_name = stencila_agents::convenience::resolve_default_agent_name("default");
        let default_session = AgentSession::new(&default_name);

        let mut color_registry = AgentColorRegistry::new();
        color_registry.color_for(&default_name);

        let mut commands_state = CommandsState::new();
        if let Some(ref tree) = cli_tree {
            commands_state.set_cli_tree(Arc::clone(tree));
        }

        Self {
            config: AppConfig::default(),
            should_quit: false,
            mode: AppMode::default(),
            messages: vec![AppMessage::Welcome],
            input: InputState::default(),
            input_history: InputHistory::new(),
            commands_state,
            files_state: FilesState::new(),
            history_state: HistoryState::new(),
            responses_state: ResponsesState::new(),
            cancel_state: CancelState::new(),
            agents_state: AgentsState::new(),
            workflows_state: WorkflowsState::new(),
            mentions_state: MentionsState::new(),
            pastes: std::collections::HashMap::new(),
            paste_counter: 0,
            input_scroll: 0,
            ghost_suggestion: None,
            ghost_is_truncated: false,
            ghost_nav_offset: 0,
            running_shell_commands: Vec::new(),
            active_workflow: None,
            sessions: vec![default_session],
            active_session: 0,
            tick_count: 0,
            log_receiver,
            scroll_pinned: true,
            scroll_offset: 0,
            total_message_lines: 0,
            visible_message_height: 0,
            site_preview: None,
            upgrade_handle,
            upgrade_available: None,
            upgrade_msg_index: None,
            md_render_cache: crate::ui::markdown::MdRenderCache::default(),
            color_registry,
            cli_tree,
        }
    }

    /// The currently active agent session.
    pub fn active(&self) -> &AgentSession {
        &self.sessions[self.active_session]
    }

    /// Handle a terminal event. Returns `true` if the app should exit.
    pub fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Paste(text) => self.handle_paste(text),
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => self.scroll_up(3),
                MouseEventKind::ScrollDown => self.scroll_down(3),
                _ => {}
            },
            _ => {}
        }
        self.should_quit
    }

    /// Whether any commands or agent exchanges are currently running.
    pub fn has_running(&self) -> bool {
        !self.running_shell_commands.is_empty()
            || self
                .sessions
                .iter()
                .any(|s| !s.running_agent_exchanges.is_empty())
            || self
                .active_workflow
                .as_ref()
                .is_some_and(|w| w.run_handle.is_some())
    }

    /// Whether the active session has any running agent exchanges.
    pub fn active_session_is_running(&self) -> bool {
        !self.sessions[self.active_session]
            .running_agent_exchanges
            .is_empty()
    }
}

/// Truncate a response to a short preview: first line, max `max_chars` characters.
///
/// Appends `...` if the text was truncated.
fn truncate_preview(text: &str, max_chars: usize) -> String {
    let first_line = text.lines().next().unwrap_or("");
    if first_line.len() <= max_chars && text.lines().count() <= 1 {
        format!("{first_line}...")
    } else if first_line.len() > max_chars {
        format!("{}...", &first_line[..max_chars])
    } else {
        format!("{first_line}...")
    }
}

#[cfg(test)]
impl App {
    /// Create an `App` with a dummy log receiver for testing.
    pub(crate) fn new_for_test() -> Self {
        let (_tx, rx) = mpsc::unbounded_channel();
        Self::new(rx, None, None)
    }
}
