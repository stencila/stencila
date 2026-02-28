use std::fmt::Write;

use strum::{Display, EnumIter, EnumMessage, EnumString, IntoEnumIterator};

use crate::app::{App, AppMessage, AppMode};
use crate::autocomplete::agents::{AgentCandidate, AgentCandidateKind, AgentDefinitionInfo};
use crate::autocomplete::workflows::{WorkflowCandidate, WorkflowDefinitionInfo};
use crate::cli_commands::CliCommandNode;

/// Slash commands available in the TUI.
///
/// Note that for each variant, the comment is the description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter, EnumMessage)]
#[strum(serialize_all = "lowercase")]
pub enum SlashCommand {
    #[strum(serialize = "workflow", message = "Run a workflow")]
    Workflow,

    #[strum(serialize = "agent", message = "Start and switch agent sessions")]
    Agent,

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

    #[strum(message = "Upgrade to the latest version")]
    Upgrade,

    #[strum(message = "Exit shell mode or quit")]
    Exit,

    #[strum(message = "Quit the application")]
    Quit,
}

/// A slot in the display ordering: either a built-in command, a specific
/// CLI passthrough command (by name), or a marker for all remaining CLI
/// commands not explicitly placed.
#[derive(Debug, Clone, Copy)]
pub enum CommandSlot {
    Builtin(SlashCommand),
    Cli(&'static str),
    RemainingCli,
}

impl SlashCommand {
    /// The canonical display ordering for slash commands and CLI passthrough
    /// commands. Edit this to control exactly where each command appears.
    ///
    /// - `Builtin(X)` places a built-in slash command.
    /// - `Cli("name")` places a specific CLI passthrough command by name.
    /// - `RemainingCli` places all CLI commands not explicitly listed via `Cli`.
    ///
    /// Any CLI command referenced by `Cli` that doesn't exist in the tree is
    /// silently skipped. If `RemainingCli` is omitted, unlisted CLI commands
    /// are appended at the end.
    pub fn display_order() -> &'static [CommandSlot] {
        use CommandSlot::*;
        use SlashCommand::*;
        &[
            Builtin(Workflow),
            Cli("workflows"),
            Builtin(Agent),
            Cli("agents"),
            Cli("skills"),
            Cli("models"),
            Builtin(Cancel),
            Builtin(Clear),
            Builtin(Help),
            Builtin(New),
            Builtin(History),
            Builtin(Shell),
            RemainingCli,
            Builtin(Upgrade),
            Builtin(Exit),
            Builtin(Quit),
        ]
    }
}

impl SlashCommand {
    /// The slash-prefixed name of this command (e.g. `/help`).
    pub fn name(self) -> String {
        ["/", &self.to_string()].concat()
    }

    /// A short description for the autocomplete popup.
    pub fn description(self) -> &'static str {
        self.get_message().unwrap_or("")
    }

    /// Whether this command is temporarily hidden from autocomplete and help.
    pub fn is_hidden(self) -> bool {
        matches!(self, Self::Clear)
    }

    /// Whether a CLI command name shadows a built-in slash command.
    ///
    /// Used by autocomplete and help to avoid showing duplicate entries.
    pub fn shadows_builtin(name: &str) -> bool {
        Self::iter().any(|cmd| cmd.to_string() == name)
    }

    /// Execute this command, mutating the app state.
    pub async fn execute(self, app: &mut App, _args: &str) {
        match self {
            Self::Agent => execute_agents(app).await,
            Self::Cancel => execute_cancel(app),
            Self::Clear => execute_clear(app),
            Self::New => execute_new(app).await,
            Self::Exit => match app.mode {
                AppMode::Shell => app.exit_shell_mode(),
                AppMode::Workflow => app.exit_workflow_mode(),
                AppMode::Agent => app.should_quit = true,
            },
            Self::Help => execute_help(app),
            Self::History => execute_history(app),
            Self::Quit => app.should_quit = true,
            Self::Shell => app.enter_shell_mode(),
            Self::Upgrade => execute_upgrade(app),
            Self::Workflow => execute_workflows(app).await,
        }
    }
}

