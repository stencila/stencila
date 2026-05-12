//! `stencila credentials init` — create a local signing identity.

use clap::Args;
use eyre::Result;
use stencila_cli_utils::message;

use crate::signer;

/// Generate a local self-signed signing identity.
///
/// Creates `local-signing-cert.pem` and `local-signing-key.pem` under
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
        let result = signer::init_local_signing_identity(self.force)?;

        if result.created {
            message!("✅ Created local signing identity");
        } else {
            message!("ℹ️  Reusing existing local signing identity (pass --force to regenerate)");
        }

        message!("");
        message!("   Cert: `{}`", result.cert_path.display());
        message!("   Key:  `{}`", result.key_path.display());
        message!("   CN:   `{}`", result.common_name);
        message!("");

        message!("⚠️ This identity is self-signed and not trusted by public verifiers.");

        Ok(())
    }
}
