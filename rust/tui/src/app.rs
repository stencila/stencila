use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::style::Color;
use tokio::sync::mpsc;

use crate::{
    agent::{AgentHandle, RunningAgentExchange},
    autocomplete::cancel::CancelCandidate,
    autocomplete::models::ModelCandidate,
    autocomplete::{
        CancelState, CommandsState, FilesState, HistoryState, ModelsState, ResponsesState,
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
        /// Shell exit code (only meaningful for Shell kind).
        exit_code: Option<i32>,
    },
    /// A system/informational message (mode transitions, slash command output, etc.).
    System { content: String },
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
    /// Model picker popup state.
    pub models_state: ModelsState,

    /// Ghost suggestion suffix (the part beyond what's typed, insertable text only).
    /// Shown as dim inline text for fish-shell-style history completion.
    pub ghost_suggestion: Option<String>,
    /// Whether the ghost suggestion was truncated from a multiline history entry.
    /// When true, the UI appends a dim "…" indicator after the ghost text.
    pub ghost_is_truncated: bool,
    /// Offset for cycling ghost suggestions via Up/Down arrows.
    /// 0 = most recent prefix match (default), incremented by Up, decremented by Down.
    ghost_nav_offset: usize,

    /// Agent handle for submitting chat messages to the LLM.
    /// Created lazily on first chat submit.
    agent: Option<AgentHandle>,

    /// The selected model override (provider, `model_id`).
    /// `None` means use the default.
    selected_model: Option<(String, String)>,

    /// Agent exchanges currently running in the background.
    /// Each entry is `(message_index, running_exchange)` linking to the exchange in `messages`.
    pub running_agent_exchanges: Vec<(usize, RunningAgentExchange)>,

    /// Shell commands currently running in the background.
    /// Each entry is `(message_index, running_command)` linking to the exchange in `messages`.
    pub running_shell_commands: Vec<(usize, RunningShellCommand)>,

    /// Tick counter for pulsating sidebar animation on running exchanges.
    pub tick_count: u32,

    /// Receiver for tracing log messages captured by the TUI logging layer.
    log_receiver: mpsc::UnboundedReceiver<String>,

    /// Scroll offset for the message area (lines from the bottom).
    pub scroll_offset: u16,
    /// Total lines rendered in the last frame's message area (set by `ui::render`).
    pub total_message_lines: u16,
    /// Visible height of the message area in the last frame (set by `ui::render`).
    pub visible_message_height: u16,
}

