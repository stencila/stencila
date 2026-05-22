//! Focused graph integration tests for behavior that cannot be sufficiently
//! covered by workspace snapshot fixtures.
//!
//! Prefer extending an existing fixture, or adding a new fixture, before adding
//! tests here.

use std::fs::write;

use eyre::{OptionExt, Result};
use stencila_codecs::{DecodeOptions, Format};
use stencila_graph::{
    Graph, GraphBuilder, GraphEdge, GraphEdgeKind, WorkspaceOptions, graph_from_node,
    graph_from_path,
};
use stencila_schema::{
    Article, Block, Citation, CodeChunk, Cord, ExecuteAction, GraphAction, GraphEvidenceKind,
    Inline, Link, Node, Paragraph, Reference, Text,
};
use tempfile::tempdir;

#[tokio::test]
async fn decode_options_override_extension() -> Result<()> {
    let workspace = tempdir()?;
    write(
        workspace.path().join("report"),
        "# Extensionless\n\nDecoded.\n",
    )?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:extensionless".to_string()),
            decode_options: Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
            fail_on_decode_error: true,
            ..Default::default()
        }),
    )
    .await?;

    assert!(graph.nodes.iter().any(|node| node.id == "node:report#art_"));
    assert_graph_edge(
        &graph,
        "file:report",
        "node:report#art_",
        GraphEdgeKind::ConvertedInto,
    );

    Ok(())
}

#[tokio::test]
async fn escapes_delimiters_in_document_scopes() -> Result<()> {
    let workspace = tempdir()?;
    write(
        workspace.path().join("report#1:raw.smd"),
        "# Escaped\n\nDecoded.\n",
    )?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:escaped-document".to_string()),
            fail_on_decode_error: true,
            ..Default::default()
        }),
    )
    .await?;

    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.id == "node:report%231%3Araw.smd#art_")
    );
    assert_graph_edge(
        &graph,
        "file:report%231%3Araw.smd",
        "node:report%231%3Araw.smd#art_",
        GraphEdgeKind::ConvertedInto,
    );

    Ok(())
}

#[test]
fn resolves_read_before_write_document_reactivity() -> Result<()> {
    let mut setup = CodeChunk::new(Cord::from("x = 1\n"));
    setup.id = Some("setup".to_string());
    setup.programming_language = Some("python".to_string());

    let mut update = CodeChunk::new(Cord::from("x = x + 1\n"));
    update.id = Some("update".to_string());
    update.programming_language = Some("python".to_string());

    let node = Node::Article(Article::new(vec![
        Block::CodeChunk(setup),
        Block::CodeChunk(update),
    ]));
    let graph = graph_from_node("fixture:read-before-write", &node)?;

    let setup_id = graph
        .nodes
        .iter()
        .find_map(|node| match node.node.as_ref() {
            Node::CodeChunk(chunk) if chunk.id.as_deref() == Some("setup") => {
                Some(node.id.as_str())
            }
            _ => None,
        })
        .ok_or_eyre("setup chunk should be present")?;
    let update_id = graph
        .nodes
        .iter()
        .find_map(|node| match node.node.as_ref() {
            Node::CodeChunk(chunk) if chunk.id.as_deref() == Some("update") => {
                Some(node.id.as_str())
            }
            _ => None,
        })
        .ok_or_eyre("update chunk should be present")?;

    assert_graph_edge(&graph, update_id, setup_id, GraphEdgeKind::DependsOn);

    Ok(())
}

#[tokio::test]
async fn optionally_fails_on_invalid_environment_manifests() -> Result<()> {
    let workspace = tempdir()?;
    write(workspace.path().join("package.json"), "{ invalid json\n")?;

    graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:invalid-environment".to_string()),
            decode: false,
            ..Default::default()
        }),
    )
    .await?;

    let error = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:invalid-environment".to_string()),
            decode: false,
            fail_on_environment_error: true,
            ..Default::default()
        }),
    )
    .await
    .err()
    .ok_or_eyre("invalid manifest should be rejected in strict mode")?;

    assert!(
        error
            .to_string()
            .contains("unable to analyze environment file")
    );
    Ok(())
}

#[tokio::test]
async fn resolves_media_references_relative_to_document_file() -> Result<()> {
    let workspace = tempdir()?;
    std::fs::create_dir(workspace.path().join("assets"))?;
    std::fs::create_dir(workspace.path().join("docs"))?;
    std::fs::create_dir(workspace.path().join("docs/assets:raw"))?;
    write(workspace.path().join("assets/source.svg"), "<svg></svg>\n")?;
    write(
        workspace.path().join("docs/assets:raw/source#1% copy.svg"),
        "<svg></svg>\n",
    )?;
    write(
        workspace.path().join("docs/report.smd"),
        "# Report\n\n![](../assets/source.svg)\n\n![](assets:raw/source%231%25%20copy.svg)\n\n[source](../assets/source.svg)\n",
    )?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:document-relative-media".to_string()),
            fail_on_decode_error: true,
            ..Default::default()
        }),
    )
    .await?;

    assert!(graph.edges.iter().any(|edge| {
        edge.source == "file:assets/source.svg"
            && edge.target.starts_with("node:docs/report.smd#img_")
            && edge.kind == GraphEdgeKind::ReferencedBy
    }));
    assert!(graph.edges.iter().any(|edge| {
        edge.source == "file:docs/assets%3Araw/source%231%25%20copy.svg"
            && edge.target.starts_with("node:docs/report.smd#img_")
            && edge.kind == GraphEdgeKind::ReferencedBy
    }));
    let link_edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == "file:assets/source.svg"
                && edge.target.starts_with("node:docs/report.smd#lin_")
                && edge.kind == GraphEdgeKind::ReferencedBy
        })
        .ok_or_eyre("workspace-relative link edge should exist")?;
    assert_edge_evidence(link_edge, GraphEvidenceKind::Declared);
    assert_edge_evidence(link_edge, GraphEvidenceKind::Resolved);

    Ok(())
}

