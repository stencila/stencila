use clap::{Parser, Subcommand};
use eyre::Result;

use stencila_cli_utils::color_print::cstr;
use stencila_node_db::cli::{Migrate, Migrations};

/// Manage the workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run pending migrations on workspace database</dim>
  <b>stencila db migrate</b>

  <dim># Check migration status</dim>
  <b>stencila db migrations</b>

  <dim># Validate migrations without applying</dim>
  <b>stencila db migrate --dry-run</b>

  <dim># Work with a specific database</dim>
  <b>stencila db migrate /path/to/database.kuzu</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Migrate(Migrate),
    Migrations(Migrations),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Migrate(migrate) => migrate.run().await,
            Command::Migrations(migrations) => migrations.run().await,
        }
    }
}
