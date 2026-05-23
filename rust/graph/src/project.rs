//! Projection utilities for display-oriented graph views.
//!
//! Raw Schema graphs preserve all discovered relationships. Display projections
//! answer a narrower reader question by selecting relationship families,
//! collapsing noisy intermediates, and attaching compact labels.

use std::collections::{BTreeMap, BTreeSet};

use clap::ValueEnum;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stencila_schema::{Graph, GraphEdge, GraphEdgeKind, GraphNode};

const STRUCTURE_EDGE_KIND: GraphEdgeKind = GraphEdgeKind::PartOf;
const GRAPH_EDGE_KEY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b'/')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

const AUTO_PRESETS: [GraphProjectionPreset; 4] = [
    GraphProjectionPreset::Flow,
    GraphProjectionPreset::Deps,
    GraphProjectionPreset::Cite,
    GraphProjectionPreset::React,
];

/// User-facing graph projection preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
pub enum GraphProjectionPreset {
    /// Choose the first useful projection from the graph's relationships.
    ///
    /// Auto mode prefers focused views in the order most useful for authoring
    /// review: data/resource flow, software dependencies, citations, then
    /// executable reactivity. If no focused relationship family is present it
    /// falls back to the full graph.
    Auto,

    /// Show every graph node and edge without applying a focused projection.
    ///
    /// This is the most complete view and the noisiest one. Use it when
    /// debugging graph collection itself, checking whether expected nodes or
    /// edges exist, or comparing a projected view against the raw graph shape.
    Full,

    /// Show resource flow, data lineage, and provenance relationships.
    ///
    /// This answers questions such as which files, tables, code units, or
    /// outputs read, generated, derived, converted, used, or transcluded each
    /// other. It is intended for tracing where data and rendered outputs came
    /// from, rather than for ordering executable document updates.
    Flow,

    /// Show software imports, calls, packages, and dependency use.
    ///
    /// This focuses on code and environment relationships: imported packages,
    /// package use by source files, and calls between discovered functions or
    /// code units. It is useful for understanding the software stack behind a
    /// workspace without mixing in data products and document structure.
    Deps,

    /// Show bibliographic references, citations, and document links.
    ///
    /// This focuses on `CitedBy` and `ReferencedBy` relationships. Citation
    /// marker nodes are collapsed to their document parent by default so the
    /// graph reads as "this work is cited or referenced by this document region"
    /// instead of exposing every inline citation marker.
    Cite,

    /// Show executable document reactivity dependencies.
    ///
    /// This focuses on dependencies that decide what should update or rerun when
    /// executable document state changes, such as `DependsOn` between code
    /// chunks and `Parameterizes` from controls or parameters. It is distinct
    /// from `flow`, which tracks provenance and produced resources.
    React,
}

impl GraphProjectionPreset {
    /// Stable CLI/display name for the preset.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Full => "full",
            Self::Flow => "flow",
            Self::Deps => "deps",
            Self::Cite => "cite",
            Self::React => "react",
        }
    }
}

/// Options used to project a graph for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphProjectionOptions {
    /// Projection preset to apply.
    pub preset: GraphProjectionPreset,

    /// Include structural containment edges.
    pub include_structure_edges: Option<bool>,

    /// Include edges carrying low-confidence evidence.
    pub include_low_confidence_edges: bool,

    /// Collapse citation marker nodes to their document parent.
    pub collapse_citation_nodes: bool,
}

impl Default for GraphProjectionOptions {
    fn default() -> Self {
        Self {
            preset: GraphProjectionPreset::Auto,
            include_structure_edges: None,
            include_low_confidence_edges: true,
            collapse_citation_nodes: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedGraphProjectionOptions {
    include_structure_edges: bool,
    include_low_confidence_edges: bool,
    collapse_citation_nodes: bool,
}

/// Coarse graph node category used by display renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphViewNodeKind {
    Document,
    Workspace,
    Environment,
    Resource,
    Content,
    Code,
    Symbol,
    Function,
    Package,
    Datatable,
    Reference,
    Citation,
    Output,
    Other,
}

impl GraphViewNodeKind {
    /// Stable lowercase display key for the node kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Workspace => "workspace",
            Self::Environment => "environment",
            Self::Resource => "resource",
            Self::Content => "content",
            Self::Code => "code",
            Self::Symbol => "symbol",
            Self::Function => "function",
            Self::Package => "package",
            Self::Datatable => "datatable",
            Self::Reference => "reference",
            Self::Citation => "citation",
            Self::Output => "output",
            Self::Other => "other",
        }
    }
}

