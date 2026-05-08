//! `verify` on an unsigned PNG: must report no manifest cleanly.

use std::fs;
use std::path::PathBuf;

use stencila_content_credentials::{CredentialVerifier, VerifyAssetRequest};
use tempfile::TempDir;

/// Ensures verifying an unsigned embeddable asset returns a clean no-manifest report.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verify_unsigned_png_reports_no_manifest() {
    let tmp = TempDir::new().expect("tmp");
    let asset = tmp.path().join("clean.png");
    fs::copy(fixture_path(), &asset).expect("copy fixture");

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: false,
        })
        .await
        .expect("verify");

    assert!(!report.manifest.present, "no manifest expected");
    assert!(!report.signature.valid);
    assert!(!report.provenance.attested);
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.png")
}
