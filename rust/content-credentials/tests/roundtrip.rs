//! End-to-end sign → verify on a real PNG using a freshly-generated local signing identity.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use stencila_content_credentials::{
    AssetSnapshot, CredentialProducer, CredentialVerifier, DocumentSnapshot, Error,
    IngredientRelationship, IngredientSnapshot, ManifestKind, ProducerSnapshot, ProvenanceSnapshot,
    SignAssetRequest, SourceSnapshot, VerifyAssetRequest, init_local_signing_identity,
    signer::CredentialSignerConfig,
};
use tempfile::TempDir;

mod common;

/// Exercises the embedded-manifest path by signing and verifying a PNG with a local identity.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_then_verify_png() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Sample".to_string()),
            ..Default::default()
        })
        .await
        .expect("sign");

    assert_eq!(signed.manifest_kind, ManifestKind::Embedded);
    assert!(
        signed
            .manifest_id
            .as_deref()
            .is_some_and(|id| id.starts_with("urn:c2pa:")),
        "manifest id should be read after signing: {signed:?}"
    );
    assert!(signed.sidecar_path.is_none());
    assert!(
        signed.warnings.is_empty(),
        "signing should not produce warnings: {:?}",
        signed.warnings
    );
    assert!(signed.source_digest.starts_with("sha256:"));
    assert!(signed.signed_asset_digest.starts_with("sha256:"));
    assert_ne!(
        signed.source_digest, signed.signed_asset_digest,
        "embedding credentials mutates asset bytes"
    );

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    assert!(report.manifest.present, "manifest present");
    assert!(report.manifest.valid, "manifest valid");
    assert!(report.asset_binding.valid, "asset binding valid");
    assert!(!report.signature.trusted, "self-signed must be untrusted");
    assert!(report.provenance.attested, "stencila assertion attested");
    assert!(report.provenance.schema_known, "v1 schema known");
    assert_eq!(
        report.provenance.schema_url.as_deref(),
        Some(stencila_content_credentials::PROVENANCE_SCHEMA)
    );
    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");
    assert_eq!(assertion.asset.content_digest, signed.source_digest);
    assert_eq!(assertion.asset.asset_type, "image");
    assert_eq!(assertion.root_node.node_type, "File");
    assert_eq!(assertion.reproducibility.status, "not-checked");
}

