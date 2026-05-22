//! Deterministic graph assembly.
//!
//! This module centralizes node and edge accumulation so graph construction
//! produces stable output regardless of the traversal order used by callers.

use std::collections::{BTreeMap, BTreeSet};

use eyre::{Result, bail};
use stencila_schema::{
    Graph, GraphAction, GraphEdge, GraphEdgeKind, GraphEvidence, GraphNode, Node,
};

/// Builder for deterministic Stencila Schema graphs.
///
/// The builder owns ordering and de-duplication so document and workspace
/// collectors can focus on discovering relationships instead of serialization
/// stability.
#[derive(Debug)]
pub struct GraphBuilder {
    /// Subject represented by the graph.
    ///
    /// Keeping the subject with the builder makes the final `Graph` creation a
    /// single operation once all nodes and edges have been discovered.
    subject: String,

    /// Graph nodes keyed by graph-local id.
    ///
    /// A `BTreeMap` gives stable node order in snapshots, generated files, and
    /// downstream consumers that diff graph output.
    nodes: BTreeMap<String, GraphNode>,

    /// Graph edges keyed by their semantic tuple.
    ///
    /// A `BTreeMap` removes duplicate relationships, keeps edge order stable,
    /// and lets separate collectors merge evidence onto the same relationship.
    edges: BTreeMap<EdgeKey, EdgeMetadata>,

    /// Validation errors collected during graph assembly.
    ///
    /// Collection lets callers keep using the builder API in simple append-only
    /// code and receive all detected graph problems when the graph is built.
    errors: BTreeSet<String>,
}

/// Sortable identity for a graph edge.
///
/// This compact key separates edge ordering and de-duplication from the
/// generated Schema edge node, whose optional metadata is not part of identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    /// Source graph node id.
    ///
    /// The source is stored as a string because graph edges refer to graph-local
    /// ids rather than borrowing the underlying `GraphNode`.
    source: String,

    /// Target graph node id.
    ///
    /// The target completes the endpoint pair used to identify duplicate
    /// relationships in the edge set.
    target: String,

    /// Relationship kind.
    ///
    /// The kind is part of identity so the same endpoints may carry multiple
    /// distinct relationships when the graph model requires it.
    kind: GraphEdgeKind,
}

/// Optional metadata attached to a graph edge.
///
/// Evidence explains why the relationship is believed to exist. Actions record
/// concrete activities that caused the relationship when such activity metadata
/// is available.
#[derive(Debug, Clone, Default, PartialEq)]
struct EdgeMetadata {
    evidence: Vec<GraphEvidence>,
    actions: Vec<GraphAction>,
}

impl GraphBuilder {
    /// Create a graph builder for a graph subject.
    ///
    /// The subject identifies the resource represented by the final graph and is
    /// passed through unchanged to the generated `Graph`.
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            errors: BTreeSet::new(),
        }
    }

    /// Add a graph node for a Stencila Schema node.
    ///
    /// Repeated discoveries of the same graph node are ignored when the embedded
    /// node is identical. If the same graph id is later associated with a
    /// different node, the conflict is recorded and reported by [`Self::build`].
    pub fn add_schema_node(&mut self, id: impl Into<String>, node: Node) {
        let id = id.into();
        let node = Box::new(node);

        match self.nodes.entry(id.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(GraphNode::new(id, node));
            }
            std::collections::btree_map::Entry::Occupied(entry) if entry.get().node != node => {
                self.errors.insert(format!(
                    "graph node id `{id}` was added with conflicting embedded nodes"
                ));
            }
            std::collections::btree_map::Entry::Occupied(..) => {}
        }
    }

    /// Add a directed graph edge.
    ///
    /// Self edges are ignored because the graph represents relationships between
    /// distinct nodes, and keeping them would add noise to provenance queries.
    pub fn add_edge(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
    ) {
        self.add_edge_with_metadata(source, target, kind, [], []);
    }

    /// Add a directed graph edge with supporting evidence.
    ///
    /// If the edge already exists, new evidence items are appended when they are
    /// not already present. Edge identity remains only source, target, and kind
    /// so evidence strengthens a relationship without creating parallel edges.
    pub fn add_edge_with_evidence(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_metadata(source, target, kind, evidence, []);
    }

    /// Add a directed graph edge with associated actions.
    ///
    /// Actions should record concrete activities that caused the relationship,
    /// not static analysis or observational evidence. If the edge already
    /// exists, new actions are appended when they are not already present.
    pub fn add_edge_with_actions(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
        actions: impl IntoIterator<Item = GraphAction>,
    ) {
        self.add_edge_with_metadata(source, target, kind, [], actions);
    }

    /// Add a directed graph edge with supporting evidence and actions.
    ///
    /// This is the most explicit builder entry point for graph relationships
    /// that have both confidence/evidence metadata and concrete operation
    /// metadata.
    pub fn add_edge_with_evidence_and_actions(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
        evidence: impl IntoIterator<Item = GraphEvidence>,
        actions: impl IntoIterator<Item = GraphAction>,
    ) {
        self.add_edge_with_metadata(source, target, kind, evidence, actions);
    }

    /// Add a directed graph edge with optional metadata.
    fn add_edge_with_metadata(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
        evidence: impl IntoIterator<Item = GraphEvidence>,
        actions: impl IntoIterator<Item = GraphAction>,
    ) {
        let source = source.into();
        let target = target.into();

        if source == target {
            return;
        }

        let edge = EdgeKey {
            source,
            target,
            kind,
        };
        let stored = self.edges.entry(edge).or_default();
        for evidence in evidence {
            if !stored.evidence.contains(&evidence) {
                stored.evidence.push(evidence);
            }
        }
        for action in actions {
            if !stored.actions.contains(&action) {
                stored.actions.push(action);
            }
        }
    }

    /// Finish graph construction.
    ///
    /// Consuming the builder validates node id conflicts and dangling edge
    /// endpoints before handing the Schema graph to callers.
    pub fn build(self) -> Result<Graph> {
        let GraphBuilder {
            subject,
            nodes,
            edges,
            mut errors,
        } = self;

        for edge in edges.keys() {
            if !nodes.contains_key(&edge.source) {
                errors.insert(format!(
                    "graph edge `{}` -> `{}` ({}) has missing source node",
                    edge.source, edge.target, edge.kind
                ));
            }
            if !nodes.contains_key(&edge.target) {
                errors.insert(format!(
                    "graph edge `{}` -> `{}` ({}) has missing target node",
                    edge.source, edge.target, edge.kind
                ));
            }
        }

        if !errors.is_empty() {
            bail!(
                "invalid graph: {}",
                errors.into_iter().collect::<Vec<_>>().join("; ")
            );
        }

        Ok(Graph::new(
            subject,
            nodes.into_values().collect(),
            edges
                .into_iter()
                .map(|(edge, metadata)| {
                    let mut graph_edge = GraphEdge::new(edge.source, edge.target, edge.kind);
                    if !metadata.evidence.is_empty() {
                        graph_edge.options.evidence = Some(metadata.evidence);
                    }
                    if !metadata.actions.is_empty() {
                        graph_edge.options.actions = Some(metadata.actions);
                    }
                    graph_edge
                })
                .collect(),
        ))
    }
}
