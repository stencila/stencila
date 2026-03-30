use std::ops::Range;

/// Information about a discovered workflow definition.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WorkflowDefinitionInfo {
    pub name: String,
    pub description: String,
    pub goal: Option<String>,
    pub goal_hint: Option<String>,
}

/// A candidate in the workflow picker popup.
#[derive(Debug, Clone)]
pub struct WorkflowCandidate {
    pub name: String,
    pub info: WorkflowDefinitionInfo,
}

/// Result of accepting a workflow candidate via the `~` trigger.
pub struct WorkflowAcceptResult {
    /// The selected workflow definition.
    pub info: WorkflowDefinitionInfo,
    /// Byte range in the input to clear (the `~query` token).
    pub range: Range<usize>,
    /// Inline goal text appended after the workflow name (e.g.
    /// `~theme-creation a dark theme` → `"a dark theme"`).
    pub inline_goal: Option<String>,
}

/// State for the workflow picker popup.
///
/// Two modes of activation:
/// - **`~` trigger**: typing `~` at the start of input opens the popup with
///   live filtering (like `#` for agent mentions). Characters after `~` filter
///   by name/description.
/// - **`/workflow` command**: the popup is opened directly via [`open`] with a
///   pre-built candidate list (no live filtering).
pub struct WorkflowsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Filtered candidates shown in the popup.
    candidates: Vec<WorkflowCandidate>,
    /// Currently selected index.
    selected: usize,
    /// Byte range of the `~query` token when in tilde-trigger mode.
    token_range: Range<usize>,
    /// Whether this popup was opened via the `~` trigger (vs `/workflow`).
    tilde_mode: bool,
    /// Inline goal text extracted from `~workflow-name goal text`.
    inline_goal: Option<String>,
}

impl WorkflowsState {
    /// Create a new hidden workflows state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
            token_range: 0..0,
            tilde_mode: false,
            inline_goal: None,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of candidates.
    pub fn candidates(&self) -> &[WorkflowCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Whether this popup is in tilde-trigger mode.
    pub fn is_tilde_mode(&self) -> bool {
        self.tilde_mode
    }

    /// Open the workflow picker with a pre-built candidate list (from `/workflow` command).
    pub fn open(&mut self, candidates: Vec<WorkflowCandidate>) {
        self.candidates = candidates;
        self.selected = 0;
        self.tilde_mode = false;
        self.token_range = 0..0;
        self.visible = !self.candidates.is_empty();
    }

    /// Update the popup based on current input and cursor, filtering from
    /// the full list of discovered workflows.
    ///
    /// Called on every keystroke from `refresh_autocomplete`. If the input
    /// starts with `~`, shows workflows matching the text after `~`.
    pub fn update(&mut self, input: &str, cursor: usize, all_workflows: &[WorkflowCandidate]) {
        // If we're in /workflow (non-tilde) mode, don't interfere
        if self.visible && !self.tilde_mode {
            return;
        }

        let Some((range, query, inline_goal)) = find_tilde_token(input, cursor) else {
            if self.tilde_mode {
                self.visible = false;
                self.candidates.clear();
                self.selected = 0;
                self.inline_goal = None;
            }
            return;
        };

        self.tilde_mode = true;
        self.token_range = range;
        self.inline_goal = inline_goal.map(std::string::ToString::to_string);

        let query_lower = query.to_ascii_lowercase();
        self.candidates = all_workflows
            .iter()
            .filter(|c| {
                query_lower.is_empty()
                    || c.name.to_ascii_lowercase().contains(&query_lower)
                    || c.info
                        .description
                        .to_ascii_lowercase()
                        .contains(&query_lower)
            })
            .cloned()
            .collect();

        self.visible = !self.candidates.is_empty();

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
    /// In tilde mode, returns a [`WorkflowAcceptResult`] with the token range
    /// to clear. In `/workflow` mode, returns `None` for the range (caller
    /// should use [`accept_command`] instead).
    pub fn accept(&mut self) -> Option<WorkflowAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let info = self.candidates[self.selected].info.clone();
        let range = self.token_range.clone();
        let tilde = self.tilde_mode;
        let inline_goal = self.inline_goal.take();
        self.dismiss();

        Some(WorkflowAcceptResult {
            info,
            range: if tilde { range } else { 0..0 },
            inline_goal: if tilde { inline_goal } else { None },
        })
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.candidates.clear();
        self.selected = 0;
        self.token_range = 0..0;
        self.tilde_mode = false;
        self.inline_goal = None;
    }
}

/// Find a `~word` or `~word goal text` workflow token at position 0.
///
/// Returns `(range, name_query, inline_goal)`:
/// - `range` — byte range of the entire token (from `~` to the end of input),
///   used for clearing the input on accept.
/// - `name_query` — the workflow-name portion after `~` up to the cursor (or
///   up to the first space, whichever comes first), used for popup filtering.
/// - `inline_goal` — text after the first space following the workflow name,
///   if present. E.g. `~theme-creation a dark theme` yields
///   `Some("a dark theme")`.
///
/// Returns `None` if the input doesn't start with `~` or the cursor is before
/// the `~`.
fn find_tilde_token(input: &str, cursor: usize) -> Option<(Range<usize>, &str, Option<&str>)> {
    if !input.starts_with('~') || cursor < 1 {
        return None;
    }

    let after_tilde = &input[1..];

    // Find the end of the workflow-name portion (alphanumeric, `-`, `_`).
    let name_end = after_tilde
        .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(after_tilde.len());

    // The characters between `~` and the cursor must be within the name or
    // goal portion. Validate that the name part (before any space) contains
    // only valid chars.
    let cursor_offset = cursor - 1; // offset into after_tilde
    if cursor_offset <= name_end {
        // Cursor is within the name portion — validate all chars up to cursor.
        let up_to_cursor = &after_tilde[..cursor_offset];
        if !up_to_cursor
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
    } else {
        // Cursor is past the name — the character right after the name must
        // be a space (the separator between name and inline goal).
        if !after_tilde[name_end..].starts_with(' ') {
            return None;
        }
    }

    // The name query for filtering is the name portion up to the cursor (or
    // the full name if the cursor is past it).
    let name_query = &after_tilde[..name_end.min(cursor_offset)];

    // The entire input is the token range (name + optional goal).
    let range = 0..input.len();

    // Extract inline goal: everything after the first space past the name.
    let inline_goal = if name_end < after_tilde.len() && after_tilde.as_bytes()[name_end] == b' ' {
        let goal_text = after_tilde[name_end + 1..].trim();
        if goal_text.is_empty() {
            None
        } else {
            Some(goal_text)
        }
    } else {
        None
    };

    Some((range, name_query, inline_goal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candidates() -> Vec<WorkflowCandidate> {
        vec![
            WorkflowCandidate {
                name: "code-review".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "code-review".to_string(),
                    description: "Review code changes".to_string(),
                    goal: Some("Review the latest PR".to_string()),
                    ..Default::default()
                },
            },
            WorkflowCandidate {
                name: "deploy".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "deploy".to_string(),
                    description: "Deploy to production".to_string(),
                    goal: None,
                    ..Default::default()
                },
            },
            WorkflowCandidate {
                name: "test-suite".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "test-suite".to_string(),
                    description: "Run the test suite".to_string(),
                    goal: Some("Run all tests".to_string()),
                    ..Default::default()
                },
            },
        ]
    }

