use std::fmt::Write;

use strum::{Display, EnumIter, EnumMessage, EnumString, IntoEnumIterator};

use crate::app::{App, AppMessage, AppMode};
use crate::autocomplete::agents::{AgentCandidate, AgentCandidateKind, AgentDefinitionInfo};

/// Slash commands available in the TUI.
///
/// Note that for each variant, the comment is the description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter, EnumMessage)]
#[strum(serialize_all = "lowercase")]
pub enum SlashCommand {
    #[strum(serialize = "agents", message = "Switch agents or create new")]
    Agents,

    #[strum(message = "Cancel a running command")]
    Cancel,

    #[strum(message = "New session for current agent")]
    Clear,

    #[strum(message = "Show help information")]
    Help,

    #[strum(message = "Reset all agents and messages")]
    New,

    #[strum(message = "Show recent history")]
    History,

    #[strum(message = "Enter shell mode")]
    Shell,

    #[strum(message = "Exit shell mode or quit")]
    Exit,

    #[strum(message = "Quit the application")]
    Quit,

    #[strum(message = "Upgrade to the latest version")]
    Upgrade,
}

impl SlashCommand {
    /// The slash-prefixed name of this command (e.g. `/help`).
    pub fn name(self) -> String {
        ["/", &self.to_string()].concat()
    }

    /// Slash-prefixed alias of this command.
    #[allow(clippy::unused_self)]
    pub fn aliases(self) -> Vec<&'static str> {
        Vec::new()
    }

    /// A short description for the autocomplete popup.
    pub fn description(self) -> &'static str {
        self.get_message().unwrap_or("")
    }

    /// Whether this command's name starts with the given prefix.
    pub fn matches_prefix(self, prefix: &str) -> bool {
        self.name().starts_with(prefix)
            || self.aliases().iter().any(|alias| alias.starts_with(prefix))
    }

    /// Return all commands whose name starts with `prefix`.
    pub fn matching(prefix: &str) -> Vec<SlashCommand> {
        Self::iter()
            .filter(|cmd| cmd.matches_prefix(prefix))
            .collect()
    }

    /// Parse a command from the input text.
    ///
    /// Returns the command and the remaining arguments, or `None` if the input
    /// doesn't match any command.
    pub fn parse(input: &str) -> Option<(SlashCommand, &str)> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        // Split into command word and arguments
        let (cmd_word, args) = trimmed
            .split_once(char::is_whitespace)
            .map_or((trimmed, ""), |(c, a)| (c, a.trim()));

        Self::iter()
            .find(|cmd| cmd.name() == cmd_word || cmd.aliases().contains(&cmd_word))
            .map(|cmd| (cmd, args))
    }

    /// Execute this command, mutating the app state.
    pub fn execute(self, app: &mut App, _args: &str) {
        match self {
            Self::Agents => execute_agents(app),
            Self::Cancel => execute_cancel(app),
            Self::Clear => execute_clear(app),
            Self::New => execute_new(app),
            Self::Exit => match app.mode {
                AppMode::Shell => app.exit_shell_mode(),
                AppMode::Chat => app.should_quit = true,
            },
            Self::Help => execute_help(app),
            Self::History => execute_history(app),
            Self::Quit => app.should_quit = true,
            Self::Shell => app.enter_shell_mode(),
            Self::Upgrade => execute_upgrade(app),
        }
    }
}

fn execute_agents(app: &mut App) {
    // Existing TUI sessions
    let mut candidates: Vec<AgentCandidate> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| AgentCandidate {
            name: s.name.clone(),
            kind: AgentCandidateKind::Session {
                index: i,
                is_active: i == app.active_session,
                definition: s.definition.clone(),
            },
        })
        .collect();

    // Discovered agent definitions (from .stencila/agents/ and ~/.config/stencila/agents/)
    let session_names: Vec<&str> = app.sessions.iter().map(|s| s.name.as_str()).collect();
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let definitions = tokio::task::block_in_place(|| {
            handle.block_on(stencila_agents::agent_def::discover(
                &std::env::current_dir().unwrap_or_default(),
            ))
        });
        for def in definitions {
            // Skip definitions whose name matches an already-active session
            if session_names.contains(&def.name.as_str()) {
                continue;
            }
            candidates.push(AgentCandidate {
                name: def.name.clone(),
                kind: AgentCandidateKind::Definition(AgentDefinitionInfo {
                    name: def.name.clone(),
                    description: def.description.clone(),
                    model: def.model.clone(),
                    provider: def.provider.clone(),
                    source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                }),
            });
        }
    }

    app.agents_state.open(candidates);
}

