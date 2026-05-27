//! Import C2PA manifest provenance into workspace graphs.
//!
//! The graph keeps a semantic projection of C2PA manifests for traversal while
//! preserving the full reader JSON in evidence details so a later exporter can
//! re-project the original C2PA data without relying only on the graph subset.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use eyre::{Result, WrapErr};
use serde_json::{Value, json};
use stencila_content_credentials::{
    C2paInspection, C2paManifestSourceKind, CredentialVerifier, InspectC2paRequest,
    media::{could_have_embedded, guess_media_type, sidecar_path},
};
use stencila_schema::{
    Action, Array, ConvertAction, CreateAction, CreativeWork, ExecuteAction, GraphAction,
    GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence, GraphEvidenceKind, Node, Null, Object,
    Primitive, SoftwareApplication,
};

use crate::{GraphBuilder, ids::LocalGraphId};

/// Workspace file that may carry C2PA provenance.
#[derive(Debug, Clone)]
pub(crate) struct WorkspaceC2paCandidate {
    /// Path passed to the C2PA reader.
    pub(crate) path: PathBuf,

    /// Graph node id for the signed asset, when the path is an asset.
    pub(crate) asset_node_id: Option<String>,

    /// Graph node id for the `.c2pa` sidecar or standalone manifest file.
    pub(crate) manifest_node_id: Option<String>,

    /// Asset path paired with a standalone sidecar candidate.
    pub(crate) paired_asset_path: Option<PathBuf>,

    /// Whether an absent C2PA manifest is expected for this scan.
    ///
    /// Workspace graph construction opportunistically scans embeddable media
    /// files so signed assets are discovered without requiring a sidecar. An
    /// unsigned media file should not become a C2PA failure in strict mode,
    /// while a present sidecar or standalone `.c2pa` file still should.
    pub(crate) missing_manifest_is_ok: bool,
}

