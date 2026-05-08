//! `stencila credentials inspect` — print the full C2PA manifest as JSON.

use std::path::PathBuf;

use clap::Args;
use eyre::Result;
use stencila_cli_utils::{AsFormat, Code, ToStdout};

use crate::verifier::CredentialVerifier;

/// Print the full C2PA manifest data attached to an asset.
#[derive(Debug, Args)]
pub struct Cli {
    /// Path to the asset to inspect.
    asset: PathBuf,

    /// Output format.
    #[arg(long, short, default_value = "json")]
    r#as: AsFormat,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let verifier = CredentialVerifier::new();
        let value = verifier.inspect_asset(&self.asset).await?;
        Code::new_from(self.r#as.into(), &value)?.to_stdout();
        Ok(())
    }
}
