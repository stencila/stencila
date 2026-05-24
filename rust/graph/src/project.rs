//! Projection utilities for display-oriented graph views.
//!
//! Raw Schema graphs preserve all discovered relationships. Display projections
//! answer a narrower reader question by selecting relationship families,
//! collapsing noisy intermediates, and attaching compact labels.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use clap::ValueEnum;
use eyre::{Result, bail};
use glob::Pattern;
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
    GraphProjectionPreset::React,
    GraphProjectionPreset::Cite,
    GraphProjectionPreset::Deps,
    GraphProjectionPreset::Flow,
];

/// User-facing graph projection preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
pub enum GraphProjectionPreset {
    /// Choose the first useful projection from the graph's relationships.
    ///
    /// Auto mode scores focused views using the graph's relationships. It gives
    /// document reactivity and citations enough weight to beat generic file
    /// conversion/provenance edges, then falls back to all relationships when only
    /// structural containment is present.
    Auto,

    /// Show every graph node and edge without applying a focused projection.
    ///
    /// This is the most complete view and the noisiest one. Use it when
    /// debugging graph collection itself, checking whether expected nodes or
    /// edges exist, or comparing a projected view against the raw graph shape.
    All,

    /// Show resource flow, data lineage, and provenance relationships.
    ///
    /// This answers questions such as which files, tables, code units, or
    /// outputs read, generated, derived, or used each other. It is
    /// intended for tracing where data and rendered outputs came from, rather
    /// than for ordering executable document updates.
    Flow,

    /// Show software imports, calls, environments, packages, and dependency use.
    ///
    /// This focuses on code and environment relationships: imported packages,
    /// declared environment packages, package use by source files, manifests,
    /// lockfiles, and calls between discovered functions or code units. It is
    /// useful for understanding the software stack behind a workspace without
    /// mixing in data products and document structure.
    Deps,

    /// Show bibliographic references, citations, and external resource links.
    ///
    /// This focuses on `CitedBy` relationships plus `LinkedBy` relationships
    /// from external resources. Local file, table, and media references are
    /// left to the flow view so citation graphs stay focused on works and
    /// external resources cited or referenced by the document.
    Cite,

    /// Show executable document reactivity dependencies.
    ///
    /// This focuses on code-symbol dependency chains that decide what should
    /// update or rerun when executable document state changes. It is distinct
    /// from `flow`, which also tracks broader provenance and produced resources.
    React,
}

/// Amount of detail to include in focused graph projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum GraphProjectionDetail {
    /// Show only the main resource, code, output, and environment relationships.
    Low,

    /// Show useful data-level detail while hiding local symbol and function internals.
    Medium,

    /// Show all relationships selected by the preset.
    High,
}

impl Default for GraphProjectionDetail {
    fn default() -> Self {
        Self::Medium
    }
}

impl GraphProjectionDetail {
    /// Stable CLI/display name for the detail level.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl GraphProjectionPreset {
    /// Stable CLI/display name for the preset.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::All => "all",
            Self::Flow => "flow",
            Self::Deps => "deps",
            Self::Cite => "cite",
            Self::React => "react",
        }
    }
}

/// How structural containment should be represented in projected graph exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum GraphContainmentMode {
    /// Do not include structural containment context.
    None,

    /// Use containment to group nodes visually, without rendering PartOf edges.
    Clusters,

    /// Render containment as explicit PartOf edges.
    Edges,

    /// Use both visual groups and explicit PartOf edges.
    Both,
}

impl GraphContainmentMode {
    /// Stable CLI/display name for the mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clusters => "clusters",
            Self::Edges => "edges",
            Self::Both => "both",
        }
    }

    pub(crate) fn uses_clusters(self) -> bool {
        matches!(self, Self::Clusters | Self::Both)
    }

    fn uses_edges(self) -> bool {
        matches!(self, Self::Edges | Self::Both)
    }

    fn includes_context(self) -> bool {
        self != Self::None
    }
}

/// Broad relationship family used by display projections and legends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphEdgeFamily {
    Structure,
    DataFlow,
    Software,
    Citation,
    Reference,
}

/// Classify a graph edge kind into the user-facing question it helps answer.
pub fn edge_family(kind: GraphEdgeKind) -> GraphEdgeFamily {
    match kind {
        GraphEdgeKind::PartOf => GraphEdgeFamily::Structure,
        GraphEdgeKind::ReadBy
        | GraphEdgeKind::Generated
        | GraphEdgeKind::WrittenTo
        | GraphEdgeKind::DerivedInto
        | GraphEdgeKind::ConvertedInto => GraphEdgeFamily::DataFlow,
        GraphEdgeKind::ImportedBy
        | GraphEdgeKind::CalledBy
        | GraphEdgeKind::Declares
        | GraphEdgeKind::RequiredBy
        | GraphEdgeKind::Pins
        | GraphEdgeKind::Configures => GraphEdgeFamily::Software,
        GraphEdgeKind::CitedBy => GraphEdgeFamily::Citation,
        GraphEdgeKind::UsedBy | GraphEdgeKind::LinkedBy | GraphEdgeKind::IncludedBy => {
            GraphEdgeFamily::Reference
        }
    }
}

/// Options used to project a graph for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphProjectionOptions {
    /// Projection preset to apply.
    pub preset: GraphProjectionPreset,

    /// Amount of detail to include in focused projections.
    pub detail: GraphProjectionDetail,

    /// How to represent structural containment relationships.
    pub containment: Option<GraphContainmentMode>,

    /// Include structural containment edges.
    ///
    /// This legacy option is retained for callers that still treat structure as
    /// an edge toggle. Prefer `containment` for new code.
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
            detail: GraphProjectionDetail::default(),
            containment: None,
            include_structure_edges: None,
            include_low_confidence_edges: true,
            collapse_citation_nodes: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedGraphProjectionOptions {
    detail: GraphProjectionDetail,
    containment: GraphContainmentMode,
    include_low_confidence_edges: bool,
    collapse_citation_nodes: bool,
}

#[derive(Debug, Clone, Copy)]
struct PrimaryGraphProjectionOptions {
    preset: GraphProjectionPreset,
    detail: GraphProjectionDetail,
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
    pub detail: GraphProjectionDetail,
    pub containment: GraphContainmentMode,
    pub nodes: Vec<GraphViewNode>,
    pub edges: Vec<GraphViewEdge>,
    pub containments: Vec<GraphViewEdge>,
}

/// How a connected-node filter should traverse projected graph edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum GraphConnectedMode {
    /// Include upstream dependencies and downstream dependents of matched nodes.
    ///
    /// Traversal does not switch direction at intermediate nodes, so shared
    /// inputs do not pull in sibling consumers.
    Directed,

    /// Include the full undirected component containing matched nodes.
    Undirected,
}

impl Default for GraphConnectedMode {
    fn default() -> Self {
        Self::Directed
    }
}

/// Filter a projected graph view to nodes connected to matched seed nodes.
///
/// In directed mode, connectivity is computed as the union of upstream and
/// downstream projected non-structural edges from the seed nodes. In undirected
/// mode, connectivity is the full component over those same non-structural
/// edges. Structural ancestors are then re-added for retained nodes.
pub fn filter_graph_view_connected_to(
    view: &GraphView,
    patterns: &[String],
    mode: GraphConnectedMode,
) -> Result<GraphView> {
    if patterns.is_empty() {
        return Ok(view.clone());
    }

    let mut seeds = BTreeSet::new();
    for pattern in patterns {
        let matches = matching_node_ids(view, pattern)?;
        if matches.is_empty() {
            bail!(
                "no projected graph nodes match `{pattern}`; check the pattern or projection options"
            );
        }
        seeds.extend(matches);
    }

    let retained = connected_node_ids(view, &seeds, mode);
    Ok(retain_graph_view_nodes(view, &retained))
}

#[derive(Debug)]
struct ProjectedEdge<'a> {
    edge: &'a GraphEdge,
    source: String,
    target: String,
}

