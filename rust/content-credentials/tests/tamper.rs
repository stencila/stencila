//! Tamper a byte after signing → asset binding should report invalid.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use stencila_content_credentials::{
    CredentialProducer, CredentialVerifier, SignAssetRequest, VerifyAssetRequest, init_dev_cert,
    signer::CredentialSignerConfig,
};
use tempfile::TempDir;

mod common;

/// Ensures modifying signed asset bytes invalidates the C2PA asset binding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tamper_breaks_binding() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset = tmp.path().join("tampered.png");
    fs::copy(fixture_path(), &asset).expect("copy");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve");
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset.clone(),
            output_path: None,
            title: None,
        })
        .await
        .expect("sign");

    // Corrupt a byte near the start of the file (in the IHDR chunk, before
    // any c2pa-added chunks). The c2pa hard binding is computed over the
    // asset bytes excluding the manifest box, so a change here must
    // invalidate the binding.
    let mut bytes = fs::read(&asset).expect("read");
    // PNG signature is 8 bytes; IHDR starts at byte 8. Flip a byte in IHDR
    // payload to make sure the asset bytes really change.
    let idx = 20;
    assert!(bytes.len() > idx);
    bytes[idx] ^= 0x55;
    {
        let mut f = fs::File::create(&asset).expect("create");
        f.write_all(&bytes).expect("write");
    }

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: false,
        })
        .await
        .expect("verify");

    assert!(
        !report.asset_binding.valid,
        "expected invalid asset binding, got {report:?}"
    );
    assert!(
        report.signature.valid,
        "asset tampering should not be reported as a signature failure: {report:?}"
    );
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.png")
}
