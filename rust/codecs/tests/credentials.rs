#![allow(unsafe_code)]

use std::{
    collections::BTreeSet,
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::TempDir;

use stencila_codecs::stencila_schema::{
    Article, Block, CodeChunk, CompilationDigest, Duration, ExecutionDependency,
    ExecutionDependencyRelation, ExecutionMessage, ExecutionStatus, Figure, Heading, ImageObject,
    Inline, MessageLevel, Node, Paragraph, Text, TimeUnit,
};
use stencila_codecs::{
    CredentialProfile, CredentialsOptions, EncodeInfo, EncodeOptions, Format, Result,
    to_path_with_info,
};
use stencila_content_credentials::{
    CredentialProducer, CredentialProfile as ContentCredentialProfile, CredentialSignerConfig,
    CredentialVerifier, SignAssetRequest, VerifyAssetRequest,
    export::{ExportSigningRequest, sign_encoded_export},
    init_local_signing_identity,
    media::{has_c2pa_manifest, sha256_file, sidecar_path},
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
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let output = dir.path().join("report.md");
    let node = Node::Article(Article::new(vec![Block::Figure(Figure {
        label: Some("1".to_string()),
        caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Generated result."),
        )]))]),
        content: vec![Block::ImageObject(ImageObject::new(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                .to_string(),
        ))],
        ..Default::default()
    })]));

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
    assert!(
        info.assets
            .iter()
            .any(|asset| asset.path == document_sidecar)
    );
    let document_asset = info
        .assets
        .iter()
        .find(|asset| asset.role.as_deref() == Some("document"))
        .expect("document asset");
    assert!(document_asset.signed);
    assert_eq!(document_asset.manifest_kind.as_deref(), Some("sidecar"));
    assert!(
        document_asset
            .manifest_id
            .as_deref()
            .is_some_and(|id| id.starts_with("urn:c2pa:"))
    );
    assert_eq!(document_asset.credential_profile.as_deref(), Some("public"));
    assert!(document_asset.signing_warnings.is_empty());

    let media_asset = info
        .assets
        .iter()
        .find(|asset| {
            asset.role.as_deref() != Some("document") && asset.role.as_deref() != Some("sidecar")
        })
        .expect("extracted media asset");
    assert!(media_asset.signed);
    assert_eq!(media_asset.manifest_kind.as_deref(), Some("embedded"));
    assert_eq!(
        media_asset.title.as_deref(),
        Some("Figure 1: Generated result.")
    );
    assert!(
        media_asset
            .manifest_id
            .as_deref()
            .is_some_and(|id| id.starts_with("urn:c2pa:"))
    );
    assert_eq!(media_asset.credential_profile.as_deref(), Some("public"));
    assert!(media_asset.signing_warnings.is_empty());
    let media_path = &media_asset.path;
    assert!(media_path.exists());

    let verifier = CredentialVerifier::new();
    let document_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    assert!(document_report.manifest.from_sidecar);
    assert!(document_report.provenance.attested);

    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let active = document_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    let document_ingredients = document_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("document ingredients");
    let component = document_ingredients
        .iter()
        .find(|ingredient| ingredient["relationship"] == "componentOf")
        .expect("component ingredient");
    assert_eq!(component["title"], "Figure 1: Generated result.");
    assert_eq!(component["format"], "image/png");
    assert!(
        component["active_manifest"].is_string(),
        "component should link to the signed media manifest: {component:#?}"
    );

    let media_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: media_path.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    assert!(!media_report.manifest.from_sidecar);
    assert!(media_report.provenance.attested);
    let media_assertion = media_report
        .provenance
        .assertion
        .as_ref()
        .expect("media assertion");
    assert_eq!(media_assertion.asset.id.as_deref(), Some("exported-asset"));
    assert_eq!(media_assertion.asset.role.as_deref(), Some("figure"));
    assert_eq!(
        media_assertion.asset.title.as_deref(),
        Some("Figure 1: Generated result.")
    );
    assert!(
        media_assertion.executed_node.is_none(),
        "plain figure assets should not be represented as executed nodes"
    );
    assert_eq!(
        media_assertion
            .output_node
            .as_ref()
            .map(|node| node.node_type.as_str()),
        Some("Figure")
    );
    assert!(media_assertion.asset.content_digest.starts_with("sha256:"));

    Ok(())
}

