//! Document graph extraction.
//!
//! This module turns a single Stencila Schema document node tree into graph
//! nodes and resource-flow relationships.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use eyre::Result;
use stencila_node_stabilize::stabilize;
use stencila_schema::{
    ActionStatusType, Block, Citation, CitationGroup, CodeChunk, CodeExpression, CreativeWork,
    DateTime as SchemaDateTime, Duration as SchemaDuration, ExecuteAction, ExecutionStatus, Graph,
    GraphAction, GraphEdgeKind, GraphEvidence, Inline, Link, Node, NodeId, NodeType, Object,
    Primitive, Reference, ResearchObjectRelationKind, Timestamp, Visitor, WalkControl, WalkNode,
    research_relation_targets,
};

use crate::{
    GraphAnalysis, GraphBuilder, GraphDiagnosticKind,
    code::{CodeLanguage, DocumentCodeIndex, DocumentCodeSource},
    evidence,
    ids::LocalGraphId,
    reference::{bare_doi, has_non_local_uri_scheme},
    runtime::{RuntimeEvidenceMode, add_cached_runtime_evidence},
    source,
};

/// Build a graph for a single Stencila Schema node tree.
///
/// This is the simplest document entry point for callers that do not already
/// have a workspace graph builder to extend.
///
/// Document graphs promote the root node plus coarse boundary nodes such as
/// figures, tables, files, and executable code. Inline citations, links, media
/// references, includes, and headings contribute relationships to retained
/// containers without becoming graph nodes themselves.
pub fn graph_from_node(subject: impl Into<String>, node: &Node) -> Result<Graph> {
    Ok(graph_from_node_with_diagnostics(subject, node)?.graph)
}

/// Build a document graph and the static analysis diagnostics behind it.
///
/// This is the same analysis as [`graph_from_node`], but it also returns the
/// I/O operations in the document's code that the analyzer could not resolve.
pub fn graph_from_node_with_diagnostics(
    subject: impl Into<String>,
    node: &Node,
) -> Result<GraphAnalysis> {
    let mut builder = GraphBuilder::new(subject);
    add_document(&mut builder, "document", node, None);
    let diagnostics = builder.take_diagnostics();
    let graph_diagnostics = builder.take_graph_diagnostics();
    let mut graph = builder.build()?;
    source::set_graph_source_metadata_from_node(&mut graph, node);
    Ok(GraphAnalysis {
        graph,
        diagnostics,
        graph_diagnostics,
    })
}

/// Build a document graph and optionally merge cached runtime evidence.
///
/// The default document APIs remain execution- and cache-independent; callers
/// must explicitly select [`RuntimeEvidenceMode::Cached`].
pub fn graph_from_node_with_runtime_evidence(
    subject: impl Into<String>,
    scope: impl Into<String>,
    node: &Node,
    root: &Path,
    runtime_evidence: RuntimeEvidenceMode,
) -> Result<GraphAnalysis> {
    let mut builder = GraphBuilder::new(subject);
    add_document(&mut builder, scope, node, None);
    if runtime_evidence == RuntimeEvidenceMode::Cached {
        add_cached_runtime_evidence(&mut builder, root);
    }
    let diagnostics = builder.take_diagnostics();
    let graph_diagnostics = builder.take_graph_diagnostics();
    let mut graph = builder.build()?;
    source::set_graph_source_metadata_from_node(&mut graph, node);
    Ok(GraphAnalysis {
        graph,
        diagnostics,
        graph_diagnostics,
    })
}

/// Add document nodes and relationships to an existing graph builder.
///
/// The node tree is cloned and stabilized before graph IDs are generated so
/// repeated calls for equivalent input produce the same graph.
///
/// The optional source file id is retained for API compatibility. Workspace
/// graph construction records decoded document placement as structural
/// containment rather than as a conversion edge.
///
/// Call [`GraphBuilder::build`] after adding all documents to validate that all
/// emitted edges refer to graph nodes.
pub fn add_document(
    builder: &mut GraphBuilder,
    scope: impl Into<String>,
    node: &Node,
    _source_file_id: Option<&str>,
) -> String {
    add_document_with_reference_resolver(builder, scope, node, None)
}

/// Add document nodes with optional reference resolution.
///
/// Workspace graph construction uses this internal variant to resolve
/// document-local references against workspace files.
pub(crate) fn add_document_with_reference_resolver<'a>(
    builder: &'a mut GraphBuilder,
    scope: impl Into<String>,
    node: &Node,
    reference_resolver: Option<&'a mut DocumentReferenceResolver<'a>>,
) -> String {
    let scope = scope.into();
    let mut node = node.clone();
    stabilize(&mut node);
    let root_id =
        graph_id_for_node(&scope, &node).unwrap_or_else(|| LocalGraphId::document_root(&scope));

    let mut collector = DocumentCollector::new(builder, scope, reference_resolver);
    collector.add_schema_node(node.clone(), Some(root_id.clone()), true, false);
    node.walk(&mut collector);
    collector.finish();
    root_id
}

