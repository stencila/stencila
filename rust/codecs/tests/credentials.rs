#![allow(unsafe_code)]

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
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

fn run_git(repo: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?} failed");
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Initialise a temp git repo containing `article.smd` committed on `main`.
fn init_repo_with_article() -> Result<(TempDir, PathBuf)> {
    let dir = TempDir::new()?;
    let repo = dir.path();
    run_git(repo, &["init", "-q", "-b", "main"]);
    run_git(repo, &["config", "user.email", "test@example.com"]);
    run_git(repo, &["config", "user.name", "Test"]);
    run_git(repo, &["config", "commit.gpgsign", "false"]);

    let source_path = repo.join("article.smd");
    fs::write(&source_path, "# Hello\n\nbody\n")?;
    run_git(repo, &["add", "article.smd"]);
    run_git(repo, &["commit", "-q", "-m", "init"]);

    Ok((dir, source_path))
}

#[tokio::test]
async fn credentials_assertion_records_document_and_source() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_dev_cert(true)?;

    let (dir, source_path) = init_repo_with_article()?;
    let output = dir.path().join("article.md");
    let node = Node::Article(Article {
        title: Some(vec![Inline::Text(Text::from("My Title"))]),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("body"),
        )]))],
        ..Default::default()
    });

    let info = to_path_with_info(
        &node,
        &output,
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            from_path: Some(source_path.clone()),
            credentials: Some(CredentialsOptions {
                profile: CredentialProfile::Private,
            }),
            ..Default::default()
        }),
    )
    .await?;

    let document_sidecar = sidecar_path(&output);
    assert!(document_sidecar.exists());
    assert!(info.assets.contains(&document_sidecar));

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            trust_anchors: None,
        })
        .await?;

    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");

    assert_eq!(assertion.profile, "document-export");
    assert_eq!(assertion.document.node_type, "Article");
    assert_eq!(assertion.document.title.as_deref(), Some("My Title"));
    assert!(
        assertion
            .document
            .node_id
            .as_deref()
            .is_some_and(|id| !id.is_empty())
    );
    assert_eq!(assertion.producer.codec.as_deref(), Some("markdown"));
    assert_eq!(assertion.producer.name, "Stencila");

    let source = assertion
        .source
        .as_ref()
        .expect("source snapshot recorded");
    assert_eq!(source.path.as_deref(), Some("article.smd"));
    assert_eq!(source.dirty, Some(false));
    assert_eq!(
        source.commit.as_deref().map(str::len),
        Some(40),
        "expected commit SHA, got {source:?}"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_dirty_source_records_patch_digest() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_dev_cert(true)?;

    let (dir, source_path) = init_repo_with_article()?;
    fs::write(&source_path, "# Hello\n\nedited body\n")?;

    let output = dir.path().join("article.md");
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("body"),
        )]))],
        ..Default::default()
    });

    to_path_with_info(
        &node,
        &output,
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            from_path: Some(source_path.clone()),
            credentials: Some(CredentialsOptions {
                profile: CredentialProfile::Private,
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
    let source = report
        .provenance
        .assertion
        .as_ref()
        .and_then(|assertion| assertion.source.as_ref())
        .expect("source snapshot");
    assert_eq!(source.dirty, Some(true));
    assert!(
        source
            .patch_digest
            .as_deref()
            .is_some_and(|digest| digest.starts_with("sha256:"))
    );
    assert_eq!(
        source.commit.as_deref().map(str::len),
        Some(40),
        "expected HEAD SHA, got {source:?}"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_public_profile_redacts_dirty_patch_digest() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_dev_cert(true)?;

    let (dir, source_path) = init_repo_with_article()?;
    fs::write(&source_path, "# Hello\n\nedited body\n")?;

    let output = dir.path().join("article.md");
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("body"),
        )]))],
        ..Default::default()
    });

    to_path_with_info(
        &node,
        &output,
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            from_path: Some(source_path.clone()),
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
    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("assertion present");
    let source = assertion.source.as_ref().expect("source");
    // Public profile drops both the repository URL and the dirty patch digest.
    assert!(source.repository.is_none());
    assert_eq!(source.dirty, Some(true));
    assert!(source.patch_digest.is_none());
    assert!(
        assertion
            .privacy
            .redactions
            .iter()
            .any(|redaction| redaction.field.as_deref() == Some("source.patchDigest"))
    );

    Ok(())
}
