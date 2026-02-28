/// A discovered agent definition, ready to be launched as a new session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDefinitionInfo {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub source: String,
}

/// The result of accepting an agent candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSelection {
    /// Switch to an existing session at the given index.
    Switch(usize),
    /// Create a new session from a discovered agent definition.
    FromDefinition(AgentDefinitionInfo),
}

/// The kind of agent candidate entry.
#[derive(Debug, Clone)]
pub enum AgentCandidateKind {
    /// An existing TUI session.
    Session {
        index: usize,
        is_active: bool,
        definition: Option<AgentDefinitionInfo>,
    },
    /// A discovered agent definition (not yet a session).
    Definition(AgentDefinitionInfo),
}

/// A candidate in the agent picker popup.
#[derive(Debug, Clone)]
pub struct AgentCandidate {
    /// What kind of candidate this is.
    pub kind: AgentCandidateKind,
    /// Display name of the agent.
    pub name: String,
}

/// State for the agent picker popup triggered by `/agent`.
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
    /// Returns an `AgentSelection` indicating whether to switch or create from
    /// definition.
    pub fn accept(&mut self) -> Option<AgentSelection> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let selection = match &candidate.kind {
            AgentCandidateKind::Session { index, .. } => AgentSelection::Switch(*index),
            AgentCandidateKind::Definition(info) => AgentSelection::FromDefinition(info.clone()),
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
                name: "coder".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 1,
                    is_active: false,
                    definition: None,
                },
            },
            AgentCandidate {
                name: "reviewer".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 2,
                    is_active: false,
                    definition: None,
                },
            },
            AgentCandidate {
                name: "default".to_string(),
                kind: AgentCandidateKind::Session {
                    index: 0,
                    is_active: true,
                    definition: None,
                },
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
        assert_eq!(state.candidates().len(), 3);
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
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        state.select_prev();
        assert_eq!(state.selected(), 2);
    }

    #[test]
    fn accept_returns_switch() {
        let mut state = AgentsState::new();
        state.open(sample_candidates());

        state.select_next();
        let result = state.accept();
        assert_eq!(result, Some(AgentSelection::Switch(2)));
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_returns_from_definition() {
        let info = AgentDefinitionInfo {
            name: "coder".to_string(),
            description: "A coding agent".to_string(),
            model: Some("claude-sonnet-4-5".to_string()),
            provider: Some("anthropic".to_string()),
            source: "workspace".to_string(),
        };
        let mut state = AgentsState::new();
        state.open(vec![AgentCandidate {
            name: "coder".to_string(),
            kind: AgentCandidateKind::Definition(info.clone()),
        }]);

        let result = state.accept();
        assert_eq!(result, Some(AgentSelection::FromDefinition(info)));
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
