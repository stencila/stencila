//! Keep credential provenance focused on the asset being attested.
//!
//! Workspace graphs intentionally describe more than a single output. This
//! module selects the relevant upstream subgraph and projects it according to
//! the requested disclosure profile before it crosses a credential boundary.

use std::{
    collections::{BTreeSet, VecDeque},
    path::{Component, Path, PathBuf},
};

use eyre::{Result, bail, eyre};
use stencila_content_credentials::{
    CredentialProfile, IngredientRelationship, IngredientSnapshot, ProjectionPolicy,
    graph::{
        ASSET_ID, PROP_ASSET_ID, PROP_ASSET_ROLE, PROP_ASSET_TITLE, PROP_ASSET_TYPE,
        PROP_CONTENT_DIGEST, PROP_MEDIA_TYPE, asset_kind_for_media_type,
    },
    media,
};
use stencila_schema::{
    CodeLocation, CreateAction, ExecuteAction, File, Graph, GraphAction, GraphEdge, GraphEdgeKind,
    GraphEvidence, GraphEvidenceKind, GraphNode, ImageObject, Node, Object, Primitive,
    PropertyValue, PropertyValueOrString,
};

/// Control how much evidence is required and disclosed for an asset.
///
/// Keeping selection inputs together makes the same policy usable by graph
/// inspection and credential signing.
#[derive(Debug, Clone)]
pub struct AssetGraphOptions {
    /// Locate discovered provenance using this path when it differs from the
    /// final credential asset path.
    pub lookup_path: Option<PathBuf>,

    /// Prefer provenance linked to this generating source.
    pub source_path: Option<PathBuf>,

    /// Attach runtime evidence at this source line when available.
    pub source_line: Option<u64>,

    /// Select the disclosure level applied to the resulting graph.
    pub profile: CredentialProfile,

    /// Reject assets without source-linked provenance when true.
    pub require_source: bool,

    /// Override the asset title inferred from its filename.
    pub title: Option<String>,
}

impl Default for AssetGraphOptions {
    fn default() -> Self {
        Self {
            lookup_path: None,
            source_path: None,
            source_line: None,
            profile: CredentialProfile::Public,
            require_source: false,
            title: None,
        }
    }
}

/// Carry the complete provenance payload needed by credential production.
///
/// Returning ingredients and warnings with the graph prevents callers from
/// independently reconstructing facts that must agree with the assertion.
#[derive(Debug, Clone)]
pub struct AssetCredentialGraph {
    /// The asset-centred graph to embed as the provenance assertion.
    pub graph: Graph,
    /// Snapshots of upstream files represented in the graph.
    pub ingredients: Vec<IngredientSnapshot>,
    /// The stable identifier of the output asset node.
    pub asset_node_id: String,
    /// Non-fatal gaps discovered while selecting provenance.
    pub warnings: Vec<String>,
}