/// Exercises signing with an explicit Stencila provenance snapshot.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_with_provenance_snapshot() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Figure 1".to_string()),
            provenance: Some(ProvenanceSnapshot {
                asset: AssetSnapshot {
                    kind: "image".to_string(),
                    role: Some("figure".to_string()),
                    label: Some("fig:example".to_string()),
                    title: Some("Figure 1".to_string()),
                    ..Default::default()
                },
                root_node: DocumentSnapshot {
                    node_type: "Article".to_string(),
                    title: Some("Article".to_string()),
                    ..Default::default()
                },
                executed_node: Some(DocumentSnapshot {
                    node_type: "CodeChunk".to_string(),
                    node_id: Some("chunk-1".to_string()),
                    label_type: Some("FigureLabel".to_string()),
                    programming_language: Some("python".to_string()),
                    ..Default::default()
                }),
                producer: Some(ProducerSnapshot {
                    codec: Some("png".to_string()),
                    renderer: Some("stencila-cli".to_string()),
                    ..Default::default()
                }),
                source: Some(SourceSnapshot {
                    repository: Some("https://github.com/stencila/example".to_string()),
                    commit: Some("0123456789abcdef".to_string()),
                    path: Some("article.smd".to_string()),
                    dirty: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("sign");

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");
    assert_eq!(assertion.asset.asset_type, "image");
    assert_eq!(assertion.asset.role.as_deref(), Some("figure"));
    assert_eq!(assertion.asset.media_type, "image/png");
    assert!(assertion.asset.content_digest.starts_with("sha256:"));
    assert_eq!(assertion.asset.label.as_deref(), Some("fig:example"));
    assert_eq!(assertion.root_node.node_type, "Article");
    let node = assertion.executed_node.as_ref().expect("executed node");
    assert_eq!(node.node_type, "CodeChunk");
    assert_eq!(node.node_id.as_deref(), Some("chunk-1"));
    assert_eq!(
        assertion
            .source
            .as_ref()
            .and_then(|source| source.commit.as_deref()),
        Some("0123456789abcdef")
    );
    assert_eq!(assertion.producer.codec.as_deref(), Some("png"));
}

/// Ensures signed assets include interoperable C2PA assertions alongside Stencila provenance.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_emits_standard_c2pa_assertions() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Figure 1".to_string()),
            provenance: Some(ProvenanceSnapshot {
                asset: AssetSnapshot {
                    kind: "image".to_string(),
                    role: Some("figure".to_string()),
                    label: Some("fig:example".to_string()),
                    title: Some("Figure 1".to_string()),
                    ..Default::default()
                },
                root_node: DocumentSnapshot {
                    node_type: "Article".to_string(),
                    title: Some("Article".to_string()),
                    ..Default::default()
                },
                producer: Some(ProducerSnapshot {
                    codec: Some("png".to_string()),
                    renderer: Some("stencila-cli".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("sign");

    let manifest_json = CredentialVerifier::new()
        .inspect_asset(&asset_path, None)
        .await
        .expect("inspect");
    let assertions = active_manifest_assertions(&manifest_json);
    let expected_creator_tool = format!("Stencila {}", stencila_version::STENCILA_VERSION);

    let actions = assertion_data(assertions, "c2pa.actions.v2");
    assert_eq!(actions["actions"][0]["action"], "c2pa.created");
    assert_eq!(actions["actions"][0]["softwareAgent"]["name"], "Stencila");
    assert_eq!(
        actions["actions"][0]["softwareAgent"]["version"],
        stencila_version::STENCILA_VERSION
    );
    assert_eq!(
        actions["actions"][0]["parameters"]["org.stencila.codec"],
        "png"
    );
    assert_eq!(
        actions["actions"][0]["parameters"]["org.stencila.assetRole"],
        "figure"
    );

    let metadata = assertion_data(assertions, "c2pa.metadata");
    assert_eq!(metadata["xmp:CreatorTool"], expected_creator_tool);
    assert_eq!(metadata["xmp:Label"], "Figure 1");
    assert_eq!(metadata["dc:format"], "image/png");
    // `dc:type` uses the DCMI Type vocabulary (closed list); `image` maps to
    // `StillImage` so generic Dublin Core tooling can interpret it.
    assert_eq!(metadata["dc:type"], "StillImage");
    // `dc:title` carries the document-level title from `root_node`, distinct
    // from the per-asset `xmp:Label`.
    assert_eq!(metadata["dc:title"], "Article");
    // Internal asset ids (used only for activity references) must not leak into
    // `dc:identifier`, which is reserved for genuine external identifiers.
    assert!(
        metadata.get("dc:identifier").is_none(),
        "internal asset id must not surface as dc:identifier"
    );

    // The `figure` role yields a data-driven source type since Stencila
    // figures are typically visualizations generated from data via code.
    assert_eq!(
        actions["actions"][0]["digitalSourceType"],
        "http://cv.iptc.org/newscodes/digitalsourcetype/dataDrivenMedia"
    );

    let asset_type = assertion_data(assertions, "c2pa.asset-type");
    assert_eq!(asset_type["types"][0]["type"], "org.stencila.asset.figure");
    assert_eq!(
        asset_type["types"][0]["dc:format"], "image/png",
        "asset type v2 should expose the rendition media type"
    );
    assert!(
        assertions
            .iter()
            .all(|assertion| assertion["label"] != "cawg.training-mining"),
        "training/mining opt-out should not be asserted without an explicit policy"
    );

    let active = manifest_json["active_manifest"]
        .as_str()
        .expect("active manifest");
    let manifest = &manifest_json["manifests"][active];
    assert!(
        manifest["thumbnail"].is_object(),
        "image asset should carry a c2pa thumbnail: {manifest:?}"
    );
}

/// Ensures provenance snapshot ingredients are emitted as standard
/// `c2pa.ingredient.v3` ingredients with their relationship preserved.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_emits_ingredient_assertions() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Figure with ingredients".to_string()),
            provenance: Some(ProvenanceSnapshot {
                asset: AssetSnapshot {
                    kind: "image".to_string(),
                    role: Some("figure".to_string()),
                    ..Default::default()
                },
                root_node: DocumentSnapshot {
                    node_type: "Article".to_string(),
                    ..Default::default()
                },
                ingredients: vec![
                    IngredientSnapshot {
                        title: Some("article.smd".to_string()),
                        media_type: Some("text/smd".to_string()),
                        content_digest: Some("sha256:aaaaaaaaaaaaaaaa".to_string()),
                        relationship: IngredientRelationship::InputTo,
                        informational_uri: Some(
                            "https://github.com/example/repo/blob/main/article.smd".to_string(),
                        ),
                        ..Default::default()
                    },
                    IngredientSnapshot {
                        title: Some("data.csv".to_string()),
                        media_type: Some("text/csv".to_string()),
                        content_digest: Some("sha256:bbbbbbbbbbbbbbbb".to_string()),
                        relationship: IngredientRelationship::ComponentOf,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("sign");

    let manifest_json = CredentialVerifier::new()
        .inspect_asset(&asset_path, None)
        .await
        .expect("inspect");
    let active = manifest_json["active_manifest"]
        .as_str()
        .expect("active manifest");
    let ingredients = manifest_json["manifests"][active]["ingredients"]
        .as_array()
        .expect("ingredients array");

    assert_eq!(ingredients.len(), 2, "expected two ingredients");

    let by_title: std::collections::HashMap<&str, &Value> = ingredients
        .iter()
        .map(|ingredient| (ingredient["title"].as_str().unwrap_or_default(), ingredient))
        .collect();

    let source = by_title
        .get("article.smd")
        .copied()
        .expect("source ingredient");
    assert_eq!(source["relationship"], "inputTo");
    assert_eq!(source["format"], "text/smd");

    let component = by_title
        .get("data.csv")
        .copied()
        .expect("component ingredient");
    assert_eq!(component["relationship"], "componentOf");
    assert_eq!(component["format"], "text/csv");

    let actions = assertion_data(
        active_manifest_assertions(&manifest_json),
        "c2pa.actions.v2",
    );
    let action_list = actions["actions"].as_array().expect("actions");
    assert!(
        action_list
            .iter()
            .any(|action| action["action"] == "c2pa.placed"),
        "component ingredients should have a matching placed action"
    );

    // c2pa.created must reference its source ingredient(s) so generic C2PA
    // tooling can see what the new bytes were derived from. ComponentOf
    // ingredients are referenced via their own c2pa.placed actions instead.
    // The SDK resolves the label-based `ingredientIds` we supply into the
    // hashed-URI `ingredients` form before storing the assertion.
    let created = action_list
        .iter()
        .find(|action| action["action"] == "c2pa.created")
        .expect("c2pa.created action");
    let created_ingredients = created["parameters"]["ingredients"]
        .as_array()
        .expect("ingredients on c2pa.created");
    assert_eq!(
        created_ingredients.len(),
        1,
        "only the InputTo ingredient is a source"
    );
    assert_eq!(
        created_ingredients[0]["url"],
        "self#jumbf=c2pa.assertions/c2pa.ingredient.v3"
    );

    let placed = action_list
        .iter()
        .find(|action| action["action"] == "c2pa.placed")
        .expect("c2pa.placed action");
    let placed_ingredients = placed["parameters"]["ingredients"]
        .as_array()
        .expect("ingredients on c2pa.placed");
    assert_eq!(
        placed_ingredients[0]["url"], "self#jumbf=c2pa.assertions/c2pa.ingredient.v3__1",
        "placed action must point at the ComponentOf ingredient"
    );
}

/// Ensures public profile redactions are applied before signing the assertion.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_with_public_profile_redacts_private_snapshot_fields() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let home_path = std::env::var("HOME")
        .map(|home| format!("{home}/private-project/article.smd"))
        .unwrap_or_else(|_| "/home/alice/private-project/article.smd".to_string());

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Private Figure".to_string()),
            provenance: Some(ProvenanceSnapshot {
                asset: AssetSnapshot {
                    kind: "image".to_string(),
                    role: Some("figure".to_string()),
                    ..Default::default()
                },
                root_node: DocumentSnapshot {
                    node_type: "Article".to_string(),
                    ..Default::default()
                },
                source: Some(SourceSnapshot {
                    repository: Some("git@github.com:private/repo.git".to_string()),
                    path: Some(home_path.clone()),
                    dirty: Some(true),
                    patch_digest: Some("sha256:private-patch".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("sign");

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");
    assert!(assertion.source.as_ref().is_some_and(|source| {
        source.repository.is_none() && source.path.is_none() && source.patch_digest.is_none()
    }));
    assert!(
        assertion
            .privacy
            .redactions
            .iter()
            .any(|redaction| redaction.reason.as_deref() == Some("uri-omitted"))
    );

    let assertion_json = serde_json::to_string(assertion).expect("serialize assertion");
    assert!(!assertion_json.contains("git@github.com:private"));
    assert!(!assertion_json.contains(&home_path));
}

/// Ensures a signed output cannot change extension to a different media type.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn output_media_type_must_match_input() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let err = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path,
            output_path: Some(tmp.path().join("sample.jpg")),
            ..Default::default()
        })
        .await
        .expect_err("mismatched output media type should fail");

    assert!(matches!(err, Error::OutputMediaTypeMismatch { .. }));
}

/// Ensures a stale `.c2pa` sidecar does not shadow a valid embedded manifest.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_manifest_wins_over_stale_sidecar() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Sample".to_string()),
            ..Default::default()
        })
        .await
        .expect("sign");

    fs::write(asset_path.with_extension("c2pa"), b"stale sidecar").expect("write sidecar");

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    assert!(report.manifest.present, "manifest present");
    assert!(report.manifest.valid, "manifest valid");
    assert!(!report.manifest.from_sidecar, "embedded manifest used");
    assert!(report.asset_binding.valid, "asset binding valid");
    assert!(report.provenance.attested, "stencila assertion attested");
}

/// Ensures a malformed embedded manifest is reported instead of masked by a sidecar.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn malformed_embedded_manifest_does_not_fall_back_to_sidecar() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_local_signing_identity(true).expect("init local signing identity");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            title: Some("Sample".to_string()),
            ..Default::default()
        })
        .await
        .expect("sign");

    fs::write(asset_path.with_extension("c2pa"), b"stale sidecar").expect("write sidecar");
    duplicate_cabx_chunk(&asset_path);

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    assert!(
        report.manifest.present,
        "invalid embedded manifest was present"
    );
    assert!(
        !report.manifest.valid,
        "invalid embedded manifest is not valid"
    );
    assert!(!report.manifest.from_sidecar, "embedded manifest was used");
    assert!(
        report
            .problems
            .iter()
            .any(|problem| problem.starts_with("embedded C2PA manifest invalid:")),
        "expected embedded manifest error, got {report:?}"
    );
    assert!(
        !report
            .problems
            .iter()
            .any(|problem| problem.starts_with("sidecar C2PA manifest invalid:")),
        "sidecar should not mask embedded manifest errors: {report:?}"
    );
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.png")
}

fn active_manifest_assertions(manifest_json: &Value) -> &[Value] {
    let active_manifest = manifest_json["active_manifest"]
        .as_str()
        .expect("active manifest");
    manifest_json["manifests"][active_manifest]["assertions"]
        .as_array()
        .expect("assertions")
}

fn assertion_data<'a>(assertions: &'a [Value], label: &str) -> &'a Value {
    assertions
        .iter()
        .find(|assertion| assertion["label"] == label)
        .and_then(|assertion| assertion.get("data"))
        .unwrap_or_else(|| panic!("{label} assertion data"))
}

fn duplicate_cabx_chunk(path: &Path) {
    let mut bytes = fs::read(path).expect("read");
    let mut offset = 8;

    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("chunk length bytes"),
        ) as usize;
        let chunk_type_start = offset + 4;
        let data_start = offset + 8;
        let data_end = data_start + length;
        let chunk_end = data_end + 4;

        assert!(chunk_end <= bytes.len(), "invalid PNG chunk length");

        if &bytes[chunk_type_start..chunk_type_start + 4] == b"caBX" {
            let chunk = bytes[offset..chunk_end].to_vec();
            bytes.splice(chunk_end..chunk_end, chunk);
            fs::write(path, bytes).expect("write");
            return;
        }

        offset = chunk_end;
    }

    panic!("caBX chunk not found");
}