/// Display-oriented node shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphViewNode {
    pub id: String,
    pub label: String,
    pub kind: GraphViewNodeKind,
    pub node: GraphNode,
}

/// Display-oriented edge shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphViewEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub kind: GraphEdgeKind,
    pub edges: Vec<GraphEdge>,
    pub edge: GraphEdge,
    pub count: usize,
    pub evidence_count: usize,
    pub action_count: usize,
    pub low_confidence: bool,
}

/// Complete projected graph for display renderers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphView {
    pub preset: GraphProjectionPreset,
    pub nodes: Vec<GraphViewNode>,
    pub edges: Vec<GraphViewEdge>,
}

#[derive(Debug)]
struct ProjectedEdge<'a> {
    edge: &'a GraphEdge,
    source: String,
    target: String,
}

/// Project a Schema graph into a display graph view.
pub fn project_graph(graph: &Graph, options: &GraphProjectionOptions) -> GraphView {
    let preset = resolve_preset(graph, options);
    let resolved = resolve_projection_options(preset, options);
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let parent_by_id = parent_map(graph);
    let mut node_ids = BTreeSet::new();
    let mut edges = BTreeMap::new();

    for edge in &graph.edges {
        if !include_primary_edge(edge, preset, resolved.include_low_confidence_edges) {
            continue;
        }

        let source = edge.source.clone();
        let mut target = edge.target.clone();

        if preset == GraphProjectionPreset::Cite
            && resolved.collapse_citation_nodes
            && edge.kind == GraphEdgeKind::CitedBy
        {
            target = collapse_citation_target(&target, &nodes_by_id, &parent_by_id);
        }

        if !nodes_by_id.contains_key(source.as_str()) || !nodes_by_id.contains_key(target.as_str())
        {
            continue;
        }

        node_ids.insert(source.clone());
        node_ids.insert(target.clone());
        add_view_edge(
            &mut edges,
            ProjectedEdge {
                edge,
                source,
                target,
            },
        );
    }

    if preset == GraphProjectionPreset::Full {
        node_ids.extend(graph.nodes.iter().map(|node| node.id.clone()));
    }

    if resolved.include_structure_edges {
        add_structure_edges(
            graph,
            &nodes_by_id,
            &parent_by_id,
            &mut node_ids,
            &mut edges,
            resolved.include_low_confidence_edges,
            preset,
        );
    }

    let nodes = node_ids
        .into_iter()
        .filter_map(|id| nodes_by_id.get(id.as_str()).map(|node| view_node(node)))
        .collect();

    GraphView {
        preset,
        nodes,
        edges: edges.into_values().collect(),
    }
}

fn resolve_projection_options(
    preset: GraphProjectionPreset,
    options: &GraphProjectionOptions,
) -> ResolvedGraphProjectionOptions {
    ResolvedGraphProjectionOptions {
        include_structure_edges: options
            .include_structure_edges
            .unwrap_or(preset == GraphProjectionPreset::Full),
        include_low_confidence_edges: options.include_low_confidence_edges,
        collapse_citation_nodes: options.collapse_citation_nodes,
    }
}

fn resolve_preset(graph: &Graph, options: &GraphProjectionOptions) -> GraphProjectionPreset {
    if options.preset != GraphProjectionPreset::Auto {
        return options.preset;
    }

    AUTO_PRESETS
        .into_iter()
        .find(|preset| {
            graph.edges.iter().any(|edge| {
                include_primary_edge(edge, *preset, options.include_low_confidence_edges)
            })
        })
        .unwrap_or(GraphProjectionPreset::Full)
}