/// A document-local file reference discovered during graph collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentReferenceKind {
    /// A media object references external bytes through `contentUrl`.
    Media,

    /// An include block transcludes an external source.
    Include,

    /// A link points at an external source or resource.
    Link,
}

/// How a document node type participates in graph extraction and projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentGraphNodePolicy {
    /// Do not retain the node as a graph node.
    Skip,

    /// Retain the node as a graph node when it is discovered.
    Record,

    /// Retain the node and show it unconditionally in flow projections.
    RecordAndSeedFlow,
}

impl DocumentGraphNodePolicy {
    fn records(self) -> bool {
        !matches!(self, Self::Skip)
    }

    pub(crate) fn seeds_flow_projection(self) -> bool {
        matches!(self, Self::RecordAndSeedFlow)
    }
}

/// Return the graph policy for a document node type.
pub(crate) fn document_graph_node_policy(node_type: NodeType) -> DocumentGraphNodePolicy {
    match node_type {
        NodeType::CodeChunk | NodeType::Datatable | NodeType::Figure | NodeType::Table => {
            DocumentGraphNodePolicy::RecordAndSeedFlow
        }
        NodeType::Claim
        | NodeType::Evidence
        | NodeType::Protocol
        | NodeType::Question
        | NodeType::Request => DocumentGraphNodePolicy::Record,
        NodeType::Article
        | NodeType::CodeExpression
        | NodeType::File
        | NodeType::Reference
        | NodeType::SymbolicLink => DocumentGraphNodePolicy::Record,
        _ => DocumentGraphNodePolicy::Skip,
    }
}

/// Check whether an embedded document node should seed flow projections.
pub(crate) fn document_node_seeds_flow_projection(node: &Node) -> bool {
    document_graph_node_policy(node.node_type()).seeds_flow_projection()
}

/// Callback used to resolve document-local file references into graph edges.
type DocumentReferenceResolver<'a> = dyn for<'reference> FnMut(DocumentReferenceKind, &'reference str) -> Option<(String, GraphEdgeKind)>
    + 'a;

/// Start and end times for an action.
///
/// This small wrapper keeps action timing optional at the boundary where file
/// decoding and document graph construction meet.
#[derive(Debug, Clone)]
pub(crate) struct ActionTimes {
    /// Time the action started.
    ///
    /// A missing start time still allows callers to record a known end time when
    /// only completion data is available.
    pub start_time: Option<SchemaDateTime>,

    /// Time the action ended.
    ///
    /// End times are used by provenance consumers to understand when decoding or
    /// execution produced a node even when the duration is unavailable.
    pub end_time: Option<SchemaDateTime>,
}

impl ActionTimes {
    /// Create action times from a start and end.
    ///
    /// This constructor captures complete timing information for actions whose
    /// duration was measured directly.
    pub(crate) fn range(start_time: SchemaDateTime, end_time: SchemaDateTime) -> Self {
        Self {
            start_time: Some(start_time),
            end_time: Some(end_time),
        }
    }

    /// Create action times from an end time only.
    ///
    /// This constructor preserves useful completion metadata when upstream data
    /// does not contain enough information to derive a start time.
    pub(crate) fn end(end_time: SchemaDateTime) -> Self {
        Self {
            start_time: None,
            end_time: Some(end_time),
        }
    }
}

/// Visitor that collects graph boundary nodes.
///
/// The collector tracks the current structural parent while walking the Schema
/// tree so it can create `PartOf` edges without embedding entire subtrees.
struct DocumentCollector<'a> {
    /// Graph builder being extended.
    ///
    /// The collector borrows the builder so document traversal can append nodes
    /// and edges without allocating an intermediate graph representation.
    builder: &'a mut GraphBuilder,

    /// Graph id scope for this document.
    ///
    /// Scoping prevents equivalent node ids from colliding when multiple
    /// documents are added to the same workspace graph.
    scope: String,

    /// Boundary node ids discovered so far.
    ///
    /// The walk API enters structs after enum variants, so this map links Schema
    /// node ids back to graph ids when structural nesting is entered.
    boundaries: HashMap<NodeId, String>,

    /// Stack of boundary ids for entered structs.
    ///
    /// This stack mirrors every entered struct so exits can determine whether a
    /// graph parent was pushed for that struct.
    struct_stack: Vec<Option<String>>,

    /// Stack of current structural graph parents.
    ///
    /// Only graph boundary nodes are included, which makes `PartOf` edges jump
    /// over non-boundary wrapper nodes.
    parent_stack: Vec<String>,

    /// Optional resolver for file references discovered in document-local fields.
    ///
    /// Workspace graph construction uses this to resolve paths relative to the
    /// containing document file while keeping workspace path knowledge out of the
    /// document collector.
    reference_resolver: Option<&'a mut DocumentReferenceResolver<'a>>,

    /// Static code facts discovered in executable document nodes.
    code_index: DocumentCodeIndex,

    /// Mapping from authored research-object ids to graph node ids.
    declared_ids: HashMap<String, String>,

    /// Ids declared by more than one research object are ambiguous targets.
    duplicate_declared_ids: HashSet<String>,

    /// Authored relations are resolved after the complete document is known.
    pending_authored_relations: Vec<AuthoredRelation>,
}

