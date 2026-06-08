//! Deterministic graph assembly.
//!
//! This module centralizes node and edge accumulation so graph construction
//! produces stable output regardless of the traversal order used by callers.

use std::collections::{BTreeMap, BTreeSet};

use eyre::{Result, bail};
use stencila_schema::{
    Author, Cord, Graph, GraphAction, GraphEdge, GraphEdgeKind, GraphEvidence, GraphNode, Node,
    StripNode, StripScope, StripTargets,
};

const CODE_PREVIEW_MAX_LINES: usize = 8;
const CODE_PREVIEW_MAX_CHARS: usize = 600;
const CODE_PREVIEW_TRUNCATION_MARKER: &str = "\n...";

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
        let node = Box::new(shallow_node(&node));
        self.add_prepared_schema_node(id, node);
    }

    /// Add a file-backed graph node while preserving supplied Git authors.
    ///
    /// Graph nodes normally strip all author metadata because document-level
    /// authorship belongs to source documents, not graph identity. Workspace
    /// file resources are different: Git authorship is filesystem source
    /// metadata collected by the workspace graph itself, so this path restores
    /// only those supplied authors after shallowing.
    pub(crate) fn add_file_schema_node(
        &mut self,
        id: impl Into<String>,
        node: Node,
        authors: Option<&[Author]>,
    ) {
        let id = id.into();
        let mut node = shallow_node(&node);
        if let Some(authors) = authors.filter(|authors| !authors.is_empty()) {
            set_file_backed_authors(&mut node, authors.to_vec());
        }
        self.add_prepared_schema_node(id, Box::new(node));
    }

    fn add_prepared_schema_node(&mut self, id: String, node: Box<Node>) {
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

    /// Check whether a graph node id has already been added.
    pub fn contains_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
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

    /// Add structural containment from a child node to its parent container.
    pub fn add_containment(
        &mut self,
        child: impl Into<String>,
        parent: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(child, parent, GraphEdgeKind::PartOf, evidence);
    }

    /// Add a resource-read relationship from a resource to its consumer.
    pub fn add_read(
        &mut self,
        resource: impl Into<String>,
        consumer: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(resource, consumer, GraphEdgeKind::ReadBy, evidence);
    }

    /// Add a generated-output relationship from a generator to its output.
    pub fn add_generation(
        &mut self,
        generator: impl Into<String>,
        output: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(generator, output, GraphEdgeKind::Generated, evidence);
    }

    /// Add a value-write relationship from an in-memory value to a persisted resource.
    pub fn add_write(
        &mut self,
        value: impl Into<String>,
        resource: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(value, resource, GraphEdgeKind::WrittenTo, evidence);
    }

    /// Add a lineage relationship from an upstream value or resource to a downstream result.
    pub fn add_derivation(
        &mut self,
        source: impl Into<String>,
        result: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(source, result, GraphEdgeKind::DerivedInto, evidence);
    }

    /// Add a citation relationship from a cited work to its citing context.
    pub fn add_citation(
        &mut self,
        cited: impl Into<String>,
        citing: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(cited, citing, GraphEdgeKind::CitedBy, evidence);
    }

    /// Add a link relationship from a linked resource to the link or document region.
    pub fn add_link(
        &mut self,
        resource: impl Into<String>,
        linked_by: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(resource, linked_by, GraphEdgeKind::LinkedBy, evidence);
    }

    /// Add an inclusion relationship from an included source to the including document node.
    pub fn add_include(
        &mut self,
        source: impl Into<String>,
        included_by: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(source, included_by, GraphEdgeKind::IncludedBy, evidence);
    }

    /// Add a declaration relationship from a manifest or config source to what it declares.
    pub fn add_declaration(
        &mut self,
        source: impl Into<String>,
        declared: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(source, declared, GraphEdgeKind::Declares, evidence);
    }

    /// Add a package requirement relationship from a package to its environment.
    pub fn add_requirement(
        &mut self,
        package: impl Into<String>,
        environment: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(package, environment, GraphEdgeKind::RequiredBy, evidence);
    }

    /// Add a reproducibility pin from a lockfile or exact pin to its environment.
    pub fn add_pin(
        &mut self,
        pin: impl Into<String>,
        pinned: impl Into<String>,
        evidence: impl IntoIterator<Item = GraphEvidence>,
    ) {
        self.add_edge_with_evidence(pin, pinned, GraphEdgeKind::Pins, evidence);
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

/// Create a shallow graph payload for a Schema node.
///
/// Graph nodes should identify and summarize objects without carrying large
/// content, generated outputs, or metadata that belongs to source documents and
/// files.
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
            article.r#abstract = None;
            article.frontmatter = None;
            article.references = None;
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
            chunk.execution_mode = None;
            chunk.code = code_preview(&chunk.code);
            chunk.execution_bounds = None;
            chunk.label_automatically = None;
            chunk.overlay = None;
            chunk.outputs = None;
            chunk.is_echoed = None;
            chunk.is_hidden = None;
            chunk.options.overlay_compiled = None;
            Node::CodeChunk(chunk)
        }
        Node::CodeExpression(expression) => {
            let mut expression = expression.clone();
            expression.execution_mode = None;
            expression.code = code_preview(&expression.code);
            expression.execution_bounds = None;
            expression.output = None;
            Node::CodeExpression(expression)
        }
        Node::Datatable(datatable) => {
            let mut datatable = datatable.clone();
            datatable.label_automatically = None;
            datatable.notes = None;
            for column in &mut datatable.columns {
                column.values.clear();
            }
            Node::Datatable(datatable)
        }
        Node::DatatableColumn(column) => {
            let mut column = column.clone();
            column.values.clear();
            Node::DatatableColumn(column)
        }
        Node::Figure(figure) => {
            let mut figure = figure.clone();
            figure.label_automatically = None;
            figure.content = Vec::new();
            figure.options.layout = None;
            figure.options.padding = None;
            figure.options.overlay = None;
            figure.options.overlay_compiled = None;
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
            table.label_automatically = None;
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

fn set_file_backed_authors(node: &mut Node, authors: Vec<Author>) {
    match node {
        Node::AudioObject(node) => node.options.authors = Some(authors),
        Node::Datatable(node) => node.options.authors = Some(authors),
        Node::File(node) => node.options.authors = Some(authors),
        Node::ImageObject(node) => node.options.authors = Some(authors),
        Node::SoftwareSourceCode(node) => node.options.authors = Some(authors),
        Node::VideoObject(node) => node.options.authors = Some(authors),
        _ => {}
    }
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

/// Return a bounded code preview for graph display and inspection.
fn code_preview(code: &Cord) -> Cord {
    Cord::from(truncate_code_preview(&code.string))
}

fn truncate_code_preview(code: &str) -> String {
    let exceeds_chars = code.chars().count() > CODE_PREVIEW_MAX_CHARS;
    let exceeds_lines = code.lines().nth(CODE_PREVIEW_MAX_LINES).is_some();

    if !exceeds_chars && !exceeds_lines {
        return code.to_string();
    }

    let line_limited = if exceeds_lines {
        code.lines()
            .take(CODE_PREVIEW_MAX_LINES)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        code.to_string()
    };

    let marker_chars = CODE_PREVIEW_TRUNCATION_MARKER.chars().count();
    let body_chars = CODE_PREVIEW_MAX_CHARS.saturating_sub(marker_chars);
    let mut preview = line_limited.chars().take(body_chars).collect::<String>();
    preview.push_str(CODE_PREVIEW_TRUNCATION_MARKER);
    preview
}

#[cfg(test)]
mod tests {
    use eyre::{Result, eyre};
    use stencila_schema::{
        ArrayValidator, Block, CodeChunk, CodeExpression, Datatable, DatatableColumn, DateTime,
        Figure, File, ImageObject, Inline, LabelType, Paragraph, Primitive, Table, Text,
    };

    use super::*;

    #[test]
    fn add_schema_node_shallows_datatable_columns() -> Result<()> {
        let mut column = DatatableColumn::new(
            "count".to_string(),
            vec![Primitive::Integer(1), Primitive::Integer(2)],
        );
        column.validator = Some(ArrayValidator::new());
        let mut datatable = Datatable::new(vec![column]);
        datatable.caption = Some(caption("Observed counts"));
        datatable.options.date_created = Some(DateTime::new("2026-01-01T00:00:00Z".to_string()));

        let mut builder = GraphBuilder::new("test:datatable");
        builder.add_schema_node("datatable:data.csv", Node::Datatable(datatable));

        let node = only_node(builder.build()?)?;
        let Node::Datatable(datatable) = node.as_ref() else {
            return Err(eyre!("expected datatable node"));
        };

        assert_eq!(datatable.columns.len(), 1);
        assert!(datatable.columns[0].values.is_empty());
        assert!(datatable.columns[0].validator.is_some());
        assert!(datatable.caption.is_some());
        assert!(datatable.options.date_created.is_some());

        Ok(())
    }

    #[test]
    fn add_schema_node_keeps_short_code_chunk_preview() -> Result<()> {
        let mut chunk = CodeChunk::new("plot(summaries)".into());
        chunk.programming_language = Some("r".to_string());
        chunk.label_type = Some(LabelType::FigureLabel);
        chunk.label = Some("1a".to_string());
        chunk.caption = Some(caption("Treatment response"));
        chunk.overlay = Some("<svg/>".to_string());

        let mut builder = GraphBuilder::new("test:code-chunk");
        builder.add_schema_node("node:article#chunk", Node::CodeChunk(chunk));

        let node = only_node(builder.build()?)?;
        let Node::CodeChunk(chunk) = node.as_ref() else {
            return Err(eyre!("expected code chunk node"));
        };

        assert_eq!(chunk.code.to_string(), "plot(summaries)");
        assert_eq!(chunk.programming_language.as_deref(), Some("r"));
        assert_eq!(chunk.label_type, Some(LabelType::FigureLabel));
        assert_eq!(chunk.label.as_deref(), Some("1a"));
        assert!(chunk.caption.is_some());
        assert!(chunk.overlay.is_none());

        Ok(())
    }

    #[test]
    fn add_schema_node_truncates_long_code_chunk_preview() -> Result<()> {
        let code = (1..=12)
            .map(|line| format!("line_{line} <- {line}"))
            .collect::<Vec<_>>()
            .join("\n");

        let mut builder = GraphBuilder::new("test:long-code-chunk");
        builder.add_schema_node(
            "node:article#chunk",
            Node::CodeChunk(CodeChunk::new(code.into())),
        );

        let node = only_node(builder.build()?)?;
        let Node::CodeChunk(chunk) = node.as_ref() else {
            return Err(eyre!("expected code chunk node"));
        };

        let preview = chunk.code.to_string();
        assert!(preview.contains("line_8 <- 8"));
        assert!(!preview.contains("line_9 <- 9"));
        assert!(preview.ends_with(CODE_PREVIEW_TRUNCATION_MARKER));

        Ok(())
    }

    #[test]
    fn add_schema_node_truncates_unicode_code_preview_safely() -> Result<()> {
        let code = "value <- \"Δ\" # 😀\n".repeat(80);

        let mut builder = GraphBuilder::new("test:unicode-code-chunk");
        builder.add_schema_node(
            "node:article#chunk",
            Node::CodeChunk(CodeChunk::new(code.into())),
        );

        let node = only_node(builder.build()?)?;
        let Node::CodeChunk(chunk) = node.as_ref() else {
            return Err(eyre!("expected code chunk node"));
        };

        let preview = chunk.code.to_string();
        assert!(preview.chars().count() <= CODE_PREVIEW_MAX_CHARS);
        assert!(preview.ends_with(CODE_PREVIEW_TRUNCATION_MARKER));

        Ok(())
    }

    #[test]
    fn add_schema_node_truncates_code_expression_preview() -> Result<()> {
        let expression = CodeExpression::new("x + y\n".repeat(20).into());

        let mut builder = GraphBuilder::new("test:code-expression");
        builder.add_schema_node("node:article#expression", Node::CodeExpression(expression));

        let node = only_node(builder.build()?)?;
        let Node::CodeExpression(expression) = node.as_ref() else {
            return Err(eyre!("expected code expression node"));
        };

        let preview = expression.code.to_string();
        assert!(preview.ends_with(CODE_PREVIEW_TRUNCATION_MARKER));
        assert!(!preview.is_empty());

        Ok(())
    }

    #[test]
    fn add_schema_node_keeps_figure_and_table_captions() -> Result<()> {
        let mut figure = Figure::new(Vec::new());
        figure.caption = Some(caption("Microscopy overview"));
        let mut table = Table::new(Vec::new());
        table.caption = Some(caption("Summary statistics"));

        let mut figure_builder = GraphBuilder::new("test:figure");
        figure_builder.add_schema_node("node:article#figure", Node::Figure(figure));
        let Node::Figure(figure) = only_node(figure_builder.build()?)?.as_ref().clone() else {
            return Err(eyre!("expected figure node"));
        };

        let mut table_builder = GraphBuilder::new("test:table");
        table_builder.add_schema_node("node:article#table", Node::Table(table));
        let Node::Table(table) = only_node(table_builder.build()?)?.as_ref().clone() else {
            return Err(eyre!("expected table node"));
        };

        assert!(figure.caption.is_some());
        assert!(figure.content.is_empty());
        assert!(table.caption.is_some());
        assert!(table.rows.is_empty());

        Ok(())
    }

    #[test]
    fn add_schema_node_redacts_media_data_urls() -> Result<()> {
        let image = ImageObject::new("data:image/png;base64,abc123".to_string());

        let mut builder = GraphBuilder::new("test:image");
        builder.add_schema_node("image:plot.png", Node::ImageObject(image));

        let node = only_node(builder.build()?)?;
        let Node::ImageObject(image) = node.as_ref() else {
            return Err(eyre!("expected image node"));
        };

        assert_eq!(image.content_url, "data:image/png;base64,<omitted>");

        Ok(())
    }

    #[test]
    fn add_schema_node_shallows_file_content() -> Result<()> {
        let mut file = File::new("report.smd".to_string(), "report.smd".to_string());
        file.content = Some("# Full document".to_string());
        file.options.date_modified = Some(DateTime::new("2026-01-01T00:00:00Z".to_string()));

        let mut builder = GraphBuilder::new("test:file");
        builder.add_schema_node("file:report.smd", Node::File(file));

        let node = only_node(builder.build()?)?;
        let Node::File(file) = node.as_ref() else {
            return Err(eyre!("expected file node"));
        };

        assert!(file.content.is_none());
        assert!(file.options.date_modified.is_some());

        Ok(())
    }

    fn only_node(graph: Graph) -> Result<Box<Node>> {
        graph
            .nodes
            .into_iter()
            .next()
            .map(|node| node.node)
            .ok_or_else(|| eyre!("missing graph node"))
    }

    fn caption(value: &str) -> Vec<Block> {
        vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new(value.into()),
        )]))]
    }
}