#[derive(Debug, Default)]
struct WorkflowScriptFlow {
    units_by_script: BTreeMap<String, BTreeSet<String>>,
    redundant_reads: BTreeSet<(String, String)>,
    redundant_writes: BTreeSet<(String, String)>,
    nonredundant_by_script_unit: BTreeSet<(String, String)>,
}

impl WorkflowScriptFlow {
    fn is_redundant_projected_edge(&self, kind: GraphEdgeKind, source: &str, target: &str) -> bool {
        match kind {
            GraphEdgeKind::ReadBy if self.units_by_script.contains_key(target) => self
                .redundant_reads
                .contains(&(source.to_string(), target.to_string())),
            GraphEdgeKind::Generated | GraphEdgeKind::WrittenTo
                if self.units_by_script.contains_key(source) =>
            {
                self.redundant_writes
                    .contains(&(source.to_string(), target.to_string()))
            }
            GraphEdgeKind::UsedBy if self.is_workflow_script_use(source, target) => !self
                .nonredundant_by_script_unit
                .contains(&(source.to_string(), target.to_string())),
            _ => false,
        }
    }

    fn is_workflow_script_use(&self, script: &str, unit: &str) -> bool {
        self.units_by_script
            .get(script)
            .is_some_and(|units| units.contains(unit))
    }
}

/// Project a Schema graph into a display graph view.
pub fn project_graph(graph: &Graph, options: &GraphProjectionOptions) -> GraphView {
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let parent_by_id = parent_map(graph);
    let workflow_script_flow = workflow_script_flow(
        graph,
        &nodes_by_id,
        &parent_by_id,
        options.include_low_confidence_edges,
    );
    let preset = resolve_preset(
        graph,
        options,
        &nodes_by_id,
        &parent_by_id,
        &workflow_script_flow,
    );
    let resolved = resolve_projection_options(preset, options);
    let primary_options = PrimaryGraphProjectionOptions {
        preset,
        detail: resolved.detail,
        include_low_confidence_edges: resolved.include_low_confidence_edges,
        collapse_citation_nodes: resolved.collapse_citation_nodes,
    };
    let mut node_ids = BTreeSet::new();
    let mut edges = BTreeMap::new();
    let mut containments = BTreeMap::new();

    for edge in &graph.edges {
        let Some(projected) = project_primary_edge(
            edge,
            primary_options,
            &nodes_by_id,
            &parent_by_id,
            &workflow_script_flow,
        ) else {
            continue;
        };

        node_ids.insert(projected.source.clone());
        node_ids.insert(projected.target.clone());
        add_view_edge(&mut edges, projected);
    }

    if preset == GraphProjectionPreset::All {
        node_ids.extend(graph.nodes.iter().map(|node| node.id.clone()));
    }

    if preset == GraphProjectionPreset::Deps {
        add_dependency_memberships(
            graph,
            &nodes_by_id,
            &mut node_ids,
            &mut containments,
            resolved.include_low_confidence_edges,
        );
    }

    if resolved.containment.includes_context() {
        add_structure_edges(
            graph,
            &nodes_by_id,
            &parent_by_id,
            &mut node_ids,
            &mut containments,
            resolved.include_low_confidence_edges,
            preset,
        );
    }

    if resolved.containment.uses_edges() {
        for containment in containments.values() {
            edges.insert(containment.id.clone(), containment.clone());
        }
    }

    let nodes = node_ids
        .into_iter()
        .filter_map(|id| nodes_by_id.get(id.as_str()).map(|node| view_node(node)))
        .collect();

    GraphView {
        preset,
        detail: resolved.detail,
        containment: resolved.containment,
        nodes,
        edges: edges.into_values().collect(),
        containments: containments.into_values().collect(),
    }
}

fn matching_node_ids(view: &GraphView, pattern: &str) -> Result<Vec<String>> {
    let exact_id = view
        .nodes
        .iter()
        .filter(|node| node.id == pattern)
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if !exact_id.is_empty() {
        return Ok(exact_id);
    }

    let exact_text = view
        .nodes
        .iter()
        .filter(|node| node_match_texts(node).iter().any(|text| text == pattern))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if !exact_text.is_empty() {
        return Ok(exact_text);
    }

    if has_glob_metacharacters(pattern) {
        let glob = Pattern::new(pattern).map_err(|error| {
            eyre::eyre!("invalid connected-to glob pattern `{pattern}`: {error}")
        })?;
        let glob_matches = view
            .nodes
            .iter()
            .filter(|node| node_match_texts(node).iter().any(|text| glob.matches(text)))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        if !glob_matches.is_empty() {
            return Ok(glob_matches);
        }
    }

    Ok(view
        .nodes
        .iter()
        .filter(|node| {
            node_match_texts(node)
                .iter()
                .any(|text| text.contains(pattern))
        })
        .map(|node| node.id.clone())
        .collect())
}

