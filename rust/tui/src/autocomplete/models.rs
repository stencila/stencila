/// A model autocomplete candidate.
#[derive(Debug, Clone)]
pub struct ModelCandidate {
    /// Provider namespace (e.g. "anthropic").
    pub provider: String,
    /// Model API identifier (e.g. "claude-opus-4-6").
    pub model_id: String,
    /// Human-readable display name (e.g. "Claude Opus 4.6").
    pub display_name: String,
}

/// State for the model picker popup triggered by `/model`.
pub struct ModelsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// All candidates loaded when the popup was opened (for filtering).
    all_candidates: Vec<ModelCandidate>,
    /// Filtered candidates matching the current filter text.
    candidates: Vec<ModelCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
}

impl ModelsState {
    /// Create a new hidden models state.
    pub fn new() -> Self {
        Self {
            visible: false,
            all_candidates: Vec::new(),
            candidates: Vec::new(),
            selected: 0,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[ModelCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the model picker with the given candidates.
    pub fn open(&mut self, candidates: Vec<ModelCandidate>) {
        self.all_candidates = candidates;
        self.candidates = self.all_candidates.clone();
        self.selected = 0;
        self.visible = !self.candidates.is_empty();
    }

    /// Re-filter candidates by case-insensitive substring match on id, display name, or provider.
    ///
    /// Hides the popup if no candidates match, but re-shows it if matches return.
    pub fn update(&mut self, filter: &str) {
        if self.all_candidates.is_empty() {
            return;
        }

        let filter_lower = filter.to_lowercase();
        self.candidates = self
            .all_candidates
            .iter()
            .filter(|c| {
                c.model_id.to_lowercase().contains(&filter_lower)
                    || c.display_name.to_lowercase().contains(&filter_lower)
                    || c.provider.to_lowercase().contains(&filter_lower)
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
    /// Returns the selected `ModelCandidate`, or `None` if nothing to accept.
    pub fn accept(&mut self) -> Option<ModelCandidate> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = self.candidates[self.selected].clone();
        self.dismiss();
        Some(candidate)
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.all_candidates.clear();
        self.candidates.clear();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candidates() -> Vec<ModelCandidate> {
        vec![
            ModelCandidate {
                provider: "anthropic".to_string(),
                model_id: "claude-opus-4-6".to_string(),
                display_name: "Claude Opus 4.6".to_string(),
            },
            ModelCandidate {
                provider: "anthropic".to_string(),
                model_id: "claude-sonnet-4-5".to_string(),
                display_name: "Claude Sonnet 4.5".to_string(),
            },
            ModelCandidate {
                provider: "openai".to_string(),
                model_id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = ModelsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_candidates() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = ModelsState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

    #[test]
    fn filter_by_model_id() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("opus");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].model_id, "claude-opus-4-6");
    }

    #[test]
    fn filter_by_provider() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("openai");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].model_id, "gpt-4o");
    }

    #[test]
    fn filter_by_display_name() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("GPT");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].display_name, "GPT-4o");
    }

    #[test]
    fn filter_case_insensitive() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("CLAUDE");
        assert_eq!(state.candidates().len(), 2);
    }

    #[test]
    fn filter_no_match_hides() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("zzz");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn filter_recovers_after_no_match() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.update("zzz");
        assert!(!state.is_visible());

        state.update("opus");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);

        state.update("");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn select_next_wraps() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 0); // wrapped
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.select_prev();
        assert_eq!(state.selected(), 2); // wrapped to last
    }

    #[test]
    fn selection_clamped_on_filter() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.select_next();
        state.select_next();
        assert_eq!(state.selected(), 2);

        state.update("opus");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn accept_returns_candidate() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());

        state.select_next();
        let result = state.accept();
        assert!(result.is_some());
        let candidate = result.expect("accept should return Some");
        assert_eq!(candidate.model_id, "claude-sonnet-4-5");
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = ModelsState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn update_without_active_session_is_noop() {
        let mut state = ModelsState::new();
        state.update("anything");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn update_after_dismiss_is_noop() {
        let mut state = ModelsState::new();
        state.open(sample_candidates());
        state.dismiss();
        state.update("opus");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }
}
