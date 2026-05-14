//! `stencila credentials inspect` — print the full C2PA manifest.

use std::path::PathBuf;

use clap::Args;
use eyre::Result;
use stencila_cli_utils::{AsFormat, Code, ToStdout};

use crate::verifier::CredentialVerifier;

use super::resolve_trust_anchors;

/// Print the full C2PA manifest data attached to an asset.
#[derive(Debug, Args)]
pub struct Cli {
    /// Path to the asset to inspect.
    asset: PathBuf,

    /// Output format.
    #[arg(long, short, default_value = "yaml")]
    r#as: AsFormat,

    /// PEM bundle of C2PA trust anchors for local signer trust checks.
    ///
    /// Can also be supplied with `STENCILA_CREDENTIALS_TRUST_ANCHORS`.
    #[arg(long, value_name = "PEM")]
    trust_anchors: Option<PathBuf>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let trust_anchors = resolve_trust_anchors(self.trust_anchors).await?;
        let verifier = CredentialVerifier::new();
        let value = verifier.inspect_asset(&self.asset, trust_anchors).await?;
        Code::new_from(self.r#as.into(), &value)?.to_stdout();
        Ok(())
    }
}
