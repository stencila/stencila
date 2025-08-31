use std::collections::HashMap;

use rust_embed::Embed;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use common::eyre::{Context, Result, bail, eyre};
use kernel_kuzu::kuzu::{Connection, Database, Value};

/// Represents a single migration with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Migration {
    /// The version this migration applies to (e.g. "2.7.0")
    ///
    /// Note: this is the version of the schema that the migration
    /// will move the database **to** (not the version that is being
    /// moved from).
    pub version: Version,

    /// The actual migration Cypher statements
    pub cypher: String,

    /// SHA256 checksum of the migration Cypher statements
    pub checksum: String,
}

impl Migration {
    /// Create a new [Migration]
    pub fn new(name: &str, cypher: String) -> Result<Self> {
        let name = name.trim_end_matches(".cypher");

        let Some(version) = name.strip_prefix("v") else {
            bail!("Migration filename must follow pattern 'v{{VERSION}}', got: {name}");
        };

        let version = Version::parse(version)
            .wrap_err_with(|| format!("Invalid version in filename: {version}"))?;

        let mut hasher = Sha256::new();
        hasher.update(cypher.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());

        Ok(Migration {
            version,
            checksum,
            cypher,
        })
    }

    /// Validate that the migration Cypher is not empty or contain potentially
    /// dangerous operations
    pub fn validate(&self) -> Result<()> {
        let cypher = self.cypher.trim().to_lowercase();

        if cypher.is_empty() {
            bail!("Migration SQL cannot be empty");
        }

        if cypher.contains("drop database") {
            bail!("Migration contains potentially dangerous operation `DROP DATABASE`");
        }

        if cypher.contains("delete from") {
            bail!("Migration contains potentially dangerous operation `DELETE FROM`");
        }

        Ok(())
    }
}

/// Migrations embedded into binary
///
/// During development these are loaded directly from the migrations directory
/// but are embedded into the binary on release builds.
#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/migrations"]
#[exclude = "README.md"]
pub struct Migrations;

impl Migrations {
    pub fn list() -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        for name in Self::iter() {
            let Some(file) = Self::get(&name) else {
                continue;
            };

            let migration = Migration::new(&name, String::from_utf8(file.data.to_vec())?)?;
            migration.validate()?;

            migrations.push(migration);
        }

        // Sort migrations by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }
}

/// Tracks which migrations have been applied to the database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationHistory {
    /// The version this migration applies to
    pub version: Version,

    /// When this migration was applied (RFC3339 timestamp)
    pub applied_at: String,

    /// Checksum of the migration when it was applied
    pub checksum: String,
}

impl MigrationHistory {
    pub fn new(migration: &Migration) -> Result<Self> {
        Ok(Self {
            version: migration.version.clone(),
            applied_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|_| eyre!("Failed to format timestamp"))?,
            checksum: migration.checksum.clone(),
        })
    }
}

/// Migration status summary
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// Number of applied migrations
    pub applied_count: usize,

    /// Number of pending migrations
    pub pending_count: usize,

    /// List of pending migration versions
    pub pending_versions: Vec<Version>,

    /// List of applied migration versions
    pub applied_versions: Vec<Version>,
}

/// Manages migration discovery, validation, and execution
pub struct MigrationRunner<'d> {
    /// Database connection for executing migrations
    database: &'d Database,
}

impl<'d> MigrationRunner<'d> {
    /// Create a new MigrationRunner
    pub fn new(database: &'d Database) -> Self {
        Self { database }
    }

