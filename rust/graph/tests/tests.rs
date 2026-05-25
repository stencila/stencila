//! Focused graph integration tests for behavior that cannot be sufficiently
//! covered by workspace snapshot fixtures.
//!
//! Prefer extending an existing fixture, or adding a new fixture, before adding
//! tests here.

use std::{fs::write, path::Path, process::Command};

use eyre::{OptionExt, Result, bail};
use stencila_codecs::{DecodeOptions, Format};
use stencila_graph::{
    Graph, GraphBuilder, GraphEdge, GraphEdgeKind, GraphProjectionOptions, GraphProjectionPreset,
    WorkspaceOptions, graph_from_node, graph_from_path, project_graph,
};
use stencila_schema::{
    Article, Block, Citation, CodeChunk, Cord, ExecuteAction, GraphAction, GraphEvidenceKind,
    Inline, Link, Node, Paragraph, Reference, Text, WorktreeStatus,
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
    assert_graph_edge(&graph, "node:report#art_", "dir:.", GraphEdgeKind::PartOf);
    assert_no_graph_edge(
        &graph,
        "file:report",
        "node:report#art_",
        GraphEdgeKind::ConvertedInto,
    );

    Ok(())
}

#[tokio::test]
async fn records_workspace_source_metadata_from_git() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let workspace = tempdir()?;
    run_git(workspace.path(), ["init"])?;
    run_git(
        workspace.path(),
        ["config", "user.email", "test@example.org"],
    )?;
    run_git(workspace.path(), ["config", "user.name", "Test User"])?;

    write(workspace.path().join("analysis.py"), "x = 1\n")?;
    run_git(workspace.path(), ["add", "analysis.py"])?;
    run_git(workspace.path(), ["commit", "-m", "initial"])?;
    run_git(
        workspace.path(),
        [
            "remote",
            "add",
            "origin",
            "git@github.com:stencila/example.git",
        ],
    )?;
    let head = run_git(workspace.path(), ["rev-parse", "HEAD"])?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            decode: false,
            analyze_environment: false,
            ..Default::default()
        }),
    )
    .await?;

    assert_eq!(
        graph.options.repository.as_deref(),
        Some("https://github.com/stencila/example")
    );
    assert_eq!(graph.options.path.as_deref(), Some("."));
    assert_eq!(graph.options.commit.as_deref(), Some(head.as_str()));
    assert_eq!(graph.options.worktree_status, Some(WorktreeStatus::Clean));

    write(workspace.path().join("notes.txt"), "not tracked\n")?;
    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            decode: false,
            analyze_environment: false,
            ..Default::default()
        }),
    )
    .await?;
    assert_eq!(
        graph.options.worktree_status,
        Some(WorktreeStatus::Untracked)
    );

    std::fs::remove_file(workspace.path().join("notes.txt"))?;
    write(workspace.path().join("analysis.py"), "x = 2\n")?;
    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            decode: false,
            analyze_environment: false,
            ..Default::default()
        }),
    )
    .await?;
    assert_eq!(graph.options.commit.as_deref(), Some(head.as_str()));
    assert_eq!(graph.options.worktree_status, Some(WorktreeStatus::Dirty));

    Ok(())
}

