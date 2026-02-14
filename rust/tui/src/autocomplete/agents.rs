/// The result of accepting an agent candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSelection {
    /// Switch to an existing session at the given index.
    Switch(usize),
    /// Create a new agent (open wizard).
    New,
}

/// A candidate in the agent picker popup.
#[derive(Debug, Clone)]
pub struct AgentCandidate {
    /// Session index (unused for the "new" entry).
    pub index: usize,
    /// Display name of the agent.
    pub name: String,
    /// Whether this is the currently active agent.
    pub is_active: bool,
    /// Whether this entry represents "create new agent".
    pub is_new: bool,
}

/// State for the agent picker popup triggered by `/agents`.
pub struct AgentsState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Agent candidates.
    candidates: Vec<AgentCandidate>,
    /// Currently selected index.
    selected: usize,
}

impl AgentsState {
    /// Create a new hidden agents state.
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
    pub fn candidates(&self) -> &[AgentCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the agent picker with the given candidates.
    pub fn open(&mut self, candidates: Vec<AgentCandidate>) {
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
    /// Returns an `AgentSelection` indicating whether to switch or create new.
    pub fn accept(&mut self) -> Option<AgentSelection> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let selection = if candidate.is_new {
            AgentSelection::New
        } else {
            AgentSelection::Switch(candidate.index)
        };
        self.dismiss();
        Some(selection)
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

    fn sample_candidates() -> Vec<AgentCandidate> {
        vec![
            AgentCandidate {
                index: 0,
                name: "default".to_string(),
                is_active: true,
                is_new: false,
            },
            AgentCandidate {
                index: 1,
                name: "coder".to_string(),
                is_active: false,
                is_new: false,
            },
            AgentCandidate {
                index: 2,
                name: "reviewer".to_string(),
                is_active: false,
                is_new: false,
            },
            AgentCandidate {
                index: 0,
                name: "+ new agent".to_string(),
                is_active: false,
                is_new: true,
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = AgentsState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_candidates() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 4);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = AgentsState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

    #[test]
    fn select_next_wraps() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 3);
        state.select_next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        state.select_prev();
        assert_eq!(state.selected(), 3);
    }

    #[test]
    fn accept_returns_switch() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        state.select_next();
        let result = state.accept();
        assert_eq!(result, Some(AgentSelection::Switch(1)));
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_returns_new() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        // Select the last candidate (the "new" entry)
        state.select_next();
        state.select_next();
        state.select_next();
        let result = state.accept();
        assert_eq!(result, Some(AgentSelection::New));
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = AgentsState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }
}