fn include_primary_edge(
    edge: &GraphEdge,
    preset: GraphProjectionPreset,
    include_low_confidence_edges: bool,
) -> bool {
    if !include_low_confidence_edges && has_low_confidence(edge) {
        return false;
    }

    if edge.kind == STRUCTURE_EDGE_KIND {
        return false;
    }

    edge_kind_in_preset(edge.kind, preset)
}

fn edge_kind_in_preset(kind: GraphEdgeKind, preset: GraphProjectionPreset) -> bool {
    if preset == GraphProjectionPreset::Full {
        return true;
    }

    match kind {
        GraphEdgeKind::UsedBy => matches!(
            preset,
            GraphProjectionPreset::Flow | GraphProjectionPreset::Deps
        ),
        GraphEdgeKind::ReadBy
        | GraphEdgeKind::Generated
        | GraphEdgeKind::DerivedInto
        | GraphEdgeKind::ConvertedInto
        | GraphEdgeKind::TranscludedBy => preset == GraphProjectionPreset::Flow,
        GraphEdgeKind::CalledBy | GraphEdgeKind::ImportedBy => {
            preset == GraphProjectionPreset::Deps
        }
        GraphEdgeKind::Parameterizes | GraphEdgeKind::DependsOn => {
            preset == GraphProjectionPreset::React
        }
        GraphEdgeKind::CitedBy | GraphEdgeKind::ReferencedBy => {
            preset == GraphProjectionPreset::Cite
        }
        GraphEdgeKind::PartOf => false,
    }
}

fn add_structure_edges(
    graph: &Graph,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
    node_ids: &mut BTreeSet<String>,
    edges: &mut BTreeMap<String, GraphViewEdge>,
    include_low_confidence_edges: bool,
    preset: GraphProjectionPreset,
) {
    let mut structure_edges = BTreeMap::new();

    for edge in &graph.edges {
        if edge.kind != STRUCTURE_EDGE_KIND
            || (!include_low_confidence_edges && has_low_confidence(edge))
            || !nodes_by_id.contains_key(edge.source.as_str())
            || !nodes_by_id.contains_key(edge.target.as_str())
        {
            continue;
        }

        structure_edges.insert(
            structure_edge_key(&edge.source, &edge.target),
            ProjectedEdge {
                edge,
                source: edge.source.clone(),
                target: edge.target.clone(),
            },
        );
    }

    if preset == GraphProjectionPreset::Full {
        for edge in structure_edges.into_values() {
            add_view_edge(edges, edge);
        }
        return;
    }

    let seeds = node_ids.iter().cloned().collect::<Vec<_>>();
    for seed in seeds {
        let mut child = seed;
        let mut visited = BTreeSet::new();

        while visited.insert(child.clone()) {
            let Some(parent) = parent_by_id.get(&child) else {
                break;
            };
            if !nodes_by_id.contains_key(parent.as_str()) {
                break;
            }

            let key = structure_edge_key(&child, parent);
            let Some(edge) = structure_edges.get(&key) else {
                break;
            };

            node_ids.insert(parent.clone());
            add_view_edge(
                edges,
                ProjectedEdge {
                    edge: edge.edge,
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                },
            );
            child = parent.clone();
        }
    }
}

fn add_view_edge(edges: &mut BTreeMap<String, GraphViewEdge>, projected: ProjectedEdge) {
    let key = edge_key(&projected.source, &projected.target, projected.edge.kind);

    if let Some(existing) = edges.get_mut(&key) {
        if !existing.edges.contains(projected.edge) {
            existing.edges.push(projected.edge.clone());
            update_edge_summary(existing);
        }
        return;
    }

    edges.insert(
        key.clone(),
        GraphViewEdge {
            id: key,
            source: projected.source,
            target: projected.target,
            label: edge_label(projected.edge.kind),
            kind: projected.edge.kind,
            edges: vec![projected.edge.clone()],
            edge: projected.edge.clone(),
            count: 1,
            evidence_count: evidence_count(projected.edge),
            action_count: action_count(projected.edge),
            low_confidence: has_low_confidence(projected.edge),
        },
    );
}