/// Build C2PA scan candidates from workspace files.
pub(crate) fn candidates_from_files(
    files: impl IntoIterator<Item = (PathBuf, String)>,
) -> Vec<WorkspaceC2paCandidate> {
    let files = files.into_iter().collect::<Vec<_>>();
    let node_ids_by_path = files
        .iter()
        .map(|(path, node_id)| (path.clone(), node_id.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut consumed_sidecars = BTreeSet::new();
    let mut candidates = Vec::new();

    for (path, node_id) in &files {
        if is_c2pa_path(path) {
            continue;
        }

        let sidecar = sidecar_path(path);
        let sidecar_node_id = node_ids_by_path.get(&sidecar).cloned();
        let has_sidecar = sidecar_node_id.is_some();
        if has_sidecar {
            consumed_sidecars.insert(sidecar.clone());
        }

        let should_scan = has_sidecar
            || guess_media_type(path)
                .ok()
                .is_some_and(|media_type| could_have_embedded(&media_type));

        if should_scan {
            candidates.push(WorkspaceC2paCandidate {
                path: path.clone(),
                asset_node_id: Some(node_id.clone()),
                manifest_node_id: sidecar_node_id,
                paired_asset_path: None,
                missing_manifest_is_ok: !has_sidecar,
            });
        }
    }

    for (path, node_id) in files {
        if is_c2pa_path(&path) && !consumed_sidecars.contains(&path) {
            candidates.push(WorkspaceC2paCandidate {
                path,
                asset_node_id: None,
                manifest_node_id: Some(node_id),
                paired_asset_path: None,
                missing_manifest_is_ok: false,
            });
        }
    }

    candidates
}

/// Import C2PA provenance for workspace scan candidates.
pub(crate) async fn add_c2pa_from_candidates(
    builder: &mut GraphBuilder,
    candidates: Vec<WorkspaceC2paCandidate>,
    fail_on_error: bool,
) -> Result<()> {
    let verifier = CredentialVerifier::new();

    for candidate in candidates {
        let inspection = verifier
            .inspect_c2pa(InspectC2paRequest {
                path: candidate.path.clone(),
                paired_asset_path: candidate.paired_asset_path.clone(),
                trust_anchors: None,
            })
            .await;

        match inspection {
            Ok(inspection) => add_inspection(builder, &candidate, &inspection)?,
            Err(error) if candidate.missing_manifest_is_ok && error.is_missing_c2pa_manifest() => {}
            Err(error) if fail_on_error => {
                return Err(error).wrap_err_with(|| {
                    format!("unable to inspect C2PA at {}", candidate.path.display())
                });
            }
            Err(..) => {}
        }
    }

    Ok(())
}

fn add_inspection(
    builder: &mut GraphBuilder,
    candidate: &WorkspaceC2paCandidate,
    inspection: &C2paInspection,
) -> Result<()> {
    let Some((active_manifest_id, manifest)) = active_manifest(&inspection.reader_json) else {
        return Ok(());
    };

    let scope = credential_scope(candidate, inspection);
    let credential_id = LocalGraphId::credential(&scope, active_manifest_id);
    add_credential_node(builder, &credential_id, inspection, manifest);

    let raw_evidence = raw_c2pa_evidence(inspection, candidate, &credential_id)?;
    let projected_evidence = projected_c2pa_evidence(inspection, candidate, active_manifest_id);

    let asset_id = candidate.asset_node_id.as_deref();
    let manifest_file_id = candidate.manifest_node_id.as_deref();

    match (asset_id, manifest_file_id, inspection.source_kind) {
        (Some(asset_id), _, C2paManifestSourceKind::Embedded) => {
            builder.add_edge_with_evidence(
                asset_id,
                &credential_id,
                GraphEdgeKind::LinkedBy,
                [raw_evidence],
            );
            builder.add_containment(&credential_id, asset_id, [projected_evidence.clone()]);
        }
        (Some(asset_id), Some(manifest_file_id), C2paManifestSourceKind::Sidecar) => {
            builder.add_edge_with_evidence(
                asset_id,
                &credential_id,
                GraphEdgeKind::LinkedBy,
                [raw_evidence],
            );
            builder.add_containment(
                &credential_id,
                manifest_file_id,
                [projected_evidence.clone()],
            );
        }
        (None, Some(manifest_file_id), C2paManifestSourceKind::Standalone) => {
            builder.add_edge_with_evidence(
                manifest_file_id,
                &credential_id,
                GraphEdgeKind::LinkedBy,
                [raw_evidence],
            );
            builder.add_containment(
                &credential_id,
                manifest_file_id,
                [projected_evidence.clone()],
            );
        }
        _ => {}
    }

    let ingredient_ids = add_ingredients(
        builder,
        &scope,
        &credential_id,
        asset_id,
        manifest,
        &projected_evidence,
    );
    add_actions(
        builder,
        &scope,
        asset_id,
        manifest,
        &ingredient_ids,
        &projected_evidence,
    );
    add_stencila_assertion(builder, &scope, asset_id, inspection, &projected_evidence);
    add_ai_disclosure(builder, &scope, asset_id, manifest, &projected_evidence);

    Ok(())
}

fn add_credential_node(
    builder: &mut GraphBuilder,
    credential_id: &str,
    inspection: &C2paInspection,
    manifest: &Value,
) {
    if builder.contains_node(credential_id) {
        return;
    }

    let mut credential = CreativeWork::new();
    credential.id = Some(credential_id.to_string());
    credential.options.name = manifest
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            inspection
                .reader_json
                .get("active_manifest")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| Some("C2PA content credential".to_string()));
    credential.options.description = Some(match inspection.source_kind {
        C2paManifestSourceKind::Embedded => "Embedded C2PA content credential".to_string(),
        C2paManifestSourceKind::Sidecar => "C2PA sidecar content credential".to_string(),
        C2paManifestSourceKind::Standalone => "Standalone C2PA manifest store".to_string(),
    });

    builder.add_schema_node(credential_id, Node::CreativeWork(credential));
}

fn add_ingredients(
    builder: &mut GraphBuilder,
    scope: &str,
    credential_id: &str,
    asset_id: Option<&str>,
    manifest: &Value,
    evidence: &GraphEvidence,
) -> Vec<String> {
    let mut ingredient_ids = Vec::new();
    let Some(ingredients) = manifest.get("ingredients").and_then(Value::as_array) else {
        return ingredient_ids;
    };

    for (index, ingredient) in ingredients.iter().enumerate() {
        let identity = ingredient_identity(ingredient, index);
        let ingredient_id = LocalGraphId::credential_ingredient(scope, index, &identity);
        add_ingredient_node(builder, &ingredient_id, ingredient, index);
        builder.add_containment(&ingredient_id, credential_id, [evidence.clone()]);

        if let Some(asset_id) = asset_id {
            let relationship = ingredient.get("relationship").and_then(Value::as_str);
            let kind = match relationship {
                Some("componentOf") => GraphEdgeKind::PartOf,
                Some("parentOf" | "inputTo") => GraphEdgeKind::DerivedInto,
                _ => GraphEdgeKind::UsedBy,
            };
            builder.add_edge_with_evidence(&ingredient_id, asset_id, kind, [evidence.clone()]);
        }

        ingredient_ids.push(ingredient_id);
    }

    ingredient_ids
}

fn add_ingredient_node(
    builder: &mut GraphBuilder,
    ingredient_id: &str,
    ingredient: &Value,
    index: usize,
) {
    if builder.contains_node(ingredient_id) {
        return;
    }

    let mut node = CreativeWork::new();
    node.id = Some(ingredient_id.to_string());
    node.options.name = ingredient
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| Some(format!("C2PA ingredient {}", index + 1)));
    node.options.description = ingredient
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            ingredient
                .get("relationship")
                .and_then(Value::as_str)
                .map(|relationship| format!("C2PA {relationship} ingredient"))
        });
    node.options.url = ingredient
        .get("informational_URI")
        .or_else(|| ingredient.get("informationalUri"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    builder.add_schema_node(ingredient_id, Node::CreativeWork(node));
}

fn add_actions(
    builder: &mut GraphBuilder,
    scope: &str,
    asset_id: Option<&str>,
    manifest: &Value,
    ingredient_ids: &[String],
    evidence: &GraphEvidence,
) {
    let Some(asset_id) = asset_id else {
        return;
    };
    let Some(actions) = actions(manifest) else {
        return;
    };

    for (index, action) in actions.iter().enumerate() {
        let kind = action.get("action").and_then(Value::as_str);
        let edge_kind = match kind {
            Some("c2pa.created") => GraphEdgeKind::Generated,
            Some("c2pa.opened" | "c2pa.converted") => GraphEdgeKind::ConvertedInto,
            Some("c2pa.placed") => GraphEdgeKind::PartOf,
            Some("org.stencila.executed") => GraphEdgeKind::Generated,
            _ => GraphEdgeKind::Generated,
        };
        let graph_action = graph_action(scope, action, index);

        let mut sources = action_ingredient_ids(action, ingredient_ids);
        if sources.is_empty()
            && let Some(software_id) = add_action_software(builder, scope, action)
        {
            sources.push(software_id);
        }

        for source in sources {
            builder.add_edge_with_evidence_and_actions(
                source,
                asset_id,
                edge_kind,
                [evidence.clone()],
                [graph_action.clone()],
            );
        }
    }
}

fn add_stencila_assertion(
    builder: &mut GraphBuilder,
    scope: &str,
    asset_id: Option<&str>,
    inspection: &C2paInspection,
    evidence: &GraphEvidence,
) {
    let Some(graph) = inspection.report.provenance.assertion.as_ref() else {
        return;
    };

    let mut id_map = BTreeMap::new();
    for (index, graph_node) in graph.nodes.iter().enumerate() {
        let id = if graph_node.id == "asset:signed" {
            asset_id
                .map(ToString::to_string)
                .unwrap_or_else(|| scoped_stencila_graph_node_id(scope, index, &graph_node.id))
        } else {
            scoped_stencila_graph_node_id(scope, index, &graph_node.id)
        };
        id_map.insert(graph_node.id.clone(), id.clone());

        if Some(id.as_str()) == asset_id {
            continue;
        }
        builder.add_schema_node(id, graph_node.node.as_ref().clone());
    }

    for edge in &graph.edges {
        let source = id_map
            .get(&edge.source)
            .cloned()
            .unwrap_or_else(|| scoped_stencila_graph_node_id(scope, 0, &edge.source));
        let target = id_map
            .get(&edge.target)
            .cloned()
            .unwrap_or_else(|| scoped_stencila_graph_node_id(scope, 0, &edge.target));
        let actions = edge.options.actions.clone().unwrap_or_default();
        let mut evidence_items = edge.options.evidence.clone().unwrap_or_default();
        evidence_items.push(evidence.clone());

        builder.add_edge_with_evidence_and_actions(
            source,
            target,
            edge.kind,
            evidence_items,
            actions,
        );
    }
}

fn add_ai_disclosure(
    builder: &mut GraphBuilder,
    scope: &str,
    asset_id: Option<&str>,
    manifest: &Value,
    evidence: &GraphEvidence,
) {
    let Some(asset_id) = asset_id else {
        return;
    };

    for assertion in manifest
        .get("assertions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|assertion| {
            assertion.get("label").and_then(Value::as_str) == Some("c2pa.ai-disclosure")
        })
    {
        let data = assertion.get("data").unwrap_or(assertion);
        let name = data
            .get("modelName")
            .or_else(|| data.get("model_name"))
            .or_else(|| data.get("model"))
            .and_then(Value::as_str)
            .unwrap_or("AI model");
        let identity = data
            .get("modelIdentifier")
            .or_else(|| data.get("model_identifier"))
            .and_then(Value::as_str)
            .unwrap_or(name);
        let software_id = add_software_agent(builder, scope, identity, name, None, None);
        builder.add_generation(software_id, asset_id, [evidence.clone()]);
    }
}