#[tokio::test]
async fn credentials_static_figure_media_is_component_ingredient() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let image = dir.path().join("static.png");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.png"),
        &image,
    )?;
    let original_digest = sha256_file(&image)?;

    let output = dir.path().join("report.md");
    let node = Node::Article(Article::new(vec![Block::Figure(Figure {
        label: Some("1".to_string()),
        caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Static result."),
        )]))]),
        content: vec![Block::ImageObject(ImageObject::new(
            image.to_string_lossy().to_string(),
        ))],
        ..Default::default()
    })]));

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

    assert_eq!(
        sha256_file(&image)?,
        original_digest,
        "static referenced media should not be modified while signing the document"
    );
    assert!(
        !has_c2pa_manifest(&image, Some("image/png")),
        "static referenced media should not receive C2PA metadata"
    );
    assert!(
        info.assets
            .iter()
            .all(|asset| asset.path != image || !asset.signed),
        "static referenced media should not be reported as a signed export asset"
    );

    let verifier = CredentialVerifier::new();
    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let active = document_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    let document_ingredients = document_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("document ingredients");
    let component = document_ingredients
        .iter()
        .find(|ingredient| ingredient["relationship"] == "componentOf")
        .expect("component ingredient");
    assert_eq!(component["title"], "Figure 1: Static result.");
    assert_eq!(component["format"], "image/png");
    assert!(
        component["active_manifest"].is_string(),
        "static media component should link to the media manifest: {component:#?}"
    );
    assert!(
        component["thumbnail"]["identifier"]
            .as_str()
            .is_some_and(|identifier| identifier.contains("c2pa.thumbnail.ingredient")),
        "static media component should have a parent-side ingredient thumbnail: {component:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_reuses_existing_static_media_manifest() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let image = dir.path().join("static.png");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.png"),
        &image,
    )?;

    let producer = CredentialProducer::new(CredentialSignerConfig::resolve(None, None)?);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: image.clone(),
            media_type: Some("image/png".to_string()),
            title: Some("Existing media manifest".to_string()),
            credential_profile: ContentCredentialProfile::Public,
            ..Default::default()
        })
        .await?;
    let original_digest = sha256_file(&image)?;
    let original_manifest_id = signed.manifest_id.clone();

    let output = dir.path().join("report.md");
    let node = Node::Article(Article::new(vec![Block::Figure(Figure {
        label: Some("1".to_string()),
        caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Existing static result."),
        )]))]),
        content: vec![Block::ImageObject(ImageObject::new(
            image.to_string_lossy().to_string(),
        ))],
        ..Default::default()
    })]));

    to_path_with_info(
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

    assert_eq!(
        sha256_file(&image)?,
        original_digest,
        "existing media manifest should be used without re-signing the media file"
    );

    let verifier = CredentialVerifier::new();
    let image_manifest = verifier.inspect_asset(&image, None).await?;
    assert_eq!(
        image_manifest["active_manifest"].as_str(),
        original_manifest_id.as_deref()
    );

    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let active = document_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    let component = document_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("document ingredients")
        .iter()
        .find(|ingredient| ingredient["relationship"] == "componentOf")
        .expect("component ingredient");
    assert_eq!(
        component["active_manifest"].as_str(),
        original_manifest_id.as_deref(),
        "document component should link to the existing media manifest"
    );
    assert!(
        component["thumbnail"]["identifier"]
            .as_str()
            .is_some_and(|identifier| identifier.contains("c2pa.thumbnail.ingredient")),
        "existing static media component should have a parent-side ingredient thumbnail: {component:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_pdf_embeds_component_ingredients_without_side_assets() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let output = dir.path().join("report.pdf");
    let static_image = dir.path().join("static.png");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.png"),
        &static_image,
    )?;
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.pdf"),
        &output,
    )?;

    let mut chunk = CodeChunk::new("plot(1)".into());
    chunk.programming_language = Some("r".to_string());
    chunk.outputs = Some(vec![Node::ImageObject(ImageObject::new(
        PNG_DATA_URI.to_string(),
    ))]);
    let node = Node::Article(Article {
        content: vec![
            Block::Heading(Heading::new(
                1,
                vec![Inline::Text(Text::from("PDF Report"))],
            )),
            Block::Figure(Figure {
                label: Some("1".to_string()),
                caption: Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                    Text::from("Static PDF result."),
                )]))]),
                content: vec![Block::ImageObject(ImageObject::new(
                    static_image.to_string_lossy().to_string(),
                ))],
                ..Default::default()
            }),
            Block::CodeChunk(chunk),
        ],
        ..Default::default()
    });

    let mut info = EncodeInfo::none();
    sign_encoded_export(ExportSigningRequest {
        node: &node,
        codec_name: "pdf",
        output_path: &output,
        source_path: None,
        source_ranges: None,
        media_type_hint: Some("application/pdf".to_string()),
        credential_profile: ContentCredentialProfile::Public,
        info: &mut info,
    })
    .await?;

    let document_asset = info
        .assets
        .iter()
        .find(|asset| asset.role.as_deref() == Some("document"))
        .expect("document asset");
    assert!(document_asset.signed);
    assert_eq!(document_asset.manifest_kind.as_deref(), Some("embedded"));
    assert!(document_asset.sidecar_path.is_none());

    let verifier = CredentialVerifier::new();
    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let active = document_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    assert_eq!(
        document_manifest["manifests"][active]["title"], "PDF Report",
        "document manifest title should use the article title, not the output file name"
    );
    let assertions = document_manifest["manifests"][active]["assertions"]
        .as_array()
        .expect("assertions");
    let metadata = assertions
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.metadata")
        .expect("metadata assertion");
    assert!(
        metadata["data"].get("dc:title").is_none(),
        "dc:title is not permitted in c2pa.metadata"
    );
    assert_eq!(
        metadata["data"]["xmp:Label"], "report.pdf",
        "the rendition label should remain the output file name"
    );
    let provenance = assertions
        .iter()
        .find(|assertion| assertion["label"] == "org.stencila.provenance")
        .expect("Stencila provenance assertion");
    assert_eq!(
        provenance["data"]["rootNode"]["title"], "PDF Report",
        "provenance should capture a top-level heading as the article title"
    );
    let document_ingredients = document_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("document ingredients");
    let components = document_ingredients
        .iter()
        .filter(|ingredient| ingredient["relationship"] == "componentOf")
        .collect::<Vec<_>>();
    assert_eq!(
        components.len(),
        2,
        "expected static and embedded components"
    );
    let component_labels = components
        .iter()
        .filter_map(|ingredient| ingredient["label"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        component_labels.len(),
        components.len(),
        "component ingredient labels should be unique: {components:#?}"
    );
    assert!(
        components
            .iter()
            .any(|component| component["title"] == "Figure 1: Static PDF result."),
        "static media should be represented as a component: {components:#?}"
    );
    for component in components {
        assert_eq!(component["format"], "image/png");
        assert!(
            component["active_manifest"].is_string(),
            "PDF component should link to a temporary signed media manifest: {component:#?}"
        );
        assert!(
            component["validation_results"]["activeManifest"].is_object(),
            "PDF component should validate through its child manifest: {component:#?}"
        );
    }
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output,
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    assert!(
        report.manifest.valid,
        "embedded PDF manifest should validate without metadata errors: {report:?}"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_sign_smd_with_stencila_media_type() -> Result<()> {
    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

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
            require_repro_exact: false,
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
    init_local_signing_identity(true)?;

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
    assert!(
        info.assets
            .iter()
            .any(|asset| asset.path == document_sidecar)
    );

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;

    let assertion = report
        .provenance
        .assertion
        .as_ref()
        .expect("parsed Stencila assertion");

    assert_eq!(assertion.asset.role.as_deref(), Some("document-export"));
    assert_eq!(assertion.root_node.node_type, "Article");
    assert_eq!(assertion.root_node.title.as_deref(), Some("My Title"));
    // Root nodes do not record a `nodeId`: the stabilized path is empty by
    // definition, and `nodeType` already conveys the kind of node.
    assert!(assertion.root_node.node_id.is_none());
    assert_eq!(assertion.producer.codec.as_deref(), Some("markdown"));
    assert_eq!(assertion.producer.name, "Stencila");

    let source = assertion.source.as_ref().expect("source snapshot recorded");
    assert_eq!(source.path.as_deref(), Some("article.smd"));
    assert_eq!(source.dirty, Some(false));
    assert_eq!(
        source.commit.as_deref().map(str::len),
        Some(40),
        "expected commit SHA, got {source:?}"
    );

    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let active = document_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    let ingredients = document_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("document ingredients");
    let source_ingredient = ingredients
        .iter()
        .find(|ingredient| ingredient["format"] == "text/smd")
        .expect("source document ingredient");
    assert_eq!(source_ingredient["relationship"], "inputTo");
    let source_manifest_id = source_ingredient["active_manifest"]
        .as_str()
        .expect("source ingredient manifest id");
    let source_manifest_assertions =
        document_manifest["manifests"][source_manifest_id]["assertions"]
            .as_array()
            .expect("source ingredient manifest assertions");
    let source_actions = source_manifest_assertions
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.actions.v2")
        .and_then(|assertion| assertion["data"]["actions"].as_array())
        .expect("source ingredient manifest actions");
    assert_eq!(source_actions[0]["action"], "c2pa.created");
    assert!(
        source_actions[0]["when"]
            .as_str()
            .is_some_and(|when| !when.is_empty()),
        "source input ingredient created action should use source creation timestamp: {source_actions:#?}"
    );

    let actions = document_manifest["manifests"][active]["assertions"]
        .as_array()
        .expect("assertions")
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.actions.v2")
        .and_then(|assertion| assertion["data"]["actions"].as_array())
        .expect("actions");
    assert_eq!(actions[0]["action"], "c2pa.created");
    assert!(
        actions
            .iter()
            .all(|action| action["action"] != "c2pa.opened"),
        "document source should be an input, not an opened parent: {actions:#?}"
    );

    Ok(())
}

/// 1×1 transparent PNG used to drive the markdown codec's media-extraction
/// path so the test can exercise per-asset signing of an extracted figure.
const PNG_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[tokio::test]
async fn credentials_per_asset_snapshots_split_document_and_chunk_execution() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let repo = dir.path();
    run_git(repo, &["init", "-q", "-b", "main"]);
    run_git(repo, &["config", "user.email", "test@example.com"]);
    run_git(repo, &["config", "user.name", "Test"]);
    run_git(repo, &["config", "commit.gpgsign", "false"]);
    run_git(
        repo,
        &["remote", "add", "origin", "https://github.com/example/repo"],
    );
    let source_path = dir.path().join("analysis.smd");
    fs::write(&source_path, "# Analysis\n\n```python\nvalue = 1\n```\n")?;
    fs::write(dir.path().join("uv.lock"), "version = 1\n")?;
    fs::write(dir.path().join("data.csv"), "x\n1\n")?;
    run_git(repo, &["add", "analysis.smd"]);
    run_git(repo, &["commit", "-q", "-m", "init"]);

    let output = dir.path().join("analysis.md");
    let mut chunk = CodeChunk::new("value = 1".into());
    chunk.programming_language = Some("python".to_string());
    chunk.outputs = Some(vec![Node::ImageObject(ImageObject::new(
        PNG_DATA_URI.to_string(),
    ))]);
    chunk.options.execution_digest = Some(CompilationDigest::new(42));
    chunk.options.execution_status = Some(ExecutionStatus::Succeeded);
    chunk.options.execution_count = Some(3);
    chunk.options.execution_duration = Some(Duration::new(250, TimeUnit::Millisecond));
    chunk.options.execution_instance = Some("python-main".to_string());
    chunk.options.execution_dependencies = Some(vec![ExecutionDependency::new(
        ExecutionDependencyRelation::Reads,
        "File".to_string(),
        "data.csv".to_string(),
    )]);
    chunk.options.execution_messages = Some(vec![ExecutionMessage::new(
        MessageLevel::Warning,
        "cached data was used".to_string(),
    )]);
    let node = Node::Article(Article {
        title: Some(vec![Inline::Text(Text::from("Analysis"))]),
        content: vec![Block::CodeChunk(chunk)],
        ..Default::default()
    });
    // The codec sees a stabilized tree when credentials are requested, so the
    // CodeChunk at `content[0]` records its stabilized path-based identifier
    // rather than the random per-load UID.
    let chunk_id = "cdc_content-0".to_string();

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

    // The markdown codec extracts data-URI media into <output>.media. Find the
    // figure that was extracted so we can verify its per-asset credentials.
    let figure_asset = info
        .assets
        .iter()
        .find(|asset| {
            asset
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        })
        .expect("extracted figure asset");
    assert_eq!(figure_asset.role.as_deref(), Some("computational-output"));
    assert_eq!(figure_asset.node_type.as_deref(), Some("CodeChunk"));
    assert_eq!(figure_asset.node_id.as_deref(), Some(chunk_id.as_str()));

    let verifier = CredentialVerifier::new();

    // Document-level snapshot: the Article carries no execution metadata of
    // its own, so the document export's snapshot has no execution facts.
    let document_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: output.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    let document = document_report
        .provenance
        .assertion
        .as_ref()
        .expect("document assertion");
    assert_eq!(document.asset.role.as_deref(), Some("document-export"));
    assert!(document.executed_node.is_none());
    assert!(document.output_node.is_none());
    assert!(
        document.execution.is_none(),
        "document snapshot should not aggregate chunk execution: {:?}",
        document.execution
    );

    let document_manifest = verifier.inspect_asset(&output, None).await?;
    let document_active = document_manifest["active_manifest"]
        .as_str()
        .expect("document active manifest");
    let document_ingredients = document_manifest["manifests"][document_active]["ingredients"]
        .as_array()
        .expect("document ingredients");
    let document_environment_ingredients: Vec<_> = document_ingredients
        .iter()
        .filter(|ingredient| {
            ingredient["relationship"] == "inputTo"
                && ingredient["title"] == "Execution environment"
        })
        .collect();
    assert_eq!(
        document_environment_ingredients.len(),
        1,
        "document manifest should carry one execution environment input: {document_ingredients:#?}"
    );
    let document_environment = document_environment_ingredients[0];
    assert_eq!(document_environment["format"], "application/json");
    let document_environment_description = document_environment["description"]
        .as_str()
        .expect("environment ingredient description");
    assert!(
        document_environment_description.starts_with("Execution environment:"),
        "environment ingredient description should be informative: {document_environment:#?}"
    );
    assert!(
        document_environment_description.contains("OS ")
            && document_environment_description.contains("architecture ")
            && document_environment_description.contains("runtimes ")
            && document_environment_description.contains("lockfiles "),
        "environment ingredient description should summarize reproducibility fields: {document_environment_description}"
    );
    assert!(
        document_environment["validation_results"]["activeManifest"].is_object(),
        "environment ingredient should carry real validation results: {document_environment:#?}"
    );
    let environment_manifest_id = document_environment["active_manifest"]
        .as_str()
        .expect("environment ingredient manifest id");
    let environment_manifest_assertions =
        document_manifest["manifests"][environment_manifest_id]["assertions"]
            .as_array()
            .expect("environment ingredient manifest assertions");
    let environment_provenance = environment_manifest_assertions
        .iter()
        .find(|assertion| assertion["label"] == "org.stencila.provenance")
        .expect("environment provenance assertion");
    assert_eq!(
        environment_provenance["data"]["rootNode"]["nodeType"],
        "EnvironmentRecord"
    );
    let document_actions = document_manifest["manifests"][document_active]["assertions"]
        .as_array()
        .expect("document assertions")
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.actions.v2")
        .and_then(|assertion| assertion["data"]["actions"].as_array())
        .expect("document actions");
    let document_created = document_actions
        .iter()
        .find(|action| action["action"] == "c2pa.created")
        .expect("document created action");
    assert_eq!(
        document_created["parameters"]["ingredients"]
            .as_array()
            .map(Vec::len),
        Some(2),
        "document created action should reference source and environment inputs: {document_created:#?}"
    );

    // Per-asset snapshot for the extracted figure: subject is the CodeChunk,
    // so the chunk's execution facts land here, not on the document.
    let figure_report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: figure_asset.path.clone(),
            require_trusted_signer: false,
            require_stencila_assertion: true,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    let figure = figure_report
        .provenance
        .assertion
        .as_ref()
        .expect("figure assertion");
    assert_eq!(figure.asset.role.as_deref(), Some("computational-output"));
    assert_eq!(figure.root_node.node_type, "Article");
    let executed_node = figure.executed_node.as_ref().expect("executed node");
    assert_eq!(executed_node.node_type, "CodeChunk");
    assert_eq!(
        executed_node.programming_language.as_deref(),
        Some("python")
    );
    let source_range = executed_node
        .source_range
        .as_ref()
        .expect("executed node source range");
    assert_eq!(source_range.start_line, 3);
    assert_eq!(source_range.start_column, 1);
    assert_eq!(source_range.end_line, 6);
    assert_eq!(source_range.end_column, 1);
    let output_node = figure.output_node.as_ref().expect("output node");
    assert_eq!(output_node.node_type, "ImageObject");
    assert!(
        output_node.content_url.is_none(),
        "data URI output content should be represented by the signed asset, not duplicated in the assertion"
    );
    let figure_json = serde_json::to_string(figure).expect("serialize figure assertion");
    assert!(
        !figure_json.contains("data:image/"),
        "figure assertion should not embed data URI payloads"
    );

    let execution = figure
        .execution
        .as_ref()
        .expect("figure execution snapshot");
    assert_eq!(execution.status.as_deref(), Some("succeeded"));
    assert_eq!(execution.duration_ms, Some(250));
    assert_eq!(execution.count, Some(3));
    assert_eq!(
        execution
            .digests
            .as_ref()
            .and_then(|digests| digests.state.as_deref()),
        Some("stencila:000000000000002a")
    );
    assert_eq!(
        execution
            .kernel
            .as_ref()
            .and_then(|kernel| kernel.name.as_deref()),
        Some("python-main")
    );
    assert_eq!(
        execution
            .kernel
            .as_ref()
            .and_then(|kernel| kernel.language.as_deref()),
        Some("python")
    );
    assert_eq!(execution.dependencies.len(), 1);
    assert_eq!(
        execution.dependencies[0].node_id.as_deref(),
        Some("data.csv")
    );
    assert_eq!(execution.messages.len(), 1);
    assert_eq!(
        execution.messages[0].message.as_deref(),
        Some("cached data was used")
    );

    let figure_manifest = verifier.inspect_asset(&figure_asset.path, None).await?;
    let active = figure_manifest["active_manifest"]
        .as_str()
        .expect("active manifest");
    let ingredients = figure_manifest["manifests"][active]["ingredients"]
        .as_array()
        .expect("figure ingredients");
    assert_eq!(
        ingredients.len(),
        1,
        "figure manifest should reference only the executed code ingredient"
    );
    for ingredient in ingredients {
        assert!(
            ingredient["active_manifest"].is_string(),
            "ingredient should link to its own manifest: {ingredient:#?}"
        );
    }
    assert!(
        ingredients
            .iter()
            .all(|ingredient| ingredient["relationship"] != "parentOf"),
        "executed figure manifests should not reference the wider source document: {ingredients:#?}"
    );

    let code = ingredients
        .iter()
        .find(|ingredient| {
            ingredient["relationship"] == "inputTo" && ingredient["format"] == "text/x-python"
        })
        .expect("executed code ingredient");
    assert_eq!(code["format"], "text/x-python");
    assert_eq!(code["title"], "Python CodeChunk content-0");
    assert!(
        code["informational_URI"]
            .as_str()
            .is_some_and(|uri| uri.ends_with("analysis.smd#L3-L5")),
        "code ingredient URI should point at source lines: {code:#?}"
    );
    assert!(
        code["validation_results"]["activeManifest"].is_object(),
        "code ingredient should carry real validation results: {code:#?}"
    );
    let code_manifest_id = code["active_manifest"]
        .as_str()
        .expect("code ingredient manifest id");
    let code_manifest_assertions = figure_manifest["manifests"][code_manifest_id]["assertions"]
        .as_array()
        .expect("code ingredient manifest assertions");
    let code_actions = code_manifest_assertions
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.actions.v2")
        .and_then(|assertion| assertion["data"]["actions"].as_array())
        .expect("code ingredient manifest actions");
    assert_eq!(code_actions[0]["action"], "c2pa.created");
    assert!(
        code_actions[0]["when"]
            .as_str()
            .is_some_and(|when| !when.is_empty()),
        "executed code input ingredient created action should use source creation timestamp: {code_actions:#?}"
    );

    let actions = figure_manifest["manifests"][active]["assertions"]
        .as_array()
        .expect("assertions")
        .iter()
        .find(|assertion| assertion["label"] == "c2pa.actions.v2")
        .and_then(|assertion| assertion["data"]["actions"].as_array())
        .expect("actions");
    assert_eq!(actions[0]["action"], "c2pa.created");
    assert!(
        actions
            .iter()
            .all(|action| action["action"] != "c2pa.opened"),
        "executed figure manifests should not include an opened action: {actions:#?}"
    );
    assert!(
        actions.iter().any(|action| {
            action["action"] == "org.stencila.executed"
                && action["parameters"]["org.stencila.execution"]["status"] == "succeeded"
                && action["parameters"]["org.stencila.execution"]["durationMs"] == 250
        }),
        "expected org.stencila.executed action with execution details: {actions:#?}"
    );
    let executed_action = actions
        .iter()
        .find(|action| action["action"] == "org.stencila.executed")
        .expect("executed action");
    assert_eq!(
        executed_action["parameters"]["ingredients"]
            .as_array()
            .map(Vec::len),
        Some(1),
        "executed action should reference the executed code input: {executed_action:#?}"
    );
    let figure_manifest_json =
        serde_json::to_string(&figure_manifest).expect("serialize figure manifest");
    assert!(
        !figure_manifest_json.contains("ingredient.unknownProvenance"),
        "executed code ingredient should carry provenance: {figure_manifest_json}"
    );

    // Environment fields hold for both snapshots; assert on the figure's.
    let environment = figure.environment.as_ref().expect("environment");
    assert!(environment.os.is_some());
    assert!(environment.architecture.is_some());
    assert!(environment.runtimes.iter().any(|runtime| {
        runtime.name.as_deref() == Some("stencila") && runtime.version.is_some()
    }));
    assert!(
        environment.lockfiles.iter().any(|lockfile| {
            lockfile
                .path
                .as_deref()
                .is_some_and(|path| path.ends_with("uv.lock"))
                && lockfile
                    .digest
                    .as_deref()
                    .is_some_and(|digest| digest.starts_with("sha256:"))
        }),
        "lockfiles: {:?}",
        environment.lockfiles
    );

    Ok(())
}

#[tokio::test]
async fn credentials_dirty_source_records_patch_digest() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

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
            require_repro_exact: false,
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
async fn credentials_untracked_source_is_dirty_without_commit() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

    let dir = TempDir::new()?;
    let repo = dir.path();
    run_git(repo, &["init", "-q", "-b", "main"]);
    run_git(repo, &["config", "user.email", "test@example.com"]);
    run_git(repo, &["config", "user.name", "Test"]);
    run_git(repo, &["config", "commit.gpgsign", "false"]);
    fs::write(repo.join("README.md"), "repo\n")?;
    run_git(repo, &["add", "README.md"]);
    run_git(repo, &["commit", "-q", "-m", "init"]);

    let source_path = repo.join("draft.smd");
    fs::write(&source_path, "# Draft\n\nbody\n")?;

    let output = dir.path().join("draft.md");
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
            from_path: Some(source_path),
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
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;
    let source = report
        .provenance
        .assertion
        .as_ref()
        .and_then(|assertion| assertion.source.as_ref())
        .expect("source snapshot");

    assert_eq!(source.path.as_deref(), Some("draft.smd"));
    assert_eq!(source.dirty, Some(true));
    assert!(
        source.commit.is_none(),
        "untracked files are not present in HEAD and should not claim a commit"
    );

    Ok(())
}

#[tokio::test]
async fn credentials_public_profile_redacts_dirty_patch_digest() -> Result<()> {
    if !git_available() {
        return Ok(());
    }

    let _config = set_isolated_config_dir();
    init_local_signing_identity(true)?;

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
            require_repro_exact: false,
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
