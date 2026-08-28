//! Graphs paired with the static analysis diagnostics that produced them.
//!
//! Graph construction is conservative: an I/O path that cannot be proven from
//! source produces no resource node and no edge. That decision is correct but
//! invisible, so the analyzer also records why it declined. Those records are
//! kept beside the graph rather than inside it, because they describe the
//! analyzer's confidence rather than the workspace's contents.

use derive_more::Display;

use crate::{Graph, code::StaticAnalysisDiagnostic};

/// A diagnostic emitted while collecting a graph rather than while analyzing
/// executable code.
#[derive(Debug, Display, Clone, PartialEq, Eq)]
#[display("{message}")]
pub struct GraphDiagnostic {
    /// The kind of graph collection problem.
    pub kind: GraphDiagnosticKind,

    /// The graph node that declared the problematic relationship or id.
    pub source: String,

    /// The authored target, when the diagnostic concerns a relationship.
    pub target: Option<String>,

    /// An actionable explanation suitable for logs and command-line output.
    pub message: String,
}

/// The kinds of authoring problems detected during graph collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphDiagnosticKind {
    /// More than one research object declared the same id.
    DuplicateResearchObjectId,

    /// A relation did not contain a target.
    EmptyResearchRelationTarget,

    /// A relation target was repeated on the same source and kind.
    DuplicateResearchRelationTarget,

    /// A relation pointed back to its source.
    SelfReferentialResearchRelation,

    /// A non-URI relation target could not be found in the document graph.
    UnresolvedResearchRelationTarget,
}

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

    /// Graph-structure and authored-relation diagnostics.
    pub graph_diagnostics: Vec<GraphDiagnostic>,
}

impl GraphAnalysis {
    /// Discard the diagnostics and return the graph.
    pub fn into_graph(self) -> Graph {
        self.graph
    }
}
