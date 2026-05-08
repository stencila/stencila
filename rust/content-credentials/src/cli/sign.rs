//! `stencila credentials sign` — sign an existing asset with a C2PA manifest.

use std::path::PathBuf;

use clap::Args;
use eyre::Result;
use stencila_cli_utils::message;

use crate::{
    producer::{CredentialProducer, ManifestKind, SignAssetRequest},
    signer::CredentialSignerConfig,
};

/// Sign an asset with a C2PA manifest carrying the
/// `org.stencila.provenance` assertion.
///
/// For PNG, JPEG, WebP, and SVG the manifest is embedded directly in the
/// asset. For other formats (including PDF) the manifest is written to a
/// `.c2pa` sidecar file next to the asset.
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

    /// Title to record in the manifest. Defaults to the asset filename.
    #[arg(long)]
    title: Option<String>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let signer_config = CredentialSignerConfig::resolve(self.cert, self.key)?;
        let producer = CredentialProducer::new(signer_config);

        let request = SignAssetRequest {
            input_path: self.input,
            output_path: self.output,
            title: self.title,
        };
        let signed_asset = producer.sign_exported_asset(request).await?;

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
        message!("   Assertion: `{}`", signed_asset.assertion_label);
        message!("   Schema:    `{}`", signed_asset.assertion_schema);
        message!("   Signed digest: `{}`", signed_asset.signed_asset_digest);
        if signed_asset.source_digest != signed_asset.signed_asset_digest {
            message!("   Source digest: `{}`", signed_asset.source_digest);
        }

        Ok(())
    }
}