fn edge_key(source: &str, target: &str, kind: GraphEdgeKind) -> String {
    format!(
        "edge:{}:{}:{}",
        kind,
        utf8_percent_encode(source, GRAPH_EDGE_KEY_ENCODE_SET),
        utf8_percent_encode(target, GRAPH_EDGE_KEY_ENCODE_SET)
    )
}

fn update_edge_summary(edge: &mut GraphViewEdge) {
    edge.count = edge.edges.len();
    edge.evidence_count = edge.edges.iter().map(evidence_count).sum();
    edge.action_count = edge.edges.iter().map(action_count).sum();
    edge.low_confidence = edge.edges.iter().any(has_low_confidence);
}

fn structure_edge_key(source: &str, target: &str) -> String {
    format!("{source}\0{target}")
}

fn has_low_confidence(edge: &GraphEdge) -> bool {
    edge.options.evidence.as_deref().is_some_and(|evidence| {
        evidence.iter().any(|item| {
            item.confidence
                .is_some_and(|confidence| confidence.to_string() == "Low")
        })
    })
}

fn evidence_count(edge: &GraphEdge) -> usize {
    edge.options
        .evidence
        .as_deref()
        .map_or(0, |evidence| evidence.len())
}

fn action_count(edge: &GraphEdge) -> usize {
    edge.options
        .actions
        .as_deref()
        .map_or(0, |actions| actions.len())
}

fn parent_map(graph: &Graph) -> BTreeMap<String, String> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == STRUCTURE_EDGE_KIND)
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect()
}

fn collapse_citation_target(
    target: &str,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
) -> String {
    let mut current = target.to_string();
    let mut visited = BTreeSet::new();

    while node_kind(nodes_by_id.get(current.as_str()).copied()) == GraphViewNodeKind::Citation {
        if !visited.insert(current.clone()) {
            break;
        }

        let Some(parent) = parent_by_id.get(&current) else {
            break;
        };
        current = parent.clone();
    }

    current
}

fn view_node(node: &GraphNode) -> GraphViewNode {
    GraphViewNode {
        id: node.id.clone(),
        label: node_label(node),
        kind: node_kind(Some(node)),
        node: node.clone(),
    }
}

fn node_kind(node: Option<&GraphNode>) -> GraphViewNodeKind {
    let Some(node) = node else {
        return GraphViewNodeKind::Other;
    };

    match graph_id_namespace(&node.id) {
        "dir" => return GraphViewNodeKind::Workspace,
        "environment" => return GraphViewNodeKind::Environment,
        "file" | "symlink" | "resource" | "code-file" => return GraphViewNodeKind::Resource,
        "code" => return GraphViewNodeKind::Code,
        "symbol" => return GraphViewNodeKind::Symbol,
        "function" | "workflow-rule" => return GraphViewNodeKind::Function,
        "package" => return GraphViewNodeKind::Package,
        "column" => return GraphViewNodeKind::Datatable,
        "reference" => return GraphViewNodeKind::Reference,
        "output" => return GraphViewNodeKind::Output,
        _ => {}
    }

    let node_type = schema_node_type(node);
    match node_type.as_deref() {
        Some("Citation") => GraphViewNodeKind::Citation,
        Some("Reference") => GraphViewNodeKind::Reference,
        Some("CreativeWork") => {
            if graph_id_namespace(&node.id) == "resource" {
                GraphViewNodeKind::Resource
            } else {
                GraphViewNodeKind::Reference
            }
        }
        Some(
            "CodeBlock" | "CodeChunk" | "CodeExpression" | "CodeInline" | "SoftwareSourceCode",
        ) => GraphViewNodeKind::Code,
        Some("Variable") => GraphViewNodeKind::Symbol,
        Some("Function") => GraphViewNodeKind::Function,
        Some("Datatable" | "DatatableColumn") => GraphViewNodeKind::Datatable,
        Some("Directory") => GraphViewNodeKind::Workspace,
        Some("File" | "SymbolicLink") => GraphViewNodeKind::Resource,
        Some("Article" | "Collection" | "Prompt") => GraphViewNodeKind::Document,
        Some(
            "AudioObject" | "CitationGroup" | "Figure" | "Heading" | "ImageObject" | "IncludeBlock"
            | "Link" | "MediaObject" | "Table" | "VideoObject",
        ) => GraphViewNodeKind::Content,
        _ if node.id.contains("#citation") => GraphViewNodeKind::Citation,
        _ => GraphViewNodeKind::Other,
    }
}

