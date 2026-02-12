/// A history autocomplete candidate with a short preview and the full entry text.
#[derive(Debug, Clone)]
pub struct HistoryCandidate {
    /// Short preview (first line, truncated for multiline entries).
    pub preview: String,
    /// Full text of the history entry.
    pub full_text: String,
}

/// State for the history autocomplete popup triggered by `/history`.
pub struct HistoryState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// All entries loaded when the popup was opened (newest first).
    all_entries: Vec<HistoryCandidate>,
    /// Filtered candidates matching the current filter text.
    candidates: Vec<HistoryCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
}

impl HistoryState {
    /// Create a new hidden history autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            all_entries: Vec::new(),
            candidates: Vec::new(),
            selected: 0,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[HistoryCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the popup with the given entries (as `(preview, full_text)` tuples).
    ///
    /// Shows the popup immediately with all entries visible.
    pub fn open(&mut self, entries: Vec<(String, String)>) {
        self.all_entries = entries
            .into_iter()
            .map(|(preview, full_text)| HistoryCandidate { preview, full_text })
            .collect();
        self.candidates = self.all_entries.clone();
        self.selected = 0;
        self.visible = !self.candidates.is_empty();
    }

    /// Re-filter candidates by case-insensitive substring match on the preview.
    ///
    /// Hides the popup if no candidates match, but re-shows it if matches return
    /// (e.g. after the user deletes characters).
    pub fn update(&mut self, filter: &str) {
        // Only act while we have an active history session (all_entries populated by open())
        if self.all_entries.is_empty() {
            return;
        }

        let filter_lower = filter.to_lowercase();
        self.candidates = self
            .all_entries
            .iter()
            .filter(|c| c.preview.to_lowercase().contains(&filter_lower))
            .cloned()
            .collect();

        self.visible = !self.candidates.is_empty();

        // Clamp selection to valid range
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
    /// Returns the full text of the selected entry, or `None` if nothing to accept.
    pub fn accept(&mut self) -> Option<String> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }
        let full_text = self.candidates[self.selected].full_text.clone();
        self.dismiss();
        Some(full_text)
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.all_entries.clear();
        self.candidates.clear();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<(String, String)> {
        vec![
            ("echo hello".to_string(), "echo hello".to_string()),
            ("ls -la".to_string(), "ls -la".to_string()),
            (
                "multi line ...".to_string(),
                "multi line\ncommand here".to_string(),
            ),
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = HistoryState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_popup() {
        let mut state = HistoryState::new();
        state.open(sample_entries());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = HistoryState::new();
        state.open(vec![]);
        assert!(!state.is_visible());
    }

    #[test]
    fn dismiss_resets_state() {
        let mut state = HistoryState::new();
        state.open(sample_entries());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn filter_narrows_candidates() {
        let mut state = HistoryState::new();
        state.open(sample_entries());
        assert_eq!(state.candidates().len(), 3);

        state.update("echo");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].preview, "echo hello");
    }

    #[test]
    fn filter_case_insensitive() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        state.update("ECHO");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].preview, "echo hello");
    }

    #[test]
    fn filter_no_match_hides() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        state.update("zzz");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn filter_recovers_after_no_match() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        // No matches — popup hides
        state.update("zzz");
        assert!(!state.is_visible());

        // Matches return — popup re-appears
        state.update("echo");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);

        // Empty filter shows all again
        state.update("");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn filter_empty_shows_all() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        state.update("");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn select_next_wraps() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 0); // wrapped
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = HistoryState::new();
        state.open(sample_entries());
        assert_eq!(state.selected(), 0);

        state.select_prev();
        assert_eq!(state.selected(), 2); // wrapped to last
    }

    #[test]
    fn selection_clamped_on_filter() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        // Select last item
        state.select_next();
        state.select_next();
        assert_eq!(state.selected(), 2);

        // Filter to fewer items — selection should clamp
        state.update("echo");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn accept_returns_full_text() {
        let mut state = HistoryState::new();
        state.open(sample_entries());

        // Select the multiline entry
        state.select_next();
        state.select_next();
        let result = state.accept();
        assert_eq!(result, Some("multi line\ncommand here".to_string()));
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = HistoryState::new();
        assert_eq!(state.accept(), None);
    }

    #[test]
    fn update_without_active_session_is_noop() {
        let mut state = HistoryState::new();
        // No open() called — update should not show anything
        state.update("anything");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn update_after_dismiss_is_noop() {
        let mut state = HistoryState::new();
        state.open(sample_entries());
        state.dismiss();
        // dismiss() clears all_entries, so update should not revive the popup
        state.update("echo");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }
}
