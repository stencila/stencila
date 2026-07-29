use std::path::PathBuf;

use eyre::Result;
use stencila_content_credentials::{CredentialProfile, media};
use stencila_graph::{
    AssetGraphOptions, GraphEdgeKind, WorkspaceOptions, credential_graph_for_asset, graph_from_path,
};
use stencila_schema::{
    Author, Graph, GraphEdge, GraphEvidenceKind, GraphNode, ImageObject, Node, Person,
    SoftwareSourceCode,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/code-python-r-dataframe-provenance")
}

#[tokio::test]
async fn extracts_only_selected_asset_lineage() -> Result<()> {
    let root = fixture();
    let graph = graph_from_path(
        &root,
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    let asset = root.join("figures/python-counts.png");
    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &asset,
        &media::sha256_file(&asset)?,
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("scripts/analysis.py")),
            profile: CredentialProfile::Public,
            require_source: true,
            ..Default::default()
        },
    )?;

    let ids = prepared
        .graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"asset:signed"));
    assert!(ids.contains(&"code:scripts/analysis.py"));
    assert!(ids.contains(&"datatable:data/raw/observations.csv"));
    assert!(!ids.contains(&"image:figures/r-counts.png"));
    assert!(!ids.contains(&"code:scripts/analysis.R"));
    assert!(
        prepared
            .graph
            .edges
            .iter()
            .any(|edge| edge.target == "asset:signed")
    );

    let json = serde_json::to_string(&prepared.graph)?;
    assert!(!json.contains(&root.to_string_lossy().to_string()));
    Ok(())
}

#[tokio::test]
async fn synthesizes_a_missing_runtime_output() -> Result<()> {
    let root = fixture();
    let graph = graph_from_path(
        &root,
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &root.join("figures/runtime.png"),
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("scripts/analysis.py")),
            source_line: Some(10),
            require_source: true,
            ..Default::default()
        },
    )?;
    assert!(
        prepared
            .graph
            .edges
            .iter()
            .any(|edge| edge.source == "code:scripts/analysis.py" && edge.target == "asset:signed")
    );
    Ok(())
}

/// The generating source file is in scope as a unit, including members that
/// fed other outputs. Static analysis attributes the write to the file rather
/// than to a symbol, so its members are the only route to the inputs they read.
#[tokio::test]
async fn retains_generating_file_members_to_reach_their_inputs() -> Result<()> {
    let root = fixture();
    let graph = graph_from_path(
        &root,
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    let asset = root.join("figures/python-counts.png");
    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &asset,
        &media::sha256_file(&asset)?,
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("scripts/analysis.py")),
            profile: CredentialProfile::Public,
            require_source: true,
            ..Default::default()
        },
    )?;

    let ids = prepared
        .graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    // `df` reads the input CSV, so reaching it is the point of the traversal.
    assert!(ids.contains(&"symbol:scripts/analysis.py:python:df"));
    assert!(ids.contains(&"datatable:data/raw/observations.csv"));
    // Members of other files stay out, as do the outputs they wrote.
    assert!(!ids.contains(&"symbol:scripts/analysis.R:r:counts"));
    assert!(!ids.contains(&"datatable:outputs/python-clean.csv"));
    Ok(())
}

/// A caller-named source is an assertion, not something the workspace showed
/// us, and the assertion has to say so.
#[tokio::test]
async fn declares_synthesized_edges_without_a_runtime_observation() -> Result<()> {
    let root = fixture();
    let graph = graph_from_path(
        &root,
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &root.join("figures/runtime.png"),
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("scripts/analysis.py")),
            source_line: None,
            require_source: true,
            ..Default::default()
        },
    )?;

    let edge = prepared
        .graph
        .edges
        .iter()
        .find(|edge| edge.source == "code:scripts/analysis.py" && edge.target == "asset:signed")
        .expect("synthesized generation edge");
    let evidence = edge
        .options
        .evidence
        .as_ref()
        .and_then(|evidence| evidence.first())
        .expect("evidence on a synthesized edge");
    assert_eq!(evidence.kind, GraphEvidenceKind::Declared);
    assert!(
        prepared
            .warnings
            .iter()
            .any(|warning| warning.contains("no source-to-output generation edge"))
    );
    Ok(())
}

