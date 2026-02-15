use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use futures::FutureExt;
use ratatui::style::Color;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    agent::{AgentHandle, ResponseSegment, RunningAgentExchange},
    autocomplete::agents::{AgentDefinitionInfo, AgentSelection},
    autocomplete::cancel::CancelCandidate,
    autocomplete::mentions::MentionCandidate,
    autocomplete::responses::ResponseCandidate,
    autocomplete::{
        AgentsState, CancelState, CommandsState, FilesState, HistoryState, MentionsState,
        ResponsesState,
    },
    commands::SlashCommand,
    history::InputHistory,
    input::InputState,
    shell::RunningShellCommand,
};

/// The current input mode of the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Chat mode — input is sent as chat messages (default).
    #[default]
    Chat,
    /// Shell mode — input is sent to the system shell.
    Shell,
}

/// The kind of exchange, determining sidebar color.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExchangeKind {
    Chat,
    Shell,
}

impl ExchangeKind {
    /// Display name for this kind (shown below input area).
    pub fn label(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Shell => "shell",
        }
    }

    /// Sidebar color for this kind.
    pub fn color(self) -> Color {
        match self {
            Self::Chat => Color::Blue,
            Self::Shell => Color::Yellow,
        }
    }
}

impl From<AppMode> for ExchangeKind {
    fn from(mode: AppMode) -> Self {
        match mode {
            AppMode::Chat => Self::Chat,
            AppMode::Shell => Self::Shell,
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

/// Fixed palette of colors for agent sessions.
const AGENT_COLORS: [Color; 6] = [
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Red,
];

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

    /// Color for this agent session, based on its index.
    pub fn color(index: usize) -> Color {
        AGENT_COLORS[index % AGENT_COLORS.len()]
    }
}

/// A message displayed in the messages area.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// The initial welcome message.
    Welcome,
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
    },
    /// A system/informational message (mode transitions, slash command output, etc.).
    System { content: String },
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
    /// Agent mention autocomplete popup state (triggered by `#`).
    pub mentions_state: MentionsState,

    /// Stored paste contents keyed by paste number. Large pastes are inserted
    /// as `[Paste #N: preview…]` tokens in the input buffer; the full text is
    /// kept here and expanded at submit time (same pattern as response refs).
    pub pastes: std::collections::HashMap<usize, String>,
    /// Counter for generating paste numbers.
    paste_counter: usize,

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

    /// Background upgrade check handle, consumed once resolved.
    upgrade_handle: Option<JoinHandle<Option<String>>>,
    /// Set when a newer version is available (from the background upgrade check).
    pub upgrade_available: Option<String>,
    /// Message index of a running `/upgrade` shell command, if any.
    /// Used to clear `upgrade_available` when the upgrade succeeds.
    upgrade_msg_index: Option<usize>,
}

