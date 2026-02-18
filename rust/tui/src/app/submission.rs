use crate::{agent::AgentHandle, commands::SlashCommand};

use super::{App, AppMessage, AppMode, ExchangeKind, ExchangeStatus};

impl App {
    /// Submit the current input as a user message or slash command.
    pub(super) fn submit_input(&mut self) {
        // Trailing backslash means "insert newline, don't submit".
        // This gives users a way to enter multiline input even when
        // Shift+Enter / Alt+Enter aren't available (e.g. some terminals
        // send a literal `\` for Shift+Enter).
        if self.input.text().ends_with('\\') {
            let len = self.input.text().len();
            self.input.replace_range(len - 1..len, "");
            self.input.insert_newline();
            return;
        }

        let text = self.input.take();
        self.input_scroll = 0;

        // Workflow goal submission: when in workflow mode and not yet running,
        // empty input uses the default goal (if available).
        if let Some(workflow) = &self.active_workflow
            && workflow.run_handle.is_none()
        {
            let goal = if text.trim().is_empty() {
                workflow.info.goal.clone()
            } else {
                Some(text.clone())
            };

            if let Some(goal) = goal {
                self.submit_workflow_goal(goal);
            }
            // If no goal provided and no default, do nothing (keep waiting)
            return;
        }

        // Workflow interview answer submission: when a workflow is waiting
        // for user input, send the answer through the oneshot channel.
        if let Some(workflow) = &mut self.active_workflow
            && let Some(pending) = workflow.pending_interview.take()
        {
            if text.trim().is_empty() {
                // Put the pending interview back if input was empty
                workflow.pending_interview = Some(pending);
                return;
            }
            self.messages.push(AppMessage::System {
                content: format!("\u{1f4ac} {text}"),
            });
            let _ = pending.answer_tx.send(text);
            self.scroll_pinned = true;
            self.scroll_offset = 0;
            return;
        }

        if text.trim().is_empty() {
            return;
        }

        self.dismiss_all_autocomplete();

        // Expand paste and response references for the actual request text.
        // History stores the original (unexpanded) text so refs remain visible.
        let expanded = self.expand_paste_refs(&text);
        let expanded = self.expand_response_refs(&expanded);

        if let Some((cmd, args)) = SlashCommand::parse(&text) {
            // Slash commands work all modes
            cmd.execute(self, args);
        } else {
            // Other handling is dependent on app mode
            match self.mode {
                AppMode::Agent => {
                    if let Some(mention) = self.parse_agent_mention(&expanded) {
                        self.input_history.push_tagged(text, AppMode::Agent);
                        self.execute_agent_mention(mention);
                    } else if let Some(cmd) = expanded.strip_prefix('!')
                        && !cmd.trim().is_empty()
                    {
                        let cmd = cmd.to_string();
                        self.input_history
                            .push_tagged(format!("!{cmd}"), AppMode::Agent);
                        self.spawn_shell_command(cmd);
                    } else {
                        self.input_history.push_tagged(text, AppMode::Agent);
                        self.submit_agent_message(expanded);
                    }
                }
                AppMode::Shell => {
                    self.input_history.push_tagged(text, AppMode::Shell);
                    self.spawn_shell_command(expanded);
                }
                AppMode::Workflow => {
                    // No-op: workflow is running, input is ignored
                }
            }
        }

        // Pin scroll to bottom so the user sees their new message
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Spawn a shell command as a background task.
    pub(super) fn spawn_shell_command(&mut self, command: String) {
        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Running,
            request: command.clone(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: None,
            agent_name: None,
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

    /// Submit a chat message to the active agent session.
    pub(super) fn submit_agent_message(&mut self, text: String) {
        let session_idx = self.active_session;
        let session = &mut self.sessions[session_idx];

        // Lazily create the agent handle on first use
        if session.agent.is_none() {
            session.agent = AgentHandle::spawn(&session.name);
        }

        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Agent,
            status: ExchangeStatus::Running,
            request: text.clone(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: Some(session_idx),
            agent_name: None,
        });
        let msg_index = self.messages.len() - 1;

        let exchange = self.sessions[session_idx]
            .agent
            .as_ref()
            .and_then(|agent| agent.submit(text));

        if let Some(running) = exchange {
            self.sessions[session_idx]
                .running_agent_exchanges
                .push((msg_index, running));
        } else {
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

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{App, AppMessage, AppMode};

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
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
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::System { content } if content.contains("Available commands")
        )));
    }

    #[tokio::test]
    async fn slash_clear_resets_active_session() {
        let mut app = App::new_for_test();
        for c in "/clear".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], AppMessage::Welcome));
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
        for c in "/notacmd".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.input.is_empty());
        // Should be treated as a user message (agent exchange)
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::Exchange { request, .. } if request == "/notacmd"
        )));
    }

    #[tokio::test]
    async fn bare_dollar_treated_as_user_message() {
        let mut app = App::new_for_test();
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // "$" alone is a user message — should create an exchange
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::Exchange { request, .. } if request == "$"
        )));
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
        assert_eq!(app.mode, AppMode::Agent);
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
}
