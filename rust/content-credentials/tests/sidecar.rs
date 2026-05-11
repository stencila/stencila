//! Sign + verify a non-embeddable format (PDF) via a sidecar `.c2pa`.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use c2pa::Builder;
use serde_json::json;
use stencila_content_credentials::{
    CredentialProducer, CredentialVerifier, Error, ManifestKind, Result, SignAssetRequest,
    VerifyAssetRequest, init_dev_cert, signer::CredentialSignerConfig,
};
use tempfile::{NamedTempFile, TempDir};

mod common;

/// Exercises the sidecar path by signing and verifying a PDF with a `.c2pa` manifest.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sign_and_verify_pdf_sidecar() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset = tmp.path().join("doc.pdf");
    fs::write(&asset, MINIMAL_PDF).expect("write fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset.clone(),
            ..Default::default()
        })
        .await
        .expect("sign");

    assert_eq!(signed.manifest_kind, ManifestKind::Sidecar);
    assert!(
        signed
            .manifest_id
            .as_deref()
            .is_some_and(|id| id.starts_with("urn:c2pa:")),
        "manifest id should be read from sidecar after signing: {signed:?}"
    );
    assert!(
        signed.warnings.is_empty(),
        "signing should not produce warnings: {:?}",
        signed.warnings
    );
    let sidecar = signed.sidecar_path.as_ref().expect("sidecar present");
    assert!(sidecar.exists(), "sidecar file written");
    assert!(sidecar.to_string_lossy().ends_with("doc.c2pa"));

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await
        .expect("verify");

    assert!(report.manifest.present);
    assert!(report.manifest.from_sidecar, "verified via sidecar");
    assert!(report.asset_binding.valid);
    assert!(report.provenance.attested);
}

/// Ensures a `.c2pa` output path is rejected instead of overwriting the asset with its sidecar.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_output_cannot_equal_asset_output() {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true).expect("init dev cert");

    let tmp = TempDir::new().expect("tmp");
    let asset = tmp.path().join("doc.pdf");
    fs::copy(fixture_path(), &asset).expect("copy fixture");

    let signer = CredentialSignerConfig::resolve(None, None).expect("resolve signer");
    let producer = CredentialProducer::new(signer);
    let err = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset,
            output_path: Some(tmp.path().join("doc.c2pa")),
            ..Default::default()
        })
        .await
        .expect_err("conflicting output should fail");

    assert!(matches!(err, Error::OutputSidecarConflict(_)));
}

/// Ensures sidecar signing persists the asset stream c2pa signed, not the original bytes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_signing_persists_rewritten_asset() -> Result<()> {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true)?;

    let tmp = TempDir::new()?;
    let asset = tmp.path().join("embedded.gif");
    fs::write(&asset, MINIMAL_GIF)?;
    embed_manifest_in_place(&asset)?;

    let embedded_len = fs::metadata(&asset)?.len();
    assert!(
        embedded_len > MINIMAL_GIF.len() as u64,
        "test setup should create a GIF with embedded credentials"
    );

    let signer = CredentialSignerConfig::resolve(None, None)?;
    let producer = CredentialProducer::new(signer);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset.clone(),
            ..Default::default()
        })
        .await?;

    assert_eq!(signed.manifest_kind, ManifestKind::Sidecar);
    assert!(
        fs::metadata(&asset)?.len() < embedded_len,
        "sidecar signing should strip the old embedded manifest from the asset"
    );

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;

    assert!(report.manifest.from_sidecar, "verified via sidecar");
    assert!(
        report.asset_binding.valid,
        "sidecar must bind to the persisted asset bytes: {report:?}"
    );

    Ok(())
}

/// Ensures embedded manifests win for SDK-supported formats even when Stencila signs them via sidecar.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_sdk_supported_format_wins_over_stale_sidecar() -> Result<()> {
    let _guard = common::set_isolated_config_dir();
    let _ = init_dev_cert(true)?;

    let tmp = TempDir::new()?;
    let asset = tmp.path().join("embedded.gif");
    fs::write(&asset, MINIMAL_GIF)?;
    embed_manifest_in_place(&asset)?;
    fs::write(asset.with_extension("c2pa"), b"stale sidecar")?;

    let verifier = CredentialVerifier::new();
    let report = verifier
        .verify_asset(VerifyAssetRequest {
            asset_path: asset,
            require_trusted_signer: false,
            require_stencila_assertion: false,
            require_repro_exact: false,
            trust_anchors: None,
        })
        .await?;

    assert!(report.manifest.present, "embedded manifest should be read");
    assert!(report.manifest.valid, "embedded manifest should validate");
    assert!(
        !report.manifest.from_sidecar,
        "stale sidecar must not shadow embedded manifest: {report:?}"
    );
    assert!(report.asset_binding.valid, "asset binding should validate");

    Ok(())
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sample.pdf")
}

fn embed_manifest_in_place(asset: &Path) -> Result<()> {
    let signer = CredentialSignerConfig::resolve(None, None)?.create_signer()?;
    let definition = json!({
        "claim_generator_info": [{
            "name": "Stencila test",
            "version": stencila_content_credentials::schema::PROVENANCE_SCHEMA,
        }],
        "title": "Embedded GIF",
        "format": "image/gif",
        "assertions": [
            {
                "label": "c2pa.actions.v2",
                "data": { "actions": [{ "action": "c2pa.created" }] }
            }
        ]
    });

    #[allow(deprecated)]
    let mut builder = Builder::from_json(&definition.to_string())?;
    builder.set_no_embed(false);

    let parent = asset.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)?;
    {
        let mut source = File::open(asset)?;
        let file = tmp.as_file_mut();
        builder.sign(signer.as_ref(), "image/gif", &mut source, file)?;
        file.flush()?;
    }
    tmp.persist(asset).map_err(|err| Error::Io(err.error))?;

    Ok(())
}

const MINIMAL_GIF: &[u8] = b"GIF89a\
\x01\x00\x01\x00\x80\x00\x00\
\x00\x00\x00\xff\xff\xff\
!\xf9\x04\x01\x00\x00\x00\x00\
,\x00\x00\x00\x00\x01\x00\x01\x00\x00\
\x02\x02D\x01\x00;";

const MINIMAL_PDF: &[u8] = b"%PDF-1.1
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] >>
endobj
xref
0 4
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
trailer
<< /Root 1 0 R /Size 4 >>
startxref
184
%%EOF
";
