use super::{AgentColorRegistry, AgentSession, App, AppMessage, AppMode};

impl App {
    /// Dismiss all autocomplete popups and ghost suggestion.
    pub(super) fn dismiss_all_autocomplete(&mut self) {
        self.cancel_state.dismiss();
        self.agents_state.dismiss();
        self.workflows_state.dismiss();
        self.mentions_state.dismiss();
        self.commands_state.dismiss();
        self.files_state.dismiss();
        self.history_state.dismiss();
        self.responses_state.dismiss();
        self.ghost_suggestion = None;
        self.ghost_is_truncated = false;
    }

    /// Re-filter all autocomplete states based on current input.
    pub(super) fn refresh_autocomplete(&mut self) {
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
    pub(super) fn navigate_history_up(&mut self) {
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
    pub(super) fn navigate_history_down(&mut self) {
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
    pub(super) fn any_popup_visible(&self) -> bool {
        self.cancel_state.is_visible()
            || self.agents_state.is_visible()
            || self.workflows_state.is_visible()
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
    pub(super) fn refresh_ghost_suggestion(&mut self) {
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
    pub(super) fn accept_ghost_word(&mut self) {
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
    pub(super) fn accept_ghost_all(&mut self) {
        let Some(ghost) = self.ghost_suggestion.take() else {
            return;
        };
        self.ghost_is_truncated = false;
        self.input.insert_str(&ghost);
    }

    /// Scroll up by the given number of lines.
    pub(super) fn scroll_up(&mut self, lines: u16) {
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
    pub(super) fn scroll_down(&mut self, lines: u16) {
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
    pub(super) fn scroll_to_bottom(&mut self) {
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Reset the active agent session: cancel all running work (shell
    /// commands and agent exchanges across all sessions), drop the active
    /// session's agent handle, and clear all messages back to the welcome
    /// screen.
    ///
    /// All running work must be cancelled before clearing messages to prevent
    /// stale `msg_index` values from mutating unrelated future messages.
    pub fn reset_active_session(&mut self) {
        self.cancel_all_running();

        let session = &mut self.sessions[self.active_session];
        session.agent = None;
        session.context_usage_percent = 0;

        self.active_workflow = None;
        if self.mode == AppMode::Workflow {
            self.mode = AppMode::Agent;
        }
        self.messages.clear();
        self.messages.push(AppMessage::Welcome);
        self.md_render_cache.clear();
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Reset everything: cancel all running work, drop all sessions, create a
    /// fresh default session, and clear all messages. Equivalent to restarting.
    pub fn reset_all(&mut self) {
        self.cancel_all_running();
        self.active_workflow = None;
        if self.mode == AppMode::Workflow {
            self.mode = AppMode::Agent;
        }
        // Drop all sessions (and their agent handles)
        self.sessions.clear();
        self.color_registry = AgentColorRegistry::new();
        let default_name = stencila_agents::convenience::resolve_default_agent_name("default");
        self.color_registry.color_for(&default_name);
        self.sessions.push(AgentSession::new(default_name));
        self.active_session = 0;
        self.messages.clear();
        self.messages.push(AppMessage::Welcome);
        self.scroll_pinned = true;
        self.scroll_offset = 0;
        self.md_render_cache.clear();
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
        self.mode = AppMode::Agent;
        self.dismiss_all_autocomplete();
        self.messages.push(AppMessage::System {
            content: "Exiting shell mode.".to_string(),
        });
    }

    /// Exit workflow input mode and return to agent mode.
    ///
    /// This does not cancel or clear the active workflow. Any running workflow
    /// continues in the background and its events remain visible in messages.
    pub fn exit_workflow_mode(&mut self) {
        self.mode = AppMode::Agent;
        self.input.clear();
        self.input_scroll = 0;
        self.dismiss_all_autocomplete();
        self.messages.push(AppMessage::System {
            content: "Exiting workflow mode.".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{App, AppMode};
    use crossterm::event::Event;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    // --- Scroll tests ---

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

    // --- Ghost suggestion tests ---

    /// Helper: set up an app with history entries and type a prefix.
    fn app_with_history_and_prefix(entries: &[&str], prefix: &str) -> App {
        let mut app = App::new_for_test();
        for &entry in entries {
            app.input_history
                .push_tagged(entry.to_string(), AppMode::Agent);
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
            .push_tagged("/help me".to_string(), AppMode::Agent);

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
            .push_tagged("hello".to_string(), AppMode::Agent);
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
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Agent);

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
            .push_tagged("foo\nbar".to_string(), AppMode::Agent);

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
            .push_tagged("hello world\nsecond line".to_string(), AppMode::Agent);

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
}
