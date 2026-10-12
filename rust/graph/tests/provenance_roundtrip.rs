//! A lab account survives signing, workspace import and re-export without stronger claims.
use eyre::{OptionExt, Result};
use serde_json::json;
use std::{fs, path::PathBuf};
use stencila_content_credentials::{
    CredentialProducer, CredentialProfile, PreparedSignAssetRequest, init_local_signing_identity,
    media, signer::CredentialSignerConfig,
};
use stencila_graph::{
    AssetGraphOptions, GraphEdgeKind, WorkspaceOptions, credential_graph_for_asset, graph_from_path,
};
use stencila_schema::{Graph, GraphEvidenceKind};
use tempfile::TempDir;

#[path = "../../content-credentials/tests/common/mod.rs"]
mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lab_account_sign_import_and_reexport_preserves_meaning() -> Result<()> {
    let _guard = common::set_isolated_config_dir();
    init_local_signing_identity(true)?;
    let tmp = TempDir::new()?;
    let asset = tmp.path().join("figure.png");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.png"),
        &asset,
    )?;
    let recorded = json!({"type":"GraphEvidence","kind":"Recorded","source":"lab:run-service","details":{"basis":"host-recorded","entryDigest":"capture"}});
    let graph: Graph = serde_json::from_value(json!({
        "type":"Graph", "subject":"file://figure.png",
        "identifiers":[
            {"type":"PropertyValue","propertyId":"org.stencila.nodeType","value":"SoftwareSourceCode"},
            {"type":"PropertyValue","propertyId":"org.stencila.execution","value":{"outcome":"exited"}}
        ],
        "nodes":[
            {"type":"GraphNode","id":"asset:signed","node":{"type":"File","name":"figure.png","path":"figure.png"}},
            {"type":"GraphNode","id":"run","node":{"type":"ExecuteAction","name":"run","actionStatus":"CompletedActionStatus"},"evidence":[recorded.clone()]},
            {"type":"GraphNode","id":"change","node":{"type":"Action","name":"change"},"evidence":[recorded.clone()]},
            {"type":"GraphNode","id":"step","node":{"type":"ExecuteAction","name":"step","agent":{"type":"Agent","name":"default","description":"Lab agent"}}},
            {"type":"GraphNode","id":"turn","node":{"type":"ExecuteAction","name":"turn"}},
            {"type":"GraphNode","id":"model-input","node":{"type":"CreativeWork","name":"model-input"}},
            {"type":"GraphNode","id":"instruction","node":{"type":"CreativeWork","name":"instruction"}},
            {"type":"GraphNode","id":"payload","node":{"type":"CreativeWork","name":"payload"}}
        ],
        "edges":[
            {"type":"GraphEdge","source":"run","target":"asset:signed","kind":"Collected","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"change","target":"run","kind":"Precedes","role":"includes","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"change","target":"step","kind":"PartOf","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"step","target":"turn","kind":"PartOf","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"model-input","target":"step","kind":"UsedBy","role":"model-input","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"instruction","target":"turn","kind":"UsedBy","role":"invocation","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"payload","target":"change","kind":"UsedBy","role":"submitted","evidence":[recorded.clone()]},
            {"type":"GraphEdge","source":"payload","target":"change","kind":"UsedBy","role":"subject","evidence":[recorded.clone()]}
        ],
        "gaps":[{"type":"GraphGap","state":"Unobserved","source":"lab:run-service","target":{"type":"GraphGapTarget","kind":"Item"},"description":"Writes are not observed."}]
    }))?;
    CredentialProducer::new(CredentialSignerConfig::resolve(None, None)?)
        .sign_prepared_asset(PreparedSignAssetRequest {
            input_path: asset.clone(),
            media_type: "image/png".into(),
            output_path: None,
            title: "figure".into(),
            assertion: graph,
            ingredients: vec![],
            embed: true,
            soft_bindings: vec![],
            credential_profile: CredentialProfile::Full,
        })
        .await?;
    let imported = graph_from_path(
        tmp.path(),
        Some(WorkspaceOptions {
            decode: false,
            analyze_environment: false,
            source_metadata: false,
            ..Default::default()
        }),
    )
    .await?;
    assert!(imported.options.gaps.as_ref().is_none_or(Vec::is_empty));
    let credential = imported
        .nodes
        .iter()
        .find(|node| {
            node.options.gaps.as_ref().is_some_and(|gaps| {
                gaps.iter()
                    .any(|gap| gap.description == "Writes are not observed.")
            })
        })
        .ok_or_eyre("the credential should own its imported account's gaps")?;
    assert_eq!(credential.options.evidence.as_ref().map(Vec::len), Some(1));
    assert_eq!(
        credential
            .options
            .evidence
            .as_ref()
            .and_then(|items| items.first())
            .map(|item| item.kind),
        Some(GraphEvidenceKind::Attested)
    );
    let collected = imported
        .edges
        .iter()
        .find(|edge| edge.kind == GraphEdgeKind::Collected)
        .ok_or_eyre("collected relationship lost")?;
    assert_eq!(collected.options.evidence.as_ref().map(Vec::len), Some(1));
    assert!(collected.options.evidence.as_ref().is_some_and(|items| {
        items
            .iter()
            .all(|item| item.confidence.is_none() && item.kind == GraphEvidenceKind::Recorded)
    }));
    assert_eq!(
        imported
            .edges
            .iter()
            .filter(|edge| edge.options.role.as_deref() == Some("submitted")
                || edge.options.role.as_deref() == Some("subject"))
            .count(),
        2
    );
    assert!(
        !imported
            .edges
            .iter()
            .any(|edge| edge.target == "image:figure.png" && edge.kind == GraphEdgeKind::Generated)
    );
    let prepared = credential_graph_for_asset(
        &imported,
        tmp.path(),
        &asset,
        &media::sha256_file(&asset)?,
        "image/png",
        &AssetGraphOptions {
            profile: CredentialProfile::Full,
            ..Default::default()
        },
    )?;
    let json = serde_json::to_value(prepared.graph)?;
    let names = json["nodes"]
        .as_array()
        .ok_or_eyre("nodes")?
        .iter()
        .filter_map(|node| node["node"]["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"change"));
    assert!(names.contains(&"step"));
    assert!(names.contains(&"turn"));
    assert!(names.contains(&"model-input"));
    assert!(names.contains(&"instruction"));
    Ok(())
}