fn connected_node_ids(
    view: &GraphView,
    seeds: &BTreeSet<String>,
    mode: GraphConnectedMode,
) -> BTreeSet<String> {
    let visible_node_ids = view
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut incoming = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut outgoing = BTreeMap::<&str, BTreeSet<&str>>::new();

    for edge in view
        .edges
        .iter()
        .filter(|edge| edge.kind != STRUCTURE_EDGE_KIND)
    {
        if !visible_node_ids.contains(edge.source.as_str())
            || !visible_node_ids.contains(edge.target.as_str())
        {
            continue;
        }

        outgoing
            .entry(edge.source.as_str())
            .or_default()
            .insert(edge.target.as_str());
        incoming
            .entry(edge.target.as_str())
            .or_default()
            .insert(edge.source.as_str());
    }

    let visible_seeds = seeds
        .iter()
        .filter(|id| visible_node_ids.contains(id.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();
    let expanded_seeds = containment_descendant_node_ids(view, &visible_seeds, &visible_node_ids);
    let mut retained = expanded_seeds.clone();

    match mode {
        GraphConnectedMode::Directed => {
            retained.extend(directed_reachable_node_ids(&expanded_seeds, &incoming));
            retained.extend(directed_reachable_node_ids(&expanded_seeds, &outgoing));
        }
        GraphConnectedMode::Undirected => {
            retained.extend(undirected_reachable_node_ids(
                &expanded_seeds,
                &incoming,
                &outgoing,
            ));
        }
    }

    retained
}

fn containment_descendant_node_ids(
    view: &GraphView,
    seeds: &BTreeSet<String>,
    visible_node_ids: &BTreeSet<&str>,
) -> BTreeSet<String> {
    let mut children = BTreeMap::<&str, BTreeSet<&str>>::new();

    for edge in &view.containments {
        if visible_node_ids.contains(edge.source.as_str())
            && visible_node_ids.contains(edge.target.as_str())
        {
            children
                .entry(edge.target.as_str())
                .or_default()
                .insert(edge.source.as_str());
        }
    }

    let mut retained = seeds.clone();
    let mut queue = seeds.iter().map(String::as_str).collect::<VecDeque<_>>();

    while let Some(id) = queue.pop_front() {
        for child in children.get(id).into_iter().flatten() {
            if retained.insert((*child).to_string()) {
                queue.push_back(child);
            }
        }
    }

    retained
}

fn directed_reachable_node_ids(
    seeds: &BTreeSet<String>,
    adjacency: &BTreeMap<&str, BTreeSet<&str>>,
) -> BTreeSet<String> {
    let mut retained = BTreeSet::new();
    let mut queue = seeds.iter().map(String::as_str).collect::<VecDeque<_>>();

    while let Some(id) = queue.pop_front() {
        for adjacent in adjacency.get(id).into_iter().flatten() {
            if retained.insert((*adjacent).to_string()) {
                queue.push_back(adjacent);
            }
        }
    }

    retained
}

fn undirected_reachable_node_ids(
    seeds: &BTreeSet<String>,
    incoming: &BTreeMap<&str, BTreeSet<&str>>,
    outgoing: &BTreeMap<&str, BTreeSet<&str>>,
) -> BTreeSet<String> {
    let mut retained = BTreeSet::new();
    let mut queue = seeds.iter().map(String::as_str).collect::<VecDeque<_>>();

    while let Some(id) = queue.pop_front() {
        for adjacent in incoming
            .get(id)
            .into_iter()
            .chain(outgoing.get(id))
            .flatten()
        {
            if retained.insert((*adjacent).to_string()) {
                queue.push_back(adjacent);
            }
        }
    }

    retained
}

fn retain_graph_view_nodes(view: &GraphView, retained: &BTreeSet<String>) -> GraphView {
    let mut contextual = retained.clone();
    let mut changed = true;

    while changed {
        changed = false;
        for edge in &view.containments {
            if contextual.contains(&edge.source) && contextual.insert(edge.target.clone()) {
                changed = true;
            }
        }
    }

    GraphView {
        preset: view.preset,
        detail: view.detail,
        containment: view.containment,
        nodes: view
            .nodes
            .iter()
            .filter(|node| contextual.contains(&node.id))
            .cloned()
            .collect(),
        edges: view
            .edges
            .iter()
            .filter(|edge| contextual.contains(&edge.source) && contextual.contains(&edge.target))
            .cloned()
            .collect(),
        containments: view
            .containments
            .iter()
            .filter(|edge| contextual.contains(&edge.source) && contextual.contains(&edge.target))
            .cloned()
            .collect(),
    }
}

fn node_match_texts(node: &GraphViewNode) -> Vec<String> {
    let mut texts = BTreeSet::from([node.id.clone(), node.label.clone()]);
    let value = serde_json::to_value(node.node.node.as_ref()).unwrap_or(Value::Null);

    for key in ["id", "name", "title", "path", "url", "target"] {
        let Some(text) = string_value(value.get(key)) else {
            continue;
        };

        texts.insert(text.clone());
        if let Some(basename) = basename(&text) {
            texts.insert(basename);
        }
    }

    texts.into_iter().collect()
}

fn basename(value: &str) -> Option<String> {
    let basename = value.rsplit(['/', '\\']).next().unwrap_or(value);
    (!basename.is_empty() && basename != value).then(|| basename.to_string())
}

fn has_glob_metacharacters(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

fn resolve_projection_options(
    preset: GraphProjectionPreset,
    options: &GraphProjectionOptions,
) -> ResolvedGraphProjectionOptions {
    ResolvedGraphProjectionOptions {
        containment: options.containment.unwrap_or_else(|| {
            options.include_structure_edges.map_or_else(
                || default_containment_mode(preset),
                |include| {
                    if include {
                        GraphContainmentMode::Edges
                    } else {
                        GraphContainmentMode::None
                    }
                },
            )
        }),
        detail: options.detail,
        include_low_confidence_edges: options.include_low_confidence_edges,
        collapse_citation_nodes: options.collapse_citation_nodes,
    }
}

fn default_containment_mode(preset: GraphProjectionPreset) -> GraphContainmentMode {
    if preset == GraphProjectionPreset::All {
        GraphContainmentMode::Edges
    } else {
        GraphContainmentMode::Clusters
    }
}

fn resolve_preset(
    graph: &Graph,
    options: &GraphProjectionOptions,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
    workflow_script_flow: &WorkflowScriptFlow,
) -> GraphProjectionPreset {
    if options.preset != GraphProjectionPreset::Auto {
        return options.preset;
    }

    let scores = AUTO_PRESETS
        .into_iter()
        .map(|preset| {
            (
                preset,
                preset_score(
                    graph,
                    PrimaryGraphProjectionOptions {
                        preset,
                        detail: options.detail,
                        include_low_confidence_edges: options.include_low_confidence_edges,
                        collapse_citation_nodes: options.collapse_citation_nodes,
                    },
                    nodes_by_id,
                    parent_by_id,
                    workflow_script_flow,
                ),
            )
        })
        .collect::<Vec<_>>();
    let best_score = scores.iter().map(|(.., score)| *score).max().unwrap_or(0);

    if best_score == 0 {
        GraphProjectionPreset::All
    } else {
        scores
            .into_iter()
            .find(|(.., score)| *score == best_score)
            .map(|(preset, ..)| preset)
            .unwrap_or(GraphProjectionPreset::All)
    }
}

fn project_primary_edge<'a>(
    edge: &'a GraphEdge,
    options: PrimaryGraphProjectionOptions,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
    workflow_script_flow: &WorkflowScriptFlow,
) -> Option<ProjectedEdge<'a>> {
    if !options.include_low_confidence_edges && has_low_confidence(edge) {
        return None;
    }

    if edge_family(edge.kind) == GraphEdgeFamily::Structure {
        return None;
    }

    if edge_score(edge, options.preset, nodes_by_id) == 0 {
        return None;
    }

    if options.preset == GraphProjectionPreset::Flow
        && options.detail != GraphProjectionDetail::High
        && edge.kind == GraphEdgeKind::CalledBy
        && edge_has_local_code_internal_endpoint(edge, nodes_by_id)
    {
        return None;
    }

    let mut source = edge.source.clone();
    let mut target = edge.target.clone();
    let mut endpoints_projected = false;

    if options.preset == GraphProjectionPreset::Cite
        && options.collapse_citation_nodes
        && edge.kind == GraphEdgeKind::CitedBy
    {
        target = collapse_citation_target(&target, nodes_by_id, parent_by_id);
        endpoints_projected = target != edge.target;
    }

    if options.preset == GraphProjectionPreset::Flow
        && options.detail != GraphProjectionDetail::High
    {
        source = collapse_local_code_internal_endpoint(&source, nodes_by_id, parent_by_id);
        target = collapse_local_code_internal_endpoint(&target, nodes_by_id, parent_by_id);
        endpoints_projected = source != edge.source || target != edge.target;
    }

    if (endpoints_projected && source == target)
        || !nodes_by_id.contains_key(source.as_str())
        || !nodes_by_id.contains_key(target.as_str())
    {
        return None;
    }

    if options.preset == GraphProjectionPreset::Flow
        && options.detail != GraphProjectionDetail::High
        && workflow_script_flow.is_redundant_projected_edge(edge.kind, &source, &target)
    {
        return None;
    }

    include_edge_for_detail(
        edge.kind,
        &source,
        &target,
        options.preset,
        options.detail,
        nodes_by_id,
    )
    .then_some(ProjectedEdge {
        edge,
        source,
        target,
    })
}

fn preset_score(
    graph: &Graph,
    options: PrimaryGraphProjectionOptions,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
    workflow_script_flow: &WorkflowScriptFlow,
) -> usize {
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            project_primary_edge(
                edge,
                options,
                nodes_by_id,
                parent_by_id,
                workflow_script_flow,
            )
        })
        .map(|projected| edge_score(projected.edge, options.preset, nodes_by_id))
        .sum()
}