#[test]
fn adds_citation_and_external_link_provenance() -> Result<()> {
    let mut citation = Citation::new("smith2020".to_string());
    let mut reference = Reference::new();
    reference.id = Some("smith2020".to_string());
    reference.doi = Some("10.1234/example".to_string());
    citation.options.cites = Some(reference);

    let node = Node::Article(Article::new(vec![Block::Paragraph(Paragraph::new(vec![
        text("See "),
        Inline::Citation(citation),
        text(" and "),
        Inline::Link(Link::new(
            vec![text("data")],
            "https://example.org/data".to_string(),
        )),
        text("."),
    ]))]));

    let graph = graph_from_node("fixture:citation-link", &node)?;

    let citation_edge = graph
        .edges
        .iter()
        .find(|edge| edge.kind == GraphEdgeKind::CitedBy)
        .ok_or_eyre("citation edge should exist")?;
    assert!(citation_edge.source == "reference:document#smith2020");
    assert!(citation_edge.target.starts_with("node:document#cit_"));
    assert_edge_evidence(citation_edge, GraphEvidenceKind::Declared);
    assert_edge_evidence(citation_edge, GraphEvidenceKind::Resolved);

    let link_edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.kind == GraphEdgeKind::ReferencedBy
                && edge.source == "resource:https%3A//example.org/data"
        })
        .ok_or_eyre("external link edge should exist")?;
    assert!(link_edge.target.starts_with("node:document#lin_"));
    assert_edge_evidence(link_edge, GraphEvidenceKind::Declared);

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn rejects_non_utf8_workspace_paths() -> Result<()> {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let workspace = tempdir()?;
    let name = OsString::from_vec(vec![b'r', b'e', b'p', b'o', b'r', b't', 0xff]);
    write(workspace.path().join(name), "not utf-8\n")?;

    let error = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:non-utf8".to_string()),
            decode: false,
            ..Default::default()
        }),
    )
    .await
    .err()
    .ok_or_eyre("non-UTF-8 workspace path should be rejected")?;

    assert!(error.to_string().contains("UTF-8"));
    Ok(())
}

#[test]
fn builder_rejects_dangling_edges() -> Result<()> {
    let mut builder = GraphBuilder::new("test:graph");
    builder.add_schema_node("node:source", Node::String("source".to_string()));
    builder.add_edge("node:source", "node:missing", GraphEdgeKind::Generated);

    let error = builder
        .build()
        .err()
        .ok_or_eyre("graph build should reject dangling edge")?;

    assert!(error.to_string().contains("missing target node"));
    Ok(())
}

#[test]
fn builder_rejects_conflicting_nodes() -> Result<()> {
    let mut builder = GraphBuilder::new("test:graph");
    builder.add_schema_node("node:conflict", Node::String("first".to_string()));
    builder.add_schema_node("node:conflict", Node::String("second".to_string()));

    let error = builder
        .build()
        .err()
        .ok_or_eyre("graph build should reject conflicting nodes")?;

    assert!(error.to_string().contains("conflicting embedded nodes"));
    Ok(())
}

#[test]
fn builder_merges_edge_actions() -> Result<()> {
    let mut builder = GraphBuilder::new("test:graph");
    builder.add_schema_node("node:source", Node::String("source".to_string()));
    builder.add_schema_node("node:target", Node::String("target".to_string()));

    let mut action = ExecuteAction::new();
    action.id = Some("action:execute:test".to_string());
    let graph_action = GraphAction::ExecuteAction(action);

    builder.add_edge_with_actions(
        "node:source",
        "node:target",
        GraphEdgeKind::Generated,
        vec![graph_action.clone()],
    );
    builder.add_edge_with_actions(
        "node:source",
        "node:target",
        GraphEdgeKind::Generated,
        vec![graph_action],
    );

    let graph = builder.build()?;
    let edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == "node:source"
                && edge.target == "node:target"
                && edge.kind == GraphEdgeKind::Generated
        })
        .ok_or_eyre("merged edge should exist")?;

    assert_eq!(
        edge.options.actions.as_deref().map(|actions| actions.len()),
        Some(1)
    );
    Ok(())
}

fn text(value: &str) -> Inline {
    Inline::Text(Text::new(value.into()))
}

fn assert_graph_edge(graph: &Graph, source: &str, target: &str, kind: GraphEdgeKind) {
    assert!(
        graph
            .edges
            .iter()
            .any(|edge| edge.source == source && edge.target == target && edge.kind == kind),
        "missing edge {source} -> {target} ({kind})"
    );
}

fn assert_edge_evidence(edge: &GraphEdge, kind: GraphEvidenceKind) {
    assert!(
        edge.options
            .evidence
            .as_deref()
            .is_some_and(|evidence| evidence.iter().any(|evidence| evidence.kind == kind)),
        "edge {} -> {} ({}) should have {kind} evidence",
        edge.source,
        edge.target,
        edge.kind
    );
}
