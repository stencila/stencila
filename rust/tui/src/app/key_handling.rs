use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::autocomplete::agents::AgentSelection;

use super::{App, AppMode};

impl App {
    /// Dispatch a key event.
    pub(super) async fn handle_key(&mut self, key: &KeyEvent) {
        let consumed = (self.cancel_state.is_visible() && self.handle_cancel_autocomplete(key))
            || (self.agents_state.is_visible() && self.handle_agents_autocomplete(key))
            || (self.workflows_state.is_visible() && self.handle_workflows_autocomplete(key))
            || (self.mentions_state.is_visible() && self.handle_mentions_autocomplete(key))
            || (self.history_state.is_visible() && self.handle_history_autocomplete(key))
            || (self.commands_state.is_visible() && self.handle_commands_autocomplete(key).await)
            || (self.files_state.is_visible() && self.handle_files_autocomplete(key))
            || (self.responses_state.is_visible() && self.handle_responses_autocomplete(key));

        // Snapshot text *after* autocomplete handlers (which may set both
        // input text and command_usage_hint together). Subsequent edits
        // in handle_normal_key will clear the hint.
        let text_before = self.input.text().to_string();

        if !consumed {
            self.handle_normal_key(key).await;
            // Only refresh autocomplete after normal key handling — autocomplete
            // handlers manage their own state (e.g. Esc dismisses without re-trigger).
            self.refresh_autocomplete();
        }

        // Clear the command usage hint when input text changes (user typed
        // or deleted something), but preserve it across non-editing keys
        // like arrow navigation, Esc, etc.
        if self.command_usage_hint.is_some() && self.input.text() != text_before {
            self.command_usage_hint = None;
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
    async fn handle_commands_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab) => {
                if let Some(name) = self.commands_state.accept() {
                    self.input.set_text(&name);
                    // Re-trigger autocomplete to support drill-down into
                    // CLI command children (e.g. "/skills " → show subcommands)
                    self.commands_state.update(self.input.text());
                    // If the accepted command is a leaf with required args,
                    // show usage hint as ghost text.
                    if !self.commands_state.is_visible()
                        && let Some(hint) = self.cli_usage_hint_for_input()
                    {
                        self.command_usage_hint = Some(format!(" {hint}"));
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.commands_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.commands_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.commands_state.select_next(),
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if let Some(name) = self.commands_state.accept() {
                    self.input.set_text(&name);
                }
                // If the command requires positional args, don't submit —
                // show the usage hint as ghost text instead.
                if let Some(hint) = self.cli_usage_hint_for_input() {
                    self.command_usage_hint = Some(format!(" {hint}"));
                } else {
                    self.submit_input().await;
                }
            }
            _ => return false,
        }
        true
    }

    /// Extract CLI command path words from the current input (e.g. "/mcp add" → ["mcp", "add"])
    /// and return the usage hint if the leaf node has required positional args.
    fn cli_usage_hint_for_input(&self) -> Option<String> {
        let text = self.input.text();
        let trimmed = text.trim();
        let without_slash = trimmed.strip_prefix('/')?;
        let path: Vec<String> = without_slash.split_whitespace().map(String::from).collect();
        if path.is_empty() {
            return None;
        }
        self.commands_state.usage_hint_for(&path)
    }

    /// Handle a key event when the files autocomplete popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_files_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            // Tab: accept file, or drill into directory
            (KeyModifiers::NONE, KeyCode::Tab) => {
                let use_at_prefix = self.mode == AppMode::Agent;
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
                let use_at_prefix = self.mode == AppMode::Agent;
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
    #[allow(clippy::too_many_lines)]
    async fn handle_normal_key(&mut self, key: &KeyEvent) {
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
                        AppMode::Agent => {
                            self.should_quit = true;
                        }
                        AppMode::Shell => {
                            // In shell mode: clear input line (standard shell behavior)
                            self.input.clear();
                            self.input_scroll = 0;
                        }
                        AppMode::Workflow => {
                            self.input.clear();
                            self.input_scroll = 0;
                        }
                    }
                }
            }

            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                if self.mode == AppMode::Agent {
                    self.enter_shell_mode();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                if self.mode == AppMode::Shell {
                    self.exit_shell_mode();
                } else if self.mode == AppMode::Workflow {
                    self.exit_workflow_mode();
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
                self.submit_input().await;
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
                    // Selecting an agent while in shell or workflow mode should
                    // transition back to agent mode so the user can chat.
                    if self.mode == AppMode::Shell {
                        self.exit_shell_mode();
                    } else if self.mode == AppMode::Workflow {
                        // Exit workflow input mode but keep the workflow
                        // running in the background.
                        self.exit_workflow_mode();
                    }

                    match selection {
                        AgentSelection::Switch(index) => self.switch_to_session(index),
                        AgentSelection::FromDefinition(info) => {
                            self.create_session_from_definition(&info);
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

    /// Handle a key event when the workflow picker popup is visible.
    ///
    /// Returns `true` if the key was consumed.
    fn handle_workflows_autocomplete(&mut self, key: &KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Enter) => {
                if let Some(info) = self.workflows_state.accept() {
                    self.activate_workflow(info);
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => self.workflows_state.dismiss(),
            (KeyModifiers::NONE, KeyCode::Up) => self.workflows_state.select_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.workflows_state.select_next(),
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

    /// Handle pasted text — insert as-is for short pastes, or as a
    /// `[Paste #N: preview…]` token for large ones.
    pub(super) fn handle_paste(&mut self, text: &str) {
        const PASTE_THRESHOLD: usize = 80;
        const PASTE_PREVIEW_CHARS: usize = 20;

        // Normalize line endings: many terminals send \r or \r\n in paste
        // events when in raw mode, but the input buffer uses \n exclusively.
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let text = text.as_str();

        let char_count = text.chars().count();

        if char_count <= PASTE_THRESHOLD {
            self.input.insert_str(text);
        } else {
            self.paste_counter += 1;
            let n = self.paste_counter;
            self.pastes.insert(n, text.to_string());

            let prefix: String = text
                .trim_start()
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
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{AgentSession, App, AppMessage, AppMode, ExchangeKind};

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
        let app = App::new_for_test().await;
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn ctrl_c_quits_in_chat_mode() {
        let mut app = App::new_for_test().await;
        let quit = app
            .handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .await;
        assert!(quit);
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn ctrl_c_clears_input_in_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();

        // Type some text
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "hello");

        // Ctrl+C should clear input, not quit
        let quit = app
            .handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .await;
        assert!(!quit);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn ctrl_c_noop_on_empty_input_in_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();

        let quit = app
            .handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .await;
        assert!(!quit);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn ctrl_d_exits_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Agent);
    }

    #[tokio::test]
    async fn ctrl_d_noop_in_chat_mode() {
        let mut app = App::new_for_test().await;
        assert_eq!(app.mode, AppMode::Agent);

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Agent);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn typing_and_submit() {
        let mut app = App::new_for_test().await;

        // Type "hello"
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "hello");

        // Submit — without a tokio runtime the agent is unavailable,
        // so the exchange is created as Failed.
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(app.input.is_empty());
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(
            &app.messages[1],
            AppMessage::Exchange { kind: ExchangeKind::Agent, request, .. } if request == "hello"
        ));
    }

    #[tokio::test]
    async fn empty_submit_ignored() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        // Only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[tokio::test]
    async fn ctrl_l_resets_active_session() {
        let mut app = App::new_for_test().await;

        // Type and submit a message
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.messages.len(), 2);

        // Ctrl+L resets to welcome
        app.handle_event(&key_event(KeyCode::Char('l'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn shift_enter_inserts_newline() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::SHIFT))
            .await;
        app.handle_event(&key_event(KeyCode::Char('b'), KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "a\nb");
    }

    #[tokio::test]
    async fn up_down_moves_cursor_in_multiline() {
        let mut app = App::new_for_test().await;
        // Paste multiline text: "abc\ndef"
        app.handle_event(&Event::Paste("abc\ndef".to_string()))
            .await;
        // Cursor at end (pos 7, line 1, col 3)
        assert_eq!(app.input.cursor(), 7);

        // Up moves to same column on previous line
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.cursor(), 3); // end of "abc"

        // Down moves back
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.cursor(), 7); // end of "def"
    }

    #[tokio::test]
    async fn alt_enter_inserts_newline() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT))
            .await;
        app.handle_event(&key_event(KeyCode::Char('y'), KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "x\ny");
    }

    #[tokio::test]
    async fn trailing_backslash_enter_inserts_newline() {
        let mut app = App::new_for_test().await;
        // Type "hello\" then press Enter — should insert newline, not submit
        for c in "hello\\".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "hello\n");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);