    /// Check if the migrations table exists
    pub fn migrations_table_exists(&self) -> Result<bool> {
        let connection = Connection::new(self.database)?;

        let mut result = connection.query("CALL show_tables() RETURN name")?;

        for _i in 0..result.get_num_tuples() {
            let row = result
                .next()
                .ok_or_else(|| eyre!("Expected row from query result"))?;
            if let Value::String(table_name) = &row[0]
                && table_name == "_migrations"
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get all applied migrations from the database
    pub fn get_applied_migrations(&self) -> Result<HashMap<Version, MigrationHistory>> {
        if !self.migrations_table_exists()? {
            return Ok(HashMap::new());
        }

        let connection = Connection::new(self.database)?;

        let mut result =
            connection.query("MATCH (m:_migrations) RETURN m.version, m.appliedAt, m.checksum")?;

        let mut applied = HashMap::new();

        for _i in 0..result.get_num_tuples() {
            let row = result
                .next()
                .ok_or_else(|| eyre!("Expected row from query result"))?;

            let version = match &row[0] {
                Value::String(v) => Version::parse(v)?,
                _ => bail!("Invalid version in database"),
            };

            let applied_at = match &row[1] {
                Value::Timestamp(ts) => ts
                    .format(&Rfc3339)
                    .map_err(|_| eyre!("Failed to format timestamp"))?,
                Value::String(s) => s.clone(),
                _ => bail!("Invalid applied_at timestamp in database"),
            };

            let checksum = match &row[2] {
                Value::String(cs) => cs.clone(),
                _ => bail!("Invalid checksum in database"),
            };

            let history = MigrationHistory {
                version: version.clone(),
                applied_at,
                checksum,
            };

            applied.insert(version, history);
        }

        Ok(applied)
    }

    /// Find migrations that haven't been applied yet
    pub fn find_pending_migrations(&self) -> Result<Vec<Migration>> {
        let all_migrations = Migrations::list()?;
        let applied = self.get_applied_migrations()?;

        let mut pending = Vec::new();

        for migration in all_migrations {
            if let Some(applied_migration) = applied.get(&migration.version) {
                if applied_migration.checksum != migration.checksum {
                    bail!(
                        "Checksum mismatch for migration {}: expected {}, got {}",
                        migration.version,
                        applied_migration.checksum,
                        migration.checksum
                    );
                }
            } else {
                pending.push(migration);
            }
        }

        Ok(pending)
    }

    /// Validate that there are no gaps in the migration sequence
    pub fn validate_migration_sequence(&self, migrations: &[Migration]) -> Result<()> {
        if migrations.is_empty() {
            return Ok(());
        }

        // Check for version gaps - each migration should have a unique version
        let mut versions: Vec<&Version> = migrations.iter().map(|m| &m.version).collect();
        versions.sort();

        for window in versions.windows(2) {
            if window[0] == window[1] {
                bail!("Duplicate migration version: {}", window[0]);
            }
        }

        Ok(())
    }

    /// Execute a single migration with transaction support
    pub fn execute_migration(&self, migration: &Migration, dry_run: bool) -> Result<()> {
        if dry_run {
            // For dry-run, just validate the migration without executing
            migration.validate()?;
            return Ok(());
        }

        let connection = Connection::new(self.database)?;

        // Check if migration is already applied
        let applied_migrations = self.get_applied_migrations()?;
        if let Some(applied_migration) = applied_migrations.get(&migration.version) {
            if applied_migration.checksum != migration.checksum {
                bail!(
                    "Migration {} already applied with different checksum. Expected: {}, got: {}",
                    migration.version,
                    applied_migration.checksum,
                    migration.checksum
                );
            }

            // Migration already applied with same checksum, skip
            return Ok(());
        }

        // Begin transaction
        connection.query("BEGIN TRANSACTION").wrap_err_with(|| {
            format!(
                "Failed to begin transaction for migration {}",
                migration.version
            )
        })?;

        let migration_result = self.execute_migration_statements(&connection, migration);

        match migration_result {
            Ok(()) => {
                // Record successful migration in history
                self.record_migration(&connection, migration)?;

                // Commit transaction
                connection.query("COMMIT").wrap_err_with(|| {
                    format!("Failed to commit migration {}", migration.version)
                })?;

                Ok(())
            }
            Err(error) => {
                // Rollback transaction on failure
                if let Err(rollback_error) = connection.query("ROLLBACK") {
                    bail!(
                        "Migration {} failed: {}. Additionally, rollback failed: {}",
                        migration.version,
                        error,
                        rollback_error
                    );
                }
                Err(error).wrap_err_with(|| format!("Migration {} failed", migration.version))
            }
        }
    }

    /// Execute the Cypher statements in a migration
    fn execute_migration_statements(
        &self,
        connection: &Connection,
        migration: &Migration,
    ) -> Result<()> {
        // Split the Cypher by semicolons and execute each statement
        let statements: Vec<&str> = migration
            .cypher
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("//") && !s.starts_with("--"))
            .collect();

        for statement in statements {
            connection
                .query(statement)
                .wrap_err_with(|| format!("Failed to execute statement: {}", statement))?;
        }

        Ok(())
    }

    /// Record a successfully applied migration in the _migrations table
    fn record_migration(&self, connection: &Connection, migration: &Migration) -> Result<()> {
        let history = MigrationHistory::new(migration)?;

        // Create Cypher statement to insert migration record
        let cypher = format!(
            "CREATE (m:_migrations {{version: '{}', appliedAt: timestamp('{}'), checksum: '{}'}})",
            history.version, history.applied_at, history.checksum
        );

        connection
            .query(&cypher)
            .wrap_err("Failed to record migration in _migrations table")?;

        Ok(())
    }

    /// Execute all pending migrations in sequence
    pub fn execute_pending_migrations(&self, dry_run: bool) -> Result<Vec<Migration>> {
        let pending_migrations = self.find_pending_migrations()?;

        if pending_migrations.is_empty() {
            return Ok(Vec::new());
        }

        // Validate the migration sequence before executing
        self.validate_migration_sequence(&pending_migrations)?;

        let mut executed_migrations = Vec::new();

        for migration in &pending_migrations {
            self.execute_migration(migration, dry_run)
                .wrap_err_with(|| format!("Failed to execute migration {}", migration.version))?;

            executed_migrations.push(migration.clone());
        }

        Ok(executed_migrations)
    }

    /// Get a summary of the migration status
    pub fn get_migration_status(&self) -> Result<MigrationStatus> {
        let applied_migrations = self.get_applied_migrations()?;
        let pending_migrations = self.find_pending_migrations()?;

        let mut applied_versions: Vec<Version> = applied_migrations.keys().cloned().collect();
        applied_versions.sort();

        let pending_versions: Vec<Version> = pending_migrations
            .iter()
            .map(|m| m.version.clone())
            .collect();

        Ok(MigrationStatus {
            applied_count: applied_migrations.len(),
            pending_count: pending_migrations.len(),
            applied_versions,
            pending_versions,
        })
    }
}

#[cfg(test)]
mod tests {
    use common::chrono;
    use kernel_kuzu::kuzu::SystemConfig;

