/// A cancel autocomplete candidate representing a running exchange.
#[derive(Debug, Clone)]
pub struct CancelCandidate {
    /// The exchange number (1-based, as displayed in the UI).
    pub exchange_num: usize,
    /// Index into `app.messages` for this exchange.
    pub msg_index: usize,
    /// Short preview of the request text.
    pub request_preview: String,
}

/// Result of accepting a cancel candidate.
pub struct CancelAcceptResult {
    /// Index into `app.messages` for the exchange to cancel.
    pub msg_index: usize,
}

/// State for the cancel picker popup triggered by `/cancel`.
pub struct CancelState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Running exchange candidates.
    candidates: Vec<CancelCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
}

impl CancelState {
    /// Create a new hidden cancel state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of candidates.
    pub fn candidates(&self) -> &[CancelCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the cancel picker with the given candidates.
    pub fn open(&mut self, candidates: Vec<CancelCandidate>) {
        self.candidates = candidates;
        self.selected = 0;
        self.visible = !self.candidates.is_empty();
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
    /// Returns the message index to cancel, or `None` if nothing to accept.
    pub fn accept(&mut self) -> Option<CancelAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let msg_index = candidate.msg_index;

        self.dismiss();

        Some(CancelAcceptResult { msg_index })
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

    fn sample_candidates() -> Vec<CancelCandidate> {
        vec![
            CancelCandidate {
                exchange_num: 2,
                msg_index: 3,
                request_preview: "sleep 30".to_string(),
            },
            CancelCandidate {
                exchange_num: 4,
                msg_index: 7,
                request_preview: "make build".to_string(),
            },
            CancelCandidate {
                exchange_num: 5,
                msg_index: 9,
                request_preview: "cargo test".to_string(),
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = CancelState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_candidates() {
        let mut state = CancelState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = CancelState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

    #[test]
    fn select_next_wraps() {
        let mut state = CancelState::new();
        state.open(sample_candidates());
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
        let mut state = CancelState::new();
        state.open(sample_candidates());
        assert_eq!(state.selected(), 0);

        state.select_prev();
        assert_eq!(state.selected(), 2); // wrapped to last
    }

    #[test]
    fn accept_returns_result() {
        let mut state = CancelState::new();
        state.open(sample_candidates());

        // Select the second candidate
        state.select_next();
        let result = state.accept();
        assert!(result.is_some());
        let result = result.expect("accept should return Some");
        assert_eq!(result.msg_index, 7);
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = CancelState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = CancelState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }
}