#[test]
fn graph_from_node_preserves_source_metadata() -> Result<()> {
    let mut article = Article::new(vec![Block::Paragraph(Paragraph::new(vec![text(
        "metadata",
    )]))]);
    article.options.repository = Some("https://github.com/stencila/example".to_string());
    article.options.path = Some("docs/report.smd".to_string());
    article.options.commit = Some("0123456789abcdef0123456789abcdef01234567".to_string());
    article.options.worktree_status = Some(WorktreeStatus::Dirty);

    let graph = graph_from_node("fixture:source-metadata", &Node::Article(article))?;

    assert_eq!(
        graph.options.repository.as_deref(),
        Some("https://github.com/stencila/example")
    );
    assert_eq!(graph.options.path.as_deref(), Some("docs/report.smd"));
    assert_eq!(
        graph.options.commit.as_deref(),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
    assert_eq!(graph.options.worktree_status, Some(WorktreeStatus::Dirty));

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
        "node:report%231%3Araw.smd#art_",
        "dir:.",
        GraphEdgeKind::PartOf,
    );
    assert_no_graph_edge(
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
    let symbol_id = graph
        .nodes
        .iter()
        .find_map(|node| match node.node.as_ref() {
            Node::Variable(variable) if variable.name == "x" => Some(node.id.as_str()),
            _ => None,
        })
        .ok_or_eyre("symbol x should be present")?;

    assert_graph_edge(&graph, setup_id, symbol_id, GraphEdgeKind::Generated);
    assert_graph_edge(&graph, symbol_id, update_id, GraphEdgeKind::UsedBy);

    Ok(())
}

#[test]
fn contains_recorded_outputs_in_executable_nodes() -> Result<()> {
    let mut chunk = CodeChunk::new(Cord::from("print('done')\n"));
    chunk.id = Some("analysis".to_string());
    chunk.programming_language = Some("python".to_string());
    chunk.outputs = Some(vec![Node::String("done".to_string())]);

    let graph = graph_from_node(
        "fixture:recorded-output-containment",
        &Node::Article(Article::new(vec![Block::CodeChunk(chunk)])),
    )?;

    let chunk_id = graph
        .nodes
        .iter()
        .find_map(|node| match node.node.as_ref() {
            Node::CodeChunk(chunk) if chunk.id.as_deref() == Some("analysis") => {
                Some(node.id.as_str())
            }
            _ => None,
        })
        .ok_or_eyre("analysis chunk should be present")?;
    let output_id = graph
        .edges
        .iter()
        .find_map(|edge| {
            (edge.source == chunk_id && edge.kind == GraphEdgeKind::Generated)
                .then_some(edge.target.as_str())
        })
        .ok_or_eyre("analysis output should be generated")?;

    assert_graph_edge(&graph, output_id, chunk_id, GraphEdgeKind::PartOf);
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

    let article_id = graph
        .nodes
        .iter()
        .find_map(|node| {
            matches!(node.node.as_ref(), Node::Article(..)).then_some(node.id.as_str())
        })
        .ok_or_eyre("decoded article should be present")?;

    assert_graph_edge(
        &graph,
        "image:assets/source.svg",
        article_id,
        GraphEdgeKind::LinkedBy,
    );
    assert_graph_edge(
        &graph,
        "image:docs/assets%3Araw/source%231%25%20copy.svg",
        article_id,
        GraphEdgeKind::LinkedBy,
    );
    let link_edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == "image:assets/source.svg"
                && edge.target == article_id
                && edge.kind == GraphEdgeKind::LinkedBy
        })
        .ok_or_eyre("workspace-relative media/link edge should exist")?;
    assert_edge_evidence(link_edge, GraphEvidenceKind::Declared);
    assert_edge_evidence(link_edge, GraphEvidenceKind::Resolved);
    assert!(
        graph.nodes.iter().all(|node| {
            !(node.id.starts_with("node:")
                && matches!(
                    node.node.as_ref(),
                    Node::ImageObject(..) | Node::Link(..) | Node::Heading(..)
                ))
        }),
        "document syntax nodes should not be promoted to graph nodes"
    );

    let cite_view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Cite,
            ..Default::default()
        },
    );
    assert!(
        cite_view
            .edges
            .iter()
            .all(|edge| edge.kind != GraphEdgeKind::LinkedBy),
        "local media/link references should stay out of cite projection"
    );

    Ok(())
}