#[test]
fn uses_lookup_path_while_preserving_the_final_asset_identity() -> Result<()> {
    let root = PathBuf::from("/workspace");
    let mut source = SoftwareSourceCode::new("analysis.py".to_string(), "Python".to_string());
    source.id = Some("code:analysis.py".to_string());
    source.path = Some("analysis.py".to_string());
    let mut image = ImageObject::new("original.png".to_string());
    image.id = Some("image:original.png".to_string());
    let graph = Graph::new(
        "workspace:test".to_string(),
        vec![
            GraphNode::new(
                "code:analysis.py".to_string(),
                Box::new(Node::SoftwareSourceCode(source)),
            ),
            GraphNode::new(
                "image:original.png".to_string(),
                Box::new(Node::ImageObject(image)),
            ),
        ],
        vec![GraphEdge::new(
            "code:analysis.py".to_string(),
            "image:original.png".to_string(),
            GraphEdgeKind::Generated,
        )],
    );

    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &root.join("exported.png"),
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            lookup_path: Some(root.join("original.png")),
            ..Default::default()
        },
    )?;

    assert_eq!(prepared.graph.options.path.as_deref(), Some("exported.png"));
    assert!(
        prepared
            .graph
            .edges
            .iter()
            .any(|edge| { edge.source == "code:analysis.py" && edge.target == "asset:signed" })
    );
    Ok(())
}

#[test]
fn explicit_source_replaces_a_conflicting_discovered_producer() -> Result<()> {
    let root = PathBuf::from("/workspace");
    let mut old_source = SoftwareSourceCode::new("old.py".to_string(), "Python".to_string());
    old_source.id = Some("code:old.py".to_string());
    old_source.path = Some("old.py".to_string());
    let mut requested_source = SoftwareSourceCode::new("new.py".to_string(), "Python".to_string());
    requested_source.id = Some("code:new.py".to_string());
    requested_source.path = Some("new.py".to_string());
    let mut image = ImageObject::new("plot.png".to_string());
    image.id = Some("image:plot.png".to_string());
    let graph = Graph::new(
        "workspace:test".to_string(),
        vec![
            GraphNode::new(
                "code:old.py".to_string(),
                Box::new(Node::SoftwareSourceCode(old_source)),
            ),
            GraphNode::new(
                "code:new.py".to_string(),
                Box::new(Node::SoftwareSourceCode(requested_source)),
            ),
            GraphNode::new(
                "image:plot.png".to_string(),
                Box::new(Node::ImageObject(image)),
            ),
        ],
        vec![GraphEdge::new(
            "code:old.py".to_string(),
            "image:plot.png".to_string(),
            GraphEdgeKind::Generated,
        )],
    );

    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &root.join("plot.png"),
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("new.py")),
            require_source: true,
            ..Default::default()
        },
    )?;
    let ids = prepared
        .graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"code:new.py"));
    assert!(!ids.contains(&"code:old.py"));
    assert!(
        prepared
            .graph
            .edges
            .iter()
            .any(|edge| { edge.source == "code:new.py" && edge.target == "asset:signed" })
    );
    Ok(())
}