        // Continue typing and submit without backslash
        for c in "world".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "hello\nworld");
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(app.input.is_empty());
        assert_eq!(app.messages.len(), 2);
    }

    #[tokio::test]
    async fn trailing_backslash_multiline_continuation() {
        let mut app = App::new_for_test().await;
        // Build up multiple lines using trailing backslash
        for c in "line1\\".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        for c in "line2\\".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        for c in "line3".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "line1\nline2\nline3");
        // Only welcome message — nothing submitted yet
        assert_eq!(app.messages.len(), 1);
    }

    #[tokio::test]
    async fn paste_inserts_without_submit() {
        let mut app = App::new_for_test().await;
        app.handle_event(&Event::Paste("hello\nworld".to_string()))
            .await;
        assert_eq!(app.input.text(), "hello\nworld");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[tokio::test]
    async fn paste_short_inserted_verbatim() {
        let mut app = App::new_for_test().await;
        app.handle_event(&Event::Paste("short text".to_string()))
            .await;
        assert_eq!(app.input.text(), "short text");
        assert!(app.pastes.is_empty());
    }

    #[tokio::test]
    async fn paste_large_inserts_token() {
        let mut app = App::new_for_test().await;
        let long_text = "a".repeat(200);
        app.handle_event(&Event::Paste(long_text.clone())).await;
        // Buffer contains the token, not the raw text
        let input = app.input.text().to_string();
        assert!(input.starts_with("[Paste #1: "));
        assert!(input.contains("+180 chars]"));
        // Full text is stored in the pastes map
        assert_eq!(
            app.pastes.get(&1).expect("paste #1 should exist"),
            &long_text
        );
    }

    #[tokio::test]
    async fn paste_token_expanded_on_submit() {
        let mut app = App::new_for_test().await;
        let long_text = "x".repeat(200);
        app.handle_event(&Event::Paste(long_text.clone())).await;
        // Expand paste refs returns the full text
        let input = app.input.text().to_string();
        let expanded = app.expand_paste_refs(&input);
        assert_eq!(expanded, long_text);
    }

    #[tokio::test]
    async fn paste_multiple_tokens() {
        let mut app = App::new_for_test().await;
        let text1 = "a".repeat(100);
        let text2 = "b".repeat(150);
        app.handle_event(&Event::Paste(text1.clone())).await;
        app.handle_event(&Event::Paste(text2.clone())).await;
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
        let mut app = App::new_for_test().await;
        let long_text = format!("line1\nline2\n{}", "x".repeat(100));
        app.handle_event(&Event::Paste(long_text)).await;
        let input = app.input.text().to_string();
        // The token itself should not contain newlines
        assert!(!input.contains('\n'));
    }

    #[tokio::test]
    async fn paste_normalizes_crlf_to_lf() {
        let mut app = App::new_for_test().await;
        app.handle_event(&Event::Paste("hello\r\nworld".to_string()))
            .await;
        assert_eq!(app.input.text(), "hello\nworld");
    }

    #[tokio::test]
    async fn paste_normalizes_cr_to_lf() {
        let mut app = App::new_for_test().await;
        app.handle_event(&Event::Paste("hello\rworld".to_string()))
            .await;
        assert_eq!(app.input.text(), "hello\nworld");
    }

    #[tokio::test]
    async fn history_up_down() {
        let mut app = App::new_for_test().await;

        // Submit a few messages
        for msg in ["first", "second", "third"] {
            for c in msg.chars() {
                app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                    .await;
            }
            app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
                .await;
        }

        // Navigate up through history
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "third");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "second");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "first");

        // Navigate back down
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "second");
    }

    #[tokio::test]
    async fn history_preserves_draft() {
        let mut app = App::new_for_test().await;

        // Submit two entries with the same prefix
        for text in ["old first", "old second"] {
            for c in text.chars() {
                app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                    .await;
            }
            app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
                .await;
        }

        // Type a prefix
        for c in "old".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }

        // Default ghost shows " second" (most recent prefix match)
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" second"));

        // Up cycles ghost to the next older match — input stays "old"
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "old");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" first"));

        // Down cycles ghost back to the most recent match
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        assert_eq!(app.input.text(), "old");
        assert_eq!(app.ghost_suggestion.as_deref(), Some(" second"));
    }

    #[tokio::test]
    async fn ctrl_u_deletes_to_line_start() {
        let mut app = App::new_for_test().await;
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Char('u'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.input.text(), "");
    }

    #[tokio::test]
    async fn ctrl_k_deletes_to_line_end() {
        let mut app = App::new_for_test().await;
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Home, KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Char('k'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.input.text(), "");
    }

    #[tokio::test]
    async fn ctrl_s_enters_shell_mode() {
        let mut app = App::new_for_test().await;
        assert_eq!(app.mode, AppMode::Agent);

        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[tokio::test]
    async fn ctrl_s_noop_in_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        // Ctrl+S in shell mode should not do anything special
        app.handle_event(&key_event(KeyCode::Char('s'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Shell);
    }

    // --- Autocomplete integration tests ---

    #[tokio::test]
    async fn autocomplete_shows_on_slash() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE))
            .await;
        assert!(app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn autocomplete_narrows_on_typing() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE))
            .await;
        let all_count = app.commands_state.candidates().len();

        app.handle_event(&key_event(KeyCode::Char('h'), KeyModifiers::NONE))
            .await;
        assert!(app.commands_state.candidates().len() < all_count);
    }

    #[tokio::test]
    async fn autocomplete_hides_on_esc() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE))
            .await;
        assert!(app.commands_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE))
            .await;
        assert!(!app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn autocomplete_tab_accepts() {
        let mut app = App::new_for_test().await;
        // Type "/he" — matches /help and /history
        for c in "/he".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert!(app.commands_state.is_visible());

        // Tab accepts the first candidate
        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE))
            .await;
        assert!(!app.commands_state.is_visible());
        // Input should be one of the matching commands
        let text = app.input.text().to_string();
        assert!(text == "/help" || text == "/history");
    }

    #[tokio::test]
    async fn autocomplete_up_down_navigates() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE))
            .await;
        assert!(app.commands_state.is_visible());
        assert_eq!(app.commands_state.selected(), 0);

        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        assert_eq!(app.commands_state.selected(), 1);

        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE))
            .await;
        assert_eq!(app.commands_state.selected(), 0);
    }

    #[tokio::test]
    async fn autocomplete_enter_accepts_and_submits() {
        let mut app = App::new_for_test().await;
        // Type "/hel" — matches /help only
        for c in "/hel".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert!(app.commands_state.is_visible());

        // Enter should accept and submit
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(!app.commands_state.is_visible());
        assert!(app.input.is_empty());
        // Should have executed /help
        assert!(app.messages.len() >= 2);
    }

    #[tokio::test]
    async fn autocomplete_dismissed_on_submit() {
        let mut app = App::new_for_test().await;
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(!app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn agents_picker_exits_shell_mode() {
        use crate::autocomplete::agents::{AgentCandidate, AgentCandidateKind};

        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("other"));
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        // Open the agents picker and select the second session
        app.agents_state.open(vec![
            AgentCandidate {
                name: "default".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 0,
                    is_active: true,
                    definition: None,
                },
            },
            AgentCandidate {
                name: "other".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 1,
                    is_active: false,
                    definition: None,
                },
            },
        ]);
        assert!(app.agents_state.is_visible());

        // Select second entry and accept
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        assert_eq!(app.mode, AppMode::Agent);
        assert_eq!(app.active_session, 1);
        assert!(!app.agents_state.is_visible());
    }

    #[tokio::test]
    async fn agents_picker_exits_workflow_mode_but_keeps_workflow() {
        use crate::autocomplete::{
            agents::{AgentCandidate, AgentCandidateKind},
            workflows::WorkflowDefinitionInfo,
        };

        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("other"));
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("goal".to_string()),
        });
        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        app.agents_state.open(vec![
            AgentCandidate {
                name: "default".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 0,
                    is_active: true,
                    definition: None,
                },
            },
            AgentCandidate {
                name: "other".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 1,
                    is_active: false,
                    definition: None,
                },
            },
        ]);

        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        assert_eq!(app.mode, AppMode::Agent);
        assert_eq!(app.active_session, 1);
        assert!(app.active_workflow.is_some());
    }

    #[tokio::test]
    async fn autocomplete_enter_on_leaf_with_required_args_shows_hint() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills ` to show subcommands, then select "show" (has <NAME> arg)
        for c in "/skills ".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert!(app.commands_state.is_visible());

        // Navigate to "show" candidate (it may be first or second)
        let show_idx = app
            .commands_state
            .candidates()
            .iter()
            .position(|c| c.name() == "/skills show")
            .expect("show should be a candidate");
        for _ in 0..show_idx {
            app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
                .await;
        }

        // Press Enter — should NOT execute, should show hint
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        assert_eq!(app.messages.len(), initial_msg_count);
        assert_eq!(app.input.text(), "/skills show");
        assert_eq!(app.ghost_suggestion, Some(" <NAME>".to_string()));
    }

    #[tokio::test]
    async fn autocomplete_tab_on_leaf_with_required_args_shows_hint() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        // Type `/skills ` to show subcommands
        for c in "/skills ".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert!(app.commands_state.is_visible());

        // Navigate to "show" candidate
        let show_idx = app
            .commands_state
            .candidates()
            .iter()
            .position(|c| c.name() == "/skills show")
            .expect("show should be a candidate");
        for _ in 0..show_idx {
            app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE))
                .await;
        }

        // Press Tab — should accept and show hint as ghost text
        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE))
            .await;

        assert_eq!(app.input.text(), "/skills show");
        assert_eq!(app.ghost_suggestion, Some(" <NAME>".to_string()));
    }

    #[tokio::test]
    async fn usage_hint_persists_across_non_editing_keys() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        // Type `/skills show` and press Enter to trigger hint
        for c in "/skills show".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.ghost_suggestion, Some(" <NAME>".to_string()));

        // Non-editing keys should NOT clear the hint
        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE))
            .await;
        assert_eq!(app.ghost_suggestion, Some(" <NAME>".to_string()));

        // Typing a character SHOULD clear the hint
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE))
            .await;
        assert!(app.command_usage_hint.is_none());
    }
}
