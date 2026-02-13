use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

use crate::{
    autocomplete::{CommandsState, FilesState, HistoryState},
    commands::SlashCommand,
    history::InputHistory,
    input::InputState,
    shell::RunningCommand,
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

/// A message displayed in the messages area.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// A message from the user.
    User { content: String },
    /// A system/informational message.
    System { content: String },
    /// Output from a shell command.
    Shell {
        command: String,
        output: String,
        exit_code: i32,
    },
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

    /// Ghost suggestion suffix (the part beyond what's typed, insertable text only).
    /// Shown as dim inline text for fish-shell-style history completion.
    pub ghost_suggestion: Option<String>,
    /// Whether the ghost suggestion was truncated from a multiline history entry.
    /// When true, the UI appends a dim "…" indicator after the ghost text.
    pub ghost_is_truncated: bool,

    /// A shell command currently running in the background.
    pub running_command: Option<RunningCommand>,

    /// Scroll offset for the message area (lines from the bottom).
    pub scroll_offset: u16,
    /// Total lines rendered in the last frame's message area (set by `ui::render`).
    pub total_message_lines: u16,
    /// Visible height of the message area in the last frame (set by `ui::render`).
    pub visible_message_height: u16,
}

impl App {
    /// Create a new App with a welcome banner.
    pub fn new() -> Self {
        let version = stencila_version::STENCILA_VERSION;
        let welcome = format!("Stencila {version} — Ctrl+C to quit, /help for commands");

        Self {
            should_quit: false,
            mode: AppMode::default(),
            messages: vec![AppMessage::System { content: welcome }],
            input: InputState::default(),
            input_history: InputHistory::new(),
            commands_state: CommandsState::new(),
            files_state: FilesState::new(),
            history_state: HistoryState::new(),
            ghost_suggestion: None,
            ghost_is_truncated: false,
            running_command: None,
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

    /// Dispatch a key event.
    fn handle_key(&mut self, key: &KeyEvent) {
        // While a shell command is running, only Ctrl+C is accepted
        if self.running_command.is_some() {
            if matches!(
                (key.modifiers, key.code),
                (KeyModifiers::CONTROL, KeyCode::Char('c'))
            ) {
                self.cancel_running_command();
            }
            return;
        }

        let consumed = (self.history_state.is_visible() && self.handle_history_autocomplete(key))
            || (self.commands_state.is_visible() && self.handle_commands_autocomplete(key))
            || (self.files_state.is_visible() && self.handle_files_autocomplete(key));

        if !consumed {
            self.handle_normal_key(key);
            // Only refresh autocomplete after normal key handling — autocomplete
            // handlers manage their own state (e.g. Esc dismisses without re-trigger).
            self.refresh_autocomplete();
        }

        // Ghost suggestion always refreshes (input/cursor may have changed in either path).
        self.refresh_ghost_suggestion();
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

    /// Handle normal key input (no autocomplete popup intercept).
    fn handle_normal_key(&mut self, key: &KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => match self.mode {
                AppMode::Chat => {
                    self.should_quit = true;
                }
                AppMode::Shell => {
                    // In shell mode: clear input line (standard shell behavior), no-op if empty
                    self.input.clear();
                }
            },
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                if self.mode == AppMode::Shell {
                    self.exit_shell_mode();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.messages.clear();
                self.scroll_offset = 0;
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

        // Slash commands work in both modes
        if let Some((cmd, args)) = SlashCommand::parse(&text) {
            cmd.execute(self, args);
        } else {
            match self.mode {
                AppMode::Chat => {
                    // Check for one-off shell command with ! prefix.
                    // Bare "!" or "!   " falls through to normal chat message.
                    if let Some(cmd) = text.strip_prefix('!')
                        && !cmd.trim().is_empty()
                    {
                        let cmd = cmd.to_string();
                        self.input_history
                            .push_tagged(format!("!{cmd}"), AppMode::Chat);
                        self.spawn_shell_command(cmd);
                    } else {
                        self.input_history.push_tagged(text.clone(), AppMode::Chat);
                        self.messages.push(AppMessage::User { content: text });
                    }
                }
                AppMode::Shell => {
                    self.input_history.push_tagged(text.clone(), AppMode::Shell);
                    self.spawn_shell_command(text);
                }
            }
        }

        // Reset scroll to bottom
        self.scroll_offset = 0;
    }

    /// Enter shell mode with a system message.
    pub fn enter_shell_mode(&mut self) {
        self.mode = AppMode::Shell;
        self.dismiss_all_autocomplete();
        self.messages.push(AppMessage::System {
            content: "Entering shell mode. Commands are sent to your shell. Use /exit or Ctrl+D to return.".to_string(),
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
        self.running_command = Some(crate::shell::spawn_command(command));
    }

    /// Cancel the currently running shell command.
    fn cancel_running_command(&mut self) {
        if let Some(running) = self.running_command.take() {
            let command = running.cancel();
            self.messages.push(AppMessage::Shell {
                command,
                output: "[cancelled]".to_string(),
                exit_code: -1,
            });
        }
    }

    /// Poll the running command for completion. Called on tick events.
    pub fn poll_running_command(&mut self) {
        let Some(running) = &mut self.running_command else {
            return;
        };

        if let Some(result) = running.try_take_result() {
            let command = self
                .running_command
                .take()
                .map(|r| r.command().to_string())
                .unwrap_or_default();
            self.messages.push(AppMessage::Shell {
                command,
                output: result.output,
                exit_code: result.exit_code,
            });
        }
    }

    /// Dismiss all autocomplete popups and ghost suggestion.
    fn dismiss_all_autocomplete(&mut self) {
        self.commands_state.dismiss();
        self.files_state.dismiss();
        self.history_state.dismiss();
        self.ghost_suggestion = None;
        self.ghost_is_truncated = false;
    }

    /// Re-filter all autocomplete states based on current input.
    fn refresh_autocomplete(&mut self) {
        self.history_state.update(self.input.text());
        self.commands_state.update(self.input.text());
        self.files_state
            .update(self.input.text(), self.input.cursor());
    }

    /// Navigate to the previous (older) history entry, filtered by current mode.
    fn navigate_history_up(&mut self) {
        let current = self.input.text().to_string();
        if let Some(entry) = self.input_history.navigate_up_filtered(&current, self.mode) {
            self.input.set_text(entry);
        }
    }

    /// Navigate to the next (newer) history entry, or back to draft, filtered by current mode.
    fn navigate_history_down(&mut self) {
        if let Some(entry) = self.input_history.navigate_down_filtered(self.mode) {
            self.input.set_text(entry);
        }
    }

    /// Whether any autocomplete popup is currently visible.
    fn any_popup_visible(&self) -> bool {
        self.history_state.is_visible()
            || self.commands_state.is_visible()
            || self.files_state.is_visible()
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
            .prefix_match(text, self.mode)
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
        let app = App::new();
        assert_eq!(app.messages.len(), 1);
        assert!(
            matches!(&app.messages[0], AppMessage::System { content } if content.contains("Stencila"))
        );
    }

    #[test]
    fn ctrl_c_quits_in_chat_mode() {
        let mut app = App::new();
        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(quit);
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_clears_input_in_shell_mode() {
        let mut app = App::new();
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
        let mut app = App::new();
        app.enter_shell_mode();

        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!quit);
        assert!(!app.should_quit);
    }

    #[test]
    fn ctrl_d_exits_shell_mode() {
        let mut app = App::new();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
    }

    #[test]
    fn ctrl_d_noop_in_chat_mode() {
        let mut app = App::new();
        assert_eq!(app.mode, AppMode::Chat);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[test]
    fn typing_and_submit() {
        let mut app = App::new();

        // Type "hello"
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.input.text(), "hello");

        // Submit
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.input.is_empty());
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(&app.messages[1], AppMessage::User { content } if content == "hello"));
    }

    #[test]
    fn empty_submit_ignored() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn ctrl_l_clears() {
        let mut app = App::new();

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
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('b'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "a\nb");
    }

    #[test]
    fn up_down_moves_cursor_in_multiline() {
        let mut app = App::new();
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
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        app.handle_event(&key_event(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "x\ny");
    }

    #[test]
    fn paste_inserts_without_submit() {
        let mut app = App::new();
        app.handle_event(&Event::Paste("hello\nworld".to_string()));
        assert_eq!(app.input.text(), "hello\nworld");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn history_up_down() {
        let mut app = App::new();

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
        let mut app = App::new();

        // Submit something to have history
        for c in "old".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));

        // Type a draft
        for c in "draft".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Navigate up, then back down to draft
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "old");
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "draft");
    }

    #[test]
    fn ctrl_u_deletes_to_line_start() {
        let mut app = App::new();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn ctrl_k_deletes_to_line_end() {
        let mut app = App::new();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Home, KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn scroll_bounds() {
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
        for c in "/clear".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.messages.is_empty());
    }

    #[test]
    fn slash_quit_exits() {
        let mut app = App::new();
        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[test]
    fn unknown_slash_treated_as_user_message() {
        let mut app = App::new();
        for c in "/unknown".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Not a command, so it's a user message
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(&app.messages[1], AppMessage::User { content } if content == "/unknown"));
    }

    #[test]
    fn bare_bang_treated_as_user_message() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('!'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // "!" should be treated as a normal chat message, not silently discarded
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(&app.messages[1], AppMessage::User { content } if content == "!"));
    }

    // --- Autocomplete integration tests ---

    #[test]
    fn autocomplete_shows_on_slash() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[test]
    fn autocomplete_narrows_on_typing() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        let all_count = app.commands_state.candidates().len();

        app.handle_event(&key_event(KeyCode::Char('h'), KeyModifiers::NONE));
        assert!(app.commands_state.candidates().len() < all_count);
    }

    #[test]
    fn autocomplete_hides_on_esc() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    #[test]
    fn autocomplete_tab_accepts() {
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.commands_state.is_visible());
    }

    // --- Shell mode tests ---

    #[test]
    fn slash_shell_enters_shell_mode() {
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
        app.enter_shell_mode();

        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let quit = app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(quit);
    }

    #[test]
    fn autocomplete_works_in_shell_mode() {
        let mut app = App::new();
        app.enter_shell_mode();

        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(app.commands_state.is_visible());
    }

    #[test]
    fn enter_and_exit_shell_mode_messages() {
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
        let mut app = App::new();
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
}