/// A rendered asset is genuinely observed at the call site that produced it.
#[tokio::test]
async fn observes_synthesized_edges_with_a_source_line() -> Result<()> {
    let root = fixture();
    let graph = graph_from_path(
        &root,
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    let prepared = credential_graph_for_asset(
        &graph,
        &root,
        &root.join("figures/runtime.png"),
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            source_path: Some(root.join("scripts/analysis.py")),
            source_line: Some(10),
            require_source: true,
            ..Default::default()
        },
    )?;

    let edge = prepared
        .graph
        .edges
        .iter()
        .find(|edge| edge.source == "code:scripts/analysis.py" && edge.target == "asset:signed")
        .expect("synthesized generation edge");
    let evidence = edge
        .options
        .evidence
        .as_ref()
        .and_then(|evidence| evidence.first())
        .expect("evidence on a synthesized edge");
    assert_eq!(evidence.kind, GraphEvidenceKind::Observed);
    assert_eq!(
        evidence
            .code_location
            .as_ref()
            .and_then(|location| location.start_line),
        Some(10)
    );
    Ok(())
}

#[tokio::test]
async fn detects_python_credentials_sign_outputs() -> Result<()> {
    let directory = tempfile::tempdir()?;
    std::fs::write(
        directory.path().join("analysis.py"),
        "from stencila import credentials\ncredentials.sign(plot, output=\"figure.png\")\ncredentials.sign(\"in-place.png\")\n",
    )?;
    std::fs::write(directory.path().join("figure.png"), b"not-a-real-png")?;
    std::fs::write(directory.path().join("in-place.png"), b"not-a-real-png")?;

    let graph = graph_from_path(
        directory.path(),
        Some(WorkspaceOptions {
            include_c2pa: false,
            source_metadata: false,
            git_file_authors: false,
            ..Default::default()
        }),
    )
    .await?;
    assert!(graph.edges.iter().any(|edge| {
        edge.source == "code:analysis.py"
            && edge.target == "image:figure.png"
            && edge.kind == GraphEdgeKind::Generated
    }));
    // Signing an existing asset in place is also a write to that path.
    assert!(graph.edges.iter().any(|edge| {
        edge.source == "code:analysis.py"
            && edge.target == "image:in-place.png"
            && edge.kind == GraphEdgeKind::Generated
    }));
    Ok(())
}

#[test]
fn public_profile_removes_personal_contact_metadata() -> Result<()> {
    let mut author = Person::new();
    author.options.name = Some("Alice Example".to_string());
    author.options.emails = Some(vec!["alice@example.test".to_string()]);
    let mut source = SoftwareSourceCode::new("analysis.py".to_string(), "Python".to_string());
    source.id = Some("code:analysis.py".to_string());
    source.path = Some("/home/alice/project/analysis.py".to_string());
    source.options.authors = Some(vec![Author::Person(author)]);
    let mut image = ImageObject::new("figure.png".to_string());
    image.id = Some("image:figure.png".to_string());
    let graph = Graph::new(
        "workspace:project".to_string(),
        vec![
            GraphNode::new(
                "code:analysis.py".to_string(),
                Box::new(Node::SoftwareSourceCode(source)),
            ),
            GraphNode::new(
                "image:figure.png".to_string(),
                Box::new(Node::ImageObject(image)),
            ),
        ],
        vec![GraphEdge::new(
            "code:analysis.py".to_string(),
            "image:figure.png".to_string(),
            GraphEdgeKind::Generated,
        )],
    );
    let root = PathBuf::from("/home/alice/project");
    let asset = root.join("figure.png");

    let public = credential_graph_for_asset(
        &graph,
        &root,
        &asset,
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            profile: CredentialProfile::Public,
            ..Default::default()
        },
    )?;
    let private = credential_graph_for_asset(
        &graph,
        &root,
        &asset,
        "sha256:test",
        "image/png",
        &AssetGraphOptions {
            profile: CredentialProfile::Private,
            ..Default::default()
        },
    )?;

    let public_json = serde_json::to_string(&public.graph)?;
    let private_json = serde_json::to_string(&private.graph)?;
    assert!(!public_json.contains("alice@example.test"));
    assert!(!public_json.contains("/home/alice"));
    assert!(private_json.contains("alice@example.test"));
    Ok(())
}
