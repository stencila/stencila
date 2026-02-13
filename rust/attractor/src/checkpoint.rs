use std::path::Path;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::Context;
use crate::error::AttractorResult;

/// A serializable snapshot of pipeline execution state, used for
/// resuming from a checkpoint after interruption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// ISO 8601 timestamp when the checkpoint was created.
    pub timestamp: String,

    /// ID of the last completed node. On resume, execution continues
    /// from the node *after* this one in the traversal.
    pub current_node: String,

    /// Nodes that have already completed (regardless of status).
    pub completed_nodes: Vec<String>,

    /// Outcome status for each completed node (e.g., `"success"`, `"fail"`).
    ///
    /// Used on resume to reconstruct accurate `node_outcomes` so that
    /// goal-gate checks don't vacuously pass for previously failed nodes.
    /// Absent in legacy checkpoints — defaults to empty, which triggers
    /// the "assume success" fallback in resume logic.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub node_statuses: IndexMap<String, String>,

    /// Per-node retry counts.
    pub node_retries: IndexMap<String, u32>,

    /// Snapshot of context values at checkpoint time.
    #[serde(rename = "context")]
    pub context_values: IndexMap<String, Value>,

    /// Log entries accumulated so far.
    pub logs: Vec<String>,

    /// The next node to execute on resume. When present, this takes
    /// precedence over heuristic edge selection from `current_node`.
    /// Stored by the engine which knows which edge was selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_node_id: Option<String>,
}

impl Checkpoint {
    /// Create a checkpoint from the current execution state.
    ///
    /// `node_statuses` records the outcome status string for each completed
    /// node, enabling accurate goal-gate restoration on resume.
    ///
    /// `next_node_id` should be set when the engine knows which edge was
    /// selected after this node completed (e.g., via the edge-selection
    /// algorithm). When absent, resume will use heuristic fallback.
    #[must_use]
    pub fn from_context(
        context: &Context,
        current_node: impl Into<String>,
        completed_nodes: Vec<String>,
        node_statuses: IndexMap<String, String>,
        node_retries: IndexMap<String, u32>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            current_node: current_node.into(),
            completed_nodes,
            node_statuses,
            node_retries,
            context_values: context.snapshot(),
            logs: context.logs(),
            next_node_id: None,
        }
    }

    /// Set the next node to resume from.
    #[must_use]
    pub fn with_next_node(mut self, next_node_id: impl Into<String>) -> Self {
        self.next_node_id = Some(next_node_id.into());
        self
    }

    /// Save this checkpoint to a file as pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or the checkpoint
    /// cannot be serialized.
    pub fn save(&self, path: &Path) -> AttractorResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a checkpoint from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid JSON.
    pub fn load(path: &Path) -> AttractorResult<Self> {
        let data = std::fs::read_to_string(path)?;
        let checkpoint: Self = serde_json::from_str(&data)?;
        Ok(checkpoint)
    }
}