fn execute_cancel(app: &mut App) {
    let candidates = app.running_exchange_candidates();
    match candidates.len() {
        0 => {
            app.messages.push(AppMessage::System {
                content: "Nothing running.".to_string(),
            });
        }
        1 => {
            // Single running command — cancel immediately without popup
            let msg_index = candidates[0].msg_index;
            app.cancel_by_msg_index(msg_index);
        }
        _ => {
            // Multiple running commands — open picker popup
            app.cancel_state.open(candidates);
        }
    }
}

fn execute_help(app: &mut App) {
    let mut help = String::from("Available commands:\n");
    for cmd in SlashCommand::iter() {
        let _ = writeln!(help, "  {:12} {}", cmd.name(), cmd.description());
    }
    help.push_str("\nKey bindings:\n");
    help.push_str("  Enter          Send message / run command\n");
    help.push_str("  Alt+Enter      Insert newline\n");
    help.push_str("  Up/Down        History / cursor movement\n");
    help.push_str("  Ctrl+C         Cancel running / quit (chat) / clear (shell)\n");
    help.push_str("  Ctrl+D         Exit shell mode\n");
    help.push_str("  Ctrl+L         New session for current agent\n");
    help.push_str("  Ctrl+A         Cycle agents\n");
    help.push_str("  PageUp/Down    Scroll messages\n");
    help.push_str("  Esc            Scroll to bottom (when scrolled up)\n");
    help.push_str("  Scroll wheel   Scroll messages\n");
    help.push_str("  Shift+drag     Select text (terminal native copy)\n");
    help.push_str("  !command       Run a shell command from chat mode\n");
    help.push_str("  #agent prompt  Send prompt to agent (switches back after)\n");
    help.push_str("  #agent prompt& Send prompt to agent (stays on agent)\n");
    help.push_str("  #agent         Switch to agent session");
    app.messages.push(AppMessage::System { content: help });
}

fn execute_clear(app: &mut App) {
    app.reset_active_session();
}

fn execute_new(app: &mut App) {
    app.reset_all();
}

fn execute_history(app: &mut App) {
    let entries = app.input_history.entries_for_mode(app.mode, 20);
    if entries.is_empty() {
        app.messages.push(AppMessage::System {
            content: "No history entries.".to_string(),
        });
        return;
    }

    app.history_state.open(entries);
}

