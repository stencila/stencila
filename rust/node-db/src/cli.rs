#![allow(clippy::print_stdout)]

use std::{env::current_dir, path::PathBuf};

use clap::Args;
use eyre::{Result, bail};
use itertools::Itertools;
use tokio::fs::remove_file;

use stencila_ask::ask;
use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use stencila_dirs::{closest_stencila_dir, stencila_db_file};
use stencila_kernel_kuzu::kuzu::{Database, SystemConfig};

use crate::{NodeDatabase, migrations::MigrationRunner};

/// Create a new document database
#[derive(Debug, Default, Args)]
#[command(after_long_help = NEW_AFTER_LONG_HELP)]
pub struct New {
    /// Path to the database file
    ///
    /// If not specified, creates a new workspace database.
    path: Option<PathBuf>,

    /// Overwrite the database if it already exists
    #[arg(long, short)]
    force: bool,
}

impl New {
    pub async fn run(self) -> Result<()> {
        let path = match self.path {
            Some(path) => path,
            None => {
                // Create a .stencila directory if one does not yet exists and return the path to the
                // database inside it
                let stencila_dir = closest_stencila_dir(&current_dir()?, true).await?;
                stencila_db_file(&stencila_dir, false).await?
            }
        };

        if path.exists() {
            if !self.force
                && ask(&format!(
                    "Database already exists at `{}`. Do you really want to overwrite it?",
                    path.display()
                ))
                .await?
                .is_no()
            {
                return Ok(());
            }

            remove_file(&path).await?;
        }

        NodeDatabase::new(&path)?;

        message!("🎂 Created a new document database at `{}`", path.display());

        Ok(())
    }
}

pub static NEW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a document database in the current workspace</dim>
  <b>stencila db new</b>

  <dim># Create a document database at a specific path</dim>
  <b>stencila db new path/to/my-database.kuzu</b>

  <dim># Overwrite the database if it already exists</dim>
  <b>stencila db new temp.kuzu --force</b>
"
);

/// Run pending database migrations
#[derive(Debug, Default, Args)]
#[command(after_long_help = MIGRATE_AFTER_LONG_HELP)]
pub struct Migrate {
    /// Path to the database file
    ///
    /// If not specified, uses the workspace database
    db: Option<PathBuf>,

    /// Preview migrations without applying them
    #[arg(long, short)]
    dry_run: bool,
}

impl Migrate {
    pub async fn run(self) -> Result<()> {
        let db_path = resolve_database(self.db).await?;

        let database = Database::new(&db_path, SystemConfig::default())?;
        let runner = MigrationRunner::new(&database);

        let executed_migrations = runner.execute_pending_migrations(self.dry_run)?;

        if executed_migrations.is_empty() {
            message("✅ No pending migrations to apply");
            return Ok(());
        }

        let versions = executed_migrations
            .iter()
            .map(|migration| migration.version.to_string())
            .join(" → ");

        if self.dry_run {
            message!(
                "📋 Would apply {} migration(s): {versions}",
                executed_migrations.len()
            );
        } else {
            message!(
                "⏩ Applied {} migration(s): {versions}",
                executed_migrations.len()
            );
        }

        Ok(())
    }
}

pub static MIGRATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Apply pending migrations to workspace database</dim>
  <b>stencila db migrate</b>

  <dim># Preview what migrations would be applied</dim>
  <b>stencila db migrate --dry-run</b>

  <dim># Apply migrations to a specific database</dim>
  <b>stencila db migrate path/to/my-database.kuzu</b>
"
);

/// Show applied and pending migrations
#[derive(Debug, Default, Args)]
#[command(after_long_help = MIGRATIONS_AFTER_LONG_HELP)]
pub struct Migrations {
    /// Path to the database file
    db: Option<PathBuf>,

    /// Output format
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl Migrations {
    pub async fn run(self) -> Result<()> {
        let db_path = resolve_database(self.db).await?;
        let database = Database::new(&db_path, SystemConfig::default())?;
        let runner = MigrationRunner::new(&database);

        let status = runner.get_migration_status()?;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &status)?.to_stdout();
            return Ok(());
        }

        if status.applied_count + status.pending_count == 0 {
            message(cstr!(
                "✅ No migrations required for this database, it is up to date with this version of Stencila"
            ));
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Version", "Status"]);

        for version in &status.applied_versions {
            table.add_row([
                Cell::new(version.to_string()).add_attribute(Attribute::Bold),
                Cell::new("Applied").fg(Color::Green),
            ]);
        }

        for version in &status.pending_versions {
            table.add_row([
                Cell::new(version.to_string()).add_attribute(Attribute::Bold),
                Cell::new("Pending").fg(Color::Green),
            ]);
        }

        table.to_stdout();

        if status.pending_count == 0 {
            message("🎉 All migrations have been successfully applied");
        } else {
            message("💡 Run *stencila db migrate* to apply pending migrations");
        }

        if let Err(error) = runner.check_version_compatibility() {
            tracing::warn!("Failed to check version compatibility: {error}");
        }

        Ok(())
    }
}

pub static MIGRATIONS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show applied and pending migrations for the workspace database</dim>
  <b>stencila db migrations</b>

  <dim># Output migrations as JSON</dim>
  <b>stencila db migrations --as json</b>

  <dim># Show migrations for a specific database</dim>
  <b>stencila db migrations path/to/database.kuzu</b>
"
);

/// Resolve the database path, using workspace default if not specified
async fn resolve_database(path: Option<PathBuf>) -> Result<PathBuf> {
    let path = match path {
        Some(path) => Ok(path),
        None => stencila_db_file(&closest_stencila_dir(&current_dir()?, false).await?, false).await,
    }?;

    if !path.exists() {
        bail!("No database at `{}`", path.display())
    }

    Ok(path)
}
