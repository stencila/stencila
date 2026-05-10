//! Verify a C2PA asset produced outside the Stencila credential producer.

use std::{fs, path::PathBuf};

use c2pa::{Builder, Context};
use serde_json::json;
use stencila_content_credentials::{
    CredentialVerifier, VerifyAssetRequest, init_dev_cert, signer::CredentialSignerConfig,
};
use tempfile::{NamedTempFile, TempDir};

mod common;

/// Ensures Stencila can validate a C2PA manifest that does not carry the
/// Stencila-specific provenance assertion.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verify_external_c2pa_asset_without_stencila_assertion() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset = tmp.path().join("external.png");
    fs::copy(fixture_path(), &asset).expect("copy fixture");

    sign_with_external_manifest(&asset);

    let report = CredentialVerifier::new()
        .verify_asset(VerifyAssetRequest {
            asset_path: asset.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    assert!(report.manifest.present, "manifest should be present");
    assert!(
        report.manifest.valid,
        "manifest should be valid: {report:?}"
    );
    assert!(report.asset_binding.valid, "asset binding should validate");
    assert!(
        report.signature.valid,
        "claim signature should validate: {report:?}"
    );
    assert!(
        !report.signature.trusted,
        "local dev signer should remain untrusted"
    );
    assert!(
        !report.provenance.assertion_present,
        "non-Stencila asset should not report Stencila provenance"
    );
    assert!(!report.provenance.attested);
    assert!(
        report.summary.is_empty(),
        "no Stencila summary should be projected"
    );
    assert!(
        report.problems.is_empty(),
        "unknown non-Stencila assertions should not be problems: {:?}",
        report.problems
    );

    let required_report = CredentialVerifier::new()
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify with requirement");
    assert!(
        required_report
            .problems
            .iter()
            .any(|problem| problem.contains("--require stencila-assertion")),
        "strict Stencila assertion requirement should be reported"
    );
}

fn sign_with_external_manifest(asset: &std::path::Path) {
    let signer = CredentialSignerConfig::resolve(None, None)
        .expect("resolve signer")
        .create_signer()
        .expect("create signer");
    let definition = json!({
        "claim_generator_info": [{
            "name": "External C2PA Test Generator",
            "version": "1.0.0",
        }],
        "title": "External C2PA sample",
        "format": "image/png",
        "assertions": [
            {
                "label": "c2pa.actions.v2",
                "data": {
                    "actions": [{
                        "action": "c2pa.created",
                        "softwareAgent": {
                            "name": "External C2PA Test Generator",
                            "version": "1.0.0"
                        }
                    }]
                }
            },
            {
                "label": "com.example.external.assertion",
                "data": {
                    "note": "unknown assertion used to exercise validator tolerance"
                }
            }
        ]
    });

    let mut builder = Builder::from_context(Context::new())
        .with_definition(definition.to_string())
        .expect("builder");
    builder.set_no_embed(false);

    let parent = asset.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent).expect("tmp asset");
    {
        let mut source = fs::File::open(asset).expect("open asset");
        builder
            .sign(signer.as_ref(), "image/png", &mut source, tmp.as_file_mut())
            .expect("sign external asset");
    }
    tmp.persist(asset).expect("persist signed asset");
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.png")
}
