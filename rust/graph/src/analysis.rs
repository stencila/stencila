//! Graphs paired with the static analysis diagnostics that produced them.
//!
//! Graph construction is conservative: an I/O path that cannot be proven from
//! source produces no resource node and no edge. That decision is correct but
//! invisible, so the analyzer also records why it declined. Those records are
//! kept beside the graph rather than inside it, because they describe the
//! analyzer's confidence rather than the workspace's contents.

use crate::{Graph, code::StaticAnalysisDiagnostic};

/// A graph together with the diagnostics collected while building it.
///
/// Consumers that only need the graph keep using [`crate::graph_from_path`] and
/// [`crate::graph_from_node`]. Callers that want to report unresolved I/O — such
/// as `stencila graph --explain` — use the analysis entry points instead.
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    /// The graph built from the workspace or document.
    pub graph: Graph,

    /// Static analysis diagnostics, ordered by scope and source position.
    pub diagnostics: Vec<StaticAnalysisDiagnostic>,
}

impl GraphAnalysis {
    /// Discard the diagnostics and return the graph.
    pub fn into_graph(self) -> Graph {
        self.graph
    }
}