#[derive(Debug, Clone)]
struct AuthoredRelation {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}

impl<'a> DocumentCollector<'a> {
    /// Create a collector for a graph builder and scope.
    ///
    /// Each document traversal gets fresh stacks so parent tracking remains
    /// local to that document even when many documents share a builder.
    fn new(
        builder: &'a mut GraphBuilder,
        scope: String,
        reference_resolver: Option<&'a mut DocumentReferenceResolver<'a>>,
    ) -> Self {
        Self {
            builder,
            scope,
            boundaries: HashMap::new(),
            struct_stack: Vec::new(),
            parent_stack: Vec::new(),
            reference_resolver,
            code_index: DocumentCodeIndex::default(),
            declared_ids: HashMap::new(),
            duplicate_declared_ids: HashSet::new(),
            pending_authored_relations: Vec::new(),
        }
    }

    /// Add a Schema node when it should appear in the graph.
    ///
    /// Policy filtering keeps the graph compact while `force` lets roots and
    /// execution outputs be represented even when they are not recorded types.
    fn add_schema_node(
        &mut self,
        node: Node,
        fallback_id: Option<String>,
        force: bool,
        structural: bool,
    ) -> Option<String> {
        let node_type = node.node_type();
        if !force && !document_graph_node_policy(node_type).records() {
            return None;
        }

        let node_id = node.node_id();
        let graph_id = node_id
            .as_ref()
            .map(|node_id| graph_id_for_node_id(&self.scope, node_id))
            .or(fallback_id)?;

        if let Some(node_id) = &node_id {
            self.boundaries.insert(node_id.clone(), graph_id.clone());
        }

        self.builder.add_schema_node(graph_id.clone(), node.clone());

        if let Some(id) = node.as_research_object().and_then(|research| research.id()) {
            if self.duplicate_declared_ids.contains(id) {
                // The first duplicate already produced the actionable diagnostic.
            } else if self
                .declared_ids
                .insert(id.to_string(), graph_id.clone())
                .is_some()
            {
                self.declared_ids.remove(id);
                self.duplicate_declared_ids.insert(id.to_string());
                self.builder.add_graph_diagnostic(crate::GraphDiagnostic {
                    kind: crate::GraphDiagnosticKind::DuplicateResearchObjectId,
                    source: graph_id.clone(),
                    target: None,
                    message: format!(
                        "research object id `{id}` is declared more than once; relations to it are ambiguous"
                    ),
                });
            }
        }

        if structural && let Some(parent_id) = self.parent_stack.last() {
            self.builder
                .add_containment(graph_id.clone(), parent_id, vec![evidence::computed()]);
        }

        self.add_document_reference(&graph_id, &node);
        self.add_authored_relations(&graph_id, &node);

        match node {
            Node::CodeChunk(chunk) => {
                if let Some(node_id) = node_id {
                    self.add_code_chunk_execution(&graph_id, &node_id, &chunk);
                    self.add_static_code_facts(
                        &graph_id,
                        chunk.node_type(),
                        &node_id,
                        chunk.code.to_string().as_str(),
                        chunk.programming_language.as_deref(),
                    );
                }
            }
            Node::CodeExpression(expression) => {
                if let Some(node_id) = node_id {
                    self.add_code_expression_execution(&graph_id, &node_id, &expression);
                    self.add_static_code_facts(
                        &graph_id,
                        expression.node_type(),
                        &node_id,
                        expression.code.to_string().as_str(),
                        expression.programming_language.as_deref(),
                    );
                }
            }
            Node::Citation(citation) => {
                self.add_citation_reference(&graph_id, &citation);
            }
            Node::Link(link) => {
                self.add_link_reference(&graph_id, &link);
            }
            _ => {}
        }

        Some(graph_id)
    }

    /// Add relationships represented by a non-boundary document syntax node.
    ///
    /// Inline links, citations, media references, and includes are useful graph
    /// facts, but the syntax occurrence itself usually is not. Attach those
    /// facts to the nearest retained document container instead.
    fn add_non_boundary_relationships(&mut self, node: &Node) {
        let Some(target_id) = self.parent_stack.last().cloned() else {
            return;
        };

        match node {
            Node::Citation(citation) => self.add_citation_reference(&target_id, citation),
            Node::CitationGroup(group) => self.add_citation_group_references(&target_id, group),
            Node::Link(link) => self.add_link_reference(&target_id, link),
            Node::AudioObject(_)
            | Node::ImageObject(_)
            | Node::MediaObject(_)
            | Node::VideoObject(_)
            | Node::IncludeBlock(_) => {
                self.add_document_reference(&target_id, node);
            }
            _ => {}
        }
    }