    // --- Basic state ---

    #[test]
    fn initially_hidden() {
        let state = WorkflowsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    // --- /workflow command mode (open) ---

    #[test]
    fn open_shows_candidates() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
        assert_eq!(state.selected(), 0);
        assert!(!state.is_tilde_mode());
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = WorkflowsState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

    // --- Navigation ---

    #[test]
    fn select_next_wraps() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());

        state.select_prev();
        assert_eq!(state.selected(), 2);
    }

    // --- Accept ---

    #[test]
    fn accept_returns_info() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());

        state.select_next();
        let result = state.accept();
        assert!(result.is_some());
        let result = result.expect("just checked");
        assert_eq!(
            result.info,
            WorkflowDefinitionInfo {
                name: "deploy".to_string(),
                description: "Deploy to production".to_string(),
                ..Default::default()
            }
        );
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = WorkflowsState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    // --- Tilde trigger mode ---

    #[test]
    fn tilde_triggers_all_candidates() {
        let mut state = WorkflowsState::new();
        state.update("~", 1, &sample_candidates());
        assert!(state.is_visible());
        assert!(state.is_tilde_mode());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn tilde_with_query_filters_by_name() {
        let mut state = WorkflowsState::new();
        state.update("~dep", 4, &sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn tilde_filters_by_description() {
        let mut state = WorkflowsState::new();
        state.update("~review", 7, &sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "code-review");
    }

    #[test]
    fn tilde_case_insensitive() {
        let mut state = WorkflowsState::new();
        state.update("~DEPLOY", 7, &sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn tilde_no_match_hides() {
        let mut state = WorkflowsState::new();
        state.update("~zzz", 4, &sample_candidates());
        assert!(!state.is_visible());
    }

    #[test]
    fn tilde_mid_input_hidden() {
        let mut state = WorkflowsState::new();
        state.update("ask ~dep", 8, &sample_candidates());
        assert!(!state.is_visible());
    }

    #[test]
    fn tilde_accept_returns_range() {
        let mut state = WorkflowsState::new();
        state.update("~dep", 4, &sample_candidates());
        assert!(state.is_visible());

        let result = state.accept().expect("should accept");
        assert_eq!(result.range, 0..4);
        assert_eq!(result.info.name, "deploy");
        assert!(!state.is_visible());
    }

    #[test]
    fn tilde_selection_clamped_on_filter() {
        let mut state = WorkflowsState::new();
        state.update("~", 1, &sample_candidates());
        state.select_next();
        state.select_next();
        assert_eq!(state.selected(), 2);

        state.update("~dep", 4, &sample_candidates());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn tilde_dismiss_on_delete_past_tilde() {
        let mut state = WorkflowsState::new();
        state.update("~dep", 4, &sample_candidates());
        assert!(state.is_visible());
        assert!(state.is_tilde_mode());

        // Simulate deleting all text
        state.update("", 0, &sample_candidates());
        assert!(!state.is_visible());
    }

    #[test]
    fn update_does_not_interfere_with_command_mode() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert!(!state.is_tilde_mode());

        // update() should not change anything when in command mode
        state.update("~dep", 4, &sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3); // unchanged
    }

    // --- find_tilde_token tests ---

    #[test]
    fn find_tilde_bare() {
        let result = find_tilde_token("~", 1);
        assert_eq!(result, Some((0..1, "", None)));
    }

    #[test]
    fn find_tilde_with_name() {
        let result = find_tilde_token("~deploy", 7);
        assert_eq!(result, Some((0..7, "deploy", None)));
    }

    #[test]
    fn find_tilde_partial_name() {
        let result = find_tilde_token("~dep", 4);
        assert_eq!(result, Some((0..4, "dep", None)));
    }

    #[test]
    fn find_tilde_with_hyphen() {
        let result = find_tilde_token("~code-review", 12);
        assert_eq!(result, Some((0..12, "code-review", None)));
    }

    #[test]
    fn find_tilde_with_underscore() {
        let result = find_tilde_token("~my_workflow", 12);
        assert_eq!(result, Some((0..12, "my_workflow", None)));
    }

    #[test]
    fn find_tilde_mid_input_none() {
        let result = find_tilde_token("ask ~deploy", 11);
        assert!(result.is_none());
    }

    #[test]
    fn find_tilde_cursor_mid_token() {
        let result = find_tilde_token("~deploy", 4);
        assert_eq!(result, Some((0..7, "dep", None)));
    }

    #[test]
    fn find_tilde_none_without_tilde() {
        let result = find_tilde_token("hello", 5);
        assert!(result.is_none());
    }

    #[test]
    fn find_tilde_cursor_at_zero() {
        let result = find_tilde_token("~deploy", 0);
        assert!(result.is_none());
    }

    #[test]
    fn find_tilde_invalid_chars() {
        let result = find_tilde_token("~foo.bar", 8);
        assert!(result.is_none());
    }

    // --- Inline goal tests ---

    #[test]
    fn find_tilde_with_inline_goal() {
        let result = find_tilde_token("~deploy to staging", 18);
        assert_eq!(result, Some((0..18, "deploy", Some("to staging"))));
    }

    #[test]
    fn find_tilde_with_inline_goal_cursor_in_name() {
        let result = find_tilde_token("~deploy to staging", 4);
        assert_eq!(result, Some((0..18, "dep", Some("to staging"))));
    }

    #[test]
    fn find_tilde_space_only_no_goal() {
        let result = find_tilde_token("~deploy ", 8);
        assert_eq!(result, Some((0..8, "deploy", None)));
    }

    #[test]
    fn find_tilde_space_cursor_at_space() {
        let result = find_tilde_token("~deploy goal", 8);
        assert_eq!(result, Some((0..12, "deploy", Some("goal"))));
    }

    #[test]
    fn find_tilde_inline_goal_with_special_chars() {
        let result = find_tilde_token("~theme-creation a theme for example.com", 39);
        assert_eq!(
            result,
            Some((0..39, "theme-creation", Some("a theme for example.com")))
        );
    }

    #[test]
    fn tilde_with_inline_goal_filters_by_name() {
        let mut state = WorkflowsState::new();
        state.update("~deploy to staging", 18, &sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn tilde_accept_with_inline_goal() {
        let mut state = WorkflowsState::new();
        state.update("~deploy to staging", 18, &sample_candidates());
        assert!(state.is_visible());

        let result = state.accept().expect("should accept");
        assert_eq!(result.range, 0..18);
        assert_eq!(result.info.name, "deploy");
        assert_eq!(result.inline_goal, Some("to staging".to_string()));
    }

    #[test]
    fn tilde_accept_without_inline_goal() {
        let mut state = WorkflowsState::new();
        state.update("~deploy", 7, &sample_candidates());
        assert!(state.is_visible());

        let result = state.accept().expect("should accept");
        assert_eq!(result.range, 0..7);
        assert_eq!(result.info.name, "deploy");
        assert_eq!(result.inline_goal, None);
    }

    #[test]
    fn command_mode_accept_has_no_inline_goal() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());

        let result = state.accept().expect("should accept");
        assert_eq!(result.inline_goal, None);
    }
}