fn scoped_stencila_graph_node_id(scope: &str, index: usize, id: &str) -> String {
    LocalGraphId::credential_ingredient(scope, index, &format!("stencila-graph:{id}"))
}

fn add_action_software(builder: &mut GraphBuilder, scope: &str, action: &Value) -> Option<String> {
    let software_agent = action.get("softwareAgent")?;
    let name = software_agent
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| software_agent.as_str())?;
    let version = software_agent.get("version").and_then(Value::as_str);

    Some(add_software_agent(
        builder, scope, name, name, version, None,
    ))
}

fn add_software_agent(
    builder: &mut GraphBuilder,
    scope: &str,
    identity: &str,
    name: &str,
    version: Option<&str>,
    provider: Option<&str>,
) -> String {
    let software_id = LocalGraphId::credential_software(scope, identity);
    if builder.contains_node(&software_id) {
        return software_id;
    }

    let mut software = SoftwareApplication::new(name.to_string());
    software.id = Some(software_id.clone());
    software.options.software_version = version.map(ToString::to_string);
    software.options.description =
        provider.map(|provider| format!("C2PA provenance software from {provider}"));
    builder.add_schema_node(&software_id, Node::SoftwareApplication(software));

    software_id
}

fn graph_action(scope: &str, action: &Value, index: usize) -> GraphAction {
    let kind = action
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("c2pa.action");
    let description = action
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let action_id = format!("action:c2pa:{scope}:{index}");

    match kind {
        "c2pa.created" => {
            let mut action = CreateAction::new();
            action.id = Some(action_id);
            action.options.name = Some(kind.to_string());
            action.options.description = description;
            GraphAction::CreateAction(action)
        }
        "c2pa.converted" => {
            let mut action = ConvertAction::new();
            action.id = Some(action_id);
            action.options.name = Some(kind.to_string());
            action.options.description = description;
            GraphAction::ConvertAction(action)
        }
        "org.stencila.executed" => {
            let mut action = ExecuteAction::new();
            action.id = Some(action_id);
            action.options.name = Some(kind.to_string());
            action.options.description = description;
            GraphAction::ExecuteAction(action)
        }
        _ => {
            let mut action = Action::new();
            action.id = Some(action_id);
            action.options.name = Some(kind.to_string());
            action.options.description = description;
            GraphAction::Action(action)
        }
    }
}