fn workflow_script_flow(
    graph: &Graph,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
    include_low_confidence_edges: bool,
) -> WorkflowScriptFlow {
    let mut flow = WorkflowScriptFlow::default();

    for edge in &graph.edges {
        if edge.kind != GraphEdgeKind::UsedBy
            || (!include_low_confidence_edges && has_low_confidence(edge))
            || !is_code_node(&edge.source, nodes_by_id)
            || !is_workflow_unit(&edge.target)
        {
            continue;
        }

        flow.units_by_script
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());
    }

    if flow.units_by_script.is_empty() {
        return flow;
    }

    let mut unit_reads = BTreeSet::new();
    let mut unit_writes = BTreeSet::new();
    let mut script_reads = BTreeSet::new();
    let mut script_writes = BTreeSet::new();

    for edge in &graph.edges {
        if (!include_low_confidence_edges && has_low_confidence(edge))
            || !matches!(
                edge.kind,
                GraphEdgeKind::ReadBy | GraphEdgeKind::Generated | GraphEdgeKind::WrittenTo
            )
        {
            continue;
        }

        let source = collapse_local_code_internal_endpoint(&edge.source, nodes_by_id, parent_by_id);
        let target = collapse_local_code_internal_endpoint(&edge.target, nodes_by_id, parent_by_id);

        if source == target
            || !nodes_by_id.contains_key(source.as_str())
            || !nodes_by_id.contains_key(target.as_str())
        {
            continue;
        }

        match edge.kind {
            GraphEdgeKind::ReadBy => {
                if is_workflow_unit(&target) {
                    unit_reads.insert((source.clone(), target.clone()));
                }
                if flow.units_by_script.contains_key(&target) {
                    script_reads.insert((source, target));
                }
            }
            GraphEdgeKind::Generated | GraphEdgeKind::WrittenTo => {
                if is_workflow_unit(&source) {
                    unit_writes.insert((source.clone(), target.clone()));
                }
                if flow.units_by_script.contains_key(&source) {
                    script_writes.insert((source, target));
                }
            }
            _ => {}
        }
    }

    for (resource, script) in script_reads {
        let units = flow
            .units_by_script
            .get(&script)
            .cloned()
            .unwrap_or_default();
        let mut redundant = true;

        for unit in units {
            if unit_reads.contains(&(resource.clone(), unit.clone())) {
                continue;
            }

            redundant = false;
            flow.nonredundant_by_script_unit
                .insert((script.clone(), unit));
        }

        if redundant {
            flow.redundant_reads.insert((resource, script));
        }
    }

    for (script, resource) in script_writes {
        let units = flow
            .units_by_script
            .get(&script)
            .cloned()
            .unwrap_or_default();
        let mut redundant = true;

        for unit in units {
            if unit_writes.contains(&(unit.clone(), resource.clone())) {
                continue;
            }

            redundant = false;
            flow.nonredundant_by_script_unit
                .insert((script.clone(), unit));
        }

        if redundant {
            flow.redundant_writes.insert((script, resource));
        }
    }

    flow
}

fn include_edge_for_detail(
    kind: GraphEdgeKind,
    source: &str,
    target: &str,
    preset: GraphProjectionPreset,
    detail: GraphProjectionDetail,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
) -> bool {
    if matches!(
        preset,
        GraphProjectionPreset::All | GraphProjectionPreset::React | GraphProjectionPreset::Cite
    ) || detail == GraphProjectionDetail::High
    {
        return true;
    }

    let source_kind = node_kind(nodes_by_id.get(source).copied());
    let target_kind = node_kind(nodes_by_id.get(target).copied());
    let source_internal = is_local_code_internal(source, source_kind);
    let target_internal = is_local_code_internal(target, target_kind);
    let source_datatable_detail = is_datatable_detail_node(source, source_kind, nodes_by_id);
    let target_datatable_detail = is_datatable_detail_node(target, target_kind, nodes_by_id);

    match preset {
        GraphProjectionPreset::Flow => match detail {
            GraphProjectionDetail::Low => {
                !source_internal
                    && !target_internal
                    && !source_datatable_detail
                    && !target_datatable_detail
            }
            GraphProjectionDetail::Medium => {
                !source_internal
                    && !target_internal
                    && (!(source_datatable_detail || target_datatable_detail)
                        || kind == GraphEdgeKind::DerivedInto)
            }
            GraphProjectionDetail::High => true,
        },
        GraphProjectionPreset::Deps => !source_internal && !target_internal,
        GraphProjectionPreset::Auto
        | GraphProjectionPreset::All
        | GraphProjectionPreset::Cite
        | GraphProjectionPreset::React => true,
    }
}

fn is_local_code_internal(id: &str, kind: GraphViewNodeKind) -> bool {
    kind == GraphViewNodeKind::Symbol
        || (kind == GraphViewNodeKind::Function && graph_id_namespace(id) != "workflow-unit")
}

fn is_code_node(id: &str, nodes_by_id: &BTreeMap<&str, &GraphNode>) -> bool {
    node_kind(nodes_by_id.get(id).copied()) == GraphViewNodeKind::Code
}

fn is_workflow_unit(id: &str) -> bool {
    graph_id_namespace(id) == "workflow-unit"
}

fn edge_has_local_code_internal_endpoint(
    edge: &GraphEdge,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
) -> bool {
    [edge.source.as_str(), edge.target.as_str()]
        .into_iter()
        .any(|id| is_local_code_internal(id, node_kind(nodes_by_id.get(id).copied())))
}

fn is_datatable_detail_node(
    id: &str,
    kind: GraphViewNodeKind,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
) -> bool {
    kind == GraphViewNodeKind::Datatable
        && (graph_id_namespace(id) == "column"
            || nodes_by_id
                .get(id)
                .copied()
                .and_then(schema_node_type)
                .as_deref()
                == Some("DatatableColumn"))
}

fn edge_score(
    edge: &GraphEdge,
    preset: GraphProjectionPreset,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
) -> usize {
    if preset == GraphProjectionPreset::All {
        return usize::from(edge_family(edge.kind) != GraphEdgeFamily::Structure);
    }

    match edge.kind {
        GraphEdgeKind::UsedBy => {
            if preset == GraphProjectionPreset::React && is_reactive_use_edge(edge, nodes_by_id) {
                20
            } else {
                matches!(
                    preset,
                    GraphProjectionPreset::Flow | GraphProjectionPreset::Deps
                )
                .then_some(1)
                .unwrap_or(0)
            }
        }
        GraphEdgeKind::ReadBy
        | GraphEdgeKind::Generated
        | GraphEdgeKind::WrittenTo
        | GraphEdgeKind::DerivedInto
        | GraphEdgeKind::ConvertedInto
        | GraphEdgeKind::IncludedBy => {
            if preset == GraphProjectionPreset::React
                && edge.kind == GraphEdgeKind::Generated
                && is_reactive_generation_edge(edge, nodes_by_id)
            {
                20
            } else if preset == GraphProjectionPreset::Deps
                && edge_targets_environment(edge, nodes_by_id)
            {
                match edge.kind {
                    GraphEdgeKind::DerivedInto => 6,
                    _ => 0,
                }
            } else if preset == GraphProjectionPreset::Flow {
                match edge.kind {
                    GraphEdgeKind::ReadBy | GraphEdgeKind::DerivedInto => 5,
                    GraphEdgeKind::IncludedBy => 4,
                    GraphEdgeKind::Generated | GraphEdgeKind::WrittenTo => 3,
                    GraphEdgeKind::ConvertedInto => 0,
                    _ => 0,
                }
            } else {
                0
            }
        }
        GraphEdgeKind::CalledBy => {
            if matches!(
                preset,
                GraphProjectionPreset::Deps | GraphProjectionPreset::Flow
            ) {
                4
            } else {
                0
            }
        }
        GraphEdgeKind::ImportedBy => {
            if preset == GraphProjectionPreset::Deps {
                4
            } else {
                0
            }
        }
        GraphEdgeKind::CitedBy => {
            if preset == GraphProjectionPreset::Cite {
                20
            } else {
                0
            }
        }
        GraphEdgeKind::LinkedBy => {
            let targets_environment = edge_targets_environment(edge, nodes_by_id);
            if (preset == GraphProjectionPreset::Deps && targets_environment)
                || (preset == GraphProjectionPreset::Cite
                    && edge_sources_external_resource(edge, nodes_by_id))
            {
                4
            } else {
                0
            }
        }
        GraphEdgeKind::Declares | GraphEdgeKind::Pins => {
            if preset == GraphProjectionPreset::Deps && edge_targets_environment(edge, nodes_by_id)
            {
                8
            } else {
                0
            }
        }
        GraphEdgeKind::RequiredBy => {
            if preset == GraphProjectionPreset::Deps && edge_targets_environment(edge, nodes_by_id)
            {
                10
            } else {
                0
            }
        }
        GraphEdgeKind::Configures => {
            if preset == GraphProjectionPreset::Deps || preset == GraphProjectionPreset::Flow {
                3
            } else {
                0
            }
        }
        GraphEdgeKind::PartOf => {
            if preset == GraphProjectionPreset::Deps && is_dependency_membership(edge, nodes_by_id)
            {
                10
            } else {
                0
            }
        }
    }
}