#[tokio::test]
async fn represents_media_files_as_media_objects() -> Result<()> {
    let workspace = tempdir()?;
    write(workspace.path().join("plot.png"), "not a real png\n")?;
    write(workspace.path().join("interview.mp3"), "not a real mp3\n")?;
    write(workspace.path().join("demo.mp4"), "not a real mp4\n")?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:media-files".to_string()),
            ..Default::default()
        }),
    )
    .await?;

    assert!(graph.nodes.iter().any(|node| {
        node.id == "image:plot.png"
            && matches!(
                node.node.as_ref(),
                Node::ImageObject(image)
                    if image.content_url == "plot.png"
                        && image.media_type.as_deref() == Some("image/png")
            )
    }));
    assert!(graph.nodes.iter().any(|node| {
        node.id == "audio:interview.mp3"
            && matches!(
                node.node.as_ref(),
                Node::AudioObject(audio)
                    if audio.content_url == "interview.mp3"
                        && audio.media_type.as_deref() == Some("audio/mp3")
            )
    }));
    assert!(graph.nodes.iter().any(|node| {
        node.id == "video:demo.mp4"
            && matches!(
                node.node.as_ref(),
                Node::VideoObject(video)
                    if video.content_url == "demo.mp4"
                        && video.media_type.as_deref() == Some("video/mp4")
            )
    }));

    Ok(())
}

#[tokio::test]
async fn resolves_code_literals_to_observed_code_files() -> Result<()> {
    let workspace = tempdir()?;
    write(
        workspace.path().join("analysis.py"),
        r#"
from pathlib import Path
source = Path("helper.py").read_text()
"#,
    )?;
    write(workspace.path().join("helper.py"), "VALUE = 1\n")?;

    let graph = graph_from_path(
        workspace.path(),
        Some(WorkspaceOptions {
            subject: Some("fixture:observed-code-reference".to_string()),
            ..Default::default()
        }),
    )
    .await?;

    assert_graph_edge(
        &graph,
        "code:helper.py",
        "symbol:analysis.py:python:source",
        GraphEdgeKind::ReadBy,
    );
    assert!(
        !graph
            .nodes
            .iter()
            .any(|node| node.id == "file-ref:analysis.py:helper.py"),
        "observed code file should not also be represented as a synthetic file"
    );

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
    let article_id = graph
        .nodes
        .iter()
        .find_map(|node| {
            matches!(node.node.as_ref(), Node::Article(..)).then_some(node.id.as_str())
        })
        .ok_or_eyre("article should be present")?;
    assert!(citation_edge.source == "reference:document#smith2020");
    assert_eq!(citation_edge.target, article_id);
    assert_edge_evidence(citation_edge, GraphEvidenceKind::Declared);
    assert_edge_evidence(citation_edge, GraphEvidenceKind::Resolved);

    let link_edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.kind == GraphEdgeKind::LinkedBy
                && edge.source == "resource:https%3A//example.org/data"
        })
        .ok_or_eyre("external link edge should exist")?;
    assert_eq!(link_edge.target, article_id);
    assert_edge_evidence(link_edge, GraphEvidenceKind::Declared);
    assert!(
        graph.nodes.iter().all(|node| {
            !(node.id.starts_with("node:")
                && matches!(
                    node.node.as_ref(),
                    Node::Citation(..) | Node::Link(..) | Node::Heading(..)
                ))
        }),
        "inline citation and link markers should not be graph nodes"
    );

    let cite_view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Cite,
            ..Default::default()
        },
    );
    assert_eq!(cite_view.edges.len(), 2);
    assert!(cite_view.edges.iter().any(|edge| {
        edge.kind == GraphEdgeKind::CitedBy
            && edge.source == "reference:document#smith2020"
            && edge.target == article_id
    }));
    assert!(cite_view.edges.iter().any(|edge| {
        edge.kind == GraphEdgeKind::LinkedBy
            && edge.source == "resource:https%3A//example.org/data"
            && edge.target == article_id
    }));

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

fn assert_no_graph_edge(graph: &Graph, source: &str, target: &str, kind: GraphEdgeKind) {
    assert!(
        graph
            .edges
            .iter()
            .all(|edge| !(edge.source == source && edge.target == target && edge.kind == kind)),
        "unexpected edge {source} -> {target} ({kind})"
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

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn run_git<const N: usize>(repo: &Path, args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git command failed: {stderr}");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