    /// Add a walked node, preserving relationships even when it is not retained.
    fn add_walked_node(&mut self, node: Node) {
        if self
            .add_schema_node(node.clone(), None, false, true)
            .is_none()
        {
            self.add_non_boundary_relationships(&node);
        }
    }

    /// Add static code graph facts for a supported executable document node.
    fn add_static_code_facts(
        &mut self,
        graph_id: &str,
        node_type: NodeType,
        node_id: &NodeId,
        code: &str,
        programming_language: Option<&str>,
    ) {
        let Some(language) = programming_language.and_then(CodeLanguage::from_programming_language)
        else {
            return;
        };

        if let Some(reference_resolver) = self.reference_resolver.as_deref_mut() {
            let mut resolver = |literal: &str| {
                reference_resolver(DocumentReferenceKind::Link, literal)
                    .map(|(file_id, _edge_kind)| file_id)
            };
            self.code_index.add_unit(
                self.builder,
                &self.scope,
                DocumentCodeSource {
                    unit_id: graph_id,
                    node_type,
                    node_id,
                    code,
                    language,
                },
                Some(&mut resolver),
            );
        } else {
            self.code_index.add_unit(
                self.builder,
                &self.scope,
                DocumentCodeSource {
                    unit_id: graph_id,
                    node_type,
                    node_id,
                    code,
                    language,
                },
                None,
            );
        }
    }

    /// Finish relationships that require the complete document-order code view.
    fn finish(&mut self) {
        self.add_pending_authored_relations();
        self.code_index.finish(self.builder, &self.scope);
    }

    /// Collect structured relations and legacy relation-shaped extra fields.
    fn add_authored_relations(&mut self, source: &str, node: &Node) {
        let Some(research) = node.as_research_object() else {
            return;
        };

        if let Some(relations) = research.relations() {
            for relation in relations {
                self.pending_authored_relations.push(AuthoredRelation {
                    source: source.to_string(),
                    target: relation.target.clone(),
                    kind: relation.kind.into(),
                });
            }
        }

        if let Some(extra) = research.extra() {
            for (key, value) in extra.iter() {
                let Some(kind) = ResearchObjectRelationKind::from_authored_key(key) else {
                    continue;
                };
                for target in research_relation_targets(value) {
                    self.pending_authored_relations.push(AuthoredRelation {
                        source: source.to_string(),
                        target,
                        kind: kind.into(),
                    });
                }
            }
        }
    }

    /// Resolve all relations after every research object has been discovered.
    fn add_pending_authored_relations(&mut self) {
        let relations = std::mem::take(&mut self.pending_authored_relations);
        let mut seen = HashSet::new();

        for relation in relations {
            let target = relation.target.trim();
            if target.is_empty() {
                self.add_relation_diagnostic(
                    GraphDiagnosticKind::EmptyResearchRelationTarget,
                    &relation,
                    "research relation has an empty target; add an object id or absolute URI",
                );
                continue;
            }

            let duplicate_key = (
                relation.source.clone(),
                relation.kind,
                target.trim_start_matches('#').to_string(),
            );
            if !seen.insert(duplicate_key) {
                self.add_relation_diagnostic(
                    GraphDiagnosticKind::DuplicateResearchRelationTarget,
                    &relation,
                    "relation target is repeated for the same source and relation kind",
                );
                continue;
            }

            let Some(resolved) = self.resolve_authored_relation_target(target) else {
                self.add_relation_diagnostic(
                    GraphDiagnosticKind::UnresolvedResearchRelationTarget,
                    &relation,
                    &format!(
                        "unable to resolve research relation target `{target}`; use a declared id, graph node id, or absolute URI"
                    ),
                );
                continue;
            };

            if resolved == relation.source {
                self.add_relation_diagnostic(
                    GraphDiagnosticKind::SelfReferentialResearchRelation,
                    &relation,
                    "research relation points to its own source; choose a different target",
                );
                continue;
            }

            let mut evidence = vec![evidence::declared()];
            if resolved != target {
                evidence.push(resolved_declaration_evidence(target, &resolved));
            }
            self.builder
                .add_edge_with_evidence(relation.source, resolved, relation.kind, evidence);
        }
    }

    fn resolve_authored_relation_target(&mut self, target: &str) -> Option<String> {
        let authored_id = target.strip_prefix('#').unwrap_or(target);
        if let Some(graph_id) = self.declared_ids.get(authored_id) {
            return Some(graph_id.clone());
        }
        if self.builder.contains_node(target) {
            return Some(target.to_string());
        }
        if has_non_local_uri_scheme(target) {
            let resource_id = LocalGraphId::resource(target);
            let mut resource = CreativeWork::new();
            resource.options.url = Some(target.to_string());
            self.builder
                .add_schema_node(resource_id.clone(), Node::CreativeWork(resource));
            return Some(resource_id);
        }
        None
    }

