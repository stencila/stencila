use crate::commands::SlashCommand;

/// State for the autocomplete popup shown when typing `/` commands.
pub struct CommandsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Filtered candidates matching the current input prefix.
    candidates: Vec<SlashCommand>,
    /// Currently selected index within `candidates`.
    selected: usize,
}

impl CommandsState {
    /// Create a new hidden autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
        }
    }

    /// Whether the autocomplete popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[SlashCommand] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Update the autocomplete state based on the current input text.
    ///
    /// Shows the popup when the input starts with `/` and there are matches.
    /// Hides it otherwise.
    pub fn update(&mut self, input: &str) {
        let trimmed = input.trim_start();

        // Only trigger for slash-prefixed input that's a single word (no spaces yet)
        if trimmed.starts_with('/') && !trimmed.contains(char::is_whitespace) {
            let candidates = SlashCommand::matching(trimmed);
            if candidates.is_empty() {
                self.dismiss();
            } else {
                // If the input exactly matches one command and it's the only candidate, hide
                if candidates.len() == 1 && candidates[0].name() == trimmed {
                    self.dismiss();
                } else {
                    self.candidates = candidates;
                    // Clamp selection to valid range
                    if self.selected >= self.candidates.len() {
                        self.selected = 0;
                    }
                    self.visible = true;
                }
            }
        } else {
            self.dismiss();
        }
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
    /// Returns the full command name to insert into the input, or `None` if
    /// there's nothing to accept.
    pub fn accept(&mut self) -> Option<&'static str> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }
        let name = self.candidates[self.selected].name();
        self.dismiss();
        Some(name)
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
    fn filters_by_prefix() {
        let mut state = CommandsState::new();
        state.update("/h");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 2); // /help, /history
        assert!(state.candidates().contains(&SlashCommand::Help));
        assert!(state.candidates().contains(&SlashCommand::History));
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
    fn hides_when_space_present() {
        let mut state = CommandsState::new();
        state.update("/model gpt");
        assert!(!state.is_visible());
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