fn edge_targets_environment(edge: &GraphEdge, nodes_by_id: &BTreeMap<&str, &GraphNode>) -> bool {
    node_kind(nodes_by_id.get(edge.target.as_str()).copied()) == GraphViewNodeKind::Environment
}

fn edge_sources_external_resource(
    edge: &GraphEdge,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
) -> bool {
    graph_id_namespace(&edge.source) == "resource"
        && !edge_targets_environment(edge, nodes_by_id)
        && node_kind(nodes_by_id.get(edge.source.as_str()).copied()) == GraphViewNodeKind::Resource
}

fn is_reactive_generation_edge(edge: &GraphEdge, nodes_by_id: &BTreeMap<&str, &GraphNode>) -> bool {
    node_kind(nodes_by_id.get(edge.source.as_str()).copied()) == GraphViewNodeKind::Code
        && matches!(
            node_kind(nodes_by_id.get(edge.target.as_str()).copied()),
            GraphViewNodeKind::Symbol | GraphViewNodeKind::Function
        )
}

fn is_reactive_use_edge(edge: &GraphEdge, nodes_by_id: &BTreeMap<&str, &GraphNode>) -> bool {
    matches!(
        node_kind(nodes_by_id.get(edge.source.as_str()).copied()),
        GraphViewNodeKind::Symbol | GraphViewNodeKind::Function
    ) && node_kind(nodes_by_id.get(edge.target.as_str()).copied()) == GraphViewNodeKind::Code
}

fn is_dependency_membership(edge: &GraphEdge, nodes_by_id: &BTreeMap<&str, &GraphNode>) -> bool {
    edge.kind == STRUCTURE_EDGE_KIND
        && node_kind(nodes_by_id.get(edge.source.as_str()).copied()) == GraphViewNodeKind::Package
        && node_kind(nodes_by_id.get(edge.target.as_str()).copied())
            == GraphViewNodeKind::Environment
}