fn raw_c2pa_evidence(
    inspection: &C2paInspection,
    candidate: &WorkspaceC2paCandidate,
    credential_id: &str,
) -> Result<GraphEvidence> {
    let mut evidence = c2pa_evidence(inspection);
    evidence.options.description = Some("Raw C2PA manifest inspection".to_string());
    let mut details = json!({
        "credentialId": credential_id,
        "sourceKind": inspection.source_kind,
        "verification": inspection.report,
        "reader": inspection.reader_json,
    });
    add_candidate_node_ids(&mut details, candidate);
    evidence.options.details = Some(object_from_json(details)?);

    Ok(evidence)
}

fn projected_c2pa_evidence(
    inspection: &C2paInspection,
    candidate: &WorkspaceC2paCandidate,
    active_manifest_id: &str,
) -> GraphEvidence {
    let mut evidence = c2pa_evidence(inspection);
    evidence.options.description = Some("Projected from C2PA manifest".to_string());
    let mut details = json!({
        "activeManifest": active_manifest_id,
        "sourceKind": inspection.source_kind,
    });
    add_candidate_node_ids(&mut details, candidate);
    evidence.options.details = object_from_json(details).ok();
    evidence
}

fn add_candidate_node_ids(details: &mut Value, candidate: &WorkspaceC2paCandidate) {
    let Value::Object(object) = details else {
        return;
    };

    if let Some(asset_node_id) = candidate.asset_node_id.as_deref() {
        object.insert("assetNodeId".to_string(), json!(asset_node_id));
    }
    if let Some(manifest_node_id) = candidate.manifest_node_id.as_deref() {
        object.insert("manifestNodeId".to_string(), json!(manifest_node_id));
    }
}

fn c2pa_evidence(inspection: &C2paInspection) -> GraphEvidence {
    let attested = inspection.report.manifest.valid
        && inspection.report.signature.valid
        && inspection.report.asset_binding.valid;
    let mut evidence = GraphEvidence::new(if attested {
        GraphEvidenceKind::Attested
    } else {
        GraphEvidenceKind::Imported
    });
    evidence.confidence = Some(if attested {
        GraphEvidenceConfidence::Certain
    } else if inspection.report.manifest.present {
        GraphEvidenceConfidence::Medium
    } else {
        GraphEvidenceConfidence::Low
    });
    evidence
}

