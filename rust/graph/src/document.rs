//! Document graph extraction.
//!
//! This module turns a single Stencila Schema document node tree into graph
//! nodes and resource-flow relationships while keeping the embedded graph nodes
//! shallow.

use std::collections::HashMap;

use eyre::Result;
use stencila_node_stabilize::stabilize;
use stencila_schema::{
    ActionStatusType, Block, Citation, CodeChunk, CodeExpression, CreativeWork,
    DateTime as SchemaDateTime, Duration as SchemaDuration, ExecuteAction, ExecutionStatus, Graph,
    GraphAction, GraphEdgeKind, GraphEvidence, Inline, Link, Node, NodeId, NodeType, Reference,
    StripNode, StripScope, StripTargets, Timestamp, Visitor, WalkControl, WalkNode,
};

use crate::{
    GraphBuilder,
    code::{CodeLanguage, DocumentCodeIndex},
    evidence,
    ids::LocalGraphId,
    reference::has_non_local_uri_scheme,
};

/// Build a graph for a single Stencila Schema node tree.
///
/// This is the simplest document entry point for callers that do not already
/// have a workspace graph builder to extend.
///
/// Document graphs promote the root node plus coarse boundary nodes such as
/// figures, tables, media objects, includes, files, and executable code. This
/// keeps the graph useful for provenance and reactivity without embedding every
/// prose node in the document.
pub fn graph_from_node(subject: impl Into<String>, node: &Node) -> Result<Graph> {
    let mut builder = GraphBuilder::new(subject);
    add_document(&mut builder, "document", node, None);
    builder.build()
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

    /// Internal document anchors keyed by authored id or stable node id.
    ///
    /// Links may refer to nodes later in the document, so anchors are collected
    /// during traversal and pending internal link edges are resolved afterward.
    anchors: HashMap<String, String>,

    /// Internal link references waiting for all document anchors to be known.
    pending_internal_references: Vec<(String, String)>,

    /// Optional resolver for file references discovered in document-local fields.
    ///
    /// Workspace graph construction uses this to resolve paths relative to the
    /// containing document file while keeping workspace path knowledge out of the
    /// document collector.
    reference_resolver: Option<&'a mut DocumentReferenceResolver<'a>>,

    /// Static code facts discovered in executable document nodes.
    code_index: DocumentCodeIndex,
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
            anchors: HashMap::new(),
            pending_internal_references: Vec::new(),
            reference_resolver,
            code_index: DocumentCodeIndex,
        }
    }

    /// Resolve edges that needed a complete document anchor index.
    fn finish(&mut self) {
        let pending = std::mem::take(&mut self.pending_internal_references);
        for (link_id, target) in pending {
            let Some(target_id) = self.anchors.get(&target) else {
                continue;
            };

            self.builder
                .add_link(target_id, link_id, evidence::declared_and_resolved());
        }
    }

    /// Add a Schema node when it should appear in the graph.
    ///
    /// Boundary filtering keeps the graph compact while `force` lets roots and
    /// execution outputs be represented even when they are not boundary types.
    fn add_schema_node(
        &mut self,
        node: Node,
        fallback_id: Option<String>,
        force: bool,
        structural: bool,
    ) -> Option<String> {
        let node_type = node.node_type();
        if !force && !is_boundary_node_type(node_type) {
            return None;
        }

        let node_id = node.node_id();
        let graph_id = node_id
            .as_ref()
            .map(|node_id| graph_id_for_node_id(&self.scope, node_id))
            .or(fallback_id)?;

        if let Some(node_id) = &node_id {
            self.boundaries.insert(node_id.clone(), graph_id.clone());
            self.anchors.insert(node_id.to_string(), graph_id.clone());
        }

        if let Some(id) = authored_id(&node) {
            self.anchors.insert(id.to_string(), graph_id.clone());
        }

        let embedded = shallow_node(&node);
        self.builder.add_schema_node(graph_id.clone(), embedded);

        if structural && let Some(parent_id) = self.parent_stack.last() {
            self.builder
                .add_containment(graph_id.clone(), parent_id, vec![evidence::computed()]);
        }

        self.add_document_reference(&graph_id, &node);

        match node {
            Node::CodeChunk(chunk) => {
                if let Some(node_id) = node_id {
                    self.add_code_chunk_execution(&graph_id, &node_id, &chunk);
                    self.add_static_code_facts(
                        &graph_id,
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

    /// Add static code graph facts for a supported executable document node.
    fn add_static_code_facts(
        &mut self,
        graph_id: &str,
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
                graph_id,
                code,
                language,
                Some(&mut resolver),
            );
        } else {
            self.code_index
                .add_unit(self.builder, &self.scope, graph_id, code, language, None);
        }
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

        match edge_kind {
            GraphEdgeKind::IncludedBy => {
                self.builder
                    .add_include(file_id, graph_id, evidence::declared_and_resolved());
            }
            GraphEdgeKind::LinkedBy => {
                self.builder
                    .add_link(file_id, graph_id, evidence::declared_and_resolved());
            }
            _ => self.builder.add_edge_with_evidence(
                file_id,
                graph_id,
                edge_kind,
                evidence::declared_and_resolved(),
            ),
        }
        true
    }

    /// Add citation provenance from a cited reference to the citation node.
    fn add_citation_reference(&mut self, citation_id: &str, citation: &Citation) {
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
            reference_id,
            citation_id,
            declared_citation_evidence(citation),
        );
    }

    /// Add link provenance from the linked target to the link node.
    fn add_link_reference(&mut self, link_id: &str, link: &Link) {
        let target = link.target.trim();
        if target.is_empty() {
            return;
        }

        if let Some(anchor) = internal_reference_anchor(target) {
            self.pending_internal_references
                .push((link_id.to_string(), anchor.to_string()));
            return;
        }

        if self.add_document_reference(link_id, &Node::Link(link.clone())) {
            return;
        }

        if has_non_local_uri_scheme(target) {
            let resource_id = LocalGraphId::resource(target);
            let mut resource = CreativeWork::new();
            resource.options.url = Some(target.to_string());

            self.builder
                .add_schema_node(resource_id.clone(), Node::CreativeWork(resource));
            self.builder
                .add_link(resource_id, link_id, vec![evidence::declared()]);
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
        self.add_schema_node(node.clone(), None, false, true);
        WalkControl::Continue
    }

    /// Visit a block enum value.
    ///
    /// Blocks are common graph boundaries in documents, so this hook gives the
    /// collector access before the walk enters the block's struct fields.
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        self.add_schema_node(block.clone().into(), None, false, true);
        WalkControl::Continue
    }

    /// Visit an inline enum value.
    ///
    /// Inline media and executable expressions can be graph boundaries, so this
    /// hook records them without promoting every inline text node.
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        self.add_schema_node(inline.clone().into(), None, false, true);
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

/// Return an authored schema `id` for boundary nodes that expose one.
fn authored_id(node: &Node) -> Option<&str> {
    match node {
        Node::Article(node) => node.id.as_deref(),
        Node::AudioObject(node) => node.id.as_deref(),
        Node::Citation(node) => node.id.as_deref(),
        Node::CitationGroup(node) => node.id.as_deref(),
        Node::CodeChunk(node) => node.id.as_deref(),
        Node::CodeExpression(node) => node.id.as_deref(),
        Node::Datatable(node) => node.id.as_deref(),
        Node::Figure(node) => node.id.as_deref(),
        Node::File(node) => node.id.as_deref(),
        Node::Heading(node) => node.id.as_deref(),
        Node::ImageObject(node) => node.id.as_deref(),
        Node::IncludeBlock(node) => node.id.as_deref(),
        Node::Link(node) => node.id.as_deref(),
        Node::MediaObject(node) => node.id.as_deref(),
        Node::Reference(node) => node.id.as_deref(),
        Node::SymbolicLink(node) => node.id.as_deref(),
        Node::Table(node) => node.id.as_deref(),
        Node::VideoObject(node) => node.id.as_deref(),
        _ => None,
    }
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

/// Return the target anchor for an internal document reference.
fn internal_reference_anchor(target: &str) -> Option<&str> {
    let target = target.strip_prefix('#')?;
    (!target.is_empty()).then_some(target)
}

/// Create a lightweight reference node for an unresolved citation target.
fn reference_from_citation_target(target: &str) -> Reference {
    let mut reference = Reference::new();
    reference.id = Some(target.to_string());

    if let Some(doi) = target
        .strip_prefix("doi:")
        .or_else(|| target.strip_prefix("https://doi.org/"))
        .or_else(|| target.strip_prefix("http://doi.org/"))
    {
        reference.doi = Some(doi.to_string());
    } else if has_non_local_uri_scheme(target) {
        reference.url = Some(target.to_string());
    }

    reference
}

/// Evidence for a citation edge.
fn declared_citation_evidence(citation: &Citation) -> Vec<GraphEvidence> {
    if citation.options.cites.is_some() {
        evidence::declared_and_resolved()
    } else {
        vec![evidence::declared()]
    }
}

/// Check whether a node type is represented as a graph boundary.
///
/// The graph intentionally includes coarse document objects and executable
/// nodes instead of every prose node, keeping relationship graphs inspectable.
fn is_boundary_node_type(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Article
            | NodeType::AudioObject
            | NodeType::Citation
            | NodeType::CitationGroup
            | NodeType::CodeChunk
            | NodeType::CodeExpression
            | NodeType::Datatable
            | NodeType::Figure
            | NodeType::File
            | NodeType::Heading
            | NodeType::ImageObject
            | NodeType::IncludeBlock
            | NodeType::Link
            | NodeType::MediaObject
            | NodeType::Reference
            | NodeType::SymbolicLink
            | NodeType::Table
            | NodeType::VideoObject
    )
}

/// Create a shallow graph payload for a Schema node.
///
/// Graph nodes should identify and summarize document objects without carrying
/// large content, generated outputs, or metadata that belongs to other views.
fn shallow_node(node: &Node) -> Node {
    macro_rules! redact_media_node {
        ($media:expr, $variant:ident) => {{
            let mut media = $media.clone();
            redact_data_url(&mut media.content_url);
            Node::$variant(media)
        }};
    }

    let mut node = match node {
        Node::Article(article) => {
            let mut article = article.clone();
            article.content = Vec::new();
            article.options.repository = None;
            article.options.path = None;
            article.options.commit = None;
            Node::Article(article)
        }
        Node::Citation(citation) => {
            let mut citation = citation.clone();
            citation.options.cites = None;
            citation.options.content = None;
            citation.options.compilation_messages = None;
            Node::Citation(citation)
        }
        Node::CitationGroup(group) => {
            let mut group = group.clone();
            group.items.clear();
            group.content = None;
            Node::CitationGroup(group)
        }
        Node::CodeChunk(chunk) => {
            let mut chunk = chunk.clone();
            chunk.caption = None;
            chunk.outputs = None;
            Node::CodeChunk(chunk)
        }
        Node::CodeExpression(expression) => {
            let mut expression = expression.clone();
            expression.output = None;
            Node::CodeExpression(expression)
        }
        Node::Datatable(datatable) => {
            let mut datatable = datatable.clone();
            datatable.caption = None;
            datatable.notes = None;
            for column in &mut datatable.columns {
                column.values.clear();
                column.validator = None;
            }
            Node::Datatable(datatable)
        }
        Node::Figure(figure) => {
            let mut figure = figure.clone();
            figure.caption = None;
            figure.content = Vec::new();
            Node::Figure(figure)
        }
        Node::File(file) => {
            let mut file = file.clone();
            file.content = None;
            Node::File(file)
        }
        Node::Heading(heading) => {
            let mut heading = heading.clone();
            heading.content = Vec::new();
            Node::Heading(heading)
        }
        Node::ImageObject(image) => redact_media_node!(image, ImageObject),
        Node::Link(link) => {
            let mut link = link.clone();
            link.content = Vec::new();
            link.compilation_messages = None;
            Node::Link(link)
        }
        Node::AudioObject(audio) => redact_media_node!(audio, AudioObject),
        Node::MediaObject(media) => redact_media_node!(media, MediaObject),
        Node::Reference(reference) => {
            let mut reference = reference.clone();
            reference.options.content = None;
            Node::Reference(reference)
        }
        Node::VideoObject(video) => redact_media_node!(video, VideoObject),
        Node::Table(table) => {
            let mut table = table.clone();
            table.caption = None;
            table.notes = None;
            table.rows.clear();
            Node::Table(table)
        }
        _ => node.clone(),
    };

    strip_graph_metadata(&mut node);
    node
}

/// Strip metadata scopes that are not part of graph identity.
///
/// Removing volatile or heavyweight fields keeps graph snapshots stable and
/// prevents embedded nodes from duplicating provenance already encoded as edges.
fn strip_graph_metadata(node: &mut Node) {
    node.strip(&StripTargets::scopes(vec![
        StripScope::Authors,
        StripScope::Provenance,
        StripScope::Archive,
        StripScope::Compilation,
        StripScope::Execution,
        StripScope::Temporary,
        StripScope::Timestamps,
    ]));
}

/// Replace embedded data URL bodies with a placeholder.
///
/// Media graph nodes need to retain the URL kind while avoiding large binary
/// payloads in graph output and snapshot fixtures.
fn redact_data_url(content_url: &mut String) {
    if let Some((metadata, ..)) = content_url.split_once(',')
        && metadata.starts_with("data:")
    {
        *content_url = format!("{metadata},<omitted>");
    }
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