fn node_label(node: &GraphNode) -> String {
    let value = serde_json::to_value(node.node.as_ref()).unwrap_or(Value::Null);

    for key in ["name", "title", "path", "url", "target", "id"] {
        if let Some(label) = string_value(value.get(key)) {
            return compact_label(&label);
        }
    }

    compact_label(&node.id)
}

fn graph_id_namespace(id: &str) -> &str {
    id.split_once(':').map_or(id, |(namespace, ..)| namespace)
}

fn schema_node_type(node: &GraphNode) -> Option<String> {
    let value = serde_json::to_value(node.node.as_ref()).ok()?;
    string_value(value.get("type"))
}

fn string_value(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
        Value::Array(values) => {
            let text = values
                .iter()
                .filter_map(|item| string_value(Some(item)))
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            (!text.is_empty()).then_some(text)
        }
        Value::Object(object) => string_value(
            object
                .get("value")
                .or_else(|| object.get("text"))
                .or_else(|| object.get("content")),
        ),
        _ => None,
    }
}

fn compact_label(label: &str) -> String {
    let without_namespace = label
        .split_once(':')
        .filter(|(prefix, ..)| prefix.chars().all(|char| char.is_ascii_lowercase()))
        .map_or(label, |(.., value)| value);
    let suffix = without_namespace
        .rsplit(['/', '#'])
        .next()
        .unwrap_or(without_namespace);

    if suffix.chars().count() > 42 {
        let mut value = suffix.chars().take(39).collect::<String>();
        value.push_str("...");
        value
    } else {
        suffix.to_string()
    }
}

