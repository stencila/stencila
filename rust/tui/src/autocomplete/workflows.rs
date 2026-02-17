/// Information about a discovered workflow definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDefinitionInfo {
    pub name: String,
    pub description: String,
    pub goal: Option<String>,
}

/// A candidate in the workflow picker popup.
#[derive(Debug, Clone)]
pub struct WorkflowCandidate {
    pub name: String,
    pub info: WorkflowDefinitionInfo,
}

/// State for the workflow picker popup triggered by `/workflows`.
pub struct WorkflowsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Workflow candidates.
    candidates: Vec<WorkflowCandidate>,
    /// Currently selected index.
    selected: usize,
}

impl WorkflowsState {
    /// Create a new hidden workflows state.
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
    pub fn candidates(&self) -> &[WorkflowCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the workflow picker with the given candidates.
    pub fn open(&mut self, candidates: Vec<WorkflowCandidate>) {
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
    /// Returns the `WorkflowDefinitionInfo` for the selected workflow, or `None`
    /// if the popup is hidden or empty.
    pub fn accept(&mut self) -> Option<WorkflowDefinitionInfo> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let info = self.candidates[self.selected].info.clone();
        self.dismiss();
        Some(info)
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

    fn sample_candidates() -> Vec<WorkflowCandidate> {
        vec![
            WorkflowCandidate {
                name: "code-review".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "code-review".to_string(),
                    description: "Review code changes".to_string(),
                    goal: Some("Review the latest PR".to_string()),
                },
            },
            WorkflowCandidate {
                name: "deploy".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "deploy".to_string(),
                    description: "Deploy to production".to_string(),
                    goal: None,
                },
            },
            WorkflowCandidate {
                name: "test-suite".to_string(),
                info: WorkflowDefinitionInfo {
                    name: "test-suite".to_string(),
                    description: "Run the test suite".to_string(),
                    goal: Some("Run all tests".to_string()),
                },
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = WorkflowsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_candidates() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = WorkflowsState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

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

    #[test]
    fn accept_returns_info() {
        let mut state = WorkflowsState::new();
        state.open(sample_candidates());

        state.select_next();
        let result = state.accept();
        assert_eq!(
            result,
            Some(WorkflowDefinitionInfo {
                name: "deploy".to_string(),
                description: "Deploy to production".to_string(),
                goal: None,
            })
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
}