    fn add_relation_diagnostic(
        &mut self,
        kind: crate::GraphDiagnosticKind,
        relation: &AuthoredRelation,
        detail: &str,
    ) {
        self.builder.add_graph_diagnostic(crate::GraphDiagnostic {
            kind,
            source: relation.source.clone(),
            target: Some(relation.target.clone()),
            message: format!("{}: {detail}", relation.source),
        });
    }

    /// Add a dependency edge for a local file reference if the caller can resolve it.
    fn add_document_reference(&mut self, graph_id: &str, node: &Node) -> bool {
        let Some((kind, reference)) = document_reference(node) else {
            return false;
        };
        let Some(reference_resolver) = &mut self.reference_resolver else {
            return false;
        };
        let Some((file_id, edge_kind)) = reference_resolver(kind, reference) else {
            return false;
        };
        let evidence = vec![resolved_declaration_evidence(reference, &file_id)];

        match edge_kind {
            GraphEdgeKind::IncludedBy => {
                self.builder.add_include(file_id, graph_id, evidence);
            }
            GraphEdgeKind::LinkedBy => {
                self.builder.add_link(file_id, graph_id, evidence);
            }
            _ => self
                .builder
                .add_edge_with_evidence(file_id, graph_id, edge_kind, evidence),
        }
        true
    }

    /// Add citation provenance from a cited reference to a document target.
    fn add_citation_reference(&mut self, target_id: &str, citation: &Citation) {
        let target = citation.target.trim();
        if target.is_empty() {
            return;
        }

        let mut reference = citation
            .options
            .cites
            .clone()
            .unwrap_or_else(|| reference_from_citation_target(target));
        let reference_id = LocalGraphId::reference(&self.scope, target);
        if reference.id.is_none() {
            reference.id = Some(target.to_string());
        }

        self.builder
            .add_schema_node(reference_id.clone(), Node::Reference(reference));
        self.builder.add_citation(
            &reference_id,
            target_id,
            declared_citation_evidence(citation, &reference_id),
        );
    }

    /// Add citation provenance for every citation in a grouped citation marker.
    fn add_citation_group_references(&mut self, target_id: &str, group: &CitationGroup) {
        for citation in &group.items {
            self.add_citation_reference(target_id, citation);
        }
    }

    /// Add link provenance from the linked target to a document target.
    fn add_link_reference(&mut self, target_id: &str, link: &Link) {
        let target = link.target.trim();
        if target.is_empty() {
            return;
        }

        if target.starts_with('#') {
            return;
        }

        if self.add_document_reference(target_id, &Node::Link(link.clone())) {
            return;
        }

        if has_non_local_uri_scheme(target) {
            let resource_id = LocalGraphId::resource(target);
            let mut resource = CreativeWork::new();
            resource.options.url = Some(target.to_string());

            self.builder
                .add_schema_node(resource_id.clone(), Node::CreativeWork(resource));
            self.builder
                .add_link(resource_id, target_id, vec![evidence::declared()]);
        }
    }

    /// Add execution provenance for a code chunk.
    ///
    /// Execution actions are attached only to generated output edges. A
    /// status-only execution remains source-node metadata rather than becoming
    /// a graph action without a resource-flow edge.
    fn add_code_chunk_execution(&mut self, code_id: &str, node_id: &NodeId, chunk: &CodeChunk) {
        let outputs = chunk.outputs.as_deref().unwrap_or_default();
        if outputs.is_empty() {
            return;
        }

        self.add_execute_action(
            code_id,
            node_id,
            chunk.options.execution_status,
            chunk.options.execution_ended.as_ref(),
            chunk.options.execution_duration.as_ref(),
            outputs.iter().enumerate(),
        );
    }

    /// Add execution provenance for a code expression.
    ///
    /// Code expressions have at most one output, so this adapts the output into
    /// the same indexed iterator shape used by code chunks.
    fn add_code_expression_execution(
        &mut self,
        expression_id: &str,
        node_id: &NodeId,
        expression: &CodeExpression,
    ) {
        let output = expression
            .output
            .as_deref()
            .map(|node| std::iter::once((0, node)));
        if output.is_none() {
            return;
        }

        self.add_execute_action(
            expression_id,
            node_id,
            expression.options.execution_status,
            expression.options.execution_ended.as_ref(),
            expression.options.execution_duration.as_ref(),
            output.into_iter().flatten(),
        );
    }