impl App {
    /// Create a new App with a welcome banner.
    pub fn new(log_receiver: mpsc::UnboundedReceiver<String>) -> Self {
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
            models_state: ModelsState::new(),
            ghost_suggestion: None,
            ghost_is_truncated: false,
            ghost_nav_offset: 0,
            running_shell_commands: Vec::new(),
            agent: None,
            selected_model: None,
            running_agent_exchanges: Vec::new(),
            tick_count: 0,
            log_receiver,
            scroll_offset: 0,
            total_message_lines: 0,
            visible_message_height: 0,
        }
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
        !self.running_shell_commands.is_empty() || !self.running_agent_exchanges.is_empty()
    }

    /// Dispatch a key event.
    fn handle_key(&mut self, key: &KeyEvent) {
        let consumed = (self.cancel_state.is_visible() && self.handle_cancel_autocomplete(key))
            || (self.models_state.is_visible() && self.handle_models_autocomplete(key))
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

    /// Handle a key event when the model picker popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_models_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(candidate) = self.models_state.accept() {
                    self.set_model(&candidate);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.models_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.models_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.models_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.delete_char_before();
                self.models_state.update(self.input.text());
            }
            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                self.models_state.update(self.input.text());
            }
            _ => return false,
        }
        true
    }

    /// Switch to a new model, resetting the agent session.
    fn set_model(&mut self, candidate: &ModelCandidate) {
        self.selected_model = Some((candidate.provider.clone(), candidate.model_id.clone()));
        self.cancel_all_running();
        self.agent = None;
        self.input.clear();
        self.messages.push(AppMessage::System {
            content: format!(
                "Switched to {} ({}). New session started.",
                candidate.display_name, candidate.provider
            ),
        });
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
                if let Some(result) = self.files_state.accept_tab() {
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
                if let Some(result) = self.files_state.accept_enter() {
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
                self.clear_messages();
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

            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                self.scroll_offset = 0;
            }

            _ => {}
        }
    }

    /// Handle pasted text — insert as-is without triggering submit.
    fn handle_paste(&mut self, text: &str) {
        self.input.insert_str(text);
        self.scroll_offset = 0;
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

        // Expand response references for the actual request text.
        // History stores the original (unexpanded) text so refs remain visible.
        let expanded = self.expand_response_refs(&text);

        // Slash commands work in both modes
        if let Some((cmd, args)) = SlashCommand::parse(&text) {
            cmd.execute(self, args);
        } else {
            match self.mode {
                AppMode::Chat => {
                    // Check for one-off shell command with $ prefix.
                    // Bare "$" or "$   " falls through to normal chat message.
                    if let Some(cmd) = expanded.strip_prefix('$')
                        && !cmd.trim().is_empty()
                    {
                        let cmd = cmd.to_string();
                        self.input_history
                            .push_tagged(format!("${cmd}"), AppMode::Chat);
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

        // Reset scroll to bottom
        self.scroll_offset = 0;
    }

    /// Build response candidates from existing exchanges.
    ///
    /// Returns `(exchange_number, truncated_preview)` tuples for exchanges that
    /// have a response, ordered newest first.
    pub fn response_candidates(&self) -> Vec<(usize, String)> {
        let mut exchange_num = 0usize;
        let mut candidates = Vec::new();

        for message in &self.messages {
            if let AppMessage::Exchange {
                response: Some(resp),
                ..
            } = message
            {
                exchange_num += 1;
                let preview = truncate_preview(resp, 50);
                candidates.push((exchange_num, preview));
            } else if matches!(message, AppMessage::Exchange { .. }) {
                exchange_num += 1;
            }
        }

        candidates.reverse();
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
    /// Searches both `running_commands` and `running_agent_exchanges` for
    /// the matching `msg_index`, removes it, and marks the exchange as cancelled.
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

        if let Some(pos) = self
            .running_agent_exchanges
            .iter()
            .position(|(idx, _)| *idx == msg_index)
        {
            let (idx, exchange) = self.running_agent_exchanges.remove(pos);
            exchange.cancel();
            Self::update_exchange_at(
                &mut self.messages,
                idx,
                ExchangeStatus::Failed,
                Some("[cancelled]".to_string()),
                None,
            );
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

    /// Clear all messages and cancel anything running.
    pub fn clear_messages(&mut self) {
        self.cancel_all_running();
        self.messages.clear();
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
            exit_code: None,
        });
        let msg_index = self.messages.len() - 1;
        let running = crate::shell::spawn_command(command);
        self.running_shell_commands.push((msg_index, running));
    }

    /// Cancel the most recent running command or agent exchange.
    ///
    /// Compares the highest message index across both `running_commands`
    /// and `running_agent_exchanges` to cancel whichever started most recently.
    fn cancel_most_recent_running(&mut self) {
        let cmd_max = self.running_shell_commands.last().map(|(idx, _)| *idx);
        let agent_max = self.running_agent_exchanges.last().map(|(idx, _)| *idx);

        match (cmd_max, agent_max) {
            (Some(c), Some(a)) if a > c => {
                if let Some((idx, exchange)) = self.running_agent_exchanges.pop() {
                    exchange.cancel();
                    Self::update_exchange_at(
                        &mut self.messages,
                        idx,
                        ExchangeStatus::Failed,
                        Some("[cancelled]".to_string()),
                        None,
                    );
                }
            }
            (Some(_), _) => {
                if let Some(entry) = self.running_shell_commands.pop() {
                    Self::cancel_entry(&mut self.messages, entry);
                }
            }
            (None, Some(_)) => {
                if let Some((idx, exchange)) = self.running_agent_exchanges.pop() {
                    exchange.cancel();
                    Self::update_exchange_at(
                        &mut self.messages,
                        idx,
                        ExchangeStatus::Failed,
                        Some("[cancelled]".to_string()),
                        None,
                    );
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
        for (idx, exchange) in self.running_agent_exchanges.drain(..) {
            exchange.cancel();
            Self::update_exchange_at(
                &mut self.messages,
                idx,
                ExchangeStatus::Failed,
                Some("[cancelled]".to_string()),
                None,
            );
        }
    }

    /// Cancel a single running command and mark its exchange as failed.
    fn cancel_entry(messages: &mut [AppMessage], (msg_index, running): (usize, RunningShellCommand)) {
        let _command = running.cancel();
        Self::update_exchange_at(
            messages,
            msg_index,
            ExchangeStatus::Failed,
            Some("[cancelled]".to_string()),
            Some(-1),
        );
    }

    /// Update the exchange at `msg_index` in-place.
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
            exit_code: ec,
            ..
        }) = messages.get_mut(msg_index)
        {
            *status = new_status;
            *resp = response;
            *ec = exit_code;
        }
    }

    /// Submit a chat message to the agent session.
    fn submit_agent_message(&mut self, text: String) {
        // Lazily create the agent handle on first use
        if self.agent.is_none() {
            self.agent = AgentHandle::spawn(self.selected_model.clone());
        }

        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Chat,
            status: ExchangeStatus::Running,
            request: text.clone(),
            response: None,
            exit_code: None,
        });
        let msg_index = self.messages.len() - 1;

        let exchange = self.agent.as_ref().and_then(|agent| agent.submit(text));

        match exchange {
            Some(running) => {
                self.running_agent_exchanges.push((msg_index, running));
            }
            None => {
                // No runtime or agent task shut down
                Self::update_exchange_at(
                    &mut self.messages,
                    msg_index,
                    ExchangeStatus::Failed,
                    Some("Agent session unavailable".to_string()),
                    None,
                );
            }
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
            Self::update_exchange_at(
                &mut self.messages,
                msg_index,
                status,
                Some(result.output),
                Some(result.exit_code),
            );
        }
    }

    /// Poll all running agent exchanges for progress. Called on tick events.
    pub fn poll_running_agent_exchanges(&mut self) {
        let mut completed = Vec::new();
        for (i, (msg_index, exchange)) in self.running_agent_exchanges.iter().enumerate() {
            if let Some(result) = exchange.try_take_result() {
                completed.push((i, *msg_index, result));
            } else {
                // Streaming update: refresh the response text
                let text = exchange.current_text();
                if !text.is_empty() {
                    Self::update_exchange_response(&mut self.messages, *msg_index, text);
                }
            }
        }

        // Process completions in reverse order to safely remove by index
        for (i, msg_index, result) in completed.into_iter().rev() {
            self.running_agent_exchanges.remove(i);
            let status = if result.error.is_some() {
                ExchangeStatus::Failed
            } else {
                ExchangeStatus::Succeeded
            };
            let response = if let Some(err) = result.error {
                if result.text.is_empty() {
                    Some(err)
                } else {
                    Some(format!("{}\n\nError: {err}", result.text))
                }
            } else if result.text.is_empty() {
                None
            } else {
                Some(result.text)
            };
            Self::update_exchange_at(&mut self.messages, msg_index, status, response, None);
        }
    }

    /// Drain pending log messages from the tracing channel and display them
    /// as system messages. Called on tick events.
    pub fn poll_log_events(&mut self) {
        let mut received = false;
        while let Ok(msg) = self.log_receiver.try_recv() {
            self.messages.push(AppMessage::System { content: msg });
            received = true;
        }
        if received {
            self.scroll_offset = 0;
        }
    }

    /// Update only the response field of an exchange without changing status.
    ///
    /// Used for streaming updates while the exchange is still running.
    fn update_exchange_response(messages: &mut [AppMessage], msg_index: usize, text: String) {
        if let Some(AppMessage::Exchange { response, .. }) = messages.get_mut(msg_index) {
            *response = Some(text);
        }
    }

    /// Dismiss all autocomplete popups and ghost suggestion.
    fn dismiss_all_autocomplete(&mut self) {
        self.cancel_state.dismiss();
        self.models_state.dismiss();
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
        let exchanges = self.response_candidates();
        let input = self.input.text().to_string();
        let cursor = self.input.cursor();
        self.responses_state.update(&input, cursor, &exchanges);
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
            || self.models_state.is_visible()
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
                match suffix.find('\n') {
                    // Suffix starts with newline — first line is exact match,
                    // nothing useful to show as ghost text.
                    Some(0) => None,
                    // Truncate at first newline, flag as truncated for UI "…" indicator.
                    Some(pos) => Some((suffix[..pos].to_string(), true)),
                    // Single-line match — use full suffix.
                    None => Some((suffix.to_string(), false)),
                }
            });

        if let Some((suffix, truncated)) = result {
            self.ghost_suggestion = Some(suffix);
            self.ghost_is_truncated = truncated;
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
        let max_scroll = self
            .total_message_lines
            .saturating_sub(self.visible_message_height);
        self.scroll_offset = self.scroll_offset.saturating_add(lines).min(max_scroll);
    }

    /// Scroll down by the given number of lines.
    fn scroll_down(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
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
        Self::new(rx)
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

    #[test]
    fn welcome_message() {
        let app = App::new_for_test();
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[test]
    fn ctrl_c_quits_in_chat_mode() {
        let mut app = App::new_for_test();
        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(quit);
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_clears_input_in_shell_mode() {
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

    #[test]
    fn ctrl_c_noop_on_empty_input_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!quit);
        assert!(!app.should_quit);
    }

    #[test]
    fn ctrl_d_exits_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
    }

    #[test]
    fn ctrl_d_noop_in_chat_mode() {
        let mut app = App::new_for_test();
        assert_eq!(app.mode, AppMode::Chat);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[test]
    fn typing_and_submit() {
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

    #[test]
    fn empty_submit_ignored() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn ctrl_l_clears() {
        let mut app = App::new_for_test();

        // Type and submit a message
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.messages.len(), 2);

        // Clear
        app.handle_event(&key_event(KeyCode::Char('l'), KeyModifiers::CONTROL));
        assert!(app.messages.is_empty());
    }

    #[test]
    fn shift_enter_inserts_newline() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('b'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "a\nb");
    }

    #[test]
    fn up_down_moves_cursor_in_multiline() {
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

    #[test]
    fn alt_enter_inserts_newline() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        app.handle_event(&key_event(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "x\ny");
    }

    #[test]
    fn paste_inserts_without_submit() {
        let mut app = App::new_for_test();
        app.handle_event(&Event::Paste("hello\nworld".to_string()));
        assert_eq!(app.input.text(), "hello\nworld");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn history_up_down() {
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

    #[test]
    fn history_preserves_draft() {
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

    #[test]
    fn ctrl_u_deletes_to_line_start() {
        let mut app = App::new_for_test();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn ctrl_k_deletes_to_line_end() {
        let mut app = App::new_for_test();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Home, KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn scroll_bounds() {
        let mut app = App::new_for_test();
        // Simulate a frame that rendered 20 total lines with 10 visible
        app.total_message_lines = 20;
        app.visible_message_height = 10;

        app.scroll_up(5);
        assert_eq!(app.scroll_offset, 5);

        app.scroll_up(10);
        assert_eq!(app.scroll_offset, 10); // clamped to max (20 - 10)

        app.scroll_down(3);
        assert_eq!(app.scroll_offset, 7);

        app.scroll_down(100);
        assert_eq!(app.scroll_offset, 0);
    }

    // --- Slash command integration tests ---

    #[test]
    fn slash_help_shows_system_message() {
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

    #[test]
    fn slash_clear_clears_messages() {
        let mut app = App::new_for_test();
        for c in "/clear".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.messages.is_empty());
    }

    #[test]
    fn slash_quit_exits() {
        let mut app = App::new_for_test();
        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[test]
    fn unknown_slash_treated_as_user_message() {
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

    #[test]
    fn bare_dollar_treated_as_user_message() {
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

    #[test]
    fn ctrl_s_enters_shell_mode() {
        let mut app = App::new_for_test();
        assert_eq!(app.mode, AppMode::Chat);

        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[test]
    fn ctrl_s_noop_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        // Ctrl+S in shell mode should not do anything special
        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Shell);
    }

    // --- Autocomplete integration tests ---

    #[test]
    fn autocomplete_shows_on_slash() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[test]
    fn autocomplete_narrows_on_typing() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        let all_count = app.commands_state.candidates().len();

        app.handle_event(&key_event(KeyCode::Char('h'), KeyModifiers::NONE));
        assert!(app.commands_state.candidates().len() < all_count);
    }

    #[test]
    fn autocomplete_hides_on_esc() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    #[test]
    fn autocomplete_tab_accepts() {
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

    #[test]
    fn autocomplete_up_down_navigates() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
        assert_eq!(app.commands_state.selected(), 0);

        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.commands_state.selected(), 1);

        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.commands_state.selected(), 0);
    }

    #[test]
    fn autocomplete_enter_accepts_and_submits() {
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

    #[test]
    fn autocomplete_dismissed_on_submit() {
        let mut app = App::new_for_test();
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    // --- Shell mode tests ---

    #[test]
    fn slash_shell_enters_shell_mode() {
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

    #[test]
    fn slash_exit_in_shell_mode_returns_to_chat() {
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

    #[test]
    fn slash_quit_quits_from_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[test]
    fn autocomplete_works_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();

        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[test]
    fn enter_and_exit_shell_mode_messages() {
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

    #[test]
    fn ghost_appears_on_prefix_match() {
        let app = app_with_history_and_prefix(&["hello world"], "hel");
        assert_eq!(app.ghost_suggestion.as_deref(), Some("lo world"));
    }

    #[test]
    fn ghost_clears_when_input_diverges() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Type 'x' — "helx" no longer matches "hello world"
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn ghost_clears_when_cursor_not_at_end() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Move cursor left — no longer at end
        app.handle_event(&key_event(KeyCode::Left, KeyModifiers::NONE));
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn ghost_clears_when_multiline() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        // Insert newline — input becomes multiline
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn ghost_clears_when_popup_visible() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("/help me".to_string(), AppMode::Chat);

        // Type "/" — triggers command autocomplete popup
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
        // Ghost should be None because popup is visible
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn ghost_not_shown_for_empty_input() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello".to_string(), AppMode::Chat);
        // No typing — ghost should not appear
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn ghost_not_shown_for_exact_match() {
        let app = app_with_history_and_prefix(&["hello"], "hello");
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn tab_accepts_ghost_word() {
        let mut app = app_with_history_and_prefix(&["cargo test --release"], "cargo");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" test --release"));

        // Tab accepts " test"
        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test");
        // Ghost should refresh to " --release"
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" --release"));
    }

    #[test]
    fn multiple_tabs_accept_word_by_word() {
        let mut app = app_with_history_and_prefix(&["cargo test --release"], "cargo");

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test");

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "cargo test --release");

        // No more ghost text
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn right_at_end_accepts_all_ghost() {
        let mut app = app_with_history_and_prefix(&["hello world"], "hel");
        assert!(app.ghost_suggestion.is_some());

        app.handle_event(&key_event(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "hello world");
        assert!(app.ghost_suggestion.is_none());
    }

    #[test]
    fn right_in_middle_moves_cursor() {
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

    #[test]
    fn ghost_multiline_history_shows_first_line_suffix() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Chat);

        for c in "hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        // Ghost contains only insertable text; ellipsis is a UI-only indicator
        assert_eq!(app.ghost_suggestion.as_deref(), Some("lo world"));
        assert!(app.ghost_is_truncated);
    }

    #[test]
    fn ghost_multiline_exact_first_line_shows_nothing() {
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

    #[test]
    fn accept_all_ghost_multiline() {
        let mut app = App::new_for_test();
        app.input_history
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Chat);

        for c in "hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.ghost_suggestion.as_deref(), Some("lo world"));

        // Right accepts all — inserts exactly the ghost text, no ellipsis
        app.handle_event(&key_event(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "hello world");
        assert!(!app.ghost_is_truncated);
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
            exit_code: Some(0),
        });
        // Exchange 2: no response yet
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Chat,
            status: ExchangeStatus::Running,
            request: "what is rust".to_string(),
            response: None,
            exit_code: None,
        });
        // Exchange 3: has response
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Succeeded,
            request: "ls -la".to_string(),
            response: Some("total 42\ndrwxr-xr-x 2 user user 4096".to_string()),
            exit_code: Some(0),
        });
        app
    }

    #[test]
    fn response_candidates_returns_correct_list() {
        let app = app_with_exchanges();
        let candidates = app.response_candidates();
        // Exchange 1 and 3 have responses; newest first
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].0, 3); // newest first
        assert_eq!(candidates[1].0, 1);
    }

    #[test]
    fn hash_triggers_response_autocomplete() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('#'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 2);
    }

    #[test]
    fn hash_with_digit_filters_responses() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('#'), KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('1'), KeyModifiers::NONE));
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 1);
        assert_eq!(app.responses_state.candidates()[0].number, 1);
    }

    #[test]
    fn response_esc_dismisses() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('#'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.responses_state.is_visible());
    }

    #[test]
    fn response_tab_accepts() {
        let mut app = app_with_exchanges();
        app.handle_event(&key_event(KeyCode::Char('#'), KeyModifiers::SHIFT));
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE));
        assert!(!app.responses_state.is_visible());
        // Input should contain [Response #N: ...]
        assert!(app.input.text().contains("[Response #"));
    }

    #[test]
    fn expand_response_refs_replaces_known() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("see [Response #1: hello...]");
        assert_eq!(expanded, "see hello");
    }

    #[test]
    fn expand_response_refs_leaves_unknown() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("see [Response #99: unknown...]");
        assert_eq!(expanded, "see [Response #99: unknown...]");
    }

    #[test]
    fn expand_response_refs_no_refs() {
        let app = app_with_exchanges();
        let expanded = app.expand_response_refs("plain text");
        assert_eq!(expanded, "plain text");
    }

    #[test]
    fn expand_response_refs_multiple() {
        let app = app_with_exchanges();
        let expanded =
            app.expand_response_refs("[Response #1: hello...] and [Response #3: total 42...]");
        assert_eq!(expanded, "hello and total 42\ndrwxr-xr-x 2 user user 4096");
    }

    #[test]
    fn truncate_preview_short() {
        assert_eq!(truncate_preview("hello", 50), "hello...");
    }

    #[test]
    fn truncate_preview_long() {
        let long = "a".repeat(100);
        let preview = truncate_preview(&long, 50);
        assert_eq!(preview.len(), 53); // 50 chars + "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn truncate_preview_multiline() {
        assert_eq!(truncate_preview("line one\nline two", 50), "line one...");
    }
}
