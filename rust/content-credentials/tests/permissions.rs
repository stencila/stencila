//! Regression tests for signed asset and sidecar file permissions.

#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use stencila_content_credentials::{
    CredentialProducer, Result, SignAssetRequest, init_dev_cert, signer::CredentialSignerConfig,
};
use tempfile::TempDir;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_signing_preserves_asset_permissions() -> Result<()> {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true)?;

    let tmp = TempDir::new()?;
    let asset = tmp.path().join("figure.png");
    fs::copy(fixture_path("sample.png"), &asset)?;
    fs::set_permissions(&asset, fs::Permissions::from_mode(0o644))?;

    let signer = CredentialSignerConfig::resolve(None, None)?;
    let producer = CredentialProducer::new(signer);
    producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset.clone(),
            output_path: None,
            title: None,
        })
        .await?;

    assert_eq!(file_mode(&asset)?, 0o644);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_signing_preserves_output_and_sidecar_permissions() -> Result<()> {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true)?;

    let tmp = TempDir::new()?;
    let input = tmp.path().join("input.pdf");
    let output = tmp.path().join("exported.pdf");
    fs::copy(fixture_path("sample.pdf"), &input)?;
    fs::set_permissions(&input, fs::Permissions::from_mode(0o640))?;

    let signer = CredentialSignerConfig::resolve(None, None)?;
    let producer = CredentialProducer::new(signer);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: input,
            output_path: Some(output.clone()),
            title: None,
        })
        .await?;

    assert_eq!(file_mode(&output)?, 0o640);
    let Some(sidecar_path) = signed.sidecar_path.as_deref() else {
        return Err(stencila_content_credentials::Error::other(
            "expected sidecar path for PDF",
        ));
    };
    assert_eq!(file_mode(sidecar_path)?, 0o640);
    Ok(())
}

fn file_mode(path: &Path) -> Result<u32> {
    Ok(fs::metadata(path)?.permissions().mode() & 0o777)
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}