    /// Add generated outputs and attach execution action metadata to the edges.
    ///
    /// Execution actions describe the concrete recorded operation that generated
    /// each output. They live on the resource-flow edge rather than as
    /// intermediary graph nodes.
    fn add_execute_action<'b>(
        &mut self,
        executable_id: &str,
        node_id: &NodeId,
        status: Option<ExecutionStatus>,
        execution_ended: Option<&Timestamp>,
        execution_duration: Option<&SchemaDuration>,
        outputs: impl Iterator<Item = (usize, &'b Node)>,
    ) {
        let action_id = LocalGraphId::execute_action(&self.scope, node_id);
        let mut action = ExecuteAction::new();
        action.id = Some(action_id.clone());
        action.action_status = action_status_from_execution(status);
        if let Some(action_times) = action_times_from_execution(execution_ended, execution_duration)
        {
            action.start_time = action_times.start_time;
            action.end_time = action_times.end_time;
        }

        for (index, output) in outputs {
            let fallback_id = LocalGraphId::output(&self.scope, node_id, index);
            if let Some(output_id) =
                self.add_schema_node(output.clone(), Some(fallback_id), true, false)
            {
                self.builder.add_containment(
                    output_id.clone(),
                    executable_id,
                    vec![evidence::recorded()],
                );
                self.builder.add_edge_with_evidence_and_actions(
                    executable_id,
                    output_id,
                    GraphEdgeKind::Generated,
                    vec![evidence::recorded()],
                    vec![GraphAction::ExecuteAction(action.clone())],
                );
            }
        }
    }
}

impl Visitor for DocumentCollector<'_> {
    /// Enter a Schema struct during traversal.
    ///
    /// Boundary nodes become potential structural parents only after the walk
    /// enters their underlying struct, matching the generated traversal order.
    fn enter_struct(&mut self, _node_type: NodeType, node_id: NodeId) -> WalkControl {
        let graph_id = self.boundaries.get(&node_id).cloned();
        if let Some(graph_id) = &graph_id {
            self.parent_stack.push(graph_id.clone());
        }
        self.struct_stack.push(graph_id);

        WalkControl::Continue
    }

    /// Exit a Schema struct during traversal.
    ///
    /// The collector pops a graph parent only for structs that pushed one, so
    /// non-boundary structs do not disturb parent tracking.
    fn exit_struct(&mut self) {
        if let Some(Some(..)) = self.struct_stack.pop() {
            self.parent_stack.pop();
        }
    }

    /// Visit a Schema node enum value.
    ///
    /// This catches boundary nodes that are exposed as `Node` values rather than
    /// only through block or inline enum variants.
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        self.add_walked_node(node.clone());
        WalkControl::Continue
    }

    /// Visit a block enum value.
    ///
    /// Blocks are common graph boundaries in documents, so this hook gives the
    /// collector access before the walk enters the block's struct fields.
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        self.add_walked_node(block.clone().into());
        WalkControl::Continue
    }

    /// Visit an inline enum value.
    ///
    /// Inline media and executable expressions can be graph boundaries, so this
    /// hook records them without promoting every inline text node.
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        self.add_walked_node(inline.clone().into());
        WalkControl::Continue
    }
}

/// Create a graph id for a node if it has a Schema node id.
///
/// Primitive Schema values do not carry node ids, so callers use this helper
/// when a missing id should mean no graph boundary node.
fn graph_id_for_node(scope: &str, node: &Node) -> Option<String> {
    node.node_id()
        .map(|node_id| graph_id_for_node_id(scope, &node_id))
}

/// Create a scoped graph id for a Schema node id.
///
/// Including the document scope keeps stable node ids unique across files while
/// retaining the original node id for traceability.
fn graph_id_for_node_id(scope: &str, node_id: &NodeId) -> String {
    LocalGraphId::document_node(scope, node_id)
}

/// Return a file reference represented by a document node.
fn document_reference(node: &Node) -> Option<(DocumentReferenceKind, &str)> {
    match node {
        Node::AudioObject(node) => Some((DocumentReferenceKind::Media, &node.content_url)),
        Node::ImageObject(node) => Some((DocumentReferenceKind::Media, &node.content_url)),
        Node::Link(node) => Some((DocumentReferenceKind::Link, &node.target)),
        Node::MediaObject(node) => Some((DocumentReferenceKind::Media, &node.content_url)),
        Node::VideoObject(node) => Some((DocumentReferenceKind::Media, &node.content_url)),
        Node::IncludeBlock(node) => Some((DocumentReferenceKind::Include, &node.source)),
        _ => None,
    }
}

/// Create a lightweight reference node for an unresolved citation target.
fn reference_from_citation_target(target: &str) -> Reference {
    let mut reference = Reference::new();
    reference.id = Some(target.to_string());

    if let Some(doi) = bare_doi(target) {
        reference.doi = Some(doi.to_string());
    } else if has_non_local_uri_scheme(target) {
        reference.url = Some(target.to_string());
    }

    reference
}

/// Evidence for a citation edge.
fn declared_citation_evidence(citation: &Citation, resolved_to: &str) -> Vec<GraphEvidence> {
    let mut evidence = resolved_declaration_evidence(&citation.target, resolved_to);
    if citation.options.cites.is_some()
        && let Some(details) = evidence.options.details.as_mut()
    {
        details.insert(
            "resolutionKind".to_string(),
            Primitive::String("embeddedReference".to_string()),
        );
    }
    vec![evidence]
}

