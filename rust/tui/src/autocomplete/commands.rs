use std::sync::Arc;

use strum::IntoEnumIterator;

use crate::cli_commands::CliCommandNode;
use crate::commands::SlashCommand;

/// A candidate shown in the autocomplete popup.
#[derive(Debug, Clone)]
pub enum CommandCandidate {
    Builtin(SlashCommand),
    CliCommand {
        /// Full path, e.g. `["skills"]` or `["skills", "list"]`
        path: Vec<String>,
        description: String,
    },
}

impl CommandCandidate {
    /// The slash-prefixed display name.
    pub fn name(&self) -> String {
        match self {
            Self::Builtin(cmd) => cmd.name(),
            Self::CliCommand { path, .. } => format!("/{}", path.join(" ")),
        }
    }

    /// A short description for the autocomplete popup.
    pub fn description(&self) -> &str {
        match self {
            Self::Builtin(cmd) => cmd.description(),
            Self::CliCommand { description, .. } => description,
        }
    }
}

/// State for the autocomplete popup shown when typing `/` commands.
pub struct CommandsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Filtered candidates matching the current input prefix.
    candidates: Vec<CommandCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
    /// CLI command tree, set after construction. None = no CLI commands available.
    cli_tree: Option<Arc<Vec<CliCommandNode>>>,
}