fn execute_upgrade(app: &mut App) {
    let exe = std::env::current_exe()
        .map_or_else(|_| "stencila".to_string(), |p| p.display().to_string());
    app.spawn_upgrade_command(format!("{exe} upgrade"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AgentSession;

    #[test]
    fn name_and_description() {
        assert_eq!(SlashCommand::Help.name(), "/help");
        assert!(!SlashCommand::Help.description().is_empty());
    }

    #[test]
    fn agents_command_name() {
        assert_eq!(SlashCommand::Agents.name(), "/agents");
    }

    #[test]
    fn matches_prefix_exact() {
        assert!(SlashCommand::Help.matches_prefix("/help"));
        assert!(SlashCommand::Clear.matches_prefix("/clear"));
        assert!(SlashCommand::Agents.matches_prefix("/agents"));
    }

    #[test]
    fn matches_prefix_partial() {
        assert!(SlashCommand::Help.matches_prefix("/h"));
        assert!(SlashCommand::Help.matches_prefix("/he"));
        assert!(SlashCommand::History.matches_prefix("/h"));
        assert!(!SlashCommand::Help.matches_prefix("/c"));
        assert!(SlashCommand::Agents.matches_prefix("/age"));
    }

    #[test]
    fn matches_prefix_slash_only() {
        // All commands match bare "/"
        for cmd in SlashCommand::iter() {
            assert!(cmd.matches_prefix("/"));
        }
    }

    #[test]
    fn matching_filters() {
        let all = SlashCommand::matching("/");
        assert_eq!(all.len(), SlashCommand::iter().count());

        let h_cmds = SlashCommand::matching("/h");
        assert_eq!(h_cmds.len(), 2); // /help, /history

        let s_cmds = SlashCommand::matching("/s");
        assert_eq!(s_cmds, vec![SlashCommand::Shell]);

        let q_cmds = SlashCommand::matching("/q");
        assert_eq!(q_cmds, vec![SlashCommand::Quit]);

        let none = SlashCommand::matching("/z");
        assert!(none.is_empty());

        let new_cmds = SlashCommand::matching("/new");
        assert_eq!(new_cmds, vec![SlashCommand::New]);
    }

    #[test]
    fn parse_valid_commands() {
        assert_eq!(SlashCommand::parse("/help"), Some((SlashCommand::Help, "")));
        assert_eq!(
            SlashCommand::parse("/clear"),
            Some((SlashCommand::Clear, ""))
        );
        assert_eq!(
            SlashCommand::parse("/agents"),
            Some((SlashCommand::Agents, ""))
        );
    }

    #[test]
    fn parse_with_leading_trailing_whitespace() {
        assert_eq!(
            SlashCommand::parse("  /quit  "),
            Some((SlashCommand::Quit, ""))
        );
        assert_eq!(
            SlashCommand::parse("  /history  "),
            Some((SlashCommand::History, ""))
        );
    }

    #[test]
    fn parse_unknown_command() {
        assert_eq!(SlashCommand::parse("/unknown"), None);
        // /model should no longer be recognized
        assert_eq!(SlashCommand::parse("/model"), None);
    }

    #[test]
    fn parse_not_a_command() {
        assert_eq!(SlashCommand::parse("hello"), None);
        assert_eq!(SlashCommand::parse(""), None);
    }

    #[test]
    fn execute_help_adds_message() {
        let mut app = App::new_for_test();
        let initial_count = app.messages.len();
        SlashCommand::Help.execute(&mut app, "");
        assert_eq!(app.messages.len(), initial_count + 1);
        assert!(matches!(
            &app.messages[initial_count],
            AppMessage::System { content } if content.contains("/help")
        ));
    }

    #[test]
    fn execute_clear_resets_active_session() {
        let mut app = App::new_for_test();
        assert!(!app.messages.is_empty());
        SlashCommand::Clear.execute(&mut app, "");
        // Messages reset to just the welcome message
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[test]
    fn execute_new_resets_all() {
        let mut app = App::new_for_test();
        app.sessions.push(AgentSession::new("extra"));
        app.active_session = 1;
        SlashCommand::New.execute(&mut app, "");
        // Back to a single default session
        assert_eq!(app.sessions.len(), 1);
        assert_eq!(app.active_session, 0);
        // Messages reset to just the welcome message
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[test]
    fn execute_exit_quits_in_chat_mode() {
        let mut app = App::new_for_test();
        assert!(!app.should_quit);
        SlashCommand::Exit.execute(&mut app, "");
        assert!(app.should_quit);
    }

    #[test]
    fn execute_exit_returns_to_chat_in_shell_mode() {
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);
        SlashCommand::Exit.execute(&mut app, "");
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[test]
    fn execute_quit_always_quits() {
        // From chat mode
        let mut app = App::new_for_test();
        SlashCommand::Quit.execute(&mut app, "");
        assert!(app.should_quit);

        // From shell mode
        let mut app = App::new_for_test();
        app.enter_shell_mode();
        SlashCommand::Quit.execute(&mut app, "");
        assert!(app.should_quit);
    }

    #[test]
    fn execute_shell_enters_shell_mode() {
        let mut app = App::new_for_test();
        assert_eq!(app.mode, AppMode::Chat);
        SlashCommand::Shell.execute(&mut app, "");
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[test]
    fn execute_history_empty() {
        let mut app = App::new_for_test();
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "");
        assert_eq!(app.messages.len(), initial + 1);
        assert!(matches!(
            &app.messages[initial],
            AppMessage::System { content } if content.contains("No history")
        ));
        assert!(!app.history_state.is_visible());
    }

    #[test]
    fn execute_history_opens_popup() {
        let mut app = App::new_for_test();
        app.input_history.push("first".to_string());
        app.input_history.push("second".to_string());
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "");
        // No new message — popup opened instead
        assert_eq!(app.messages.len(), initial);
        assert!(app.history_state.is_visible());
        assert_eq!(app.history_state.candidates().len(), 2);
    }

    #[test]
    fn execute_agents_single_session_opens_popup() {
        let mut app = App::new_for_test();
        SlashCommand::Agents.execute(&mut app, "");
        // Should open popup with 1 existing agent
        // (plus any discovered definitions, but in test there are none)
        assert!(app.agents_state.is_visible());
        assert!(app.agents_state.candidates().len() >= 1);
    }

    #[test]
    fn execute_agents_multiple_sessions() {
        let mut app = App::new_for_test();
        app.sessions.push(AgentSession::new("test"));
        SlashCommand::Agents.execute(&mut app, "");
        assert!(app.agents_state.is_visible());
        // 2 existing agents (plus any discovered definitions)
        assert!(app.agents_state.candidates().len() >= 2);
    }
}