/// Evidence for an authored locator enriched by successful resolution.
fn resolved_declaration_evidence(locator: &str, resolved_to: &str) -> GraphEvidence {
    let mut evidence = evidence::declared();
    evidence.options.details = Some(Object::from([
        (
            "detector",
            Primitive::String("stencila-document-reference".to_string()),
        ),
        ("locator", Primitive::String(locator.to_string())),
        ("resolvedTo", Primitive::String(resolved_to.to_string())),
    ]));
    evidence
}

/// Convert execution status into Schema action status.
///
/// Execution statuses are domain-specific, while action statuses are the common
/// provenance vocabulary used by generated graph action nodes.
fn action_status_from_execution(status: Option<ExecutionStatus>) -> Option<ActionStatusType> {
    status.map(|status| match status {
        ExecutionStatus::Scheduled | ExecutionStatus::Pending => {
            ActionStatusType::PotentialActionStatus
        }
        ExecutionStatus::Running => ActionStatusType::ActiveActionStatus,
        ExecutionStatus::Errors
        | ExecutionStatus::Exceptions
        | ExecutionStatus::Cancelled
        | ExecutionStatus::Interrupted => ActionStatusType::FailedActionStatus,
        ExecutionStatus::Skipped
        | ExecutionStatus::Locked
        | ExecutionStatus::Rejected
        | ExecutionStatus::Empty
        | ExecutionStatus::Succeeded
        | ExecutionStatus::Warnings => ActionStatusType::CompletedActionStatus,
    })
}

