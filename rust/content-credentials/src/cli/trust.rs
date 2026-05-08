//! `stencila credentials trust` - manage the official C2PA trust-list cache.

use clap::{Args, Subcommand};
use eyre::Result;
use stencila_cli_utils::{
    AsFormat, Code, Tabulated, ToStdout, message,
    tabulated::{Attribute, Cell, Color},
};

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
        }

        Ok(())
    }
}

fn print_status(status: &trust::TrustListStatus) {
    let mut table = Tabulated::new();
    table.set_header(["Field", "Value"]);
    table.add_row([
        Cell::new("C2PA trust list").add_attribute(Attribute::Bold),
        Cell::new(status.url).fg(Color::Blue),
    ]);
    table.add_row([
        Cell::new("Cache path").add_attribute(Attribute::Bold),
        Cell::new(status.path.display()).fg(Color::Rgb {
            r: 200,
            g: 160,
            b: 255,
        }),
    ]);
    table.add_row([
        Cell::new("Present").add_attribute(Attribute::Bold),
        yes_no_cell(status.present),
    ]);
    table.add_row([
        Cell::new("Fresh").add_attribute(Attribute::Bold),
        yes_no_cell(status.fresh),
    ]);

    if let Some(fetched_at) = status.fetched_at {
        table.add_row([
            Cell::new("Fetched at").add_attribute(Attribute::Bold),
            Cell::new(fetched_at.to_rfc3339()),
        ]);
    }
    if let Some(expires_at) = status.expires_at {
        table.add_row([
            Cell::new("Expires at").add_attribute(Attribute::Bold),
            Cell::new(expires_at.to_rfc3339()),
        ]);
    }
    if let Some(sha256) = &status.sha256 {
        table.add_row([
            Cell::new("SHA-256").add_attribute(Attribute::Bold),
            Cell::new(sha256),
        ]);
    }

    table.to_stdout();
}

fn yes_no_cell(value: bool) -> Cell {
    if value {
        Cell::new("yes").fg(Color::Green)
    } else {
        Cell::new("no").add_attribute(Attribute::Dim)
    }
}