/// Result of parsing a slash command input.
pub enum ParsedCommand<'a> {
    Builtin(SlashCommand, &'a str),
    CliPassthrough(CliPassthroughCmd),
}

/// A CLI passthrough command with its argv vector.
pub struct CliPassthroughCmd {
    /// The argv vector, e.g. `["skills", "list", "--as", "json"]`
    pub args: Vec<String>,
    /// Display name for the exchange, e.g. "skills list --as json"
    pub display: String,
}

impl SlashCommand {
    /// Whether this command only matches when no arguments follow.
    /// When args are present, the input falls through to CLI passthrough.
    pub fn is_exact_only(self) -> bool {
        matches!(
            self,
            Self::Agent
                | Self::Workflow
                | Self::Cancel
                | Self::Clear
                | Self::New
                | Self::Shell
                | Self::Quit
                | Self::Exit
        )
    }
}

/// Parse a command from the input text, checking built-in commands first,
/// then falling through to CLI passthrough commands.
///
/// Built-in commands classified as "exact-only" only match when no arguments
/// follow. With arguments, they fall through to CLI passthrough if the
/// command word matches a CLI tree entry (e.g. `/agents list` runs
/// `stencila agents list` instead of opening the popup). If no CLI tree
/// match exists, the built-in is executed anyway (ignoring the extra args)
/// to avoid the surprising behavior of `/quit now` being sent as chat text.
pub fn parse_command<'a>(input: &'a str, cli_tree: &[CliCommandNode]) -> Option<ParsedCommand<'a>> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }
    let without_slash = &trimmed[1..];
    let (cmd_word, args) = without_slash
        .split_once(char::is_whitespace)
        .map_or((without_slash, ""), |(c, a)| (c, a.trim()));

    let has_args = !args.is_empty();
    let builtin = cmd_word.parse::<SlashCommand>().ok();

    // 1. Non-exact-only built-in, or exact-only without args: use built-in
    if let Some(builtin) = builtin
        && (!has_args || !builtin.is_exact_only())
    {
        return Some(ParsedCommand::Builtin(builtin, args));
    }

    // 2. Try CLI tree match (exact-only built-ins with args fall through here)
    if cli_tree.iter().any(|node| node.name == cmd_word) {
        let mut cmd_args: Vec<String> = vec![cmd_word.to_string()];
        if has_args {
            cmd_args.extend(split_args(args));
        }
        let display = format!("stencila {}", cmd_args.join(" "));
        return Some(ParsedCommand::CliPassthrough(CliPassthroughCmd {
            args: cmd_args,
            display,
        }));
    }

    // 3. Exact-only built-in with args but no CLI tree match: still run
    //    the built-in rather than letting `/quit now` become chat text.
    if let Some(builtin) = builtin {
        return Some(ParsedCommand::Builtin(builtin, args));
    }

    None
}

/// Split an argument string into individual arguments.
///
/// Handles quoting (double and single quotes) and backslash escapes for
/// arguments containing spaces. Outside of quotes, `\ ` (backslash-space)
/// is treated as a literal space within the current argument.
fn split_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = args.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' if !in_single => {
                // Backslash escape: consume the next character literally
                if let Some(&next) = chars.peek() {
                    chars.next();
                    current.push(next);
                } else {
                    // Trailing backslash with nothing after — keep it
                    current.push('\\');
                }
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

async fn execute_agents(app: &mut App) {
    // Discovered agent definitions (from .stencila/agents/ and ~/.config/stencila/agents/)
    let definitions: Vec<_> =
        stencila_agents::agent_def::discover(&std::env::current_dir().unwrap_or_default()).await;

    // Existing TUI sessions
    let mut candidates: Vec<AgentCandidate> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let definition = s.definition.clone().or_else(|| {
                definitions
                    .iter()
                    .find(|d| d.name == s.name)
                    .map(|d| AgentDefinitionInfo {
                        name: d.name.clone(),
                        description: d.description.clone(),
                        model: d.model.clone(),
                        provider: d.provider.clone(),
                        source: d.source().map(|src| src.to_string()).unwrap_or_default(),
                    })
            });
            AgentCandidate {
                name: s.name.clone(),
                kind: AgentCandidateKind::Session {
                    index: i,
                    is_active: i == app.active_session,
                    definition,
                },
            }
        })
        .collect();

    // Add definitions that don't already have a session
    let session_names: Vec<&str> = app.sessions.iter().map(|s| s.name.as_str()).collect();
    for def in &definitions {
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

    app.agents_state.open(candidates);
}

