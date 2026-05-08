//! `stencila credentials trust` - manage the official C2PA trust-list cache.

use clap::{Args, Subcommand};
use eyre::Result;
use stencila_cli_utils::{AsFormat, Code, ToStdout, message};

use crate::trust;

/// Manage the cached official C2PA trust list.
#[derive(Debug, Args)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Status(Status),
    Refresh(Refresh),
}

/// Show official trust-list cache status.
#[derive(Debug, Args)]
struct Status {
    /// Output format. Defaults to text.
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

/// Refresh the official trust-list cache.
#[derive(Debug, Args)]
struct Refresh {
    /// Output format. Defaults to text.
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl Cli {
    /// Run the requested trust-list command.
    ///
    /// # Errors
    ///
    /// Returns an error if cache status cannot be read, refreshed, or written.
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Status(cmd) => cmd.run(),
            Command::Refresh(cmd) => cmd.run().await,
        }
    }
}

impl Status {
    fn run(self) -> Result<()> {
        let status = trust::status()?;
        match self.r#as {
            Some(format) => Code::new_from(format.into(), &status)?.to_stdout(),
            None => print_status(&status),
        }

        Ok(())
    }
}

impl Refresh {
    async fn run(self) -> Result<()> {
        let status = trust::refresh_official_trust_list().await?;
        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &status)?.to_stdout();
        } else {
            message!("Refreshed official C2PA trust list");
            print_status(&status);
        }

        Ok(())
    }
}

fn print_status(status: &trust::TrustListStatus) {
    message!("Official C2PA trust list: `{}`", status.url);
    message!("Cache path: `{}`", status.path.display());
    message!("Present: {}", yes_no(status.present));
    message!("Fresh: {}", yes_no(status.fresh));
    if let Some(fetched_at) = status.fetched_at {
        message!("Fetched at: {}", fetched_at.to_rfc3339());
    }
    if let Some(expires_at) = status.expires_at {
        message!("Expires at: {}", expires_at.to_rfc3339());
    }
    if let Some(sha256) = &status.sha256 {
        message!("SHA-256: `{}`", sha256);
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
