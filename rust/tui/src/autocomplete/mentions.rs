use std::ops::Range;

/// A mention autocomplete candidate representing an agent.
#[derive(Debug, Clone)]
pub struct MentionCandidate {
    /// The agent name.
    pub name: String,
    /// Agent's color.
    pub color: ratatui::style::Color,
    /// Agent definition info if it's a discovered agent.
    pub definition: Option<super::agents::AgentDefinitionInfo>,
}

/// Result of accepting a mention autocomplete candidate.
pub struct MentionAcceptResult {
    /// Byte range in the input to replace.
    pub range: Range<usize>,
    /// Text to insert (e.g. `#agent-name `).
    pub text: String,
}

/// State for the mention autocomplete popup triggered by `#` at the start of
/// input or after whitespace.
pub struct MentionsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Filtered candidates matching the current query.
    candidates: Vec<MentionCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
    /// Byte range of the `#name` token in the input.
    token_range: Range<usize>,
}

impl MentionsState {
    /// Create a new hidden mentions autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
            token_range: 0..0,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[MentionCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Update the mentions state based on current input and cursor position.
    ///
    /// `agents` is the full list of [`MentionCandidate`]s to filter against.
    pub fn update(&mut self, input: &str, cursor: usize, agents: &[MentionCandidate]) {
        if agents.is_empty() {
            self.visible = false;
            return;
        }

        let Some((range, query)) = find_mention_token(input, cursor) else {
            self.visible = false;
            return;
        };

        self.token_range = range;

        let query_lower = query.to_ascii_lowercase();
        self.candidates = agents
            .iter()
            .filter(|c| c.name.to_ascii_lowercase().starts_with(&query_lower))
            .cloned()
            .collect();

        self.visible = !self.candidates.is_empty();

        // Clamp selection
        if self.selected >= self.candidates.len() {
            self.selected = 0;
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
    /// Returns the byte range to replace and the text to insert, or `None` if
    /// nothing to accept.
    pub fn accept(&mut self) -> Option<MentionAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let text = format!("#{} ", candidate.name);
        let range = self.token_range.clone();

        self.dismiss();

        Some(MentionAcceptResult { range, text })
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.candidates.clear();
        self.selected = 0;
        self.token_range = 0..0;
    }
}

/// Find a `#word` mention token at position 0 of the input.
///
/// Returns the byte range of the token (from `#` through any trailing valid
/// mention characters) and the query string after `#` up to the cursor.
/// Returns `None` if the input doesn't start with `#` or the cursor is not
/// within the token.
///
/// The characters after `#` must be alphanumeric, hyphens, or underscores.
fn find_mention_token(input: &str, cursor: usize) -> Option<(Range<usize>, &str)> {
    if !input.starts_with('#') || cursor < 1 {
        return None;
    }

    // Everything between `#` and cursor must be valid mention chars
    let after_hash = &input[1..cursor];
    if !after_hash
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }

    // Token extends from `#` through any contiguous valid mention chars after
    // cursor too
    let mut end = cursor;
    for c in input[cursor..].chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            end += c.len_utf8();
        } else {
            break;
        }
    }

    Some((0..end, after_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::style::Color;

    fn sample_agents() -> Vec<MentionCandidate> {
        vec![
            MentionCandidate {
                name: "coder".to_string(),
                color: Color::Blue,
                definition: None,
            },
            MentionCandidate {
                name: "data-analyst".to_string(),
                color: Color::Magenta,
                definition: None,
            },
            MentionCandidate {
                name: "reviewer".to_string(),
                color: Color::Cyan,
                definition: None,
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = MentionsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn hash_triggers_all_candidates() {
        let mut state = MentionsState::new();
        state.update("#", 1, &sample_agents());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn hash_with_query_filters() {
        let mut state = MentionsState::new();
        state.update("#cod", 4, &sample_agents());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "coder");

        // Case-insensitive
        let mut state = MentionsState::new();
        state.update("#COD", 4, &sample_agents());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "coder");
    }

    #[test]
    fn hash_no_match_hides() {
        let mut state = MentionsState::new();
        state.update("#zzz", 4, &sample_agents());
        assert!(!state.is_visible());
    }

    #[test]
    fn hash_mid_input_hidden() {
        let mut state = MentionsState::new();
        state.update("ask #cod", 8, &sample_agents());
        assert!(!state.is_visible());
    }

    #[test]
    fn hash_after_space_hidden() {
        let mut state = MentionsState::new();
        state.update("hello #", 7, &sample_agents());
        assert!(!state.is_visible());
    }

    #[test]
    fn hash_not_after_nonspace_hides() {
        let mut state = MentionsState::new();
        state.update("foo#bar", 7, &sample_agents());
        assert!(!state.is_visible());
    }

    #[test]
    fn no_hash_stays_hidden() {
        let mut state = MentionsState::new();
        state.update("hello", 5, &sample_agents());
        assert!(!state.is_visible());
    }

    #[test]
    fn select_next_wraps() {
        let mut state = MentionsState::new();
        state.update("#", 1, &sample_agents());
        assert_eq!(state.selected(), 0);

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 0); // wrapped
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = MentionsState::new();
        state.update("#", 1, &sample_agents());
        assert_eq!(state.selected(), 0);

        state.select_prev();
        assert_eq!(state.selected(), 2); // wrapped to last
    }

    #[test]
    fn accept_returns_result() {
        let mut state = MentionsState::new();
        state.update("#cod", 4, &sample_agents());
        assert!(state.is_visible());

        let result = state.accept();
        assert!(result.is_some());
        let result = result.expect("accept should return Some");
        assert_eq!(result.range, 0..4);
        assert_eq!(result.text, "#coder ");
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = MentionsState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = MentionsState::new();
        state.update("#", 1, &sample_agents());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn selection_clamped_on_filter() {
        let mut state = MentionsState::new();
        state.update("#", 1, &sample_agents());
        state.select_next();
        state.select_next();
        assert_eq!(state.selected(), 2);

        // Re-filter to fewer candidates
        state.update("#cod", 4, &sample_agents());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    // --- find_mention_token tests ---

    #[test]
    fn find_mention_bare_hash() {
        let result = find_mention_token("#", 1);
        assert_eq!(result, Some((0..1, "")));
    }

    #[test]
    fn find_mention_with_name() {
        let result = find_mention_token("#coder", 6);
        assert_eq!(result, Some((0..6, "coder")));
    }

    #[test]
    fn find_mention_partial_name() {
        let result = find_mention_token("#cod", 4);
        assert_eq!(result, Some((0..4, "cod")));
    }

    #[test]
    fn find_mention_with_hyphen() {
        let result = find_mention_token("#data-analyst", 13);
        assert_eq!(result, Some((0..13, "data-analyst")));
    }

    #[test]
    fn find_mention_with_underscore() {
        let result = find_mention_token("#my_agent", 9);
        assert_eq!(result, Some((0..9, "my_agent")));
    }

    #[test]
    fn find_mention_mid_input_none() {
        let result = find_mention_token("ask #coder something", 10);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_cursor_mid_token() {
        // Cursor between "cod" and "er"
        let result = find_mention_token("#coder", 4);
        assert_eq!(result, Some((0..6, "cod")));
    }

    #[test]
    fn find_mention_none_without_hash() {
        let result = find_mention_token("hello", 5);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_none_after_nonspace() {
        let result = find_mention_token("foo#bar", 7);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_after_space_none() {
        let result = find_mention_token("hello #rev", 10);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_at_start() {
        let result = find_mention_token("#rev", 4);
        assert_eq!(result, Some((0..4, "rev")));
    }

    #[test]
    fn find_mention_hash_mid_input_after_space_none() {
        let result = find_mention_token("text #", 6);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_invalid_chars() {
        let result = find_mention_token("#foo.bar", 8);
        assert!(result.is_none());
    }

    #[test]
    fn find_mention_cursor_at_zero() {
        let result = find_mention_token("#code-reviewer review the changes", 0);
        assert!(result.is_none());
    }
}