async fn execute_workflows(app: &mut App) {
    let definitions: Vec<stencila_workflows::WorkflowInstance> =
        stencila_workflows::discover(&std::env::current_dir().unwrap_or_default()).await;

    let candidates: Vec<WorkflowCandidate> = definitions
        .into_iter()
        .map(|def| WorkflowCandidate {
            name: def.name.clone(),
            info: WorkflowDefinitionInfo {
                name: def.name.clone(),
                description: def.description.clone(),
                goal: def.goal.clone(),
            },
        })
        .collect();

    if candidates.is_empty() {
        app.messages.push(AppMessage::System {
            content: "No workflows found. Create one in .stencila/workflows/<name>/WORKFLOW.md"
                .to_string(),
        });
    } else {
        app.workflows_state.open(candidates);
    }
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

    let cli_nodes: Vec<&crate::cli_commands::CliCommandNode> = app
        .cli_tree
        .as_deref()
        .map(|tree| crate::cli_commands::visible_top_level(tree))
        .unwrap_or_default();

    let mut placed_cli: Vec<&str> = Vec::new();
    let write_cli_node =
        |help: &mut String, node: &crate::cli_commands::CliCommandNode| {
            let _ = writeln!(
                help,
                "  {:12} {}",
                format!("/{}", node.name),
                node.description
            );
        };

    for slot in SlashCommand::display_order() {
        match slot {
            CommandSlot::Builtin(cmd) if !cmd.is_hidden() => {
                let _ = writeln!(help, "  {:12} {}", cmd.name(), cmd.description());
            }
            CommandSlot::Cli(name) => {
                if let Some(node) = cli_nodes.iter().find(|n| n.name == *name) {
                    write_cli_node(&mut help, node);
                    placed_cli.push(name);
                }
            }
            CommandSlot::RemainingCli => {
                for node in &cli_nodes {
                    if !placed_cli.contains(&node.name.as_str()) {
                        write_cli_node(&mut help, node);
                    }
                }
                placed_cli.extend(cli_nodes.iter().map(|n| n.name.as_str()));
            }
            _ => {}
        }
    }

    // Safety net: any unplaced CLI commands
    for node in &cli_nodes {
        if !placed_cli.contains(&node.name.as_str()) {
            write_cli_node(&mut help, node);
        }
    }

    help.push_str("\nKey bindings:\n");
    help.push_str("  Enter          Send message / run command\n");
    help.push_str(
        "  Alt+Enter      Insert newline (Shift+Enter on supported terminals)\n\
  \\+Enter        Insert newline (trailing backslash continues input)\n",
    );
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

async fn execute_new(app: &mut App) {
    app.reset_all().await;
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
    use crate::cli_commands::test_cli_tree;

    #[test]
    fn name_and_description() {
        assert_eq!(SlashCommand::Help.name(), "/help");
        assert!(!SlashCommand::Help.description().is_empty());
    }

    #[test]
    fn agent_command_name() {
        assert_eq!(SlashCommand::Agent.name(), "/agent");
    }

    #[tokio::test]
    async fn execute_help_adds_message() {
        let mut app = App::new_for_test().await;
        let initial_count = app.messages.len();
        SlashCommand::Help.execute(&mut app, "").await;
        assert_eq!(app.messages.len(), initial_count + 1);
        assert!(matches!(
            &app.messages[initial_count],
            AppMessage::System { content } if content.contains("/help")
        ));
    }

    #[tokio::test]
    async fn execute_clear_resets_active_session() {
        let mut app = App::new_for_test().await;
        assert!(!app.messages.is_empty());
        SlashCommand::Clear.execute(&mut app, "").await;
        // Messages reset to just the welcome message
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn execute_new_resets_all() {
        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("extra"));
        app.active_session = 1;
        SlashCommand::New.execute(&mut app, "").await;
        // Back to a single default session
        assert_eq!(app.sessions.len(), 1);
        assert_eq!(app.active_session, 0);
        // Messages reset to just the welcome message
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], AppMessage::Welcome));
    }

    #[tokio::test]
    async fn execute_exit_quits_in_chat_mode() {
        let mut app = App::new_for_test().await;
        assert!(!app.should_quit);
        SlashCommand::Exit.execute(&mut app, "").await;
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn execute_exit_returns_to_chat_in_shell_mode() {
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();
        assert_eq!(app.mode, AppMode::Shell);
        SlashCommand::Exit.execute(&mut app, "").await;
        assert_eq!(app.mode, AppMode::Agent);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn execute_quit_always_quits() {
        // From chat mode
        let mut app = App::new_for_test().await;
        SlashCommand::Quit.execute(&mut app, "").await;
        assert!(app.should_quit);

        // From shell mode
        let mut app = App::new_for_test().await;
        app.enter_shell_mode();
        SlashCommand::Quit.execute(&mut app, "").await;
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn execute_shell_enters_shell_mode() {
        let mut app = App::new_for_test().await;
        assert_eq!(app.mode, AppMode::Agent);
        SlashCommand::Shell.execute(&mut app, "").await;
        assert_eq!(app.mode, AppMode::Shell);
    }

    #[tokio::test]
    async fn execute_history_empty() {
        let mut app = App::new_for_test().await;
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "").await;
        assert_eq!(app.messages.len(), initial + 1);
        assert!(matches!(
            &app.messages[initial],
            AppMessage::System { content } if content.contains("No history")
        ));
        assert!(!app.history_state.is_visible());
    }

    #[tokio::test]
    async fn execute_history_opens_popup() {
        let mut app = App::new_for_test().await;
        app.input_history.push("first".to_string());
        app.input_history.push("second".to_string());
        let initial = app.messages.len();
        SlashCommand::History.execute(&mut app, "").await;
        // No new message — popup opened instead
        assert_eq!(app.messages.len(), initial);
        assert!(app.history_state.is_visible());
        assert_eq!(app.history_state.candidates().len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn execute_agent_single_session_opens_popup() {
        let mut app = App::new_for_test().await;
        SlashCommand::Agent.execute(&mut app, "").await;
        // Should open popup with 1 existing agent
        // (plus any discovered definitions, but in test there are none)
        assert!(app.agents_state.is_visible());
        assert!(!app.agents_state.candidates().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn execute_agent_multiple_sessions() {
        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("test"));
        SlashCommand::Agent.execute(&mut app, "").await;
        assert!(app.agents_state.is_visible());
        // 2 existing agents (plus any discovered definitions)
        assert!(app.agents_state.candidates().len() >= 2);
    }

    // --- parse_command tests ---

    #[test]
    fn parse_command_builtin_exact() {
        let tree = test_cli_tree();
        let result = parse_command("/help", &tree);
        assert!(matches!(
            result,
            Some(ParsedCommand::Builtin(SlashCommand::Help, ""))
        ));
    }

    #[test]
    fn parse_command_builtin_with_args() {
        let tree = test_cli_tree();
        let result = parse_command("/help topics", &tree);
        assert!(matches!(
            result,
            Some(ParsedCommand::Builtin(SlashCommand::Help, "topics"))
        ));
    }

    #[test]
    fn parse_command_agent_exact_only_no_args() {
        let tree = test_cli_tree();
        let result = parse_command("/agent", &tree);
        assert!(matches!(
            result,
            Some(ParsedCommand::Builtin(SlashCommand::Agent, ""))
        ));
    }

    #[test]
    fn parse_command_agents_cli_passthrough_no_args() {
        let tree = test_cli_tree();
        let result = parse_command("/agents", &tree);
        match result {
            Some(ParsedCommand::CliPassthrough(cmd)) => {
                assert_eq!(cmd.args, vec!["agents"]);
                assert_eq!(cmd.display, "stencila agents");
            }
            _ => panic!("Expected CliPassthrough"),
        }
    }

    #[test]
    fn parse_command_agents_cli_passthrough_with_args() {
        let tree = test_cli_tree();
        let result = parse_command("/agents list", &tree);
        match result {
            Some(ParsedCommand::CliPassthrough(cmd)) => {
                assert_eq!(cmd.args, vec!["agents", "list"]);
                assert_eq!(cmd.display, "stencila agents list");
            }
            _ => panic!("Expected CliPassthrough"),
        }
    }

    #[test]
    fn parse_command_cli_only() {
        let tree = test_cli_tree();
        let result = parse_command("/skills list", &tree);
        match result {
            Some(ParsedCommand::CliPassthrough(cmd)) => {
                assert_eq!(cmd.args, vec!["skills", "list"]);
                assert_eq!(cmd.display, "stencila skills list");
            }
            _ => panic!("Expected CliPassthrough"),
        }
    }

    #[test]
    fn parse_command_cli_with_flags() {
        let tree = test_cli_tree();
        let result = parse_command("/skills list --as json", &tree);
        match result {
            Some(ParsedCommand::CliPassthrough(cmd)) => {
                assert_eq!(cmd.args, vec!["skills", "list", "--as", "json"]);
            }
            _ => panic!("Expected CliPassthrough"),
        }
    }

    #[test]
    fn parse_command_unknown() {
        let tree = test_cli_tree();
        assert!(parse_command("/notacmd", &tree).is_none());
    }

    #[test]
    fn parse_command_not_slash() {
        let tree = test_cli_tree();
        assert!(parse_command("hello", &tree).is_none());
    }

    #[test]
    fn parse_command_empty_cli_tree() {
        let tree: Vec<CliCommandNode> = vec![];
        // Built-ins still work
        assert!(matches!(
            parse_command("/help", &tree),
            Some(ParsedCommand::Builtin(SlashCommand::Help, ""))
        ));
        // Unknown returns None
        assert!(parse_command("/skills", &tree).is_none());
    }

    #[test]
    fn split_args_handles_quotes() {
        let args = split_args(r#"show "my skill" --verbose"#);
        assert_eq!(args, vec!["show", "my skill", "--verbose"]);
    }

    #[test]
    fn split_args_handles_single_quotes() {
        let args = split_args("show 'my skill' --verbose");
        assert_eq!(args, vec!["show", "my skill", "--verbose"]);
    }

    #[test]
    fn split_args_simple() {
        let args = split_args("list --as json");
        assert_eq!(args, vec!["list", "--as", "json"]);
    }

    #[test]
    fn split_args_backslash_escaped_space() {
        let args = split_args(r"show path\ with\ spaces --verbose");
        assert_eq!(args, vec!["show", "path with spaces", "--verbose"]);
    }

    #[test]
    fn split_args_backslash_escaped_quote() {
        let args = split_args(r#"show "it\'s a test""#);
        assert_eq!(args, vec!["show", "it's a test"]);
    }

    #[test]
    fn split_args_trailing_backslash() {
        let args = split_args(r"show trailing\");
        assert_eq!(args, vec![r"show", r"trailing\"]);
    }

    // --- Finding 2: exact-only built-ins with args but no CLI match ---

    #[test]
    fn parse_command_quit_with_args_still_quits() {
        // /quit is exact-only and "quit" is not in the CLI tree,
        // so it should still resolve to the Quit built-in.
        let tree = test_cli_tree();
        assert!(matches!(
            parse_command("/quit now", &tree),
            Some(ParsedCommand::Builtin(SlashCommand::Quit, "now"))
        ));
    }

    #[test]
    fn parse_command_exit_with_args_still_exits() {
        let tree = test_cli_tree();
        assert!(matches!(
            parse_command("/exit please", &tree),
            Some(ParsedCommand::Builtin(SlashCommand::Exit, "please"))
        ));
    }

    #[test]
    fn parse_command_clear_with_args_still_clears() {
        let tree = test_cli_tree();
        assert!(matches!(
            parse_command("/clear all", &tree),
            Some(ParsedCommand::Builtin(SlashCommand::Clear, "all"))
        ));
    }

    #[test]
    fn parse_command_shell_with_args_still_shell() {
        let tree = test_cli_tree();
        assert!(matches!(
            parse_command("/shell foo", &tree),
            Some(ParsedCommand::Builtin(SlashCommand::Shell, "foo"))
        ));
    }
}
