//! End-to-end sign → verify on a real PNG using a freshly-generated dev cert.

use std::fs;
use std::path::{Path, PathBuf};

use stencila_content_credentials::{
    CredentialProducer, CredentialVerifier, Error, ManifestKind, SignAssetRequest,
    VerifyAssetRequest, init_dev_cert, signer::CredentialSignerConfig,
};
use tempfile::TempDir;

mod common;

/// Exercises the embedded-manifest path by signing and verifying a PNG with a dev certificate.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_then_verify_png() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            output_path: None,
            title: Some("Sample".to_string()),
        })
        .await
        .expect("sign");

    assert_eq!(signed.manifest_kind, ManifestKind::Embedded);
    assert!(signed.sidecar_path.is_none());
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
        Some(stencila_content_credentials::PROVENANCE_SCHEMA_V1)
    );
    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");
    assert_eq!(assertion.asset.source_digest, signed.source_digest);
}

/// Ensures a signed output cannot change extension to a different media type.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn output_media_type_must_match_input() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let err = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path,
            output_path: Some(tmp.path().join("sample.jpg")),
            title: None,
        })
        .await
        .expect_err("mismatched output media type should fail");

    assert!(matches!(err, Error::OutputMediaTypeMismatch { .. }));
}

/// Ensures a stale `.c2pa` sidecar does not shadow a valid embedded manifest.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_manifest_wins_over_stale_sidecar() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            output_path: None,
            title: Some("Sample".to_string()),
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
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset_path = tmp.path().join("sample.png");
    fs::copy(fixture_path(), &asset_path).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            output_path: None,
            title: Some("Sample".to_string()),
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
