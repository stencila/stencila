//! `stencila credentials` subcommand.

use std::{env, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use eyre::Result;
use stencila_cli_utils::color_print::cstr;

mod init;
mod inspect;
mod sign;
mod trust;
mod verify;

const ENV_TRUST_ANCHORS: &str = "STENCILA_CREDENTIALS_TRUST_ANCHORS";

/// Manage and inspect Stencila C2PA Content Credentials
#[derive(Debug, Parser)]
#[command(visible_alias = "creds", after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Generate a local signing identity (untrusted)</dim>
  <b>stencila credentials init</b>

  <dim># Sign an existing image with a C2PA manifest</dim>
  <b>stencila credentials sign</b> <g>figure.png</g>

  <dim># Sign with an explicit title</dim>
  <b>stencila credentials sign</b> <g>figure.png</g> <c>--title</c> <y>\"Figure 4\"</y>

  <dim># Verify an asset and show the four-status report</dim>
  <b>stencila credentials verify</b> <g>figure.png</g>

  <dim># Verify and require a Stencila assertion to be present</dim>
  <b>stencila credentials verify</b> <g>figure.png</g> <c>--require</c> <y>stencila-assertion</y>

  <dim># Verify and emit the full report as JSON</dim>
  <b>stencila credentials verify</b> <g>figure.png</g> <c>--as</c> <y>json</y>

  <dim># Inspect the full manifest as JSON</dim>
  <b>stencila credentials inspect</b> <g>figure.png</g>

  <dim># Refresh the official C2PA trust list cache</dim>
  <b>stencila credentials trust refresh</b>

<bold><b>Trust</b></bold>
  Identities created by <c>credentials init</c> are local self-signed
  certificates. They are not on any C2PA trust list and will display as
  *untrusted* in third-party verifiers (Adobe, contentcredentials.org).
  Use them for local and internal workflows only.
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Init(init::Cli),
    Sign(sign::Cli),
    Verify(verify::Cli),
    Inspect(inspect::Cli),
    Trust(trust::Cli),
}

impl Cli {
    /// Run the requested credentials subcommand.
    ///
    /// # Errors
    ///
    /// Returns an error if the selected subcommand fails.
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Init(cmd) => cmd.run(),
            Command::Sign(cmd) => cmd.run().await,
            Command::Verify(cmd) => cmd.run().await,
            Command::Inspect(cmd) => cmd.run().await,
            Command::Trust(cmd) => cmd.run().await,
        }
    }
}

async fn resolve_trust_anchors(path: Option<PathBuf>) -> Result<Option<String>> {
    let path = match path {
        Some(path) => Some(path),
        None => env::var_os(ENV_TRUST_ANCHORS).map(PathBuf::from),
    };

    if let Some(path) = path {
        return Ok(Some(fs::read_to_string(path)?));
    }

    Ok(crate::trust::official_trust_anchors_best_effort().await?)
}