/// Derive action times from execution metadata.
///
/// Code nodes store an end timestamp and optional duration, so this helper
/// translates that pair into action start and end fields when possible.
fn action_times_from_execution(
    execution_ended: Option<&Timestamp>,
    execution_duration: Option<&SchemaDuration>,
) -> Option<ActionTimes> {
    let execution_ended = execution_ended?;
    let end_time = execution_ended.to_date_time().ok()?;

    Some(
        if let Some(start_time) = execution_duration
            .and_then(|duration| execution_ended.to_date_time_before(duration).ok())
        {
            ActionTimes::range(start_time, end_time)
        } else {
            ActionTimes::end(end_time)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GraphEdgeFamily, GraphProjectionOptions, GraphProjectionPreset, edge_family, project_graph,
    };
    use stencila_schema::ResearchObjectRelation;

    #[test]
    fn document_policy_records_symbolic_links_without_flow_seeding() {
        let policy = document_graph_node_policy(NodeType::SymbolicLink);

        assert!(policy.records());
        assert!(!policy.seeds_flow_projection());
    }

    #[test]
    fn records_all_research_objects_and_resolves_forward_relations() -> Result<()> {
        use stencila_schema::{Claim, Evidence, Protocol, Question, Request};

        let mut claim = Claim::new(vec![paragraph("Claim.")]);
        claim.id = Some("claim-1".to_string());
        claim.relations = Some(vec![ResearchObjectRelation::new(
            ResearchObjectRelationKind::SupportedBy,
            "#evidence-1".to_string(),
        )]);

        let mut evidence = Evidence::new(vec![paragraph("Evidence.")]);
        evidence.id = Some("evidence-1".to_string());
        let mut question = Question::new(vec![paragraph("Question?")]);
        question.id = Some("question-1".to_string());
        let mut protocol = Protocol::new(vec![paragraph("Protocol.")]);
        protocol.id = Some("protocol-1".to_string());
        let mut request = Request::new(vec![paragraph("Request.")]);
        request.id = Some("request-1".to_string());

        let graph = graph_from_node(
            "document:test",
            &Node::Article(stencila_schema::Article::new(vec![
                Block::Claim(claim),
                Block::Evidence(evidence),
                Block::Question(question),
                Block::Protocol(protocol),
                Block::Request(request),
            ])),
        )?;

        for node_type in [
            NodeType::Claim,
            NodeType::Evidence,
            NodeType::Question,
            NodeType::Protocol,
            NodeType::Request,
        ] {
            assert!(
                graph
                    .nodes
                    .iter()
                    .any(|node| node.node.node_type() == node_type)
            );
        }
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == GraphEdgeKind::SupportedBy
                && edge.source == "node:document#clm_claim-1"
                && edge.target == "node:document#evd_evidence-1"
                && edge
                    .options
                    .evidence
                    .as_ref()
                    .is_some_and(|evidence| evidence.len() == 2)
        }));

        let discourse = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Discourse,
                ..Default::default()
            },
        );
        assert_eq!(discourse.preset, GraphProjectionPreset::Discourse);
        assert!(
            discourse
                .edges
                .iter()
                .any(|edge| edge.kind == GraphEdgeKind::SupportedBy)
        );

        Ok(())
    }

    #[test]
    fn resolves_external_targets_and_reports_invalid_relations() -> Result<()> {
        let mut claim = stencila_schema::Claim::new(vec![paragraph("Claim.")]);
        claim.id = Some("claim-1".to_string());
        claim.relations = Some(vec![
            ResearchObjectRelation::new(
                ResearchObjectRelationKind::Supports,
                "https://example.org/evidence".to_string(),
            ),
            ResearchObjectRelation::new(
                ResearchObjectRelationKind::Supports,
                "missing".to_string(),
            ),
            ResearchObjectRelation::new(ResearchObjectRelationKind::Supports, "".to_string()),
            ResearchObjectRelation::new(
                ResearchObjectRelationKind::Supports,
                "#claim-1".to_string(),
            ),
        ]);

        let analysis = graph_from_node_with_diagnostics(
            "document:test",
            &Node::Article(stencila_schema::Article::new(vec![Block::Claim(claim)])),
        )?;

        assert!(analysis.graph.nodes.iter().any(|node| {
            matches!(node.node.as_ref(), Node::CreativeWork(work) if work.options.url.as_deref() == Some("https://example.org/evidence"))
        }));
        assert!(analysis.graph.edges.iter().any(|edge| {
            edge.kind == GraphEdgeKind::Supports && edge.target.starts_with("resource:")
        }));
        assert_eq!(analysis.graph_diagnostics.len(), 3);
        assert!(analysis.graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == crate::GraphDiagnosticKind::UnresolvedResearchRelationTarget
                && diagnostic.target.as_deref() == Some("missing")
        }));
        assert!(analysis.graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == crate::GraphDiagnosticKind::EmptyResearchRelationTarget
        }));
        assert!(analysis.graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == crate::GraphDiagnosticKind::SelfReferentialResearchRelation
        }));

        Ok(())
    }

    #[test]
    fn relation_kind_mapping_is_exhaustive() {
        let mapped = ResearchObjectRelationKind::all()
            .iter()
            .copied()
            .map(GraphEdgeKind::from)
            .collect::<Vec<_>>();
        assert_eq!(mapped.len(), 11);
        assert_eq!(
            mapped,
            vec![
                GraphEdgeKind::Supports,
                GraphEdgeKind::SupportedBy,
                GraphEdgeKind::Opposes,
                GraphEdgeKind::OpposedBy,
                GraphEdgeKind::Addresses,
                GraphEdgeKind::AddressedBy,
                GraphEdgeKind::Follows,
                GraphEdgeKind::Grounds,
                GraphEdgeKind::IsGroundedIn,
                GraphEdgeKind::RequestFor,
                GraphEdgeKind::RequestTarget,
            ]
        );
        assert!(
            ResearchObjectRelationKind::all()
                .iter()
                .copied()
                .map(GraphEdgeKind::from)
                .all(|kind| edge_family(kind) == GraphEdgeFamily::Discourse)
        );
    }

    #[test]
    fn deduplicates_legacy_relations_and_reports_duplicate_ids() -> Result<()> {
        let mut first = stencila_schema::Claim::new(vec![paragraph("First.")]);
        first.id = Some("same".to_string());
        first.relations = Some(vec![ResearchObjectRelation::new(
            ResearchObjectRelationKind::Supports,
            "#target".to_string(),
        )]);
        first.options.extra = Some(Object::from([(
            "supports",
            Primitive::String("#target".to_string()),
        )]));

        let mut second = stencila_schema::Claim::new(vec![paragraph("Second.")]);
        second.id = Some("same".to_string());
        let mut target = stencila_schema::Evidence::new(vec![paragraph("Target.")]);
        target.id = Some("target".to_string());

        let analysis = graph_from_node_with_diagnostics(
            "document:test",
            &Node::Article(stencila_schema::Article::new(vec![
                Block::Claim(first),
                Block::Claim(second),
                Block::Evidence(target),
            ])),
        )?;

        assert_eq!(
            analysis
                .graph
                .edges
                .iter()
                .filter(|edge| edge.kind == GraphEdgeKind::Supports)
                .count(),
            1
        );
        assert!(analysis.graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == crate::GraphDiagnosticKind::DuplicateResearchObjectId
        }));
        assert!(analysis.graph_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == crate::GraphDiagnosticKind::DuplicateResearchRelationTarget
        }));
        Ok(())
    }

    fn paragraph(value: &str) -> Block {
        Block::Paragraph(stencila_schema::Paragraph::new(vec![Inline::Text(
            stencila_schema::Text::new(value.into()),
        )]))
    }

    #[test]
    fn preserves_diagnostics_for_identical_code_in_distinct_nodes() -> Result<()> {
        let mut first = CodeChunk::new("open(path)\n".into());
        first.programming_language = Some("python".to_string());
        let mut second = CodeChunk::new("open(path)\n".into());
        second.programming_language = Some("python".to_string());
        let node = Node::Article(stencila_schema::Article::new(vec![
            Block::CodeChunk(first),
            Block::CodeChunk(second),
        ]));

        let analysis = graph_from_node_with_diagnostics("document:test", &node)?;

        assert_eq!(analysis.diagnostics.len(), 2);
        assert_ne!(
            analysis.diagnostics[0].diagnostic.node_id,
            analysis.diagnostics[1].diagnostic.node_id
        );
        Ok(())
    }
}
