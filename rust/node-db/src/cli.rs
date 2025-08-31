#![allow(clippy::print_stdout)]

use std::{env::current_dir, path::PathBuf, sync::Arc};

use cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use common::{
    clap::{self, Args},
    eyre::{Result, bail},
};
use dirs::{closest_stencila_dir, stencila_db_file};
use kernel_kuzu::kuzu::{Database, SystemConfig};

use crate::migrations::MigrationRunner;

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

        let database = Arc::new(Database::new(&db_path, SystemConfig::default())?);
        let runner = MigrationRunner::new(database);

        let executed_migrations = runner.execute_pending_migrations(self.dry_run)?;

        if executed_migrations.is_empty() {
            message("No pending migrations to apply", Some("✅"));
            return Ok(());
        }

        if self.dry_run {
            message(
                &format!("Would apply {} migration(s):", executed_migrations.len()),
                Some("📋"),
            );
            for migration in executed_migrations {
                println!("  • {}", migration.version);
            }
        } else {
            message(
                &format!("Applied {} migration(s):", executed_migrations.len()),
                Some("⏩"),
            );
            for migration in executed_migrations {
                println!("  ✓ {}", migration.version);
            }
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
        let database = Arc::new(Database::new(&db_path, SystemConfig::default())?);
        let runner = MigrationRunner::new(database);

        let status = runner.get_migration_status()?;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &status)?.to_stdout();
            return Ok(());
        }

        if status.applied_count + status.pending_count == 0 {
            message(
                cstr!(
                    "No migrations required for this database, it is up to date with this version of Stencila"
                ),
                Some("✅"),
            );
            return Ok(())
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
            message(
                cstr!("All migrations have been successfully applied"),
                Some("🎉"),
            );
        } else {
            message(
                cstr!("Run <b>stencila db migrate</> to apply pending migrations"),
                Some("💡"),
            );
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

