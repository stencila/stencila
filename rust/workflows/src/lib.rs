mod workflow_def;
pub use workflow_def::*;

mod workflow_validate;
pub use workflow_validate::*;

mod workflow_emitters;
pub use workflow_emitters::*;

mod workflow_run;
pub use workflow_run::*;

mod tools;

pub mod cli;

/// Run all known domain migrations on a [`WorkspaceDb`].
///
/// This is used by `stencila db pull` to bring a restored database up to
/// the schema version expected by the current code before applying
/// changesets.
pub fn run_migrations(db: &stencila_db::WorkspaceDb) -> Result<(), stencila_db::rusqlite::Error> {
    db.migrate(
        "workflows",
        stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
    )?;
    Ok(())
}