/// Limit a workspace graph to the provenance of one asset.
///
/// Credential assertions should not leak unrelated workspace structure. The
/// asset path identifies the final output, while the optional lookup path and
/// source select discovered provenance. Ambiguity and required-but-missing
/// source evidence are reported as errors.
pub fn credential_graph_for_asset(
    workspace_graph: &Graph,
    workspace_root: &Path,
    asset_path: &Path,
    asset_digest: &str,
    media_type: &str,
    options: &AssetGraphOptions,
) -> Result<AssetCredentialGraph> {
    let relative_asset = relative_path(workspace_root, asset_path);
    let relative_lookup = relative_path(
        workspace_root,
        options.lookup_path.as_deref().unwrap_or(asset_path),
    );
    let source_id = options
        .source_path
        .as_deref()
        .map(|path| format!("code:{}", relative_path(workspace_root, path)));

    let mut candidates = workspace_graph
        .nodes
        .iter()
        .filter(|node| node_path(node).as_deref() == Some(relative_lookup.as_str()))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();

    if candidates.len() > 1 {
        let source_matches = source_id.as_deref().map(|source_id| {
            candidates
                .iter()
                .filter(|candidate| {
                    workspace_graph.edges.iter().any(|edge| {
                        edge.source == source_id
                            && edge.target == candidate.as_str()
                            && edge.kind == GraphEdgeKind::Generated
                    })
                })
                .cloned()
                .collect::<Vec<_>>()
        });
        if let Some(source_matches) = source_matches
            && !source_matches.is_empty()
        {
            candidates = source_matches;
        }
    }
    if candidates.len() > 1 {
        bail!(
            "asset lookup path `{}` matches multiple graph nodes; supply a source that uniquely identifies one",
            relative_lookup
        );
    }

    let selected_id = candidates.first().cloned();
    let selected_has_source = selected_id.as_deref().is_some_and(|selected| {
        workspace_graph.edges.iter().any(|edge| {
            edge.target == selected
                && edge.kind == GraphEdgeKind::Generated
                && source_id.as_deref().map_or_else(
                    || edge.source.starts_with("code:"),
                    |source| edge.source == source,
                )
        })
    });
    let mut warnings = Vec::new();
    if selected_id.is_none() {
        warnings.push(format!(
            "no statically discovered output node for `{relative_lookup}`; using runtime asset identity"
        ));
    }

    let mut retained = BTreeSet::new();
    if let Some(selected) = selected_id.as_deref() {
        if let Some(source_id) = source_id.as_deref() {
            retained.insert(selected.to_string());
            if selected_has_source {
                retain_upstream(workspace_graph, source_id, &mut retained);
            }
        } else {
            retain_upstream(workspace_graph, selected, &mut retained);
        }
    }

    let has_source = selected_has_source;

    let runtime_source_available = source_id.as_deref().is_some_and(|source_id| {
        workspace_graph
            .nodes
            .iter()
            .any(|node| node.id == source_id)
    });

    if !has_source
        && runtime_source_available
        && let Some(source_id) = source_id.as_deref()
    {
        retain_upstream(workspace_graph, source_id, &mut retained);
        retained.insert(source_id.to_string());
    }

    if options.require_source && !has_source && !runtime_source_available {
        bail!("source-linked provenance is required but no generating source was found");
    }
    // A caller-supplied source is an assertion rather than a discovered fact, so
    // record the gap even when it satisfies `require_source`.
    if !has_source {
        warnings.push("no source-to-output generation edge was found".to_string());
    }

    retain_containers(workspace_graph, &mut retained);
    let selected_id_for_edges = selected_id.as_deref();

    let mut nodes = workspace_graph
        .nodes
        .iter()
        .filter(|node| {
            retained.contains(&node.id) && Some(node.id.as_str()) != selected_id_for_edges
        })
        .cloned()
        .collect::<Vec<_>>();
    let asset_node = credential_asset_node(
        &relative_asset,
        asset_digest,
        media_type,
        options.title.as_deref(),
    );
    nodes.push(asset_node);
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let endpoint = |id: &str| {
        if Some(id) == selected_id_for_edges {
            ASSET_ID.to_string()
        } else {
            id.to_string()
        }
    };
    let mut edges = workspace_graph
        .edges
        .iter()
        .filter(|edge| {
            let conflicts_with_requested_source = source_id.as_deref().is_some_and(|source_id| {
                Some(edge.target.as_str()) == selected_id_for_edges
                    && edge.kind == GraphEdgeKind::Generated
                    && edge.source.starts_with("code:")
                    && edge.source != source_id
            });
            !conflicts_with_requested_source
                && (retained.contains(&edge.source)
                    || Some(edge.source.as_str()) == selected_id_for_edges)
                && (retained.contains(&edge.target)
                    || Some(edge.target.as_str()) == selected_id_for_edges)
        })
        .cloned()
        .map(|mut edge| {
            edge.source = endpoint(&edge.source);
            edge.target = endpoint(&edge.target);
            edge
        })
        .collect::<Vec<_>>();

    if !edges.iter().any(|edge| {
        edge.target == ASSET_ID
            && edge.kind == GraphEdgeKind::Generated
            && edge.source.starts_with("code:")
    }) && let Some(source_id) = source_id
        && nodes.iter().any(|node| node.id == source_id)
    {
        let mut edge = GraphEdge::new(source_id, ASSET_ID.to_string(), GraphEdgeKind::Generated);
        if let Some(line) = options.source_line {
            let source = options
                .source_path
                .as_deref()
                .map(|path| relative_path(workspace_root, path))
                .unwrap_or_default();
            let mut location = CodeLocation::new();
            location.source = Some(source);
            location.start_line = Some(line);

            let mut evidence = GraphEvidence::new(GraphEvidenceKind::Observed);
            evidence.code_location = Some(location);
            evidence.options.description =
                Some("Observed while rendering the asset from this call site".to_string());
            evidence.options.details = Some(Object::from([
                (
                    "detector",
                    Primitive::String("stencila-python-runtime".to_string()),
                ),
                ("operation", Primitive::String("render/sign".to_string())),
            ]));
            edge.options.evidence = Some(vec![evidence]);

            let mut action = ExecuteAction::new();
            action.id = Some("action:stencila-python-render".to_string());
            action.options.name = Some("org.stencila.rendered".to_string());
            edge.options.actions = Some(vec![GraphAction::ExecuteAction(action)]);
        } else {
            // Without a runtime observation this edge rests entirely on the
            // caller naming the source, so mark it as declared rather than
            // letting it read like a discovered generation edge.
            let mut evidence = GraphEvidence::new(GraphEvidenceKind::Declared);
            evidence.options.description =
                Some("Declared by the caller; no generating source was discovered".to_string());
            evidence.options.details = Some(Object::from([(
                "detector",
                Primitive::String("stencila-python-runtime".to_string()),
            )]));
            edge.options.evidence = Some(vec![evidence]);
        }
        edges.push(edge);
    }
    if let Some(edge) = edges
        .iter_mut()
        .find(|edge| edge.target == ASSET_ID && edge.kind == GraphEdgeKind::Generated)
    {
        let mut action = CreateAction::new();
        action.id = Some("action:stencila-sign".to_string());
        action.options.name = Some("org.stencila.signed".to_string());
        edge.options
            .actions
            .get_or_insert_default()
            .push(GraphAction::CreateAction(action));
    }
    edges.sort_by(|left, right| {
        (&left.source, &left.target, left.kind).cmp(&(&right.source, &right.target, right.kind))
    });

    let mut graph = Graph::new(format!("asset:{relative_asset}"), nodes, edges);
    graph.options.path = Some(relative_asset);
    let policy = ProjectionPolicy::for_workspace(options.profile, workspace_root.to_path_buf());
    let redactions = policy
        .project_graph(&mut graph)
        .map_err(|error| eyre!(error))?;
    if !redactions.is_empty() {
        warnings.push(format!(
            "credential privacy policy applied {} redaction(s)",
            redactions.len()
        ));
    }

    let ingredients = ingredients(&graph, workspace_root);
    Ok(AssetCredentialGraph {
        graph,
        ingredients,
        asset_node_id: ASSET_ID.to_string(),
        warnings,
    })
}