fn add_dependency_memberships(
    graph: &Graph,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    node_ids: &mut BTreeSet<String>,
    containments: &mut BTreeMap<String, GraphViewEdge>,
    include_low_confidence_edges: bool,
) {
    for edge in &graph.edges {
        if (!include_low_confidence_edges && has_low_confidence(edge))
            || !is_dependency_membership(edge, nodes_by_id)
        {
            continue;
        }

        node_ids.insert(edge.source.clone());
        node_ids.insert(edge.target.clone());
        add_view_edge(
            containments,
            ProjectedEdge {
                edge,
                source: edge.source.clone(),
                target: edge.target.clone(),
            },
        );
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

    if preset == GraphProjectionPreset::All {
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

fn collapse_local_code_internal_endpoint(
    id: &str,
    nodes_by_id: &BTreeMap<&str, &GraphNode>,
    parent_by_id: &BTreeMap<String, String>,
) -> String {
    let mut current = id.to_string();
    let mut visited = BTreeSet::new();

    while is_local_code_internal(
        &current,
        node_kind(nodes_by_id.get(current.as_str()).copied()),
    ) {
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
        "function" | "workflow-unit" => return GraphViewNodeKind::Function,
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
        Some("Parameter" | "Variable") => GraphViewNodeKind::Symbol,
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

    for key in ["name", "title", "path", "url", "target"] {
        if let Some(label) = string_value(value.get(key)) {
            return compact_label(&label);
        }
    }

    if node_kind(Some(node)) == GraphViewNodeKind::Document
        && let Some(label) = document_scope_label(&node.id)
    {
        return label;
    }

    if let Some(label) = string_value(value.get("id")) {
        return compact_label(&label);
    }

    compact_label(&node.id)
}

fn graph_id_namespace(id: &str) -> &str {
    id.split_once(':').map_or(id, |(namespace, ..)| namespace)
}

fn document_scope_label(id: &str) -> Option<String> {
    let (namespace, scoped) = id.split_once(':')?;
    if namespace != "node" {
        return None;
    }

    let (scope, ..) = scoped.split_once('#')?;
    (!scope.is_empty()).then(|| compact_label(scope))
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
        Article, Datatable, DatatableColumn, Directory, File, Function, Graph, GraphEdge,
        GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence, GraphEvidenceKind, GraphNode, Node,
        Reference, SoftwareApplication, SoftwareSourceCode, Variable,
    };

    use super::*;

    #[test]
    fn selects_data_flow_projection_automatically() {
        let view = project_graph(&duplicate_edge_graph(), &GraphProjectionOptions::default());

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
    fn prefers_citation_projection_over_generic_flow_edges() {
        let view = project_graph(&graph(), &GraphProjectionOptions::default());

        assert_eq!(view.preset, GraphProjectionPreset::Cite);
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![GraphEdgeKind::CitedBy]
        );
    }

    #[test]
    fn selects_dependency_projection_for_declared_environments() {
        let view = project_graph(&environment_graph(), &GraphProjectionOptions::default());

        assert_eq!(view.preset, GraphProjectionPreset::Deps);
        assert_eq!(view.containment, GraphContainmentMode::Clusters);
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![
                GraphEdgeKind::Declares,
                GraphEdgeKind::Pins,
                GraphEdgeKind::RequiredBy
            ]
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
    fn cite_projection_keeps_external_links_only() {
        let mut graph = graph();
        graph.nodes.push(graph_node(
            "resource:https%3A//example.org/archive",
            Node::String("https://example.org/archive".to_string()),
        ));
        graph.edges.extend([
            GraphEdge::new(
                "resource:https%3A//example.org/archive".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::LinkedBy,
            ),
            GraphEdge::new(
                "file:data.csv".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::LinkedBy,
            ),
        ]);

        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Cite,
                ..Default::default()
            },
        );

        assert!(view.edges.iter().any(|edge| {
            edge.kind == GraphEdgeKind::LinkedBy
                && edge.source == "resource:https%3A//example.org/archive"
        }));
        assert!(
            !view.edges.iter().any(|edge| {
                edge.kind == GraphEdgeKind::LinkedBy && edge.source == "file:data.csv"
            }),
            "local file links should stay out of cite projection"
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
    fn uses_clusters_for_focused_containment_by_default() {
        let view = project_graph(
            &graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        assert_eq!(view.containment, GraphContainmentMode::Clusters);
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![GraphEdgeKind::ReadBy]
        );
        assert_eq!(
            view.containments
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec!["PartOf:code:analysis.py->node:document#article"]
        );
    }

    #[test]
    fn uses_all_preset_structure_defaults_after_auto_resolution() {
        let view = project_graph(&structure_only_graph(), &GraphProjectionOptions::default());

        assert_eq!(view.preset, GraphProjectionPreset::All);
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

    #[test]
    fn flow_detail_defaults_to_medium_without_local_symbols() {
        let view = project_graph(
            &detail_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.kind == GraphViewNodeKind::Symbol)
        );
        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.id == "function:analysis.py:python:read_csv")
        );
        assert!(
            view.nodes
                .iter()
                .any(|node| node.id == "column:analysis.py:data.csv:count")
        );
        assert_eq!(
            view.edges
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec![
                "DerivedInto:column:analysis.py:data.csv:count->file:plot.png",
                "Generated:code:analysis.py->file:plot.png",
                "ReadBy:file:data.csv->code:analysis.py",
            ]
        );
    }

    #[test]
    fn flow_detail_low_hides_datatable_columns() {
        let view = project_graph(
            &detail_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::Low,
                ..Default::default()
            },
        );

        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.kind == GraphViewNodeKind::Datatable)
        );
        assert_eq!(
            view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
            vec![GraphEdgeKind::Generated, GraphEdgeKind::ReadBy]
        );
    }

    #[test]
    fn flow_detail_high_includes_local_symbols_and_functions() {
        let view = project_graph(
            &detail_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        assert!(
            view.nodes
                .iter()
                .any(|node| node.id == "symbol:analysis.py:python:df")
        );
        assert!(
            view.nodes
                .iter()
                .any(|node| node.id == "function:analysis.py:python:read_csv")
        );
        assert!(
            view.edges
                .iter()
                .any(|edge| edge.kind == GraphEdgeKind::CalledBy)
        );
    }

    #[test]
    fn labels_document_roots_from_their_scope() {
        let node = graph_node(
            "node:manuscript/report.smd#art_",
            Node::Article(Article::new(Vec::new())),
        );

        assert_eq!(node_label(&node), "report.smd");
    }

    #[test]
    fn connected_filter_matches_exact_node_id() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["code:scripts/analysis.R".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/analysis.R")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:data/raw.csv")
        );
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/other.R")
        );
        Ok(())
    }

    #[test]
    fn connected_filter_prefers_exact_text_before_substring() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["analysis.R".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/analysis.R")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:scripts/analysis.R")
        );
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:archive/analysis.R.old")
        );
        Ok(())
    }

    #[test]
    fn connected_filter_matches_glob_patterns() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        for pattern in ["*/analysis.R", "**/analysis.R"] {
            let filtered = filter_graph_view_connected_to(
                &view,
                &[pattern.into()],
                GraphConnectedMode::Directed,
            )?;

            assert!(
                filtered
                    .nodes
                    .iter()
                    .any(|node| node.id == "code:scripts/analysis.R")
            );
            assert!(
                !filtered
                    .nodes
                    .iter()
                    .any(|node| node.id == "code:archive/analysis.R.old")
            );
        }
        Ok(())
    }

    #[test]
    fn connected_filter_uses_union_of_best_tier_matches() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["analysis.R".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:scripts/analysis.R")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:r-plot.png")
        );
        Ok(())
    }

    #[test]
    fn connected_filter_does_not_traverse_structure_edges() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["analysis.R".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(filtered.nodes.iter().any(|node| node.id == "dir:scripts"));
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/other.R")
        );
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:scripts/other.R")
        );
        Ok(())
    }

    #[test]
    fn connected_filter_starts_from_contained_descendants() -> Result<()> {
        let view = project_graph(
            &contained_symbol_flow_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["analysis.py".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:analysis.py")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "function:analysis.py:python:summarize")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "symbol:analysis.py:python:summarize:table")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:data/samples.tsv")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:results/python-summary.tsv")
        );
        Ok(())
    }

    #[test]
    fn flow_medium_collapses_contained_symbol_io_to_code() {
        let view = project_graph(
            &contained_symbol_flow_graph(),
            &GraphProjectionOptions::default(),
        );

        assert_eq!(view.preset, GraphProjectionPreset::Flow);
        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.kind == GraphViewNodeKind::Symbol)
        );
        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.id == "function:analysis.py:python:summarize")
        );
        assert_eq!(
            view.edges
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec![
                "ReadBy:file:data/samples.tsv->code:analysis.py",
                "WrittenTo:code:analysis.py->file:results/python-summary.tsv",
            ]
        );
    }

    #[test]
    fn flow_medium_hides_local_workflow_execution_calls() {
        let graph = workflow_execution_call_graph();
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::Medium,
                ..Default::default()
            },
        );

        assert!(
            view.edges
                .iter()
                .all(|edge| edge.kind != GraphEdgeKind::CalledBy)
        );
        assert!(
            !view
                .edges
                .iter()
                .any(|edge| edge.source == "code:main.nf"
                    && edge.target == "workflow-unit:main.nf:qc")
        );
        assert!(
            view.edges
                .iter()
                .any(|edge| edge.source == "file:data/reads.fastq"
                    && edge.target == "workflow-unit:main.nf:qc"
                    && edge.kind == GraphEdgeKind::ReadBy)
        );
        assert!(
            view.edges
                .iter()
                .any(|edge| edge.source == "workflow-unit:main.nf:qc"
                    && edge.target == "file:results/qc/M1-qc.txt"
                    && edge.kind == GraphEdgeKind::Generated)
        );

        let high = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        assert!(high.edges.iter().any(|edge| {
            edge.source == "function:main.nf:nextflow:script"
                && edge.target == "workflow-unit:main.nf:qc"
                && edge.kind == GraphEdgeKind::CalledBy
        }));
    }

    #[test]
    fn flow_medium_hides_redundant_workflow_script_io() {
        let graph = workflow_script_io_graph(false);
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::Medium,
                ..Default::default()
            },
        );

        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.id == "code:workflow/scripts/download.py")
        );
        assert!(
            view.edges
                .iter()
                .all(|edge| edge.kind != GraphEdgeKind::UsedBy)
        );
        assert!(!view.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "file:data/raw/S1.fastq"
                && edge.kind == GraphEdgeKind::Generated
        }));
        assert!(view.edges.iter().any(|edge| {
            edge.source == "workflow-unit:Snakefile:download"
                && edge.target == "file:data/raw/S1.fastq"
                && edge.kind == GraphEdgeKind::Generated
        }));

        let high = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        assert!(high.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "workflow-unit:Snakefile:download"
                && edge.kind == GraphEdgeKind::UsedBy
        }));
        assert!(high.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "file:data/raw/S1.fastq"
                && edge.kind == GraphEdgeKind::Generated
        }));
    }

    #[test]
    fn flow_medium_keeps_workflow_script_fallback_io() {
        let graph = workflow_script_io_graph(true);
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::Medium,
                ..Default::default()
            },
        );

        assert!(
            view.nodes
                .iter()
                .any(|node| node.id == "code:workflow/scripts/download.py")
        );
        assert!(view.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "workflow-unit:Snakefile:download"
                && edge.kind == GraphEdgeKind::UsedBy
        }));
        assert!(!view.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "file:data/raw/S1.fastq"
                && edge.kind == GraphEdgeKind::Generated
        }));
        assert!(view.edges.iter().any(|edge| {
            edge.source == "code:workflow/scripts/download.py"
                && edge.target == "file:logs/download.log"
                && edge.kind == GraphEdgeKind::Generated
        }));
        assert!(view.edges.iter().any(|edge| {
            edge.source == "workflow-unit:Snakefile:download"
                && edge.target == "file:data/raw/S1.fastq"
                && edge.kind == GraphEdgeKind::Generated
        }));
    }

    #[test]
    fn flow_hides_document_conversion_edges() {
        let view = project_graph(
            &converted_document_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        assert!(
            view.edges
                .iter()
                .all(|edge| edge.kind != GraphEdgeKind::ConvertedInto)
        );
        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.id == "node:report.html#art_")
        );
        assert!(
            view.edges
                .iter()
                .any(|edge| edge.source == "code:analysis.py"
                    && edge.target == "file:report.html"
                    && edge.kind == GraphEdgeKind::Generated)
        );

        let all = project_graph(
            &converted_document_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::All,
                ..Default::default()
            },
        );

        assert!(
            all.edges
                .iter()
                .any(|edge| edge.kind == GraphEdgeKind::ConvertedInto)
        );
    }

    #[test]
    fn flow_containment_uses_decoded_document_structure() {
        let view = project_graph(
            &converted_document_containment_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        assert!(
            view.edges
                .iter()
                .all(|edge| edge.kind != GraphEdgeKind::ConvertedInto)
        );
        assert_eq!(
            view.containments
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec![
                "PartOf:dir:docs->dir:.",
                "PartOf:node:docs/notebook.json#art_->dir:docs",
                "PartOf:node:docs/notebook.json#cdc_setup->node:docs/notebook.json#art_",
            ]
        );
    }

    #[test]
    fn flow_low_and_medium_keep_datatable_resources() {
        for detail in [GraphProjectionDetail::Low, GraphProjectionDetail::Medium] {
            let view = project_graph(
                &contained_datatable_symbol_flow_graph(),
                &GraphProjectionOptions {
                    preset: GraphProjectionPreset::Flow,
                    detail,
                    ..Default::default()
                },
            );

            assert!(
                !view
                    .nodes
                    .iter()
                    .any(|node| node.kind == GraphViewNodeKind::Symbol)
            );
            assert_eq!(
                view.edges
                    .iter()
                    .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                    .collect::<Vec<_>>(),
                vec![
                    "ReadBy:datatable:data/samples.tsv->code:analysis.py",
                    "WrittenTo:code:analysis.py->datatable:results/python-summary.tsv",
                ]
            );
        }
    }

    #[test]
    fn connected_filter_does_not_cross_shared_inputs_to_sibling_consumers() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["code:scripts/analysis.R".into()],
            GraphConnectedMode::Directed,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:data/raw.csv")
        );
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/other.R")
        );
        assert!(!filtered.edges.iter().any(
            |edge| edge.source == "file:data/raw.csv" && edge.target == "code:scripts/other.R"
        ));
        Ok(())
    }

    #[test]
    fn connected_filter_undirected_crosses_shared_inputs_to_sibling_consumers() -> Result<()> {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let filtered = filter_graph_view_connected_to(
            &view,
            &["code:scripts/analysis.R".into()],
            GraphConnectedMode::Undirected,
        )?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/analysis.R")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:data/raw.csv")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/other.R")
        );
        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "file:other-plot.png")
        );
        assert!(filtered.edges.iter().any(
            |edge| edge.source == "file:data/raw.csv" && edge.target == "code:scripts/other.R"
        ));
        Ok(())
    }

    #[test]
    fn connected_filter_reports_invalid_and_unmatched_patterns() {
        let view = project_graph(
            &connected_pattern_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let invalid =
            filter_graph_view_connected_to(&view, &["[".into()], GraphConnectedMode::Directed)
                .expect_err("invalid glob should error");
        assert!(
            invalid
                .to_string()
                .contains("invalid connected-to glob pattern")
        );

        let unmatched = filter_graph_view_connected_to(
            &view,
            &["missing.R".into()],
            GraphConnectedMode::Directed,
        )
        .expect_err("unmatched pattern should error");
        assert!(
            unmatched
                .to_string()
                .contains("no projected graph nodes match")
        );
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

    fn connected_pattern_graph() -> Graph {
        Graph::new(
            "test:connected-pattern".to_string(),
            vec![
                graph_node(
                    "dir:.",
                    Node::Directory(Directory::new("workspace".to_string(), ".".to_string())),
                ),
                graph_node(
                    "dir:scripts",
                    Node::Directory(Directory::new("scripts".to_string(), "scripts".to_string())),
                ),
                graph_node(
                    "code:scripts/analysis.R",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.R".to_string(),
                        path: Some("scripts/analysis.R".to_string()),
                        programming_language: "r".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "file:scripts/analysis.R",
                    Node::File(File::new(
                        "analysis.R".to_string(),
                        "scripts/analysis.R".to_string(),
                    )),
                ),
                graph_node(
                    "file:data/raw.csv",
                    Node::File(File::new("raw.csv".to_string(), "data/raw.csv".to_string())),
                ),
                graph_node(
                    "file:r-plot.png",
                    Node::File(File::new(
                        "r-plot.png".to_string(),
                        "r-plot.png".to_string(),
                    )),
                ),
                graph_node(
                    "code:scripts/other.R",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "other.R".to_string(),
                        path: Some("scripts/other.R".to_string()),
                        programming_language: "r".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "file:scripts/other.R",
                    Node::File(File::new(
                        "other.R".to_string(),
                        "scripts/other.R".to_string(),
                    )),
                ),
                graph_node(
                    "file:other-data.csv",
                    Node::File(File::new(
                        "other-data.csv".to_string(),
                        "other-data.csv".to_string(),
                    )),
                ),
                graph_node(
                    "file:other-plot.png",
                    Node::File(File::new(
                        "other-plot.png".to_string(),
                        "other-plot.png".to_string(),
                    )),
                ),
                graph_node(
                    "code:archive/analysis.R.old",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.R.old".to_string(),
                        path: Some("archive/analysis.R.old".to_string()),
                        programming_language: "r".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "file:archive-data.csv",
                    Node::File(File::new(
                        "archive-data.csv".to_string(),
                        "archive-data.csv".to_string(),
                    )),
                ),
                graph_node(
                    "file:archive-plot.png",
                    Node::File(File::new(
                        "archive-plot.png".to_string(),
                        "archive-plot.png".to_string(),
                    )),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data/raw.csv".to_string(),
                    "code:scripts/analysis.R".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:scripts/analysis.R".to_string(),
                    "file:r-plot.png".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "code:scripts/analysis.R".to_string(),
                    "file:scripts/analysis.R".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:scripts/analysis.R".to_string(),
                    "dir:scripts".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "dir:scripts".to_string(),
                    "dir:.".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:data/raw.csv".to_string(),
                    "code:scripts/other.R".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:scripts/other.R".to_string(),
                    "file:other-plot.png".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "code:scripts/other.R".to_string(),
                    "file:scripts/other.R".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:scripts/other.R".to_string(),
                    "dir:scripts".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:archive-data.csv".to_string(),
                    "code:archive/analysis.R.old".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:archive/analysis.R.old".to_string(),
                    "file:archive-plot.png".to_string(),
                    GraphEdgeKind::Generated,
                ),
            ],
        )
    }

    fn contained_symbol_flow_graph() -> Graph {
        Graph::new(
            "test:contained-symbol-flow".to_string(),
            vec![
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "function:analysis.py:python:summarize",
                    Node::Function(Function::new("summarize".to_string(), Vec::new())),
                ),
                graph_node(
                    "symbol:analysis.py:python:summarize:table",
                    Node::Variable(Variable::new("table".to_string())),
                ),
                graph_node(
                    "file:data/samples.tsv",
                    Node::File(File::new(
                        "samples.tsv".to_string(),
                        "data/samples.tsv".to_string(),
                    )),
                ),
                graph_node(
                    "file:results/python-summary.tsv",
                    Node::File(File::new(
                        "python-summary.tsv".to_string(),
                        "results/python-summary.tsv".to_string(),
                    )),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data/samples.tsv".to_string(),
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    "file:results/python-summary.tsv".to_string(),
                    GraphEdgeKind::WrittenTo,
                ),
                GraphEdge::new(
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    "function:analysis.py:python:summarize".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "function:analysis.py:python:summarize".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::PartOf,
                ),
            ],
        )
    }

    fn contained_datatable_symbol_flow_graph() -> Graph {
        Graph::new(
            "test:contained-datatable-symbol-flow".to_string(),
            vec![
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "function:analysis.py:python:summarize",
                    Node::Function(Function::new("summarize".to_string(), Vec::new())),
                ),
                graph_node(
                    "symbol:analysis.py:python:summarize:table",
                    Node::Variable(Variable::new("table".to_string())),
                ),
                graph_node(
                    "datatable:data/samples.tsv",
                    Node::Datatable(Datatable::new(Vec::new())),
                ),
                graph_node(
                    "datatable:results/python-summary.tsv",
                    Node::Datatable(Datatable::new(Vec::new())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "datatable:data/samples.tsv".to_string(),
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    "datatable:results/python-summary.tsv".to_string(),
                    GraphEdgeKind::WrittenTo,
                ),
                GraphEdge::new(
                    "symbol:analysis.py:python:summarize:table".to_string(),
                    "function:analysis.py:python:summarize".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "function:analysis.py:python:summarize".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::PartOf,
                ),
            ],
        )
    }

    fn workflow_execution_call_graph() -> Graph {
        Graph::new(
            "test:workflow-execution-call".to_string(),
            vec![
                graph_node(
                    "code:main.nf",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "main.nf".to_string(),
                        programming_language: "nextflow".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "function:main.nf:nextflow:script",
                    Node::Function(Function::new("script".to_string(), Vec::new())),
                ),
                graph_node(
                    "workflow-unit:main.nf:qc",
                    Node::Function(Function::new("qc".to_string(), Vec::new())),
                ),
                graph_node(
                    "file:data/reads.fastq",
                    Node::File(File::new(
                        "reads.fastq".to_string(),
                        "data/reads.fastq".to_string(),
                    )),
                ),
                graph_node(
                    "file:results/qc/M1-qc.txt",
                    Node::File(File::new(
                        "M1-qc.txt".to_string(),
                        "results/qc/M1-qc.txt".to_string(),
                    )),
                ),
            ],
            vec![
                GraphEdge::new(
                    "code:main.nf".to_string(),
                    "workflow-unit:main.nf:qc".to_string(),
                    GraphEdgeKind::Declares,
                ),
                GraphEdge::new(
                    "function:main.nf:nextflow:script".to_string(),
                    "code:main.nf".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "function:main.nf:nextflow:script".to_string(),
                    "workflow-unit:main.nf:qc".to_string(),
                    GraphEdgeKind::CalledBy,
                ),
                GraphEdge::new(
                    "workflow-unit:main.nf:qc".to_string(),
                    "code:main.nf".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:data/reads.fastq".to_string(),
                    "workflow-unit:main.nf:qc".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "workflow-unit:main.nf:qc".to_string(),
                    "file:results/qc/M1-qc.txt".to_string(),
                    GraphEdgeKind::Generated,
                ),
            ],
        )
    }

    fn workflow_script_io_graph(include_extra_output: bool) -> Graph {
        let mut nodes = vec![
            graph_node(
                "code:Snakefile",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "Snakefile".to_string(),
                    programming_language: "snakemake".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "workflow-unit:Snakefile:download",
                Node::Function(Function::new("download".to_string(), Vec::new())),
            ),
            graph_node(
                "code:workflow/scripts/download.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "download.py".to_string(),
                    path: Some("workflow/scripts/download.py".to_string()),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:data/raw/S1.fastq",
                Node::File(File::new(
                    "S1.fastq".to_string(),
                    "data/raw/S1.fastq".to_string(),
                )),
            ),
        ];
        let mut edges = vec![
            GraphEdge::new(
                "code:Snakefile".to_string(),
                "workflow-unit:Snakefile:download".to_string(),
                GraphEdgeKind::Declares,
            ),
            GraphEdge::new(
                "workflow-unit:Snakefile:download".to_string(),
                "code:Snakefile".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "code:workflow/scripts/download.py".to_string(),
                "workflow-unit:Snakefile:download".to_string(),
                GraphEdgeKind::UsedBy,
            ),
            GraphEdge::new(
                "code:workflow/scripts/download.py".to_string(),
                "file:data/raw/S1.fastq".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "workflow-unit:Snakefile:download".to_string(),
                "file:data/raw/S1.fastq".to_string(),
                GraphEdgeKind::Generated,
            ),
        ];

        if include_extra_output {
            nodes.push(graph_node(
                "file:logs/download.log",
                Node::File(File::new(
                    "download.log".to_string(),
                    "logs/download.log".to_string(),
                )),
            ));
            edges.push(GraphEdge::new(
                "code:workflow/scripts/download.py".to_string(),
                "file:logs/download.log".to_string(),
                GraphEdgeKind::Generated,
            ));
        }

        Graph::new("test:workflow-script-io".to_string(), nodes, edges)
    }

    fn converted_document_graph() -> Graph {
        Graph::new(
            "test:converted-document".to_string(),
            vec![
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "file:report.html",
                    Node::File(File::new(
                        "report.html".to_string(),
                        "report.html".to_string(),
                    )),
                ),
                graph_node(
                    "node:report.html#art_",
                    Node::Article(Article::new(Vec::new())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "code:analysis.py".to_string(),
                    "file:report.html".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "file:report.html".to_string(),
                    "node:report.html#art_".to_string(),
                    GraphEdgeKind::ConvertedInto,
                ),
            ],
        )
    }

    fn converted_document_containment_graph() -> Graph {
        Graph::new(
            "test:converted-document-containment".to_string(),
            vec![
                graph_node(
                    "dir:.",
                    Node::Directory(Directory::new("workspace".to_string(), ".".to_string())),
                ),
                graph_node(
                    "dir:docs",
                    Node::Directory(Directory::new("docs".to_string(), "docs".to_string())),
                ),
                graph_node(
                    "file:docs/notebook.json",
                    Node::File(File::new(
                        "notebook.json".to_string(),
                        "docs/notebook.json".to_string(),
                    )),
                ),
                graph_node(
                    "node:docs/notebook.json#art_",
                    Node::Article(Article::new(Vec::new())),
                ),
                graph_node(
                    "node:docs/notebook.json#cdc_setup",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "setup".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "dir:docs".to_string(),
                    "dir:.".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:docs/notebook.json".to_string(),
                    "dir:docs".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "node:docs/notebook.json#art_".to_string(),
                    "dir:docs".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "node:docs/notebook.json#cdc_setup".to_string(),
                    "node:docs/notebook.json#art_".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "node:docs/notebook.json#cdc_setup".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
            ],
        )
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

    fn environment_graph() -> Graph {
        Graph::new(
            "test:environment".to_string(),
            vec![
                graph_node(
                    "file:pyproject.toml",
                    Node::File(File::new(
                        "pyproject.toml".to_string(),
                        "pyproject.toml".to_string(),
                    )),
                ),
                graph_node(
                    "file:uv.lock",
                    Node::File(File::new("uv.lock".to_string(), "uv.lock".to_string())),
                ),
                graph_node(
                    "environment:python:pyproject.toml",
                    Node::SoftwareApplication(SoftwareApplication::new(
                        "Python environment declared by pyproject.toml".to_string(),
                    )),
                ),
                graph_node(
                    "package:pypi/pandas",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "pandas".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:pyproject.toml".to_string(),
                    "environment:python:pyproject.toml".to_string(),
                    GraphEdgeKind::Declares,
                ),
                GraphEdge::new(
                    "file:uv.lock".to_string(),
                    "environment:python:pyproject.toml".to_string(),
                    GraphEdgeKind::Pins,
                ),
                GraphEdge::new(
                    "package:pypi/pandas".to_string(),
                    "environment:python:pyproject.toml".to_string(),
                    GraphEdgeKind::RequiredBy,
                ),
            ],
        )
    }

    fn detail_graph() -> Graph {
        Graph::new(
            "test:detail".to_string(),
            vec![
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
                graph_node(
                    "file:plot.png",
                    Node::File(File::new("plot.png".to_string(), "plot.png".to_string())),
                ),
                graph_node(
                    "code:analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
                        programming_language: "python".to_string(),
                        ..Default::default()
                    }),
                ),
                graph_node(
                    "symbol:analysis.py:python:df",
                    Node::Variable(Variable::new("df".to_string())),
                ),
                graph_node(
                    "function:analysis.py:python:read_csv",
                    Node::Function(Function::new("read_csv".to_string(), Vec::new())),
                ),
                graph_node(
                    "column:analysis.py:data.csv:count",
                    Node::DatatableColumn(DatatableColumn::new("count".to_string(), Vec::new())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:analysis.py".to_string(),
                    "file:plot.png".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "code:analysis.py".to_string(),
                    "symbol:analysis.py:python:df".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "symbol:analysis.py:python:df".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::UsedBy,
                ),
                GraphEdge::new(
                    "function:analysis.py:python:read_csv".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::CalledBy,
                ),
                GraphEdge::new(
                    "column:analysis.py:data.csv:count".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::UsedBy,
                ),
                GraphEdge::new(
                    "column:analysis.py:data.csv:count".to_string(),
                    "file:plot.png".to_string(),
                    GraphEdgeKind::DerivedInto,
                ),
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