impl App {
    /// Create a new App with a welcome banner.
    ///
    pub fn new(
        log_receiver: mpsc::UnboundedReceiver<String>,
        upgrade_handle: Option<JoinHandle<Option<String>>>,
    ) -> Self {
        let default_name = stencila_agents::convenience::resolve_default_agent_name("default");
        let default_session = AgentSession::new(default_name);

        Self {
            should_quit: false,
            mode: AppMode::default(),
            messages: vec![AppMessage::Welcome],
            input: InputState::default(),
            input_history: InputHistory::new(),
            commands_state: CommandsState::new(),
            files_state: FilesState::new(),
            history_state: HistoryState::new(),
            responses_state: ResponsesState::new(),
            cancel_state: CancelState::new(),
            agents_state: AgentsState::new(),
            mentions_state: MentionsState::new(),
            pastes: std::collections::HashMap::new(),
            paste_counter: 0,
            ghost_suggestion: None,
            ghost_is_truncated: false,
            ghost_nav_offset: 0,
            running_shell_commands: Vec::new(),
            sessions: vec![default_session],
            active_session: 0,
            tick_count: 0,
            log_receiver,
            scroll_pinned: true,
            scroll_offset: 0,
            total_message_lines: 0,
            visible_message_height: 0,
            upgrade_handle,
            upgrade_available: None,
            upgrade_msg_index: None,
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
    }

    /// Whether the active session has any running agent exchanges.
    pub fn active_session_is_running(&self) -> bool {
        !self.sessions[self.active_session]
            .running_agent_exchanges
            .is_empty()
    }

    /// Dispatch a key event.
    fn handle_key(&mut self, key: &KeyEvent) {
        let consumed = (self.cancel_state.is_visible() && self.handle_cancel_autocomplete(key))
            || (self.agents_state.is_visible() && self.handle_agents_autocomplete(key))
            || (self.mentions_state.is_visible() && self.handle_mentions_autocomplete(key))
            || (self.history_state.is_visible() && self.handle_history_autocomplete(key))
            || (self.commands_state.is_visible() && self.handle_commands_autocomplete(key))
            || (self.files_state.is_visible() && self.handle_files_autocomplete(key))
            || (self.responses_state.is_visible() && self.handle_responses_autocomplete(key));

        if !consumed {
            self.handle_normal_key(key);
            // Only refresh autocomplete after normal key handling — autocomplete
            // handlers manage their own state (e.g. Esc dismisses without re-trigger).
            self.refresh_autocomplete();
        }

        // Ghost suggestion always refreshes (input/cursor may have changed in either path).
        self.refresh_ghost_suggestion();
    }

    /// Handle a key event when the cancel picker popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_cancel_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(result) = self.cancel_state.accept() {
                    self.cancel_by_msg_index(result.msg_index);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.cancel_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.cancel_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.cancel_state.select_next(),
            _ => return false,
        }
        true
    }

    /// Handle a key event when the history autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_history_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(full_text) = self.history_state.accept() {
                    self.input.set_text(&full_text);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.history_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.history_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.history_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.delete_char_before();
                self.history_state.update(self.input.text());
            }
            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                self.history_state.update(self.input.text());
            }
            _ => return false,
        }
        true
    }

    /// Handle a key event when the commands autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_commands_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab) => {
                if let Some(name) = self.commands_state.accept() {
                    self.input.set_text(&name);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.commands_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.commands_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.commands_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if let Some(name) = self.commands_state.accept() {
                    self.input.set_text(&name);
                }
                self.submit_input();
            }
            _ => return false,
        }
        true
    }

    /// Handle a key event when the files autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_files_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            // Tab: accept file, or drill into directory
            (KeyModifiers::NONE, KeyCode::Tab) => {
                let use_at_prefix = self.mode == AppMode::Chat;
                if let Some(result) = self.files_state.accept_tab(use_at_prefix) {
                    self.input.replace_range(result.range, &result.text);
                    if result.refresh {
                        // Directory drill-down — re-trigger update to show new contents
                        let input = self.input.text().to_string();
                        let cursor = self.input.cursor();
                        self.files_state.update(&input, cursor);
                    }
                }
            }
            // Enter: always accept and dismiss
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let use_at_prefix = self.mode == AppMode::Chat;
                if let Some(result) = self.files_state.accept_enter(use_at_prefix) {
                    self.input.replace_range(result.range, &result.text);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.files_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.files_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.files_state.select_next(),
            _ => return false,
        }
        true
    }

    /// Handle a key event when the responses autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_responses_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(result) = self.responses_state.accept() {
                    self.input.replace_range(result.range, &result.text);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.responses_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.responses_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.responses_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.delete_char_before();
                let input = self.input.text().to_string();
                let cursor = self.input.cursor();
                let exchanges = self.response_candidates();
                self.responses_state.update(&input, cursor, &exchanges);
            }
            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                let input = self.input.text().to_string();
                let cursor = self.input.cursor();
                let exchanges = self.response_candidates();
                self.responses_state.update(&input, cursor, &exchanges);
            }
            _ => return false,
        }
        true
    }

    /// Handle normal key input (no autocomplete popup intercept).
    fn handle_normal_key(&mut self, key: &KeyEvent) {
        // Reset ghost navigation offset for any key except Up/Down
        // (those keys cycle through prefix-matched ghost suggestions).
        if !matches!(key.code, KeyCode::Up | KeyCode::Down) {
            self.ghost_nav_offset = 0;
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => {
                if !self.scroll_pinned {
                    self.scroll_to_bottom();
                } else if self.has_running() {
                    self.cancel_most_recent_running();
                }
            }

            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                if self.has_running() {
                    self.cancel_most_recent_running();
                } else {
                    match self.mode {
                        AppMode::Chat => {
                            self.should_quit = true;
                        }
                        AppMode::Shell => {
                            // In shell mode: clear input line (standard shell behavior)
                            self.input.clear();
                        }
                    }
                }
            }

            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                if self.mode == AppMode::Chat {
                    self.enter_shell_mode();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                if self.mode == AppMode::Shell {
                    self.exit_shell_mode();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.reset_active_session();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                if self.sessions.len() > 1 {
                    let next = (self.active_session + 1) % self.sessions.len();
                    self.switch_to_session(next);
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => self.input.delete_to_line_start(),
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => self.input.delete_to_line_end(),

            (m, KeyCode::Enter)
                if m.contains(KeyModifiers::SHIFT) || m.contains(KeyModifiers::ALT) =>
            {
                self.input.insert_newline();
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.submit_input();
            }

            (KeyModifiers::NONE, KeyCode::Tab) => {
                if self.ghost_suggestion.is_some() {
                    self.accept_ghost_word();
                }
            }

            (KeyModifiers::CONTROL, KeyCode::Left) => self.input.move_word_left(),
            (KeyModifiers::CONTROL, KeyCode::Right) => self.input.move_word_right(),
            (KeyModifiers::NONE, KeyCode::Left) => self.input.move_left(),
            (KeyModifiers::NONE, KeyCode::Right) => {
                if self.input.cursor() == self.input.text().len() && self.ghost_suggestion.is_some()
                {
                    self.accept_ghost_all();
                } else {
                    self.input.move_right();
                }
            }
            (KeyModifiers::NONE, KeyCode::Home) => self.input.move_home(),
            (KeyModifiers::NONE, KeyCode::End) => self.input.move_end(),

            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.input.is_on_first_line() {
                    self.navigate_history_up();
                } else {
                    self.input.move_up();
                }
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.input.is_on_last_line() {
                    self.navigate_history_down();
                } else {
                    self.input.move_down();
                }
            }

            (KeyModifiers::CONTROL, KeyCode::Backspace) => self.input.delete_word_back(),
            (KeyModifiers::CONTROL, KeyCode::Delete) => self.input.delete_word_forward(),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.delete_char_before();
            }
            (KeyModifiers::NONE, KeyCode::Delete) => self.input.delete_char_at(),

            (KeyModifiers::NONE, KeyCode::PageUp) => self.scroll_up(10),
            (KeyModifiers::NONE, KeyCode::PageDown) => self.scroll_down(10),
            (KeyModifiers::CONTROL, KeyCode::End) => self.scroll_to_bottom(),

            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
            }

            _ => {}
        }
    }

    /// Handle a key event when the agent picker popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_agents_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(selection) = self.agents_state.accept() {
                    match selection {
                        AgentSelection::Switch(index) => self.switch_to_session(index),
                        AgentSelection::FromDefinition(info) => {
                            self.create_session_from_definition(info);
                        }
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.agents_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.agents_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.agents_state.select_next(),
            _ => return false,
        }
        true
    }

    /// Handle a key event when the mentions autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_mentions_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(result) = self.mentions_state.accept() {
                    self.input.replace_range(result.range, &result.text);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.mentions_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.mentions_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.mentions_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.delete_char_before();
                let input = self.input.text().to_string();
                let cursor = self.input.cursor();
                let agents = self.mention_candidates();
                self.mentions_state.update(&input, cursor, &agents);
            }
            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                let input = self.input.text().to_string();
                let cursor = self.input.cursor();
                let agents = self.mention_candidates();
                self.mentions_state.update(&input, cursor, &agents);
            }
            _ => return false,
        }
        true
    }

    fn create_session_from_definition(&mut self, info: AgentDefinitionInfo) {
        let mut session = AgentSession::new(&info.name);
        session.definition = Some(info.clone());

        self.sessions.push(session);
        self.active_session = self.sessions.len() - 1;
        self.input.clear();

        let model_info = match (&info.provider, &info.model) {
            (Some(p), Some(m)) => format!(" using {m} ({p})"),
            _ => String::new(),
        };
        self.messages.push(AppMessage::System {
            content: format!("Agent '{}' activated{model_info}.", info.name),
        });
    }

    /// Switch to the agent session at the given index.
    pub fn switch_to_session(&mut self, index: usize) {
        if index < self.sessions.len() && index != self.active_session {
            self.active_session = index;
            self.messages.push(AppMessage::System {
                content: format!("Switched to agent '{}'.", self.sessions[index].name),
            });
        }
    }

    /// Handle pasted text — insert as-is for short pastes, or as a
    /// `[Paste #N: preview…]` token for large ones.
    fn handle_paste(&mut self, text: &str) {
        const PASTE_THRESHOLD: usize = 80;
        const PASTE_PREVIEW_CHARS: usize = 20;

        let char_count = text.chars().count();

        if char_count <= PASTE_THRESHOLD {
            self.input.insert_str(text);
        } else {
            self.paste_counter += 1;
            let n = self.paste_counter;
            self.pastes.insert(n, text.to_string());

            let prefix: String = text
                .chars()
                .take(PASTE_PREVIEW_CHARS)
                .map(|c| if c == '\n' { ' ' } else { c })
                .collect();
            let remaining = char_count - PASTE_PREVIEW_CHARS;
            let token = format!("[Paste #{n}: {prefix}\u{2026} +{remaining} chars]");
            self.input.insert_str(&token);
        }

        self.refresh_autocomplete();
        self.refresh_ghost_suggestion();
    }

    /// Submit the current input as a user message or slash command.
    fn submit_input(&mut self) {
        let text = self.input.take();
        if text.trim().is_empty() {
            return;
        }

        self.dismiss_all_autocomplete();

        // Expand paste and response references for the actual request text.
        // History stores the original (unexpanded) text so refs remain visible.
        let expanded = self.expand_paste_refs(&text);
        let expanded = self.expand_response_refs(&expanded);

        // Slash commands work in both modes
        if let Some((cmd, args)) = SlashCommand::parse(&text) {
            cmd.execute(self, args);
        } else {
            match self.mode {
                AppMode::Chat => {
                    if let Some(mention) = self.parse_agent_mention(&expanded) {
                        self.input_history
                            .push_tagged(text, AppMode::Chat);
                        self.execute_agent_mention(mention);
                    } else if let Some(cmd) = expanded.strip_prefix('!')
                        && !cmd.trim().is_empty()
                    {
                        let cmd = cmd.to_string();
                        self.input_history
                            .push_tagged(format!("!{cmd}"), AppMode::Chat);
                        self.spawn_shell_command(cmd);
                    } else {
                        self.input_history.push_tagged(text, AppMode::Chat);
                        self.submit_agent_message(expanded);
                    }
                }
                AppMode::Shell => {
                    self.input_history.push_tagged(text, AppMode::Shell);
                    self.spawn_shell_command(expanded);
                }
            }
        }

        // Pin scroll to bottom so the user sees their new message
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Build response candidates from existing exchanges.
    ///
    /// Returns `(exchange_number, truncated_preview)` tuples for exchanges that
    /// have a response, ordered newest first.
    pub fn response_candidates(&self) -> Vec<ResponseCandidate> {
        let mut exchange_num = 0usize;
        let mut candidates = Vec::new();

        for message in &self.messages {
            if let AppMessage::Exchange {
                kind,
                response: Some(resp),
                agent_index,
                ..
            } = message
            {
                exchange_num += 1;
                if resp.is_empty() {
                    continue;
                }
                // Label + color: agent name/color or "shell"/yellow
                let (label, color) = if *kind == ExchangeKind::Shell {
                    ("shell".to_string(), ExchangeKind::Shell.color())
                } else {
                    let name = agent_index
                        .and_then(|idx| self.sessions.get(idx))
                        .map_or_else(|| "chat".to_string(), |s| s.name.clone());
                    let c = agent_index
                        .map(AgentSession::color)
                        .unwrap_or(ExchangeKind::Chat.color());
                    (name, c)
                };
                // First line of response as preview (no truncation — renderer handles it)
                let preview = resp.lines().next().unwrap_or("").to_string();
                candidates.push(ResponseCandidate {
                    number: exchange_num,
                    label,
                    preview,
                    color,
                });
            } else if matches!(message, AppMessage::Exchange { .. }) {
                exchange_num += 1;
            }
        }

        candidates.reverse();
        candidates
    }

    /// Build mention candidates from existing sessions and discovered agents.
    pub fn mention_candidates(&self) -> Vec<MentionCandidate> {
        let mut candidates: Vec<MentionCandidate> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.active_session)
            .map(|(i, s)| MentionCandidate {
                name: s.name.clone(),
                color: AgentSession::color(i),
                definition: s.definition.clone(),
            })
            .collect();

        // Discovered agent definitions not yet in sessions
        let session_names: Vec<&str> = self.sessions.iter().map(|s| s.name.as_str()).collect();
        let definitions = discover_agents();
        for def in definitions {
            if session_names.contains(&def.name.as_str()) {
                continue;
            }
            candidates.push(MentionCandidate {
                name: def.name.clone(),
                color: Color::DarkGray,
                definition: Some(AgentDefinitionInfo {
                    name: def.name.clone(),
                    description: def.description.clone(),
                    model: def.model.clone(),
                    provider: def.provider.clone(),
                    source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                }),
            });
        }

        candidates
    }

    /// Build cancel candidates from running exchanges.
    ///
    /// Returns a list of `CancelCandidate` for exchanges with `Running` status,
    /// ordered by their position in `messages`.
    pub fn running_exchange_candidates(&self) -> Vec<CancelCandidate> {
        let mut exchange_num = 0usize;
        let mut candidates = Vec::new();

        for (msg_index, message) in self.messages.iter().enumerate() {
            if let AppMessage::Exchange {
                status: ExchangeStatus::Running,
                request,
                ..
            } = message
            {
                exchange_num += 1;
                let preview = truncate_preview(request, 40);
                candidates.push(CancelCandidate {
                    exchange_num,
                    msg_index,
                    request_preview: preview,
                });
            } else if matches!(message, AppMessage::Exchange { .. }) {
                exchange_num += 1;
            }
        }

        candidates
    }

    /// Cancel a running command or agent exchange by its message index.
    ///
    /// Searches `running_shell_commands` and all sessions'
    /// `running_agent_exchanges` for the matching `msg_index`, removes it,
    /// and marks the exchange as cancelled.
    pub fn cancel_by_msg_index(&mut self, msg_index: usize) {
        if let Some(pos) = self
            .running_shell_commands
            .iter()
            .position(|(idx, _)| *idx == msg_index)
        {
            let entry = self.running_shell_commands.remove(pos);
            Self::cancel_entry(&mut self.messages, entry);
            return;
        }

        for session in &mut self.sessions {
            if let Some(pos) = session
                .running_agent_exchanges
                .iter()
                .position(|(idx, _)| *idx == msg_index)
            {
                let (idx, exchange) = session.running_agent_exchanges.remove(pos);
                exchange.cancel();
                Self::mark_exchange_cancelled(&mut self.messages, idx);
                return;
            }
        }
    }

    /// Expand `[Response #N: ...]` references in text with full response content.
    ///
    /// Unknown references are left as-is.
    pub fn expand_response_refs(&self, text: &str) -> String {
        // Build a map of exchange_number → response text
        let mut exchange_num = 0usize;
        let mut response_map = Vec::new();
        for message in &self.messages {
            if let AppMessage::Exchange { response, .. } = message {
                exchange_num += 1;
                if let Some(resp) = response {
                    response_map.push((exchange_num, resp.as_str()));
                }
            }
        }

        let mut result = String::with_capacity(text.len());
        let mut remaining = text;

        while let Some(start) = remaining.find("[Response #") {
            // Copy everything before the match
            result.push_str(&remaining[..start]);

            let after_prefix = &remaining[start + "[Response #".len()..];

            // Parse the number (digits until ':' or ']')
            let num_end = after_prefix
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_prefix.len());
            let num_str = &after_prefix[..num_end];

            // Find the closing ']'
            if let Some(close) = after_prefix.find(']') {
                if let Ok(num) = num_str.parse::<usize>() {
                    // Look up the response
                    if let Some((_, resp)) = response_map.iter().find(|(n, _)| *n == num) {
                        result.push_str(resp);
                        remaining = &after_prefix[close + 1..];
                        continue;
                    }
                }
                // Unknown ref or parse failure — keep original text
                result.push_str(&remaining[start..=(start + "[Response #".len() + close)]);
                remaining = &after_prefix[close + 1..];
            } else {
                // No closing bracket — keep the rest as-is
                result.push_str(&remaining[start..]);
                remaining = "";
            }
        }

        result.push_str(remaining);
        result
    }

    /// Expand `[Paste #N: ...]` tokens in text with the stored paste contents.
    ///
    /// Unknown paste references are left as-is.
    fn expand_paste_refs(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut remaining = text;

        while let Some(start) = remaining.find("[Paste #") {
            result.push_str(&remaining[..start]);

            let after_prefix = &remaining[start + "[Paste #".len()..];

            let num_end = after_prefix
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_prefix.len());
            let num_str = &after_prefix[..num_end];

            if let Some(close) = after_prefix.find(']') {
                if let Ok(num) = num_str.parse::<usize>() {
                    if let Some(content) = self.pastes.get(&num) {
                        result.push_str(content);
                        remaining = &after_prefix[close + 1..];
                        continue;
                    }
                }
                result.push_str(&remaining[start..=(start + "[Paste #".len() + close)]);
                remaining = &after_prefix[close + 1..];
            } else {
                result.push_str(&remaining[start..]);
                remaining = "";
            }
        }

        result.push_str(remaining);
        result
    }

    /// Reset the active agent session: cancel its running exchanges, drop the
    /// agent handle (so a fresh session is created on the next submit), and
    /// clear all messages.
    pub fn reset_active_session(&mut self) {
        let session = &mut self.sessions[self.active_session];
        for (idx, exchange) in session.running_agent_exchanges.drain(..) {
            exchange.cancel();
            Self::mark_exchange_cancelled(&mut self.messages, idx);
        }
        session.agent = None;
        session.context_usage_percent = 0;

        self.messages.push(AppMessage::System {
            content: format!("Context cleared for agent `{}`", session.name),
        });
    }

    /// Reset everything: cancel all running work, drop all sessions, create a
    /// fresh default session, and clear all messages. Equivalent to restarting.
    pub fn reset_all(&mut self) {
        self.cancel_all_running();
        // Drop all sessions (and their agent handles)
        self.sessions.clear();
        let default_name = stencila_agents::convenience::resolve_default_agent_name("default");
        self.sessions.push(AgentSession::new(default_name));
        self.active_session = 0;
        self.messages.clear();
        self.messages.push(AppMessage::Welcome);
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Enter shell mode with a system message.
    pub fn enter_shell_mode(&mut self) {
        self.mode = AppMode::Shell;
        self.dismiss_all_autocomplete();
        self.messages.push(AppMessage::System {
            content: "Entering shell mode.".to_string(),
        });
    }

    /// Exit shell mode and return to chat mode with a system message.
    pub fn exit_shell_mode(&mut self) {
        self.mode = AppMode::Chat;
        self.dismiss_all_autocomplete();
        self.messages.push(AppMessage::System {
            content: "Exiting shell mode.".to_string(),
        });
    }

    /// Spawn a shell command as a background task.
    fn spawn_shell_command(&mut self, command: String) {
        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Running,
            request: command.clone(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: None,
        });
        let msg_index = self.messages.len() - 1;
        let running = crate::shell::spawn_command(command);
        self.running_shell_commands.push((msg_index, running));
    }

    /// Spawn an upgrade shell command, tracking it to clear the upgrade
    /// notification on success.
    pub fn spawn_upgrade_command(&mut self, command: String) {
        self.spawn_shell_command(command);
        self.upgrade_msg_index = Some(self.messages.len() - 1);
    }

    /// Cancel the most recent running command or agent exchange.
    ///
    /// Compares the highest message index across all sessions'
    /// `running_agent_exchanges` and `running_shell_commands`.
    fn cancel_most_recent_running(&mut self) {
        let cmd_max = self.running_shell_commands.last().map(|(idx, _)| *idx);
        let agent_max = self
            .sessions
            .iter()
            .filter_map(|s| s.running_agent_exchanges.last())
            .map(|(idx, _)| *idx)
            .max();

        match (cmd_max, agent_max) {
            (Some(c), Some(a)) if a > c => {
                // Find which session has that max agent exchange
                for session in &mut self.sessions {
                    if session
                        .running_agent_exchanges
                        .last()
                        .is_some_and(|(idx, _)| *idx == a)
                    {
                        if let Some((idx, exchange)) = session.running_agent_exchanges.pop() {
                            exchange.cancel();
                            Self::mark_exchange_cancelled(&mut self.messages, idx);
                        }
                        break;
                    }
                }
            }
            (Some(_), _) => {
                if let Some(entry) = self.running_shell_commands.pop() {
                    Self::cancel_entry(&mut self.messages, entry);
                }
            }
            (None, Some(_)) => {
                // Find which session has the max agent exchange
                if let Some(a) = agent_max {
                    for session in &mut self.sessions {
                        if session
                            .running_agent_exchanges
                            .last()
                            .is_some_and(|(idx, _)| *idx == a)
                        {
                            if let Some((idx, exchange)) = session.running_agent_exchanges.pop() {
                                exchange.cancel();
                                Self::mark_exchange_cancelled(&mut self.messages, idx);
                            }
                            break;
                        }
                    }
                }
            }
            (None, None) => {}
        }
    }

    /// Cancel all running shell commands and agent exchanges.
    pub fn cancel_all_running(&mut self) {
        for entry in self.running_shell_commands.drain(..) {
            Self::cancel_entry(&mut self.messages, entry);
        }
        for session in &mut self.sessions {
            for (idx, exchange) in session.running_agent_exchanges.drain(..) {
                exchange.cancel();
                Self::mark_exchange_cancelled(&mut self.messages, idx);
            }
        }
    }

    /// Cancel a single running command and mark its exchange as cancelled.
    fn cancel_entry(
        messages: &mut [AppMessage],
        (msg_index, running): (usize, RunningShellCommand),
    ) {
        let _command = running.cancel();
        Self::update_exchange_at(
            messages,
            msg_index,
            ExchangeStatus::Cancelled,
            Some("[cancelled]".to_string()),
            Some(-1),
        );
    }

    /// Update the exchange at `msg_index` in-place.
    ///
    /// Clears `response_segments` since this is used for cancellation and
    /// shell command completion which produce only plain text.
    fn update_exchange_at(
        messages: &mut [AppMessage],
        msg_index: usize,
        new_status: ExchangeStatus,
        response: Option<String>,
        exit_code: Option<i32>,
    ) {
        if let Some(AppMessage::Exchange {
            status,
            response: resp,
            response_segments,
            exit_code: ec,
            ..
        }) = messages.get_mut(msg_index)
        {
            *status = new_status;
            *resp = response;
            *response_segments = None;
            *ec = exit_code;
        }
    }

    /// Mark the exchange at `msg_index` as cancelled without replacing its response.
    fn mark_exchange_cancelled(messages: &mut [AppMessage], msg_index: usize) {
        if let Some(AppMessage::Exchange { status, .. }) = messages.get_mut(msg_index) {
            *status = ExchangeStatus::Cancelled;
        }
    }

    /// Submit a chat message to the active agent session.
    fn submit_agent_message(&mut self, text: String) {
        let session_idx = self.active_session;
        let session = &mut self.sessions[session_idx];

        // Lazily create the agent handle on first use
        if session.agent.is_none() {
            session.agent = AgentHandle::spawn(&session.name);
        }

        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Chat,
            status: ExchangeStatus::Running,
            request: text.clone(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: Some(session_idx),
        });
        let msg_index = self.messages.len() - 1;

        let exchange = self.sessions[session_idx]
            .agent
            .as_ref()
            .and_then(|agent| agent.submit(text));

        match exchange {
            Some(running) => {
                self.sessions[session_idx]
                    .running_agent_exchanges
                    .push((msg_index, running));
            }
            None => {
                // Agent task shut down — drop the dead handle so a fresh
                // session is created automatically on the next submit.
                self.sessions[session_idx].agent = None;

                Self::update_exchange_at(
                    &mut self.messages,
                    msg_index,
                    ExchangeStatus::Failed,
                    Some("Agent unavailable, will retry with a new session".to_string()),
                    None,
                );
            }
        }
    }

    /// Parse a `#agent-name [prompt]` pattern from the input text.
    ///
    /// Returns `None` if the text doesn't start with `#name` or the name doesn't
    /// match any known session or discovered agent.
    fn parse_agent_mention(&self, text: &str) -> Option<AgentMention> {
        let trimmed = text.trim_start();
        if !trimmed.starts_with('#') {
            return None;
        }

        // Extract the name: everything after `#` up to first whitespace
        let after_hash = &trimmed[1..];
        let (name, rest) = after_hash
            .split_once(char::is_whitespace)
            .unwrap_or((after_hash, ""));

        if name.is_empty() {
            return None;
        }

        // Validate the name matches a session or discovered agent
        let name_lower = name.to_ascii_lowercase();
        let is_session = self
            .sessions
            .iter()
            .any(|s| s.name.to_ascii_lowercase() == name_lower);

        let is_definition = if !is_session {
            discover_agents()
                .iter()
                .any(|d| d.name.to_ascii_lowercase() == name_lower)
        } else {
            false
        };

        if !is_session && !is_definition {
            return None;
        }

        let prompt = rest.trim();
        if prompt.is_empty() {
            // Just `#agent-name` — switch without sending
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: None,
                switch_back: false,
            })
        } else if prompt.ends_with('&') {
            // Prompt ends with `&` — send but don't switch back
            let prompt = prompt.trim_end_matches('&').trim_end().to_string();
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: if prompt.is_empty() {
                    None
                } else {
                    Some(prompt)
                },
                switch_back: false,
            })
        } else {
            // Normal prompt — send and switch back
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: Some(prompt.to_string()),
                switch_back: true,
            })
        }
    }

    /// Execute a parsed agent mention: switch, optionally send, optionally switch back.
    fn execute_agent_mention(&mut self, mention: AgentMention) {
        let original_session = self.active_session;

        // Find or create the target session
        let name_lower = mention.agent_name.to_ascii_lowercase();
        let target_idx = self
            .sessions
            .iter()
            .position(|s| s.name.to_ascii_lowercase() == name_lower);

        let target_idx = match target_idx {
            Some(idx) => {
                if idx != self.active_session {
                    self.switch_to_session(idx);
                }
                idx
            }
            None => {
                // Create from definition
                if let Some(def) = discover_agents()
                    .into_iter()
                    .find(|d| d.name.to_ascii_lowercase() == name_lower)
                {
                    let info = AgentDefinitionInfo {
                        name: def.name.clone(),
                        description: def.description.clone(),
                        model: def.model.clone(),
                        provider: def.provider.clone(),
                        source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                    };
                    self.create_session_from_definition(info);
                    self.active_session
                } else {
                    return;
                }
            }
        };

        // Send the prompt if present
        if let Some(prompt) = mention.prompt {
            self.submit_agent_message(prompt);
        }

        // Switch back if needed
        if mention.switch_back && target_idx != original_session {
            self.active_session = original_session;
        }
    }

    /// Poll all running commands for completion. Called on tick events.
    pub fn poll_running_commands(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // Collect completed indices (iterate in reverse so removal doesn't shift later indices)
        let mut completed = Vec::new();
        for (i, (_msg_index, running)) in self.running_shell_commands.iter_mut().enumerate() {
            if let Some(result) = running.try_take_result() {
                completed.push((i, result));
            }
        }

        // Process completions in reverse order to safely remove by index
        for (i, result) in completed.into_iter().rev() {
            let (msg_index, _running) = self.running_shell_commands.remove(i);
            let status = if result.exit_code == 0 {
                ExchangeStatus::Succeeded
            } else {
                ExchangeStatus::Failed
            };
            let was_upgrade =
                self.upgrade_msg_index == Some(msg_index) && status == ExchangeStatus::Succeeded;
            if self.upgrade_msg_index == Some(msg_index) {
                self.upgrade_msg_index = None;
            }

            Self::update_exchange_at(
                &mut self.messages,
                msg_index,
                status,
                Some(result.output),
                Some(result.exit_code),
            );

            if was_upgrade {
                self.upgrade_available = None;
                self.messages.push(AppMessage::System {
                    content: "Restart to use the new version.".to_string(),
                });
            }
        }
    }

    /// Poll all running agent exchanges for progress. Called on tick events.
    ///
    /// Iterates ALL sessions (not just active) since background agents may
    /// still be streaming.
    pub fn poll_running_agent_exchanges(&mut self) {
        for session in &mut self.sessions {
            let mut completed = Vec::new();
            for (i, (msg_index, exchange)) in session.running_agent_exchanges.iter().enumerate() {
                // Update context usage from the latest event data
                let pct = exchange.context_usage_percent();
                if pct > 0 {
                    session.context_usage_percent = pct;
                }

                if let Some(result) = exchange.try_take_result() {
                    completed.push((i, *msg_index, result));
                } else {
                    // Streaming update: refresh both plain text and segments
                    let segments = exchange.current_segments();
                    if !segments.is_empty() {
                        let text = crate::agent::plain_text_from_segments(&segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            *msg_index,
                            text,
                            segments,
                        );
                    }
                }
            }

            // Process completions in reverse order to safely remove by index
            for (i, msg_index, result) in completed.into_iter().rev() {
                session.running_agent_exchanges.remove(i);
                let status = if result.error.is_some() {
                    ExchangeStatus::Failed
                } else {
                    ExchangeStatus::Succeeded
                };
                let (response, segments) = if let Some(err) = result.error {
                    if result.text.is_empty() {
                        (Some(err), None)
                    } else {
                        (
                            Some(format!("{}\n\nError: {err}", result.text)),
                            Some(result.segments),
                        )
                    }
                } else if result.text.is_empty() {
                    (None, None)
                } else {
                    (Some(result.text), Some(result.segments))
                };
                Self::update_exchange_complete(
                    &mut self.messages,
                    msg_index,
                    status,
                    response,
                    segments,
                );
            }
        }
    }

    /// Drain pending log messages from the tracing channel and display them
    /// as system messages. Called on tick events.
    pub fn poll_log_events(&mut self) {
        while let Ok(msg) = self.log_receiver.try_recv() {
            self.messages.push(AppMessage::System { content: msg });
        }
    }

    /// Poll the background upgrade check. Called on tick events.
    ///
    /// If the check has completed with a newer version, stores it in
    /// `upgrade_available` for display in the welcome banner.
    pub fn poll_upgrade_check(&mut self) {
        let Some(mut handle) = self.upgrade_handle.take() else {
            return;
        };

        match (&mut handle).now_or_never() {
            Some(Ok(Some(version))) => {
                self.upgrade_available = Some(version);
            }
            Some(Ok(None) | Err(_)) => {
                // Check completed with no upgrade or the task panicked — discard handle
            }
            None => {
                // Not ready yet — put the handle back for next tick
                self.upgrade_handle = Some(handle);
            }
        }
    }

    /// Update response text and segments during streaming without changing status.
    fn update_exchange_streaming(
        messages: &mut [AppMessage],
        msg_index: usize,
        text: String,
        segments: Vec<ResponseSegment>,
    ) {
        if let Some(AppMessage::Exchange {
            response,
            response_segments,
            ..
        }) = messages.get_mut(msg_index)
        {
            *response = Some(text);
            *response_segments = Some(segments);
        }
    }

    /// Update an exchange on completion, setting status, response, segments, and exit code.
    fn update_exchange_complete(
        messages: &mut [AppMessage],
        msg_index: usize,
        new_status: ExchangeStatus,
        response: Option<String>,
        segments: Option<Vec<ResponseSegment>>,
    ) {
        if let Some(AppMessage::Exchange {
            status,
            response: resp,
            response_segments,
            exit_code,
            ..
        }) = messages.get_mut(msg_index)
        {
            *status = new_status;
            *resp = response;
            *response_segments = segments;
            *exit_code = None;
        }
    }

    /// Dismiss all autocomplete popups and ghost suggestion.
    fn dismiss_all_autocomplete(&mut self) {
        self.cancel_state.dismiss();
        self.agents_state.dismiss();
        self.mentions_state.dismiss();
        self.commands_state.dismiss();
        self.files_state.dismiss();
        self.history_state.dismiss();
        self.responses_state.dismiss();
        self.ghost_suggestion = None;
        self.ghost_is_truncated = false;
    }

    /// Re-filter all autocomplete states based on current input.
    fn refresh_autocomplete(&mut self) {
        self.history_state.update(self.input.text());
        self.commands_state.update(self.input.text());
        self.files_state
            .update(self.input.text(), self.input.cursor());
        let input = self.input.text().to_string();
        let cursor = self.input.cursor();
        let exchanges = self.response_candidates();
        self.responses_state.update(&input, cursor, &exchanges);
        let agents = self.mention_candidates();
        self.mentions_state.update(&input, cursor, &agents);
    }

    /// Navigate to the previous (older) history entry, filtered by current mode.
    ///
    /// When input is non-empty and not already in full-replacement navigation,
    /// cycles through prefix-matched entries as ghost suggestions (input stays
    /// unchanged). When input is empty, does full replacement (standard shell behavior).
    fn navigate_history_up(&mut self) {
        let current = self.input.text().to_string();
        if !current.is_empty() && self.input_history.is_at_draft() {
            // Ghost cycling: try the next older prefix match
            let next = self.ghost_nav_offset + 1;
            if self
                .input_history
                .prefix_match_nth(&current, self.mode, next)
                .is_some()
            {
                self.ghost_nav_offset = next;
            }
            // Ghost will be updated by refresh_ghost_suggestion via ghost_nav_offset
        } else {
            // Full replacement (empty input or already navigating)
            if let Some(entry) = self.input_history.navigate_up_filtered(&current, self.mode) {
                self.input.set_text(entry);
            }
        }
    }

    /// Navigate to the next (newer) history entry, or back to draft, filtered by current mode.
    ///
    /// When ghost cycling is active (offset > 0), decrements to show a newer match.
    /// Otherwise, does full replacement navigation.
    fn navigate_history_down(&mut self) {
        if self.ghost_nav_offset > 0 {
            // Ghost cycling: show a newer prefix match
            self.ghost_nav_offset -= 1;
            // Ghost will be updated by refresh_ghost_suggestion via ghost_nav_offset
        } else if !self.input_history.is_at_draft() {
            // Full replacement navigation
            if let Some(entry) = self.input_history.navigate_down_filtered(self.mode) {
                self.input.set_text(entry);
            }
        }
    }

    /// Whether any autocomplete popup is currently visible.
    fn any_popup_visible(&self) -> bool {
        self.cancel_state.is_visible()
            || self.agents_state.is_visible()
            || self.mentions_state.is_visible()
            || self.history_state.is_visible()
            || self.commands_state.is_visible()
            || self.files_state.is_visible()
            || self.responses_state.is_visible()
    }

    /// Recompute the ghost suggestion based on current input state.
    ///
    /// Shows ghost text when: input is non-empty, cursor is at end,
    /// input is single-line, and no autocomplete popup is visible.
    fn refresh_ghost_suggestion(&mut self) {
        let text = self.input.text();
        let at_end = self.input.cursor() == text.len();

        if text.is_empty() || !at_end || !self.input.is_single_line() || self.any_popup_visible() {
            self.ghost_suggestion = None;
            self.ghost_is_truncated = false;
            return;
        }

        let result = self
            .input_history
            .prefix_match_nth(text, self.mode, self.ghost_nav_offset)
            .and_then(|matched| {
                let suffix = &matched[text.len()..];
                if suffix.is_empty() || suffix.starts_with('\n') {
                    None
                } else {
                    Some(suffix.to_string())
                }
            });

        if let Some(suffix) = result {
            self.ghost_suggestion = Some(suffix);
            // ghost_is_truncated is computed at render time based on visual line count
            self.ghost_is_truncated = false;
        } else {
            self.ghost_suggestion = None;
            self.ghost_is_truncated = false;
        }
    }

    /// Accept the next whitespace-delimited word from the ghost suggestion.
    fn accept_ghost_word(&mut self) {
        let Some(ghost) = self.ghost_suggestion.take() else {
            return;
        };

        // Find the end of the next word in the ghost suffix.
        // Skip leading whitespace, then find the next whitespace boundary.
        let trimmed_start = ghost.len() - ghost.trim_start().len();
        let word_end = if trimmed_start >= ghost.len() {
            ghost.len()
        } else {
            ghost[trimmed_start..]
                .find(char::is_whitespace)
                .map_or(ghost.len(), |pos| trimmed_start + pos)
        };

        let word = &ghost[..word_end];
        self.input.insert_str(word);
        // refresh_ghost_suggestion will be called by handle_key after this
    }

    /// Accept the entire ghost suggestion.
    fn accept_ghost_all(&mut self) {
        let Some(ghost) = self.ghost_suggestion.take() else {
            return;
        };
        self.ghost_is_truncated = false;
        self.input.insert_str(&ghost);
    }

    /// Scroll up by the given number of lines.
    fn scroll_up(&mut self, lines: u16) {
        let max_top = self
            .total_message_lines
            .saturating_sub(self.visible_message_height);
        if self.scroll_pinned {
            // Unpin and set offset to current bottom position, then scroll up
            self.scroll_pinned = false;
            self.scroll_offset = max_top.saturating_sub(lines);
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        }
    }

    /// Scroll down by the given number of lines.
    fn scroll_down(&mut self, lines: u16) {
        let max_top = self
            .total_message_lines
            .saturating_sub(self.visible_message_height);
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
        // Re-pin when we've scrolled back to (or past) the bottom
        if self.scroll_offset >= max_top {
            self.scroll_pinned = true;
            self.scroll_offset = 0;
        }
    }

    /// Snap scroll back to the bottom and re-pin.
    fn scroll_to_bottom(&mut self) {
        self.scroll_pinned = true;
        self.scroll_offset = 0;
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
        Self::new(rx, None)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[tokio::test]
    async fn welcome_message() {
        let app = App::new_for_test();
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn ctrl_c_quits_in_chat_mode() {
        let mut app = App::new_for_test();
        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(quit);
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn ctrl_c_clears_input_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        // Type some text
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.input.text(), "hello");

        // Ctrl+C should clear input, not quit
        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!quit);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn ctrl_c_noop_on_empty_input_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!quit);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn ctrl_d_exits_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
    }

    #[tokio::test]
    async fn ctrl_d_noop_in_chat_mode() {
        let mut app = App::new_for_test();
        assert_eq!(app.mode, AppMode::Chat);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn typing_and_submit() {
        let mut app = App::new_for_test();

        // Type "hello"
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.input.text(), "hello");

        // Submit — without a tokio runtime the agent is unavailable,
        // so the exchange is created as Failed.
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.input.is_empty());
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(
            &app.messages[1],
            AppMessage::Exchange { kind: ExchangeKind::Chat, request, .. } if request == "hello"
        ));
    }

    #[tokio::test]
    async fn empty_submit_ignored() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[tokio::test]
    async fn ctrl_l_resets_active_session() {
        let mut app = App::new_for_test();

        // Type and submit a message
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.messages.len(), 2);

        // Ctrl+L resets to welcome
        app.handle_event(&key_event(KeyCode::Char('l'), KeyModifiers::CONTROL));
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn shift_enter_inserts_newline() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('b'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "a\nb");
    }

    #[tokio::test]
    async fn up_down_moves_cursor_in_multiline() {
        let mut app = App::new_for_test();
        // Paste multiline text: "abc\ndef"
        app.handle_event(&Event::Paste("abc\ndef".to_string()));
        // Cursor at end (pos 7, line 1, col 3)
        assert_eq!(app.input.cursor(), 7);

        // Up moves to same column on previous line
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.cursor(), 3); // end of "abc"

        // Down moves back
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.cursor(), 7); // end of "def"
    }

    #[tokio::test]
    async fn alt_enter_inserts_newline() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        app.handle_event(&key_event(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "x\ny");
    }

    #[tokio::test]
    async fn paste_inserts_without_submit() {
        let mut app = App::new_for_test();
        app.handle_event(&Event::Paste("hello\nworld".to_string()));
        assert_eq!(app.input.text(), "hello\nworld");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[tokio::test]
    async fn paste_short_inserted_verbatim() {
        let mut app = App::new_for_test();
        app.handle_event(&Event::Paste("short text".to_string()));
        assert_eq!(app.input.text(), "short text");
        assert!(app.pastes.is_empty());
    }

    #[tokio::test]
    async fn paste_large_inserts_token() {
        let mut app = App::new_for_test();
        let long_text = "a".repeat(200);
        app.handle_event(&Event::Paste(long_text.clone()));
        // Buffer contains the token, not the raw text
        let input = app.input.text().to_string();
        assert!(input.starts_with("[Paste #1: "));
        assert!(input.contains("+180 chars]"));
        // Full text is stored in the pastes map
        assert_eq!(app.pastes.get(&1).unwrap(), &long_text);
    }

    #[tokio::test]
    async fn paste_token_expanded_on_submit() {
        let mut app = App::new_for_test();
        let long_text = "x".repeat(200);
        app.handle_event(&Event::Paste(long_text.clone()));
        // Expand paste refs returns the full text
        let input = app.input.text().to_string();
        let expanded = app.expand_paste_refs(&input);
        assert_eq!(expanded, long_text);
    }

    #[tokio::test]
    async fn paste_multiple_tokens() {
        let mut app = App::new_for_test();
        let text1 = "a".repeat(100);
        let text2 = "b".repeat(150);
        app.handle_event(&Event::Paste(text1.clone()));
        app.handle_event(&Event::Paste(text2.clone()));
        let input = app.input.text().to_string();
        assert!(input.contains("[Paste #1:"));
        assert!(input.contains("[Paste #2:"));
        assert_eq!(app.pastes.len(), 2);
        // Both expand correctly
        let expanded = app.expand_paste_refs(&input);
        assert!(expanded.contains(&text1));
        assert!(expanded.contains(&text2));
    }

    #[tokio::test]
    async fn paste_token_newlines_in_preview_replaced() {
        let mut app = App::new_for_test();
        let long_text = format!("line1\nline2\n{}", "x".repeat(100));
        app.handle_event(&Event::Paste(long_text));
        let input = app.input.text().to_string();
        // The token itself should not contain newlines
        assert!(!input.contains('\n'));
    }

    #[tokio::test]
    async fn history_up_down() {
        let mut app = App::new_for_test();

        // Submit a few messages
        for msg in ["first", "second", "third"] {
            for c in msg.chars() {
                app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
            }
            app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        }

        // Navigate up through history
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "third");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "second");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "first");

        // Navigate back down
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "second");
    }

    #[tokio::test]
    async fn history_preserves_draft() {
        let mut app = App::new_for_test();

        // Submit two entries with the same prefix
        for text in ["old first", "old second"] {
            for c in text.chars() {
                app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
            }
            app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        }

        // Type a prefix
        for c in "old".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Default ghost shows " second" (most recent prefix match)
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" second"));

        // Up cycles ghost to the next older match — input stays "old"
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "old");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" first"));

        // Down cycles ghost back to the most recent match
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "old");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" second"));
    }

    #[tokio::test]
    async fn ctrl_u_deletes_to_line_start() {
        let mut app = App::new_for_test();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[tokio::test]
    async fn ctrl_k_deletes_to_line_end() {
        let mut app = App::new_for_test();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Home, KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[tokio::test]
    async fn scroll_bounds() {
        let mut app = App::new_for_test();
        // Simulate a frame that rendered 20 total lines with 10 visible
        app.total_message_lines = 20;
        app.visible_message_height = 10;
        // max_top = 20 - 10 = 10

        assert!(app.scroll_pinned);

        // Scroll up 5: unpins, offset = max_top(10) - 5 = 5
        app.scroll_up(5);
        assert!(!app.scroll_pinned);
        assert_eq!(app.scroll_offset, 5);

        // Scroll up 10 more: 5 - 10 saturates to 0 (top of content)
        app.scroll_up(10);
        assert!(!app.scroll_pinned);
        assert_eq!(app.scroll_offset, 0);

        // Scroll down 3: 0 + 3 = 3, still not at bottom (10)
        app.scroll_down(3);
        assert!(!app.scroll_pinned);
        assert_eq!(app.scroll_offset, 3);

        // Scroll down past bottom: re-pins
        app.scroll_down(100);
        assert!(app.scroll_pinned);
        assert_eq!(app.scroll_offset, 0);
    }

    #[tokio::test]
    async fn scroll_stays_stable_during_streaming() {
        let mut app = App::new_for_test();
        app.total_message_lines = 30;
        app.visible_message_height = 10;

        // Scroll up to unpin
        app.scroll_up(5);
        assert!(!app.scroll_pinned);
        assert_eq!(app.scroll_offset, 15); // max_top(20) - 5

        // Simulate streaming: total lines grow but offset stays the same
        app.total_message_lines = 50;
        assert_eq!(app.scroll_offset, 15); // unchanged — view stays put

        // Esc snaps back to bottom
        app.scroll_to_bottom();
        assert!(app.scroll_pinned);
        assert_eq!(app.scroll_offset, 0);
    }

    // --- Slash command integration tests ---

    #[tokio::test]
    async fn slash_help_shows_system_message() {
        let mut app = App::new_for_test();
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.input.is_empty());
        // Welcome + help output
        assert_eq!(app.messages.len(), 2);
        assert!(
            matches!(&app.messages[1], AppMessage::System { content } if content.contains("/help"))
        );
    }

    #[tokio::test]
    async fn slash_clear_resets_active_session() {
        let mut app = App::new_for_test();
        for c in "/clear".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn slash_quit_exits() {
        let mut app = App::new_for_test();
        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[tokio::test]
    async fn unknown_slash_treated_as_user_message() {
        let mut app = App::new_for_test();
        for c in "/unknown".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Not a command, so it's a user message (sent to agent)
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(
            &app.messages[1],
            AppMessage::Exchange { kind: ExchangeKind::Chat, request, .. } if request == "/unknown"
        ));
    }

    #[tokio::test]
    async fn bare_dollar_treated_as_user_message() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // "$" should be treated as a normal chat message, not silently discarded
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(
            &app.messages[1],
            AppMessage::Exchange { kind: ExchangeKind::Chat, request, .. } if request == "$"
        ));
    }

    #[tokio::test]
    async fn ctrl_s_enters_shell_mode() {
        let mut app = App::new_for_test();
        assert_eq!(app.mode, AppMode::Chat);

        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[tokio::test]
    async fn ctrl_s_noop_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        // Ctrl+S in shell mode should not do anything special
        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Shell);
    }

    // --- Autocomplete integration tests ---

    #[tokio::test]
    async fn autocomplete_shows_on_slash() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn autocomplete_narrows_on_typing() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        let all_count = app.commands_state.candidates().len();

        app.handle_event(&key_event(KeyCode::Char('h'), KeyModifiers::NONE));
        assert!(app.commands_state.candidates().len() < all_count);
    }

    #[tokio::test]
    async fn autocomplete_hides_on_esc() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn autocomplete_tab_accepts() {
        let mut app = App::new_for_test();
        // Type "/he" — matches /help and /history
        for c in "/he".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert!(app.commands_state.is_visible());

        // Tab accepts the first candidate
        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
        // Input should be one of the matching commands
        let text = app.input.text().to_string();
        assert!(text == "/help" || text == "/history");
    }

    #[tokio::test]
    async fn autocomplete_up_down_navigates() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
        assert_eq!(app.commands_state.selected(), 0);

        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.commands_state.selected(), 1);

        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.commands_state.selected(), 0);
    }

    #[tokio::test]
    async fn autocomplete_enter_accepts_and_submits() {
        let mut app = App::new_for_test();
        // Type "/hel" — matches /help only
        for c in "/hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert!(app.commands_state.is_visible());

        // Enter should accept and submit
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
        assert!(app.input.is_empty());
        // Should have executed /help
        assert!(app.messages.len() >= 2);
    }

    #[tokio::test]
    async fn autocomplete_dismissed_on_submit() {
        let mut app = App::new_for_test();
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    // --- Shell mode tests ---

    #[tokio::test]
    async fn slash_shell_enters_shell_mode() {
        let mut app = App::new_for_test();
        for c in "/shell".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.mode, AppMode::Shell);
        // System message about entering shell mode
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::System { content } if content.contains("shell mode")
        )));
    }

    #[tokio::test]
    async fn slash_exit_in_shell_mode_returns_to_chat() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        for c in "/exit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn slash_quit_quits_from_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[tokio::test]
    async fn autocomplete_works_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn enter_and_exit_shell_mode_messages() {
        let mut app = App::new_for_test();
        let initial_count = app.messages.len();

        app.enter_shell_mode();
        assert_eq!(app.messages.len(), initial_count + 1);
        assert!(matches!(
            &app.messages[initial_count],
            AppMessage::System { content } if content.contains("Entering shell mode")
        ));

        app.exit_shell_mode();
        assert_eq!(app.messages.len(), initial_count + 2);
        assert!(matches!(
            &app.messages[initial_count + 1],
            AppMessage::System { content } if content.contains("Exiting shell mode")
        ));
    }

    // --- Ghost suggestion tests ---

    /// Helper: set up an app with history entries and type a prefix.
    fn app_with_history_and_prefix(entries: &[&str], prefix: &str) -> App {
        let mut app = App::new_for_test();
        for &entry in entries {
            app.input_history
                .push_tagged(entry.to_string(), AppMode::Chat);
        }
        for c in prefix.chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app
    }

    #[tokio::test]
    async fn ghost_appears_on_prefix_match() {
        let app = app_with_history_and_prefix(&["hello world"], "hel");
        assert_eq!(app.ghost_suggestion.as_deref(), Some("lo world"));
    }

    #[tokio::test]
    async fn ghost_clears_when_input_diverges() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Type 'x' — "helx" no longer matches "hello world"
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn ghost_clears_when_cursor_not_at_end() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Move cursor left — no longer at end
        app.handle_event(&key_event(KeyCode::Left, KeyModifiers::NONE));
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn ghost_clears_when_multiline() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Insert newline — input becomes multiline
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn ghost_clears_when_popup_visible() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("/help me".to_string(), AppMode::Chat);

        // Type "/" — triggers command autocomplete popup
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
        // Ghost should be None because popup is visible
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn ghost_not_shown_for_empty_input() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello".to_string(), AppMode::Chat);
        // No typing — ghost should not appear
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn ghost_not_shown_for_exact_match() {
        let app = app_with_history_and_prefix(&["hello"], "hello");
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn tab_accepts_ghost_word() {
        let mut app = app_with_history_and_prefix(&["cargo test --release"], "cargo");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" test --release"));

        // Tab accepts " test"
        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test");
        // Ghost should refresh to " --release"
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" --release"));
    }

    #[tokio::test]
    async fn multiple_tabs_accept_word_by_word() {
        let mut app = app_with_history_and_prefix(&["cargo test --release"], "cargo");

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test");

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test --release");

        // No more ghost text
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn right_at_end_accepts_all_ghost() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        app.handle_event(&key_event(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "hello world");
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn right_in_middle_moves_cursor() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        // Move cursor left first
        app.handle_event(&key_event(KeyCode::Left, KeyModifiers::NONE));
        assert!(app.ghost_suggestion.is_none());

        // Right should move cursor, not accept ghost (there is none)
        let cursor_before = app.input.cursor();
        app.handle_event(&key_event(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.input.cursor(), cursor_before + 1);
        assert_eq!(app.input.text(), "hel");
    }

    #[tokio::test]
    async fn ghost_multiline_history_shows_full_suffix() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Chat);

        for c in "hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        // Ghost contains the full suffix including newlines; visual truncation
        // is handled at render time.
        assert_eq!(
            app.ghost_suggestion.as_deref(),
            Some("lo world\nsecond line")
        );
    }

    #[tokio::test]
    async fn ghost_multiline_exact_first_line_shows_nothing() {
        let mut app = App::new_for_test();
        // History entry where the first line is an exact match for the typed input
        app.input_history
            .push_tagged("foo\nbar".to_string(), AppMode::Chat);

        for c in "foo".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        // No useful ghost to show — suffix starts with newline
        assert!(app.ghost_suggestion.is_none());
    }

    #[tokio::test]
    async fn accept_all_ghost_multiline() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Chat);

        for c in "hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(
            app.ghost_suggestion.as_deref(),
            Some("lo world\nsecond line")
        );

        // Right accepts all — inserts the full ghost text including newlines
        app.handle_event(&key_event(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "hello world\nsecond line");
    }

    // --- Response autocomplete tests ---

    /// Helper: create an app with some exchanges that have responses.
    fn app_with_exchanges() -> App {
        let mut app = App::new_for_test();
        // Exchange 1: has response
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Succeeded,
            request: "echo hello".to_string(),
            response: Some("hello".to_string()),
            response_segments: None,
            exit_code: Some(0),
            agent_index: None,
        });
        // Exchange 2: no response yet
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Chat,
            status: ExchangeStatus::Running,
            request: "what is rust".to_string(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: Some(0),
        });
        // Exchange 3: has response
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Succeeded,
            request: "ls -la".to_string(),
            response: Some("total 42\ndrwxr-xr-x 2 user user 4096".to_string()),
            response_segments: None,
            exit_code: Some(0),
            agent_index: None,
        });
        app
    }

    #[tokio::test]
    async fn response_candidates_returns_correct_list() {
        let app = app_with_exchanges();
        let candidates = app.response_candidates();
        // Exchange 1 and 3 have responses; newest first
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].number, 3); // newest first
        assert_eq!(candidates[1].number, 1);
    }

    #[tokio::test]
    async fn dollar_triggers_response_autocomplete() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 2);
    }

    #[tokio::test]
    async fn dollar_with_digit_filters_responses() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('1'), KeyModifiers::NONE));
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 1);
        assert_eq!(app.responses_state.candidates()[0].number, 1);
    }

    #[tokio::test]
    async fn response_esc_dismisses() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.responses_state.is_visible());
    }

    #[tokio::test]
    async fn response_tab_accepts() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert!(!app.responses_state.is_visible());
        // Input should contain [Response #N: ...]
        assert!(app.input.text().contains("[Response #"));
    }

    #[tokio::test]
    async fn expand_response_refs_replaces_known() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("see [Response #1: hello...]");
        assert_eq!(expanded, "see hello");
    }

    #[tokio::test]
    async fn expand_response_refs_leaves_unknown() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("see [Response #99: unknown...]");
        assert_eq!(expanded, "see [Response #99: unknown...]");
    }

    #[tokio::test]
    async fn expand_response_refs_no_refs() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("plain text");
        assert_eq!(expanded, "plain text");
    }

    #[tokio::test]
    async fn expand_response_refs_multiple() {
        let app = app_with_exchanges();
        let expanded =
            app.expand_response_refs("[Response #1: hello...] and [Response #3: total 42...]");
        assert_eq!(expanded, "hello and total 42\ndrwxr-xr-x 2 user user 4096");
    }

    #[tokio::test]
    async fn truncate_preview_short() {
        assert_eq!(truncate_preview("hello", 50), "hello...");
    }

    #[tokio::test]
    async fn truncate_preview_long() {
        let long = "a".repeat(100);
        let preview = truncate_preview(&long, 50);
        assert_eq!(preview.len(), 53); // 50 chars + "..."
        assert!(preview.ends_with("..."));
    }

    #[tokio::test]
    async fn truncate_preview_multiline() {
        assert_eq!(truncate_preview("line one\nline two", 50), "line one...");
    }

    // --- Multi-agent tests ---

    #[tokio::test]
    async fn default_session_exists() {
        let app = App::new_for_test();
        assert_eq!(app.sessions.len(), 1);
        let expected = stencila_agents::convenience::resolve_default_agent_name("default");
        assert_eq!(app.sessions[0].name, expected);
        assert_eq!(app.active_session, 0);
    }

    #[tokio::test]
    async fn switch_to_session() {
        let mut app = App::new_for_test();
        app.sessions.push(AgentSession::new("test-agent"));
        let initial = app.messages.len();

        app.switch_to_session(1);
        assert_eq!(app.active_session, 1);
        assert_eq!(app.messages.len(), initial + 1);
        assert!(matches!(
            &app.messages[initial],
            AppMessage::System { content } if content.contains("test-agent")
        ));
    }

    #[tokio::test]
    async fn switch_to_same_session_noop() {
        let mut app = App::new_for_test();
        let initial = app.messages.len();
        app.switch_to_session(0);
        assert_eq!(app.messages.len(), initial); // no message added
    }

    #[tokio::test]
    async fn ctrl_a_cycles_agents() {
        let mut app = App::new_for_test();
        app.sessions.push(AgentSession::new("agent-a"));
        app.sessions.push(AgentSession::new("agent-b"));

        // Ctrl+A cycles forward
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(app.active_session, 1);

        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(app.active_session, 2);

        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(app.active_session, 0); // wraps around
    }

    #[tokio::test]
    async fn ctrl_a_noop_single_session() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(app.active_session, 0);
    }

    #[tokio::test]
    async fn exchange_has_agent_index() {
        let mut app = App::new_for_test();
        // Submit a chat message (agent will be unavailable in test, which is fine)
        for c in "test".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));

        // The last exchange should have agent_index = Some(0)
        let exchange = app
            .messages
            .iter()
            .find(|m| matches!(m, AppMessage::Exchange { .. }));
        assert!(exchange.is_some());
        if let Some(AppMessage::Exchange { agent_index, .. }) = exchange {
            assert_eq!(*agent_index, Some(0));
        }
    }
}
