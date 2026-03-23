/// The kind of resumable item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumableKind {
    /// A workflow run that can be resumed.
    WorkflowRun,
    /// An agent session that can be resumed.
    AgentSession,
}

/// A candidate in the resume picker popup.
#[derive(Debug, Clone)]
pub struct ResumeCandidate {
    /// Kind of resumable item.
    pub kind: ResumableKind,
    /// The run/session ID (full).
    pub id: String,
    /// Name of the workflow or agent.
    pub name: String,
    /// Goal of workflow run or description of agent session.
    pub description: String,
    /// Status string (e.g. "failed", "running").
    pub status: String,
    /// ISO-8601 timestamp for sorting (most recent first).
    pub sort_timestamp: String,
    /// Human-readable relative time (e.g. "3 minutes ago").
    pub time_ago: String,
}

/// State for the resume picker popup triggered by `/resume`.
pub struct ResumeState {
    visible: bool,
    all_candidates: Vec<ResumeCandidate>,
    candidates: Vec<ResumeCandidate>,
    selected: usize,
}

impl ResumeState {
    pub fn new() -> Self {
        Self {
            visible: false,
            all_candidates: Vec::new(),
            candidates: Vec::new(),
            selected: 0,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn candidates(&self) -> &[ResumeCandidate] {
        &self.candidates
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn has_matches(&self) -> bool {
        !self.candidates.is_empty()
    }

    pub fn open(&mut self, candidates: Vec<ResumeCandidate>) {
        self.all_candidates = candidates;
        self.candidates = self.all_candidates.clone();
        self.selected = 0;
        self.visible = !self.candidates.is_empty();
    }

    pub fn update(&mut self, filter: &str) {
        if self.all_candidates.is_empty() {
            return;
        }

        if filter.is_empty() {
            self.candidates = self.all_candidates.clone();
        } else {
            let filter_lower = filter.to_ascii_lowercase();
            self.candidates = self
                .all_candidates
                .iter()
                .filter(|c| {
                    c.name.to_ascii_lowercase().contains(&filter_lower)
                        || c.description.to_ascii_lowercase().contains(&filter_lower)
                        || c.status.to_ascii_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }

        self.visible = true;

        if self.selected >= self.candidates.len() {
            self.selected = 0;
        }
    }

    pub fn select_next(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = (self.selected + 1) % self.candidates.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = if self.selected == 0 {
                self.candidates.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn accept(&mut self) -> Option<ResumeCandidate> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = self.candidates[self.selected].clone();
        self.dismiss();
        Some(candidate)
    }

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

    fn sample_candidates() -> Vec<ResumeCandidate> {
        vec![
            ResumeCandidate {
                kind: ResumableKind::WorkflowRun,
                id: "01926f3a-7b2c-7d4e-8f1a-9c3d5e7f0a1b".to_string(),
                name: "code-review".to_string(),
                description: "Review the latest PR".to_string(),
                status: "failed".to_string(),
                sort_timestamp: "2026-03-23T10:00:00Z".to_string(),
                time_ago: "3 minutes ago".to_string(),
            },
            ResumeCandidate {
                kind: ResumableKind::WorkflowRun,
                id: "01926f3b-1234-5678-9abc-def012345678".to_string(),
                name: "deploy".to_string(),
                description: "Deploy to staging".to_string(),
                status: "running".to_string(),
                sort_timestamp: "2026-03-23T09:53:00Z".to_string(),
                time_ago: "10 minutes ago".to_string(),
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = ResumeState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_shows_candidates() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 2);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn open_empty_stays_hidden() {
        let mut state = ResumeState::new();
        state.open(Vec::new());
        assert!(!state.is_visible());
    }

    #[test]
    fn filter_by_name() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("deploy");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn filter_by_goal() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("staging");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn filter_by_status() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("failed");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "code-review");
    }

    #[test]
    fn filter_case_insensitive() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("DEPLOY");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].name, "deploy");
    }

    #[test]
    fn filter_no_match_hides() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("zzz");
        assert!(state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn filter_recovers_after_no_match() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.update("zzz");
        assert!(state.is_visible());

        state.update("deploy");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);

        state.update("");
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 2);
    }

    #[test]
    fn select_next_wraps() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.select_prev();
        assert_eq!(state.selected(), 1);
    }

    #[test]
    fn selection_clamped_on_filter() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        state.select_next();
        assert_eq!(state.selected(), 1);

        state.update("code");
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn accept_returns_candidate() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());

        let result = state.accept();
        assert!(result.is_some());
        let candidate = result.expect("just checked");
        assert_eq!(candidate.name, "code-review");
        assert_eq!(candidate.kind, ResumableKind::WorkflowRun);
        assert!(!state.is_visible());
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = ResumeState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn update_without_open_is_noop() {
        let mut state = ResumeState::new();
        state.update("anything");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }

    #[test]
    fn update_after_dismiss_is_noop() {
        let mut state = ResumeState::new();
        state.open(sample_candidates());
        state.dismiss();
        state.update("code");
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
    }
}
