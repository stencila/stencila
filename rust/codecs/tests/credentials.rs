#![allow(unsafe_code)]

use std::{
    env,
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::TempDir;

use stencila_codecs::stencila_schema::{
    Article, Block, ImageObject, Inline, Node, Paragraph, Text,
};
use stencila_codecs::{
    CredentialProfile, CredentialsOptions, EncodeOptions, Format, Result, to_path_with_info,
};
use stencila_content_credentials::{
    CredentialVerifier, VerifyAssetRequest, init_dev_cert, media::sidecar_path,
};

static CONFIG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct ConfigGuard {
    _tmp: TempDir,
    _lock: MutexGuard<'static, ()>,
    prev_xdg: Option<String>,
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        match self.prev_xdg.take() {
            Some(prev) => unsafe { env::set_var("XDG_CONFIG_HOME", prev) },
            None => unsafe { env::remove_var("XDG_CONFIG_HOME") },
        }
    }
}

fn set_isolated_config_dir() -> ConfigGuard {
    let lock = CONFIG_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let tmp = TempDir::new().expect("tmp");
    let path: PathBuf = tmp.path().to_path_buf();
    let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
    unsafe { env::set_var("XDG_CONFIG_HOME", &path) };

    ConfigGuard {
        _tmp: tmp,
        _lock: lock,
        prev_xdg,
    }
}

#[tokio::test]
async fn credentials_sign_markdown_and_extracted_media() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_dev_cert(true)?;

    let dir = TempDir::new()?;
    let output = dir.path().join("report.md");
    let node = Node::Article(Article::new(vec![Block::ImageObject(ImageObject::new(
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
            .to_string(),
    ))]));

    let info = to_path_with_info(
        &node,
        &output,
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            credentials: Some(CredentialsOptions {
                profile: CredentialProfile::Public,
            }),
            ..Default::default()
        }),
    )
    .await?;

    let document_sidecar = sidecar_path(&output);
    assert!(document_sidecar.exists());
    assert!(info.assets.contains(&document_sidecar));

    let media_path = info
        .assets
        .iter()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) != Some("c2pa"))
        .expect("extracted media asset");
    assert!(media_path.exists());

    let verifier = CredentialVerifier::new();
    let document_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            trust_anchors: None,
        })
        .await?;
    assert!(document_report.manifest.from_sidecar);
    assert!(document_report.provenance.attested);

    let media_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: media_path.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            trust_anchors: None,
        })
        .await?;
    assert!(!media_report.manifest.from_sidecar);
    assert!(media_report.provenance.attested);

    Ok(())
}

#[tokio::test]
async fn credentials_sign_smd_with_stencila_media_type() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_dev_cert(true)?;

    let dir = TempDir::new()?;
    let output = dir.path().join("report.smd");
    let node = Node::Article(Article::new(vec![Block::Paragraph(Paragraph::new(vec![
        Inline::Text(Text::from("Signed Stencila Markdown.")),
    ]))]));

    let _info = to_path_with_info(
        &node,
        &output,
        Some(EncodeOptions {
            format: Some(Format::Smd),
            credentials: Some(CredentialsOptions {
                profile: CredentialProfile::Public,
            }),
            ..Default::default()
        }),
    )
    .await?;

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            trust_anchors: None,
        })
        .await?;

    assert!(report.manifest.from_sidecar);
    assert!(report.provenance.attested);
    assert_eq!(
        report
            .provenance
            .assertion
            .expect("provenance assertion")
            .asset
            .media_type,
        "text/smd"
    );

    Ok(())
}
