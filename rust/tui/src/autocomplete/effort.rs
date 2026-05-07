use stencila_agents::types::ReasoningEffort;

/// A reasoning effort candidate shown in the effort picker popup.
#[derive(Debug, Clone)]
pub struct EffortCandidate {
    /// Display label for the effort level.
    pub label: &'static str,
    /// Description shown alongside the label.
    pub description: &'static str,
    /// Reasoning effort to apply; `None` clears the override.
    pub effort: Option<ReasoningEffort>,
}

/// State for the reasoning effort picker popup triggered by `/effort`.
pub struct EffortState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Effort candidates.
    candidates: Vec<EffortCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
}

impl EffortState {
    /// Create a new hidden effort state.
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
    pub fn candidates(&self) -> &[EffortCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Open the effort picker with all supported portable effort values.
    pub fn open(&mut self, current: Option<&ReasoningEffort>) {
        self.candidates = vec![
            EffortCandidate {
                label: "low",
                description: "minimal reasoning",
                effort: Some(ReasoningEffort::Low),
            },
            EffortCandidate {
                label: "medium",
                description: "balanced reasoning",
                effort: Some(ReasoningEffort::Medium),
            },
            EffortCandidate {
                label: "high",
                description: "deeper reasoning",
                effort: Some(ReasoningEffort::High),
            },
            EffortCandidate {
                label: "xhigh",
                description: "extra-high reasoning",
                effort: Some(ReasoningEffort::Xhigh),
            },
            EffortCandidate {
                label: "default",
                description: "clear override",
                effort: None,
            },
        ];

        self.selected = self
            .candidates
            .iter()
            .position(|candidate| candidate.effort.as_ref() == current)
            .unwrap_or(0);
        self.visible = true;
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
    pub fn accept(&mut self) -> Option<EffortCandidate> {
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
        self.candidates.clear();
        self.selected = 0;
    }
}
