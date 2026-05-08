//! `stencila credentials init` — create a local dev signing identity.

use clap::Args;
use eyre::Result;
use stencila_cli_utils::message;

use crate::signer;

/// Generate a local self-signed signing identity for development.
///
/// Creates `dev-cert.pem` and `dev-key.pem` under
/// `<config>/credentials/`. The certificate is **not** trusted by
/// third-party verifiers; use it for local and internal workflows only.
#[derive(Debug, Args)]
pub struct Cli {
    /// Overwrite an existing dev cert and key.
    #[arg(long)]
    force: bool,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let result = signer::init_dev_cert(self.force)?;

        if result.created {
            message!("✅ Created dev signing identity");
        } else {
            message!("ℹ️  Reusing existing dev signing identity (pass --force to regenerate)");
        }

        message!("");
        message!("   Cert: `{}`", result.cert_path.display());
        message!("   Key:  `{}`", result.key_path.display());
        message!("   CN:   `{}`", result.common_name);
        message!("");

        message!("⚠️ This identity is untrusted outside of local development.");

        Ok(())
    }
}
