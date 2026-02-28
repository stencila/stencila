use crate::{
    agent::AgentHandle,
    commands::{ParsedCommand, parse_command},
};

use super::{App, AppMessage, AppMode, ExchangeKind, ExchangeStatus};

impl App {
    /// Submit the current input as a user message or slash command.
    pub(super) async fn submit_input(&mut self) {
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
        self.dismiss_all_autocomplete();

        // Slash commands work in all modes and always take precedence.
        // Two-tier dispatch: built-in commands first, then CLI passthrough.
        let cli_tree = self.cli_tree.clone();
        let empty_tree: Vec<crate::cli_commands::CliCommandNode> = Vec::new();
        let tree_ref = cli_tree.as_deref().unwrap_or(&empty_tree);
        if let Some(parsed) = parse_command(&text, tree_ref) {
            match parsed {
                ParsedCommand::Builtin(cmd, args) => {
                    cmd.execute(self, args).await;
                }
                ParsedCommand::CliPassthrough(cmd) => {
                    // Bare top-level command with subcommands (e.g. `/kernels`):
                    // instead of running the default `list` action, open the
                    // autocomplete picker so the user discovers subcommands.
                    if cmd.args.len() == 1 {
                        let has_children = tree_ref
                            .iter()
                            .find(|n| n.name == cmd.args[0])
                            .is_some_and(|n| !n.children.is_empty());
                        if has_children {
                            let with_space = format!("/{} ", cmd.args[0]);
                            self.input.set_text(&with_space);
                            self.commands_state.update(self.input.text());
                            return;
                        }
                    }

                    // If the leaf subcommand requires positional args that
                    // haven't been provided, show ghost text instead of
                    // executing (which would fail with a clap error).
                    if let Some(hint) =
                        crate::cli_commands::find_missing_args_hint(tree_ref, &cmd.args)
                    {
                        let slash_cmd = format!("/{}", cmd.args.join(" "));
                        self.input.set_text(&slash_cmd);
                        self.command_usage_hint = Some(format!(" {hint}"));
                        return;
                    }

                    let exe = std::env::current_exe()
                        .map_or_else(|_| "stencila".to_string(), |p| p.display().to_string());
                    // Record in history as a shell command so Up-arrow recall works.
                    // In Agent mode, prefix with `!` (the shell-escape syntax);
                    // in Shell mode, store the bare command.
                    let history_text = match self.mode {
                        AppMode::Agent => format!("!{}", cmd.display),
                        _ => cmd.display.clone(),
                    };
                    self.input_history.push_tagged(history_text, self.mode);
                    self.spawn_cli_command(exe, cmd.args, cmd.display);
                }
            }
            self.scroll_pinned = true;
            self.scroll_offset = 0;
            return;
        }

        if self.mode == AppMode::Workflow {
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
        }

        if text.trim().is_empty() {
            return;
        }

        // Expand paste and response references for the actual request text.
        // History stores the original (unexpanded) text so refs remain visible.
        let expanded = self.expand_paste_refs(&text);
        let expanded = self.expand_response_refs(&expanded);

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

        // Pin scroll to bottom so the user sees their new message
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Spawn a shell command as a background task.
    pub(super) fn spawn_shell_command(&mut self, command: String) {
        let running = crate::shell::spawn_command(command.clone());
        self.track_shell_exchange(command, running);
    }

    /// Spawn a CLI command as a direct process (argv-based, no shell).
    pub(super) fn spawn_cli_command(
        &mut self,
        program: String,
        args: Vec<String>,
        display: String,
    ) {
        let running = crate::shell::spawn_command_argv(program, args, display.clone());
        self.track_shell_exchange(display, running);
    }

    /// Push a shell-style exchange message and track the running command.
    fn track_shell_exchange(
        &mut self,
        request: String,
        running: crate::shell::RunningShellCommand,
    ) {
        self.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Running,
            request,
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: None,
            agent_name: None,
        });
        let msg_index = self.messages.len() - 1;
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
        let mut app = App::new_for_test().await;
        for c in "/help".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(app.input.is_empty());
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::System { content } if content.contains("Available commands")
        )));
    }

    #[tokio::test]
    async fn slash_clear_resets_active_session() {
        let mut app = App::new_for_test().await;
        for c in "/clear".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn slash_quit_exits() {
        let mut app = App::new_for_test().await;
        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        let quit = app
            .handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(quit);
    }

    #[tokio::test]
    async fn unknown_slash_treated_as_user_message() {
        let mut app = App::new_for_test().await;
        for c in "/notacmd".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(app.input.is_empty());
        // Should be treated as a user message (agent exchange)
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::Exchange { request, .. } if request == "/notacmd"
        )));
    }

    #[tokio::test]
    async fn bare_dollar_treated_as_user_message() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT))
            .await;
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        // "$" alone is a user message — should create an exchange
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::Exchange { request, .. } if request == "$"
        )));
    }

    // --- Shell mode tests ---

    #[tokio::test]
    async fn slash_shell_enters_shell_mode() {
        let mut app = App::new_for_test().await;
        for c in "/shell".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.mode, AppMode::Shell);
        // System message about entering shell mode
        assert!(app.messages.iter().any(|m| matches!(
            m,
            AppMessage::System { content } if content.contains("shell mode")
        )));
    }

    #[tokio::test]
    async fn slash_exit_in_shell_mode_returns_to_chat() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);

        for c in "/exit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert_eq!(app.mode, AppMode::Agent);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn slash_quit_quits_from_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();

        for c in "/quit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        let quit = app
            .handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(quit);
    }

    #[tokio::test]
    async fn autocomplete_works_in_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();

        app.handle_event(&key_event(KeyCode::Char('/'), KeyModifiers::NONE))
            .await;
        assert!(app.commands_state.is_visible());
    }

    #[tokio::test]
    async fn detached_agent_submit_creates_agent_exchange() {
        use super::super::ExchangeKind;
        use crate::autocomplete::workflows::WorkflowDefinitionInfo;

        let mut app = App::new_for_test().await;
        // Activate a workflow then detach back to agent mode
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("goal".to_string()),
        });
        app.exit_workflow_mode();
        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.active_workflow.is_some());

        let initial = app.messages.len();

        // Submit normal text — should create an agent exchange, not a workflow goal
        for c in "hello agent".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;
        assert!(app.input.is_empty());
        assert!(app.messages.len() > initial);
        assert!(app.messages[initial..].iter().any(|m| matches!(
            m,
            AppMessage::Exchange { kind: ExchangeKind::Agent, request, .. } if request == "hello agent"
        )));
    }

    #[tokio::test]
    async fn slash_exit_in_workflow_mode_keeps_workflow() {
        use crate::autocomplete::workflows::WorkflowDefinitionInfo;

        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
        });
        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        for c in "/exit".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.active_workflow.is_some());
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn enter_and_exit_shell_mode_messages() {
        let mut app = App::new_for_test().await;
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

    #[tokio::test]
    async fn bare_cli_command_with_children_opens_picker() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills` and press Enter
        for c in "/skills".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should NOT have spawned a command (no new messages)
        assert_eq!(app.messages.len(), initial_msg_count);
        // Input should be set to "/skills " to show subcommand picker
        assert_eq!(app.input.text(), "/skills ");
        // Autocomplete should be visible with subcommand candidates
        assert!(app.commands_state.is_visible());
        assert!(app.commands_state.candidates().len() >= 2); // list, show
    }

    #[tokio::test]
    async fn bare_cli_command_without_children_executes() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/formats` (no children in test tree) and press Enter
        for c in "/formats".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should have spawned a command (new exchange message)
        assert!(app.messages.len() > initial_msg_count);
        // Input should be consumed
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn cli_command_with_subcommand_executes() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills list` and press Enter — should execute, not open picker
        for c in "/skills list".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should have spawned a command
        assert!(app.messages.len() > initial_msg_count);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn cli_subcommand_with_required_args_shows_hint_on_enter() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills show` (requires <NAME>) and press Enter
        for c in "/skills show".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should NOT have spawned a command
        assert_eq!(app.messages.len(), initial_msg_count);
        // Input should be preserved with the command text
        assert_eq!(app.input.text(), "/skills show");
        // Ghost suggestion should show the usage hint
        assert_eq!(app.ghost_suggestion, Some(" <NAME>".to_string()));
    }

    #[tokio::test]
    async fn cli_subcommand_with_args_provided_executes() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills show foo` (required arg provided) and press Enter
        for c in "/skills show foo".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should have spawned a command
        assert!(app.messages.len() > initial_msg_count);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn cli_subcommand_no_required_args_executes() {
        use std::sync::Arc;

        let mut app = App::new_for_test().await;
        let tree = Arc::new(crate::cli_commands::test_cli_tree());
        app.cli_tree = Some(Arc::clone(&tree));
        app.commands_state.set_cli_tree(tree);

        let initial_msg_count = app.messages.len();

        // Type `/skills list` (no required args) and press Enter
        for c in "/skills list".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // Should have spawned a command immediately
        assert!(app.messages.len() > initial_msg_count);
        assert!(app.input.is_empty());
    }
}