/// Retain causal ancestors so the asset remains explainable.
///
/// Traversal follows `PartOf` backwards into source files on purpose. Static
/// analysis attributes a write to the file as a whole rather than to any symbol
/// within it, so a file's members are the only route from the asset to the
/// inputs those members read. The generating file is therefore in scope as a
/// unit, including members that produced other outputs.
///
/// Directories are the exception: expanding one would pull in every sibling in
/// the workspace, so containment for directories is restored afterwards by
/// [`retain_containers`] instead.
fn retain_upstream(graph: &Graph, start: &str, retained: &mut BTreeSet<String>) {
    let mut queue = VecDeque::from([start.to_string()]);
    while let Some(target) = queue.pop_front() {
        if !retained.insert(target.clone()) {
            continue;
        }
        let expands_siblings = target.starts_with("dir:");
        for edge in &graph.edges {
            if edge.target == target && !(expands_siblings && edge.kind == GraphEdgeKind::PartOf) {
                queue.push_back(edge.source.clone());
            }
        }
    }
}

/// Preserve containers needed to locate retained nodes in the workspace.
fn retain_containers(graph: &Graph, retained: &mut BTreeSet<String>) {
    loop {
        let parents = graph
            .edges
            .iter()
            .filter(|edge| edge.kind == GraphEdgeKind::PartOf && retained.contains(&edge.source))
            .map(|edge| edge.target.clone())
            .filter(|id| !retained.contains(id))
            .collect::<Vec<_>>();
        if parents.is_empty() {
            break;
        }
        retained.extend(parents);
    }
}