fn object_from_json(value: Value) -> Result<Object> {
    match value {
        Value::Object(object) => object
            .into_iter()
            .map(|(key, value)| Ok((key, primitive_from_json(value)?)))
            .collect::<Result<Vec<_>>>()
            .map(|entries| Object(entries.into_iter().collect())),
        other => Ok(Object(
            [("value".to_string(), primitive_from_json(other)?)]
                .into_iter()
                .collect(),
        )),
    }
}

fn primitive_from_json(value: Value) -> Result<Primitive> {
    Ok(match value {
        Value::Null => Primitive::Null(Null),
        Value::Bool(value) => Primitive::Boolean(value),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Primitive::Integer(value)
            } else if let Some(value) = number.as_u64() {
                Primitive::UnsignedInteger(value)
            } else {
                Primitive::Number(
                    number
                        .as_f64()
                        .ok_or_else(|| eyre::eyre!("JSON number is not representable"))?,
                )
            }
        }
        Value::String(value) => Primitive::String(value),
        Value::Array(values) => Primitive::Array(Array(
            values
                .into_iter()
                .map(primitive_from_json)
                .collect::<Result<Vec<_>>>()?,
        )),
        Value::Object(object) => Primitive::Object(Object(
            object
                .into_iter()
                .map(|(key, value)| Ok((key, primitive_from_json(value)?)))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .collect(),
        )),
    })
}

fn active_manifest(reader_json: &Value) -> Option<(&str, &Value)> {
    let active = reader_json.get("active_manifest").and_then(Value::as_str)?;
    let manifest = reader_json
        .get("manifests")
        .and_then(Value::as_object)?
        .get(active)?;
    Some((active, manifest))
}

fn actions(manifest: &Value) -> Option<&Vec<Value>> {
    manifest
        .get("assertions")
        .and_then(Value::as_array)?
        .iter()
        .find(|assertion| assertion.get("label").and_then(Value::as_str) == Some("c2pa.actions.v2"))
        .and_then(|assertion| assertion.get("data"))
        .and_then(|data| data.get("actions"))
        .and_then(Value::as_array)
}

fn action_ingredient_ids(action: &Value, ingredient_ids: &[String]) -> Vec<String> {
    let mut ids = Vec::new();
    let Some(parameters) = action.get("parameters") else {
        return ids;
    };

    if let Some(ingredients) = parameters.get("ingredients").and_then(Value::as_array) {
        for ingredient in ingredients {
            let Some(url) = ingredient.get("url").and_then(Value::as_str) else {
                continue;
            };
            if let Some(index) = ingredient_index_from_url(url)
                && let Some(id) = ingredient_ids.get(index)
            {
                ids.push(id.clone());
            }
        }
    }

    if let Some(labels) = parameters.get("ingredientIds").and_then(Value::as_array) {
        for label in labels.iter().filter_map(Value::as_str) {
            if let Some((index, _)) = ingredient_ids
                .iter()
                .enumerate()
                .find(|(_, id)| id.ends_with(label))
                && let Some(id) = ingredient_ids.get(index)
            {
                ids.push(id.clone());
            }
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

fn ingredient_index_from_url(url: &str) -> Option<usize> {
    let marker = "c2pa.ingredient.v3";
    let pos = url.rfind(marker)?;
    let suffix = &url[pos + marker.len()..];
    if suffix.is_empty() {
        return Some(0);
    }
    suffix
        .strip_prefix("__")
        .and_then(|index| index.parse::<usize>().ok())
}

fn ingredient_identity(ingredient: &Value, index: usize) -> String {
    ingredient
        .get("hash")
        .or_else(|| ingredient.get("url"))
        .or_else(|| ingredient.get("activeManifest"))
        .or_else(|| ingredient.get("title"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map_or_else(|| format!("ingredient-{index}"), ToString::to_string)
}

fn credential_scope(candidate: &WorkspaceC2paCandidate, inspection: &C2paInspection) -> String {
    candidate
        .asset_node_id
        .as_deref()
        .or(candidate.manifest_node_id.as_deref())
        .map(ToString::to_string)
        .or_else(|| {
            inspection
                .asset_path
                .as_deref()
                .or(inspection.manifest_path.as_deref())
                .and_then(Path::to_str)
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "c2pa".to_string())
}

fn is_c2pa_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("c2pa"))
}
