use std::fmt::Write;

use strum::{Display, EnumIter, EnumMessage, EnumString, IntoEnumIterator};

use crate::app::{App, AppMode, AppMessage};

/// Slash commands available in the TUI.
///
/// Note that for each variant, the comment is the description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter, EnumMessage)]
#[strum(serialize_all = "lowercase")]
pub enum SlashCommand {
    #[strum(message = "Clear messages")]
    Clear,

    #[strum(message = "Show help information")]
    Help,

    #[strum(message = "Show recent history")]
    History,

    #[strum(message = "Enter shell mode")]
    Shell,

    #[strum(message = "Exit shell mode or quit")]
    Exit,

    #[strum(message = "Quit the application")]
    Quit,
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
            Self::Clear => execute_clear(app),
            Self::Exit => match app.mode {
                AppMode::Shell => app.exit_shell_mode(),
                AppMode::Chat => app.should_quit = true,
            },
            Self::Help => execute_help(app),
            Self::History => execute_history(app),
            Self::Quit => app.should_quit = true,
            Self::Shell => app.enter_shell_mode(),
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
    help.push_str("  Ctrl+C         Quit (chat) / clear or cancel (shell)\n");
    help.push_str("  Ctrl+D         Exit shell mode\n");
    help.push_str("  Ctrl+L         Clear messages\n");
    help.push_str("  PageUp/Down    Scroll messages\n");
    help.push_str("  !command       Run a shell command from chat mode");
    app.messages.push(AppMessage::System { content: help });
}

fn execute_clear(app: &mut App) {
    app.messages.clear();
    app.scroll_offset = 0;
}

fn execute_history(app: &mut App) {
    let entries = app.input_history.entries();
    if entries.is_empty() {
        app.messages.push(AppMessage::System {
            content: "No history entries.".to_string(),
        });
        return;
    }

    // Show the last 20 entries
    let start = entries.len().saturating_sub(20);
    let mut text = String::from("Recent history:\n");
    for (i, entry) in entries[start..].iter().enumerate() {
        // Show single-line preview for multiline entries
        let preview = entry.lines().next().unwrap_or("");
        let suffix = if entry.contains('\n') { " ..." } else { "" };
        let _ = writeln!(text, "  {:3}. {preview}{suffix}", start + i + 1);
    }
    app.messages.push(AppMessage::System { content: text });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_description() {
        assert_eq!(SlashCommand::Help.name(), "/help");
        assert!(!SlashCommand::Help.description().is_empty());
    }

    #[test]
    fn matches_prefix_exact() {
        assert!(SlashCommand::Help.matches_prefix("/help"));
        assert!(SlashCommand::Clear.matches_prefix("/clear"));
    }

    #[test]
    fn matches_prefix_partial() {
        assert!(SlashCommand::Help.matches_prefix("/h"));
        assert!(SlashCommand::Help.matches_prefix("/he"));
        assert!(SlashCommand::History.matches_prefix("/h"));
        assert!(!SlashCommand::Help.matches_prefix("/c"));
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
    }

    #[test]
    fn parse_valid_commands() {
        assert_eq!(SlashCommand::parse("/help"), Some((SlashCommand::Help, "")));
        assert_eq!(
            SlashCommand::parse("/clear"),
            Some((SlashCommand::Clear, ""))
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
    }

    #[test]
    fn parse_not_a_command() {
        assert_eq!(SlashCommand::parse("hello"), None);
        assert_eq!(SlashCommand::parse(""), None);
    }

    #[test]
    fn execute_help_adds_message() {
        let mut app = App::new();
        let initial_count = app.messages.len();
        SlashCommand::Help.execute(&mut app, "");
        assert_eq!(app.messages.len(), initial_count + 1);
        assert!(matches!(
            &app.messages[initial_count],
            AppMessage::System { content } if content.contains("/help")
        ));
    }

    #[test]
    fn execute_clear_empties_messages() {
        let mut app = App::new();
        assert!(!app.messages.is_empty());
        SlashCommand::Clear.execute(&mut app, "");
        assert!(app.messages.is_empty());
    }

    #[test]
    fn execute_exit_quits_in_chat_mode() {
        let mut app = App::new();
        assert!(!app.should_quit);
        SlashCommand::Exit.execute(&mut app, "");
        assert!(app.should_quit);
    }

    #[test]
    fn execute_exit_returns_to_chat_in_shell_mode() {
        let mut app = App::new();
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);
        SlashCommand::Exit.execute(&mut app, "");
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
    }

    #[test]
    fn execute_quit_always_quits() {
        // From chat mode
        let mut app = App::new();
        SlashCommand::Quit.execute(&mut app, "");
        assert!(app.should_quit);

        // From shell mode
        let mut app = App::new();
        app.enter_shell_mode();
        SlashCommand::Quit.execute(&mut app, "");
        assert!(app.should_quit);
    }

    #[test]
    fn execute_shell_enters_shell_mode() {
        let mut app = App::new();
        assert_eq!(app.mode, AppMode::Chat);
        SlashCommand::Shell.execute(&mut app, "");
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[test]
    fn execute_history_empty() {
        let mut app = App::new();
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "");
        assert_eq!(app.messages.len(), initial + 1);
        assert!(matches!(
            &app.messages[initial],
            AppMessage::System { content } if content.contains("No history")
        ));
    }

    #[test]
    fn execute_history_with_entries() {
        let mut app = App::new();
        app.input_history.push("first".to_string());
        app.input_history.push("second".to_string());
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "");
        assert_eq!(app.messages.len(), initial + 1);
        assert!(matches!(
            &app.messages[initial],
            AppMessage::System { content } if content.contains("first") && content.contains("second")
        ));
    }
}
