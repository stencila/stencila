//! `stencila credentials sign` — sign an existing asset with a C2PA manifest.

use std::path::PathBuf;

use clap::Args;
use eyre::Result;
use serde::Serialize;
use stencila_cli_utils::{AsFormat, Code, ToStdout, message};

use crate::{
    producer::{CredentialProducer, ManifestKind, SignAssetRequest, SignedAsset},
    signer::CredentialSignerConfig,
};

/// Sign an asset with a C2PA manifest carrying the
/// `org.stencila.provenance` assertion.
///
/// For PNG, JPEG, WebP, SVG, and PDF the manifest is embedded directly in the
/// asset. For other formats the manifest is written to a `.c2pa` sidecar file
/// next to the asset.
#[derive(Debug, Args)]
pub struct Cli {
    /// Path to the asset to sign.
    input: PathBuf,

    /// Where to write the signed asset (defaults to in-place).
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Path to the signing certificate (PEM).
    #[arg(long)]
    cert: Option<PathBuf>,

    /// Path to the signing private key (PEM).
    #[arg(long)]
    key: Option<PathBuf>,

    /// Timestamp authority URL to use when signing.
    ///
    /// Can also be supplied with `STENCILA_CREDENTIALS_TSA_URL`.
    #[arg(long, value_name = "URL")]
    tsa_url: Option<String>,

    /// Title to record in the manifest. Defaults to the asset filename.
    #[arg(long)]
    title: Option<String>,

    /// Output format. Defaults to a human-readable summary.
    ///
    /// Use `json`, `yaml`, or `toml` to emit a structured signing report
    /// suitable for evidence collection or piping into other tools.
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

/// Structured result of a `stencila credentials sign` invocation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignReport {
    path: PathBuf,
    manifest_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar_path: Option<PathBuf>,
    side_assets: Vec<PathBuf>,
    warnings: Vec<String>,
    profile: &'static str,
    signing_mode: &'static str,
    assertion_label: &'static str,
    assertion_schema: &'static str,
    media_type: String,
    source_digest: String,
    signed_asset_digest: String,
}

impl From<&SignedAsset> for SignReport {
    fn from(signed: &SignedAsset) -> Self {
        Self {
            path: signed.asset_path.clone(),
            manifest_kind: signed.manifest_kind.label(),
            manifest_id: signed.manifest_id.clone(),
            sidecar_path: signed.sidecar_path.clone(),
            side_assets: signed.sidecar_path.iter().cloned().collect(),
            warnings: signed.warnings.clone(),
            profile: signed.credential_profile.label(),
            signing_mode: signed.signing_mode.label(),
            assertion_label: signed.assertion_label,
            assertion_schema: signed.assertion_schema,
            media_type: signed.media_type.clone(),
            source_digest: signed.source_digest.clone(),
            signed_asset_digest: signed.signed_asset_digest.clone(),
        }
    }
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let signer_config =
            CredentialSignerConfig::resolve_with_options(self.cert, self.key, self.tsa_url)?;
        let producer = CredentialProducer::new(signer_config);

        let request = SignAssetRequest {
            input_path: self.input,
            output_path: self.output,
            title: self.title,
            ..Default::default()
        };
        let signed_asset = producer.sign_exported_asset(request).await?;

        match self.r#as {
            Some(format) => {
                let report = SignReport::from(&signed_asset);
                Code::new_from(format.into(), &report)?.to_stdout();
            }
            None => print_human_summary(&signed_asset),
        }

        Ok(())
    }
}

fn print_human_summary(signed_asset: &SignedAsset) {
    match signed_asset.manifest_kind {
        ManifestKind::Embedded => {
            message!(
                "✅ Signed asset (embedded): `{}`",
                signed_asset.asset_path.display()
            );
        }
        ManifestKind::Sidecar => {
            message!(
                "✅ Signed asset (sidecar): `{}`",
                signed_asset.asset_path.display()
            );
            if let Some(sidecar) = &signed_asset.sidecar_path {
                message!("   Sidecar manifest: `{}`", sidecar.display());
            }
        }
    }

    message!("");
    if let Some(manifest_id) = &signed_asset.manifest_id {
        message!("   Manifest ID: `{}`", manifest_id);
    }
    message!("   Profile:  `{}`", signed_asset.credential_profile.label());
    message!("   Signer:   `{}`", signed_asset.signing_mode.label());
    message!("   Assertion: `{}`", signed_asset.assertion_label);
    message!("   Schema:    `{}`", signed_asset.assertion_schema);
    message!("   Signed digest: `{}`", signed_asset.signed_asset_digest);
    if signed_asset.source_digest != signed_asset.signed_asset_digest {
        message!("   Source digest: `{}`", signed_asset.source_digest);
    }
    for warning in &signed_asset.warnings {
        message!("   Warning: {}", warning);
    }
}