/// Read a normalized path across the file-like schema node variants.
fn node_path(node: &GraphNode) -> Option<String> {
    let path = match node.node.as_ref() {
        Node::AudioObject(audio) => audio.options.path.as_deref().unwrap_or(&audio.content_url),
        Node::Directory(directory) => &directory.path,
        Node::File(file) => &file.path,
        Node::ImageObject(image) => image.options.path.as_deref().unwrap_or(&image.content_url),
        Node::MediaObject(media) => media.options.path.as_deref().unwrap_or(&media.content_url),
        Node::SoftwareSourceCode(source) => source.path.as_deref()?,
        Node::SymbolicLink(link) => &link.path,
        Node::VideoObject(video) => video.options.path.as_deref().unwrap_or(&video.content_url),
        _ => return None,
    };
    Some(normalize_relative(path))
}

/// Replace the discovered output with a stable credential asset identity.
///
/// The runtime digest and media type describe the bytes actually signed,
/// rather than relying on information collected during workspace discovery.
fn credential_asset_node(
    path: &str,
    digest: &str,
    media_type: &str,
    title: Option<&str>,
) -> GraphNode {
    let name = title
        .map(ToOwned::to_owned)
        .or_else(|| Path::new(path).file_name()?.to_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "asset".to_string());
    let identifiers = Some(vec![
        property_identifier(PROP_ASSET_TYPE, asset_kind_for_media_type(media_type)),
        property_identifier(PROP_ASSET_ID, ASSET_ID),
        property_identifier(PROP_ASSET_ROLE, "output"),
        property_identifier(PROP_ASSET_TITLE, name.clone()),
        property_identifier(PROP_MEDIA_TYPE, media_type),
        property_identifier(PROP_CONTENT_DIGEST, digest),
    ]);
    let node = if media_type.starts_with("image/") {
        let mut image = ImageObject::new(path.to_string());
        image.id = Some(ASSET_ID.to_string());
        image.media_type = Some(media_type.to_string());
        image.options.name = Some(name);
        image.options.identifiers = identifiers;
        Node::ImageObject(image)
    } else {
        let mut file = File::new(name, path.to_string());
        file.id = Some(ASSET_ID.to_string());
        file.media_type = Some(media_type.to_string());
        file.options.identifiers = identifiers;
        Node::File(file)
    };
    GraphNode::new(ASSET_ID.to_string(), Box::new(node))
}

/// Construct a typed identifier property for the credential asset.
fn property_identifier(
    property_id: impl Into<String>,
    value: impl Into<String>,
) -> PropertyValueOrString {
    let mut property = PropertyValue::new(Primitive::String(value.into()));
    property.property_id = Some(property_id.into());
    PropertyValueOrString::PropertyValue(property)
}

/// Snapshot retained files so verifiers can identify concrete inputs.
fn ingredients(graph: &Graph, workspace_root: &Path) -> Vec<IngredientSnapshot> {
    graph
        .nodes
        .iter()
        .filter(|node| node.id != ASSET_ID)
        .filter_map(|node| {
            let path = node_path(node)?;
            let absolute = workspace_root.join(&path);
            if !absolute.is_file() {
                return None;
            }
            let media_type = media::guess_media_type(&absolute).ok()?;
            let content_digest = media::sha256_file(&absolute).ok()?;
            Some(IngredientSnapshot {
                label: Some(node.id.clone()),
                title: absolute
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(ToOwned::to_owned),
                media_type: Some(media_type),
                content_digest: Some(content_digest),
                relationship: IngredientRelationship::InputTo,
                informational_uri: Some(path),
                ..Default::default()
            })
        })
        .collect()
}

/// Produce a portable path without exposing directories outside the workspace.
fn relative_path(root: &Path, path: &Path) -> String {
    let path = if path.is_absolute() {
        path.strip_prefix(root).unwrap_or_else(|_| {
            path.file_name()
                .map(Path::new)
                .unwrap_or_else(|| Path::new("asset"))
        })
    } else {
        path
    };
    normalize_relative(&path.to_string_lossy())
}

/// Normalize relative paths for stable graph matching and serialization.
fn normalize_relative(path: &str) -> String {
    let mut components = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(value) => components.push(value.to_string_lossy().to_string()),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    components.join("/")
}
