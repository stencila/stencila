use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    sync::Arc,
};

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
    /// Create a new [Migration] from a file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| eyre!("Invalid migration filename: {}", path.display()))?
            .to_string();

        let Some(version) = name.strip_prefix("v") else {
            bail!(
                "Migration filename must follow pattern 'v{{VERSION}}.cypher', got: {}",
                path.display()
            );
        };

        let version = Version::parse(version)
            .wrap_err_with(|| format!("Invalid version in filename: {version}"))?;

        let cypher = read_to_string(path)
            .wrap_err_with(|| format!("Failed to read migration file: {}", path.display()))?;

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
    pub fn validate_migration(&self) -> Result<()> {
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
pub struct MigrationRunner {
    /// Path to the migrations directory
    migrations_dir: PathBuf,

    /// Database connection for executing migrations
    database: Arc<Database>,
}

impl MigrationRunner {
    /// Create a new MigrationRunner
    pub fn new(migrations_dir: PathBuf, database: Arc<Database>) -> Self {
        Self {
            migrations_dir,
            database,
        }
    }

    /// Discover all migration files in the migrations directory
    pub fn discover_migrations(&self) -> Result<Vec<Migration>> {
        if !self.migrations_dir.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        for entry in read_dir(&self.migrations_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension() == Some(OsStr::new("cypher")) {
                let migration = Migration::from_file(&path).wrap_err_with(|| {
                    format!("Failed to parse migration file: {}", path.display())
                })?;

                migration
                    .validate_migration()
                    .wrap_err_with(|| format!("Invalid SQL in migration: {}", migration.version))?;

                migrations.push(migration);
            }
        }

        // Sort migrations by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }

    /// Check if the migrations table exists
    pub fn migrations_table_exists(&self) -> Result<bool> {
        let connection = Connection::new(&self.database)?;

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

        let connection = Connection::new(&self.database)?;

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
        let all_migrations = self.discover_migrations()?;
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
            migration.validate_migration()?;
            return Ok(());
        }

        let connection = Connection::new(&self.database)?;

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
                self.record_migration(&connection, migration)
                    .wrap_err_with(|| {
                        format!(
                            "Failed to record migration {} in history",
                            migration.version
                        )
                    })?;

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
    use std::fs::{create_dir, write};

    use common::{chrono, tempfile::TempDir};
    use kernel_kuzu::kuzu::SystemConfig;

    use super::*;

    fn create_test_migration_file(dir: &Path, filename: &str, content: &str) -> Result<PathBuf> {
        let path = dir.join(filename);
        write(&path, content)?;
        Ok(path)
    }

    #[test]
    fn test_migration_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let migration_content =
            "ALTER TABLE `Reference` ADD COLUMN `test_column` STRING DEFAULT NULL;";
        let path = create_test_migration_file(temp_dir.path(), "v2.1.0.cypher", migration_content)?;

        let migration = Migration::from_file(&path)?;

        assert_eq!(migration.version.to_string(), "2.1.0");
        assert_eq!(migration.cypher, migration_content);
        assert!(!migration.checksum.is_empty());
        Ok(())
    }

    #[test]
    fn test_migration_from_file_invalid_name() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Missing version prefix
        let path = create_test_migration_file(
            temp_dir.path(),
            "1.0.0.cypher",
            "CREATE NODE TABLE test ();",
        )?;

        let result = Migration::from_file(&path);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("must follow pattern"));
        }

        // Invalid version
        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.0.cypher",
            "CREATE NODE TABLE test ();",
        )?;

        let result = Migration::from_file(&path);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("Invalid version"));
        }

        Ok(())
    }

    #[test]
    fn test_migration_checksum_consistency() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let content = "CREATE NODE TABLE checksum_test ();";
        let path = create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", content)?;

        let migration1 = Migration::from_file(&path)?;
        let migration2 = Migration::from_file(&path)?;

        // Same content should produce same checksum
        assert_eq!(migration1.checksum, migration2.checksum);

        // Different content should produce different checksum
        let path2 = create_test_migration_file(
            temp_dir.path(),
            "v1.1.0.cypher",
            "CREATE NODE TABLE different ();",
        )?;

        let migration3 = Migration::from_file(&path2)?;
        assert_ne!(migration1.checksum, migration3.checksum);

        Ok(())
    }

    #[test]
    fn test_migration_cypher_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Valid Cypher should pass
        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE valid ();",
        )?;

        let migration = Migration::from_file(&path)?;
        assert!(migration.validate_migration().is_ok());

        // Empty Cypher should fail
        let path = create_test_migration_file(temp_dir.path(), "v1.1.0.cypher", "   \n  \t  ")?;

        let migration = Migration::from_file(&path)?;
        let result = migration.validate_migration();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("cannot be empty"));
        }

        // Dangerous operations should fail
        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.2.0.cypher",
            "DROP DATABASE production;",
        )?;

        let migration = Migration::from_file(&path)?;
        let result = migration.validate_migration();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(
                error
                    .to_string()
                    .contains("dangerous operation `DROP DATABASE`")
            );
        }

        let path =
            create_test_migration_file(temp_dir.path(), "v1.3.0.cypher", "DELETE FROM users;")?;

        let migration = Migration::from_file(&path)?;
        let result = migration.validate_migration();
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
        let temp_dir = TempDir::new()?;
        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // For an empty database, migrations table should not exist
        assert!(!runner.migrations_table_exists()?);

        // Note: In practice, the migrations table would be created by the schema initialization
        // This test just verifies that the check works correctly
        Ok(())
    }

    #[test]
    fn test_migration_runner_discovery() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test migration files
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE test ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v2.0.0.cypher",
            "ALTER TABLE test ADD COLUMN name STRING;",
        )?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version.to_string(), "1.0.0");
        assert_eq!(migrations[1].version.to_string(), "2.0.0");
        Ok(())
    }

    #[test]
    fn test_migration_runner_validate_sequence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        let migration1 = Migration {
            version: Version::new(1, 0, 0),
            checksum: "checksum1".to_string(),
            cypher: "Cypher 1".to_string(),
        };

        let migration2 = Migration {
            version: Version::new(2, 0, 0),
            checksum: "checksum2".to_string(),
            cypher: "Cypher 2".to_string(),
        };

        // Valid sequence should pass
        let result = runner.validate_migration_sequence(&[migration1.clone(), migration2]);
        assert!(result.is_ok());

        // Duplicate version should fail
        let result = runner.validate_migration_sequence(&[migration1.clone(), migration1]);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_migration_runner_sorting() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create migrations in non-sorted order
        create_test_migration_file(
            temp_dir.path(),
            "v2.1.0.cypher",
            "CREATE NODE TABLE second ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE first ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v2.0.0.cypher",
            "CREATE NODE TABLE middle ();",
        )?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].version.to_string(), "1.0.0");
        assert_eq!(migrations[1].version.to_string(), "2.0.0");
        assert_eq!(migrations[2].version.to_string(), "2.1.0");

        Ok(())
    }

    #[test]
    fn test_migration_runner_ignores_non_migration_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create migration files
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE valid ();",
        )?;

        // Create non-migration files that should be ignored
        write(temp_dir.path().join("README.md"), "This is a readme")?;
        write(temp_dir.path().join("invalid.txt"), "Not a migration")?;

        // Create subdirectory with files (should be ignored)
        let subdir = temp_dir.path().join("subdir");
        create_dir(&subdir)?;
        write(subdir.join("v2.0.0.cypher"), "Should be ignored")?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        // Should only find the one valid migration
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version.to_string(), "1.0.0");

        Ok(())
    }

    #[test]
    fn test_migration_runner_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 0);

        Ok(())
    }

    #[test]
    fn test_migration_runner_nonexistent_directory() -> Result<()> {
        let nonexistent_dir = PathBuf::from("/nonexistent/migrations");

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(nonexistent_dir, database);
        let migrations = runner.discover_migrations()?;

        // Should return empty vector for non-existent directory
        assert_eq!(migrations.len(), 0);

        Ok(())
    }

    #[test]
    fn test_execute_migration_success() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a simple migration
        let migration_content =
            "CREATE NODE TABLE test_table (id STRING PRIMARY KEY, name STRING);";
        create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", migration_content)?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 1);
        let migration = &migrations[0];

        // Execute the migration
        runner.execute_migration(migration, false)?;

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
        let temp_dir = TempDir::new()?;

        // Create a migration with invalid Cypher
        let migration_content = "CREATE NODE TABLE invalid_syntax (id INVALID_TYPE);";
        create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", migration_content)?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 1);
        let migration = &migrations[0];

        // Execute the migration - should fail
        let result = runner.execute_migration(migration, false);
        assert!(result.is_err());

        // Verify no migration was recorded in history
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(!applied_migrations.contains_key(&migration.version));

        Ok(())
    }

    #[test]
    fn test_execute_pending_migrations() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create multiple migrations
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE first_table (id STRING PRIMARY KEY);",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v1.1.0.cypher",
            "CREATE NODE TABLE second_table (id STRING PRIMARY KEY);",
        )?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        // Execute all pending migrations
        let executed_migrations = runner.execute_pending_migrations(false)?;

        assert_eq!(executed_migrations.len(), 2);
        assert_eq!(executed_migrations[0].version.to_string(), "1.0.0");
        assert_eq!(executed_migrations[1].version.to_string(), "1.1.0");

        // Verify both migrations were recorded
        let applied_migrations = runner.get_applied_migrations()?;
        assert_eq!(applied_migrations.len(), 2);

        Ok(())
    }

    #[test]
    fn test_execute_migration_dry_run() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a migration
        let migration_content = "CREATE NODE TABLE test_table (id STRING PRIMARY KEY);";
        create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", migration_content)?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 1);
        let migration = &migrations[0];

        // Execute in dry-run mode
        runner.execute_migration(migration, true)?;

        // Verify no migration was actually applied
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(!applied_migrations.contains_key(&migration.version));

        Ok(())
    }

    #[test]
    fn test_get_migration_status() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create migrations
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
            "CREATE NODE TABLE applied_table (id STRING PRIMARY KEY);",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v1.1.0.cypher",
            "CREATE NODE TABLE pending_table (id STRING PRIMARY KEY);",
        )?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        let migrations = runner.discover_migrations()?;

        // Apply only the first migration
        runner.execute_migration(&migrations[0], false)?;

        // Get migration status
        let status = runner.get_migration_status()?;

        assert_eq!(status.applied_count, 1);
        assert_eq!(status.pending_count, 1);
        assert_eq!(status.applied_versions.len(), 1);
        assert_eq!(status.pending_versions.len(), 1);
        assert_eq!(status.applied_versions[0].to_string(), "1.0.0");
        assert_eq!(status.pending_versions[0].to_string(), "1.1.0");

        Ok(())
    }

    #[test]
    fn test_execute_migration_checksum_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a migration
        let migration_content = "CREATE NODE TABLE test_table (id STRING PRIMARY KEY);";
        create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", migration_content)?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        let migrations = runner.discover_migrations()?;
        let migration = &migrations[0];

        // Apply the migration
        runner.execute_migration(migration, false)?;

        // Try to apply again - should succeed (idempotent)
        runner.execute_migration(migration, false)?;

        // Verify still only one migration record
        let applied_migrations = runner.get_applied_migrations()?;
        assert_eq!(applied_migrations.len(), 1);

        Ok(())
    }

    #[test]
    fn test_execute_migration_multi_statement() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a migration with multiple statements
        let migration_content = r#"
            CREATE NODE TABLE first_table (id STRING PRIMARY KEY);
            CREATE NODE TABLE second_table (id STRING PRIMARY KEY);
            // This is a comment
            CREATE NODE TABLE third_table (id STRING PRIMARY KEY);
        "#;
        create_test_migration_file(temp_dir.path(), "v1.0.0.cypher", migration_content)?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        // Initialize the database with the migrations table
        let connection = Connection::new(&runner.database)?;
        connection.query("CREATE NODE TABLE IF NOT EXISTS _migrations (version STRING PRIMARY KEY, appliedAt TIMESTAMP, checksum STRING)")?;
        drop(connection);

        let migrations = runner.discover_migrations()?;

        assert_eq!(migrations.len(), 1);
        let migration = &migrations[0];

        // Execute the migration
        runner.execute_migration(migration, false)?;

        // Verify migration was recorded
        let applied_migrations = runner.get_applied_migrations()?;
        assert!(applied_migrations.contains_key(&migration.version));

        Ok(())
    }
}