/// Format an edge kind for display.
pub fn edge_label(kind: GraphEdgeKind) -> String {
    let value = kind.to_string();
    let mut label = String::with_capacity(value.len() + 4);

    for (index, char) in value.chars().enumerate() {
        if index > 0 && char.is_uppercase() {
            label.push(' ');
        }
        label.push(char);
    }

    label
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use stencila_schema::{
        Article, File, Graph, GraphEdge, GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence,
        GraphEvidenceKind, GraphNode, Node, Reference, SoftwareSourceCode,
    };

    use super::*;

    #[test]
    fn selects_data_flow_projection_automatically() {
        let view = project_graph(&graph(), &GraphProjectionOptions::default());

        assert_eq!(view.preset, GraphProjectionPreset::Flow);
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![GraphEdgeKind::ReadBy]
        );
        assert_eq!(
            view.nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["code:analysis.py", "file:data.csv"]
        );
    }

    #[test]
    fn collapses_citations_to_document_parent() {
        let view = project_graph(
            &graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Cite,
                ..Default::default()
            },
        );

        assert_eq!(view.edges.len(), 1);
        assert_eq!(view.edges[0].source, "reference:paper");
        assert_eq!(view.edges[0].target, "node:document#article");
        assert_eq!(view.edges[0].edges.len(), 1);
        assert_eq!(
            view.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
            vec![GraphViewNodeKind::Document, GraphViewNodeKind::Reference]
        );
    }

    #[test]
    fn filters_low_confidence_edges() {
        let view = project_graph(
            &graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                include_low_confidence_edges: false,
                ..Default::default()
            },
        );

        assert_eq!(view.edges.len(), 1);
        assert_eq!(view.edges[0].kind, GraphEdgeKind::ReadBy);
    }

    #[test]
    fn adds_structural_ancestors_for_projected_nodes_only() {
        let view = project_graph(
            &graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                include_structure_edges: Some(true),
                ..Default::default()
            },
        );

        assert_eq!(
            view.nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["code:analysis.py", "file:data.csv", "node:document#article"]
        );
        assert_eq!(
            view.edges
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec![
                "PartOf:code:analysis.py->node:document#article",
                "ReadBy:file:data.csv->code:analysis.py"
            ]
        );
    }

    #[test]
    fn uses_full_preset_structure_defaults_after_auto_resolution() {
        let view = project_graph(&structure_only_graph(), &GraphProjectionOptions::default());

        assert_eq!(view.preset, GraphProjectionPreset::Full);
        assert_eq!(
            view.nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["node:document#article", "node:document#figure"]
        );
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![GraphEdgeKind::PartOf]
        );
    }

    #[test]
    fn uses_stable_aggregate_edge_ids_and_summaries() {
        let view = project_graph(
            &duplicate_edge_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        assert_eq!(view.edges.len(), 1);
        assert_eq!(
            view.edges[0].id,
            "edge:ReadBy:file%3Adata.csv:code%3Aanalysis.py"
        );
        assert_eq!(view.edges[0].count, 2);
        assert_eq!(view.edges[0].evidence_count, 1);
        assert!(view.edges[0].low_confidence);
    }

    fn graph() -> Graph {
        let mut graph = Graph::new(
            "test:graph".to_string(),
            vec![
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node("reference:paper", Node::Reference(reference("Paper"))),
                graph_node(
                    "node:document#citation-1",
                    Node::String("paper".to_string()),
                ),
                graph_node(
                    "node:document#article",
                    Node::Article(Article::new(Vec::new())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                low_confidence_edge(
                    "code:analysis.py",
                    "file:plot.png",
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "reference:paper".to_string(),
                    "node:document#citation-1".to_string(),
                    GraphEdgeKind::CitedBy,
                ),
                GraphEdge::new(
                    "node:document#citation-1".to_string(),
                    "node:document#article".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "code:analysis.py".to_string(),
                    "node:document#article".to_string(),
                    GraphEdgeKind::PartOf,
                ),
            ],
        );
        graph.nodes[3].node = Box::new(Node::Citation(Default::default()));
        graph
    }

    fn structure_only_graph() -> Graph {
        Graph::new(
            "test:structure".to_string(),
            vec![
                graph_node(
                    "node:document#article",
                    Node::Article(Article::new(Vec::new())),
                ),
                graph_node("node:document#figure", Node::String("figure".to_string())),
            ],
            vec![GraphEdge::new(
                "node:document#figure".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::PartOf,
            )],
        )
    }

    fn duplicate_edge_graph() -> Graph {
        Graph::new(
            "test:duplicate".to_string(),
            vec![
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                low_confidence_edge("file:data.csv", "code:analysis.py", GraphEdgeKind::ReadBy),
            ],
        )
    }

    fn graph_node(id: &str, node: Node) -> GraphNode {
        GraphNode::new(id.to_string(), Box::new(node))
    }

    fn reference(title: &str) -> Reference {
        Reference {
            title: Some(vec![stencila_schema::Inline::Text(
                stencila_schema::Text::new(title.into()),
            )]),
            ..Default::default()
        }
    }

    fn low_confidence_edge(source: &str, target: &str, kind: GraphEdgeKind) -> GraphEdge {
        let mut edge = GraphEdge::new(source.to_string(), target.to_string(), kind);
        edge.options.evidence = Some(vec![GraphEvidence {
            kind: GraphEvidenceKind::Inferred,
            confidence: Some(GraphEvidenceConfidence::Low),
            ..Default::default()
        }]);
        edge
    }

    #[test]
    fn edge_labels_are_readable() -> Result<()> {
        assert_eq!(edge_label(GraphEdgeKind::DerivedInto), "Derived Into");
        Ok(())
    }
}