    use super::*;

    #[test]
    fn test_migration_new() -> Result<()> {
        let migration_content =
            "ALTER TABLE `Reference` ADD COLUMN `test_column` STRING DEFAULT NULL;";

        let migration = Migration::new("v2.1.0", migration_content.to_string())?;

        assert_eq!(migration.version.to_string(), "2.1.0");
        assert_eq!(migration.cypher, migration_content);
        assert!(!migration.checksum.is_empty());
        Ok(())
    }

    #[test]
    fn test_migration_new_invalid_name() -> Result<()> {
        // Missing version prefix
        let result = Migration::new("1.0.0", "CREATE NODE TABLE test ();".to_string());
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("must follow pattern"));
        }

        // Invalid version
        let result = Migration::new("v1.0", "CREATE NODE TABLE test ();".to_string());
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("Invalid version"));
        }

        Ok(())
    }

    #[test]
    fn test_migration_checksum_consistency() -> Result<()> {
        let content = "CREATE NODE TABLE checksum_test ();";

        let migration1 = Migration::new("v1.0.0", content.to_string())?;
        let migration2 = Migration::new("v1.0.0", content.to_string())?;

        // Same content should produce same checksum
        assert_eq!(migration1.checksum, migration2.checksum);

        // Different content should produce different checksum
        let migration3 = Migration::new("v1.1.0", "CREATE NODE TABLE different ();".to_string())?;
        assert_ne!(migration1.checksum, migration3.checksum);

        Ok(())
    }

    #[test]
    fn test_migration_cypher_validation() -> Result<()> {
        // Valid Cypher should pass
        let migration = Migration::new("v1.0.0", "CREATE NODE TABLE valid ();".to_string())?;
        assert!(migration.validate().is_ok());

        // Empty Cypher should fail
        let migration = Migration::new("v1.1.0", "   \n  \t  ".to_string())?;
        let result = migration.validate();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("cannot be empty"));
        }

        // Dangerous operations should fail
        let migration = Migration::new("v1.2.0", "DROP DATABASE production;".to_string())?;
        let result = migration.validate();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .to_string()
                    .contains("dangerous operation `DROP DATABASE`")
            );
        }

        let migration = Migration::new("v1.3.0", "DELETE FROM users;".to_string())?;
        let result = migration.validate();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .to_string()
                    .contains("dangerous operation `DELETE FROM`")
            );
        }

        Ok(())
    }

    #[test]
    fn test_migration_history_creation() -> Result<()> {
        let migration = Migration {
            version: Version::new(1, 0, 0),
            checksum: "abc123".to_string(),
            cypher: "CREATE NODE TABLE test ();".to_string(),
        };

        let history = MigrationHistory::new(&migration)?;

        assert_eq!(history.version, Version::new(1, 0, 0));
        assert_eq!(history.checksum, "abc123");
        assert!(!history.applied_at.is_empty());

        // Verify timestamp format (should be RFC3339)
        assert!(chrono::DateTime::parse_from_rfc3339(&history.applied_at).is_ok());

        Ok(())
    }

    #[test]
    fn test_migrations_table_exists() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // For an empty database, migrations table should not exist
        assert!(!runner.migrations_table_exists()?);

        // Note: In practice, the migrations table would be created by the schema initialization
        // This test just verifies that the check works correctly
        Ok(())
    }

    #[test]
    fn test_execute_migration_success() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        // Create a test migration directly
        let migration = Migration::new(
            "v1.0.0",
            "CREATE NODE TABLE test_table (id STRING PRIMARY KEY, name STRING);".to_string(),
        )?;

        // Execute the migration
        runner.execute_migration(&migration, false)?;

        // Verify migration was recorded in history
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(applied_migrations.contains_key(&migration.version));
        assert_eq!(
            applied_migrations[&migration.version].checksum,
            migration.checksum
        );

        Ok(())
    }

    #[test]
    fn test_execute_migration_rollback() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        // Create a migration with invalid Cypher
        let migration = Migration::new(
            "v1.0.0",
            "CREATE NODE TABLE invalid_syntax (id INVALID_TYPE);".to_string(),
        )?;

        // Execute the migration - should fail
        let result = runner.execute_migration(&migration, false);
        assert!(result.is_err());

        // Verify no migration was recorded in history
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(!applied_migrations.contains_key(&migration.version));

        Ok(())
    }

    #[test]
    fn test_execute_migration_dry_run() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // Create a test migration directly
        let migration = Migration::new(
            "v1.0.0",
            "CREATE NODE TABLE test_table (id STRING PRIMARY KEY);".to_string(),
        )?;

        // Execute in dry-run mode
        runner.execute_migration(&migration, true)?;

        // Verify no migration was actually applied (dry run doesn't create tables)
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(!applied_migrations.contains_key(&migration.version));

        Ok(())
    }

    #[test]
    fn test_execute_migration_checksum_validation() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        // Create a test migration directly
        let migration = Migration::new(
            "v1.0.0",
            "CREATE NODE TABLE test_table (id STRING PRIMARY KEY);".to_string(),
        )?;

        // Apply the migration
        runner.execute_migration(&migration, false)?;

        // Try to apply again - should succeed (idempotent)
        runner.execute_migration(&migration, false)?;

        // Verify still only one migration record
        let applied_migrations = runner.get_applied_migrations()?;
        assert_eq!(applied_migrations.len(), 1);

        Ok(())
    }

    #[test]
    fn test_execute_migration_multi_statement() -> Result<()> {
        let database = Database::new(":memory:", SystemConfig::default())
            .map_err(|e| eyre!("Failed to create test database: {}", e))?;

        let runner = MigrationRunner::new(&database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        // Create a migration with multiple statements
        let migration_content = r#"
            CREATE NODE TABLE first_table (id STRING PRIMARY KEY);
            CREATE NODE TABLE second_table (id STRING PRIMARY KEY);
            // This is a comment
            CREATE NODE TABLE third_table (id STRING PRIMARY KEY);
        "#;

        let migration = Migration::new("v1.0.0", migration_content.to_string())?;

        // Execute the migration
        runner.execute_migration(&migration, false)?;

        // Verify migration was recorded
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(applied_migrations.contains_key(&migration.version));

        Ok(())
    }
}