impl CommandsState {
    /// Create a new hidden autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
            cli_tree: None,
        }
    }

    /// Inject the CLI command tree for autocomplete.
    pub fn set_cli_tree(&mut self, tree: Arc<Vec<CliCommandNode>>) {
        self.cli_tree = Some(tree);
    }

    /// Whether the autocomplete popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[CommandCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Update the autocomplete state based on the current input text.
    ///
    /// Supports hierarchical completion:
    /// - `/` → all top-level commands
    /// - `/sk` → filter top-level by prefix
    /// - `/skills ` → show children of "skills"
    /// - `/skills li` → filter children by prefix
    /// - deeper paths → dismiss
    pub fn update(&mut self, input: &str) {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('/') {
            self.dismiss();
            return;
        }
        let without_slash = &trimmed[1..];

        let words: Vec<&str> = without_slash.split_whitespace().collect();
        let trailing_space = without_slash.ends_with(' ') || without_slash.ends_with('\t');

        match (words.len(), trailing_space) {
            // Just "/" — show everything
            (0, _) => {
                self.candidates = self.all_top_level_candidates();
            }
            // "/sk" (partial, no space) — filter top-level by prefix
            (1, false) => {
                let prefix = words[0];
                self.candidates = self.top_level_matching(prefix);
            }
            // "/skills " (complete word + space) — show children
            (1, true) => {
                let parent = words[0];
                self.candidates = self.children_of(parent);
            }
            // "/skills li" (parent + partial child) — filter children
            (2, false) => {
                let parent = words[0];
                let child_prefix = words[1];
                self.candidates = self.children_of_matching(parent, child_prefix);
            }
            // Deeper or exact two-word match — dismiss
            _ => {
                self.dismiss();
                return;
            }
        }

        if self.candidates.is_empty()
            || (self.candidates.len() == 1 && self.is_exact_single_match(trimmed))
        {
            self.dismiss();
        } else {
            self.visible = true;
            self.selected = self.selected.min(self.candidates.len().saturating_sub(1));
        }
    }

    /// Check if there's exactly one candidate that exactly matches the input.
    fn is_exact_single_match(&self, input: &str) -> bool {
        self.candidates.len() == 1 && self.candidates[0].name() == input
    }

    /// All top-level candidates: built-in commands + CLI root commands.
    fn all_top_level_candidates(&self) -> Vec<CommandCandidate> {
        let mut candidates: Vec<CommandCandidate> = SlashCommand::iter()
            .filter(|cmd| !cmd.is_hidden())
            .map(CommandCandidate::Builtin)
            .collect();

        if let Some(ref tree) = self.cli_tree {
            for node in crate::cli_commands::visible_top_level(tree) {
                candidates.push(CommandCandidate::CliCommand {
                    path: vec![node.name.clone()],
                    description: node.description.clone(),
                });
            }
        }

        candidates
    }

    /// Top-level candidates matching a prefix.
    fn top_level_matching(&self, prefix: &str) -> Vec<CommandCandidate> {
        let slash_prefix = format!("/{prefix}");
        self.all_top_level_candidates()
            .into_iter()
            .filter(|c| c.name().starts_with(&slash_prefix))
            .collect()
    }

    /// Children of a parent CLI command.
    fn children_of(&self, parent: &str) -> Vec<CommandCandidate> {
        let Some(ref tree) = self.cli_tree else {
            return Vec::new();
        };
        let Some(node) = tree.iter().find(|n| n.name == parent) else {
            return Vec::new();
        };
        node.children
            .iter()
            .map(|child| CommandCandidate::CliCommand {
                path: vec![parent.to_string(), child.name.clone()],
                description: child.description.clone(),
            })
            .collect()
    }

    /// Children of a parent matching a prefix.
    fn children_of_matching(&self, parent: &str, child_prefix: &str) -> Vec<CommandCandidate> {
        self.children_of(parent)
            .into_iter()
            .filter(|c| match c {
                CommandCandidate::CliCommand { path, .. } => path
                    .last()
                    .is_some_and(|name| name.starts_with(child_prefix)),
                CommandCandidate::Builtin(_) => false,
            })
            .collect()
    }

    /// Move selection to the next candidate, wrapping around.
    pub fn select_next(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = (self.selected + 1) % self.candidates.len();
        }
    }

    /// Move selection to the previous candidate, wrapping around.
    pub fn select_prev(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = if self.selected == 0 {
                self.candidates.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Accept the currently selected candidate.
    ///
    /// Returns the full command name to insert into the input.
    /// For commands with children, appends a trailing space to trigger drill-down.
    pub fn accept(&mut self) -> Option<String> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }
        let candidate = &self.candidates[self.selected];
        let name = candidate.name();

        // Check if this is a top-level CLI command with children
        let has_children = match candidate {
            CommandCandidate::CliCommand { path, .. } if path.len() == 1 => {
                if let Some(ref tree) = self.cli_tree {
                    tree.iter()
                        .find(|n| n.name == path[0])
                        .is_some_and(|n| !n.children.is_empty())
                } else {
                    false
                }
            }
            _ => false,
        };

        self.dismiss();

        if has_children {
            // Append trailing space to trigger drill-down on next update
            Some(format!("{name} "))
        } else {
            Some(name)
        }
    }

    /// Look up the usage hint for a CLI command path (e.g. `["mcp", "add"]`).
    ///
    /// Returns `Some("…")` when the leaf node has required positional args,
    /// `None` otherwise.
    pub fn usage_hint_for(&self, path: &[String]) -> Option<String> {
        let tree = self.cli_tree.as_deref()?;
        crate::cli_commands::find_missing_args_hint(tree, path)
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.candidates.clear();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cli_tree() -> Arc<Vec<CliCommandNode>> {
        Arc::new(crate::cli_commands::test_cli_tree())
    }

    #[test]
    fn initially_hidden() {
        let state = CommandsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn shows_on_slash() {
        let mut state = CommandsState::new();
        state.update("/");
        assert!(state.is_visible());
        assert!(!state.candidates().is_empty());
    }

    #[test]
    fn shows_builtins_and_cli_on_slash() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/");
        assert!(state.is_visible());
        // Should have both builtins and CLI commands
        let has_builtin = state
            .candidates()
            .iter()
            .any(|c| matches!(c, CommandCandidate::Builtin(SlashCommand::Help)));
        let has_cli = state
            .candidates()
            .iter()
            .any(|c| matches!(c, CommandCandidate::CliCommand { path, .. } if path == &["skills"]));
        assert!(has_builtin);
        assert!(has_cli);
    }

    #[test]
    fn filters_by_prefix() {
        let mut state = CommandsState::new();
        state.update("/h");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 2); // /help, /history
        assert!(state.candidates().iter().any(|c| c.name() == "/help"));
        assert!(state.candidates().iter().any(|c| c.name() == "/history"));
    }

    #[test]
    fn filters_cli_commands_by_prefix() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/sk");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name(), "/skills");
    }

    #[test]
    fn hides_on_no_match() {
        let mut state = CommandsState::new();
        state.update("/z");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn hides_on_exact_single_match() {
        let mut state = CommandsState::new();
        state.update("/quit");
        // /quit is the only match and is exact — hide popup
        assert!(!state.is_visible());
    }

    #[test]
    fn hides_on_non_slash_input() {
        let mut state = CommandsState::new();
        state.update("hello");
        assert!(!state.is_visible());
    }

    #[test]
    fn shows_children_on_trailing_space() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/skills ");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 2); // list, show
        assert!(
            state
                .candidates()
                .iter()
                .any(|c| c.name() == "/skills list")
        );
        assert!(
            state
                .candidates()
                .iter()
                .any(|c| c.name() == "/skills show")
        );
    }

    #[test]
    fn filters_children_by_prefix() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/skills li");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name(), "/skills list");
    }

    #[test]
    fn dismisses_on_deep_paths() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/skills list --verbose");
        assert!(!state.is_visible());
    }

    #[test]
    fn dismisses_when_no_children() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        // "models" has no children, trailing space dismisses
        state.update("/models ");
        assert!(!state.is_visible());
    }

    #[test]
    fn builtin_not_duplicated_by_cli() {
        let mut state = CommandsState::new();
        // Create a CLI tree that has a command with same name as a builtin
        let tree = Arc::new(vec![CliCommandNode {
            name: "workflows".to_string(),
            description: "CLI workflows".to_string(),
            children: vec![],
            usage_hint: String::new(),
        }]);
        state.set_cli_tree(tree);
        state.update("/");
        // "workflows" should appear only once (as a builtin)
        let wf_count = state
            .candidates()
            .iter()
            .filter(|c| c.name() == "/workflows")
            .count();
        assert_eq!(wf_count, 1);
    }

    #[test]
    fn select_next_wraps() {
        let mut state = CommandsState::new();
        state.update("/");
        let count = state.candidates().len();
        assert!(count > 1);

        for _ in 0..count {
            state.select_next();
        }
        // Wrapped back to 0
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = CommandsState::new();
        state.update("/");
        assert_eq!(state.selected(), 0);

        state.select_prev();
        // Wrapped to last
        assert_eq!(state.selected(), state.candidates().len() - 1);
    }

    #[test]
    fn accept_returns_name_and_dismisses() {
        let mut state = CommandsState::new();
        state.update("/h");
        assert!(state.is_visible());

        let name = state.accept();
        assert!(name.is_some());
        assert!(name.expect("just checked").starts_with("/h"));
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_cli_with_children_appends_space() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/sk");
        assert!(state.is_visible());
        // "skills" is the only match and has children
        let name = state.accept();
        assert_eq!(name, Some("/skills ".into()));
    }

    #[test]
    fn accept_cli_without_children_no_space() {
        let mut state = CommandsState::new();
        state.set_cli_tree(make_cli_tree());
        state.update("/mo");
        assert!(state.is_visible());
        // "models" has no children
        let name = state.accept();
        assert_eq!(name, Some("/models".into()));
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = CommandsState::new();
        assert_eq!(state.accept(), None);
    }

    #[test]
    fn dismiss_resets_state() {
        let mut state = CommandsState::new();
        state.update("/");
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn selection_clamped_on_narrowing() {
        let mut state = CommandsState::new();
        state.update("/");
        // Select a later item
        let initial_count = state.candidates().len();
        for _ in 0..initial_count - 1 {
            state.select_next();
        }
        assert_eq!(state.selected(), initial_count - 1);

        // Narrow to fewer candidates — selection should be clamped
        state.update("/h");
        assert!(state.selected() < state.candidates().len());
    }
}
