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

use common::{
    eyre::{Context, Result, bail, eyre},
    inflector::Inflector,
};
use kernel_kuzu::kuzu::{Connection, Database, Value};

/// Represents a single migration with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Migration {
    /// Unique identifier for this migration
    pub id: String,

    /// The version this migration applies to (e.g., "2.1.0")
    pub version: Version,

    /// Human-readable description of the migration
    pub description: String,

    /// The actual migration Cypher statements
    pub cypher: String,

    /// SHA256 checksum of the migration Cypher statements
    pub checksum: String,
}

impl Migration {
    /// Create a new [Migration] from a file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| eyre!("Invalid migration filename: {}", path.display()))?
            .to_string();

        let parts: Vec<&str> = id.splitn(2, "-").collect();
        if parts.len() != 2 || !parts[0].starts_with('v') {
            bail!(
                "Migration filename must follow pattern 'v{{VERSION}}-{{DESCRIPTION}}.cypher', got: {}",
                path.display()
            );
        }

        let version_str = &parts[0][1..]; // Remove 'v' prefix
        let version = Version::parse(version_str)
            .wrap_err_with(|| format!("Invalid version in filename: {version_str}"))?;

        let description = parts[1].to_sentence_case();

        let cypher = read_to_string(path)
            .wrap_err_with(|| format!("Failed to read migration file: {}", path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(cypher.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());

        Ok(Migration {
            id,
            version,
            description,
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
    /// Migration identifier
    pub migration_id: String,

    /// Version this migration was applied for
    pub version: String,

    /// Description of the migration
    pub description: String,

    /// When this migration was applied (RFC3339 timestamp)
    pub applied_at: String,

    /// Checksum of the migration when it was applied
    pub checksum: String,
}

impl MigrationHistory {
    pub fn new(migration: &Migration) -> Result<Self> {
        Ok(Self {
            migration_id: migration.id.clone(),
            version: migration.version.to_string(),
            description: migration.description.clone(),
            applied_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|e| eyre!("Failed to format timestamp: {}", e))?,
            checksum: migration.checksum.clone(),
        })
    }
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
                let migration = Migration::from_file(&path)
                    .wrap_err_with(|| format!("Failed to parse migration file: {:?}", path))?;

                migration
                    .validate_migration()
                    .wrap_err_with(|| format!("Invalid SQL in migration: {}", migration.id))?;

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
    pub fn get_applied_migrations(&self) -> Result<HashMap<String, MigrationHistory>> {
        if !self.migrations_table_exists()? {
            return Ok(HashMap::new());
        }

        let connection = Connection::new(&self.database)?;

        let mut result = connection.query(
            "MATCH (m:_migrations) RETURN m.migration_id, m.version, m.description, m.applied_at, m.checksum"
        )?;

        let mut applied = HashMap::new();

        for _i in 0..result.get_num_tuples() {
            let row = result
                .next()
                .ok_or_else(|| eyre!("Expected row from query result"))?;

            let migration_id = match &row[0] {
                Value::String(id) => id.clone(),
                _ => bail!("Invalid migration_id in database"),
            };

            let version = match &row[1] {
                Value::String(v) => v.clone(),
                _ => bail!("Invalid version in database"),
            };

            let description = match &row[2] {
                Value::String(desc) => desc.clone(),
                _ => bail!("Invalid description in database"),
            };

            let applied_at = match &row[3] {
                Value::Timestamp(ts) => ts
                    .format(&Rfc3339)
                    .map_err(|e| eyre!("Failed to format timestamp: {}", e))?,
                Value::String(s) => s.clone(),
                _ => bail!("Invalid applied_at timestamp in database"),
            };

            let checksum = match &row[4] {
                Value::String(cs) => cs.clone(),
                _ => bail!("Invalid checksum in database"),
            };

            let history = MigrationHistory {
                migration_id: migration_id.clone(),
                version,
                description,
                applied_at,
                checksum,
            };

            applied.insert(migration_id, history);
        }

        Ok(applied)
    }

    /// Find migrations that haven't been applied yet
    pub fn find_pending_migrations(&self) -> Result<Vec<Migration>> {
        let all_migrations = self.discover_migrations()?;
        let applied = self.get_applied_migrations()?;

        let mut pending = Vec::new();

        for migration in all_migrations {
            if let Some(applied_migration) = applied.get(&migration.id) {
                // Verify checksum matches
                if applied_migration.checksum != migration.checksum {
                    bail!(
                        "Checksum mismatch for migration {}: expected {}, got {}",
                        migration.id,
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
        let path = create_test_migration_file(
            temp_dir.path(),
            "v2.1.0-add-test-column.cypher",
            migration_content,
        )?;

        let migration = Migration::from_file(&path)?;

        assert_eq!(migration.version.to_string(), "2.1.0");
        assert_eq!(migration.description, "Add test column");
        assert_eq!(migration.cypher, migration_content);
        assert!(!migration.checksum.is_empty());
        Ok(())
    }

    #[test]
    fn test_migration_from_file_invalid_name() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let path = create_test_migration_file(
            temp_dir.path(),
            "invalid_name.cypher",
            "ALTER TABLE test ADD COLUMN test STRING;",
        )?;

        let result = Migration::from_file(&path);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("must follow pattern"));
        }
        Ok(())
    }

    #[test]
    fn test_migration_discovery() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test migration files
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0-initial-migration.cypher",
            "CREATE NODE TABLE test ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v2.0.0-second-migration.cypher",
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
    fn test_validate_migration_sequence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);

        let migration1 = Migration {
            id: "v1.0.0-test1".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test 1".to_string(),
            checksum: "checksum1".to_string(),
            cypher: "Cypher 1".to_string(),
        };

        let migration2 = Migration {
            id: "v2.0.0-test2".to_string(),
            version: Version::new(2, 0, 0),
            description: "Test 2".to_string(),
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
    fn test_migration_parsing_invalid_cases() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Missing version prefix
        let path = create_test_migration_file(
            temp_dir.path(),
            "1.0.0-no-v-prefix.cypher",
            "CREATE NODE TABLE test ();",
        )?;

        let result = Migration::from_file(&path);
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("must follow pattern"));
        }

        // Missing description
        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.0.0.cypher",
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
            "v1.0.invalid-version.cypher",
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
        let path =
            create_test_migration_file(temp_dir.path(), "v1.0.0-checksum-test.cypher", content)?;

        let migration1 = Migration::from_file(&path)?;
        let migration2 = Migration::from_file(&path)?;

        // Same content should produce same checksum
        assert_eq!(migration1.checksum, migration2.checksum);

        // Different content should produce different checksum
        let path2 = create_test_migration_file(
            temp_dir.path(),
            "v1.1.0-different-content.cypher",
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
            "v1.0.0-valid.cypher",
            "CREATE NODE TABLE valid ();",
        )?;

        let migration = Migration::from_file(&path)?;
        assert!(migration.validate_migration().is_ok());

        // Empty Cypher should fail
        let path =
            create_test_migration_file(temp_dir.path(), "v1.1.0-empty.cypher", "   \n  \t  ")?;

        let migration = Migration::from_file(&path)?;
        let result = migration.validate_migration();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("cannot be empty"));
        }

        // Dangerous operations should fail
        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.2.0-dangerous-drop.cypher",
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

        let path = create_test_migration_file(
            temp_dir.path(),
            "v1.3.0-dangerous-delete.cypher",
            "DELETE FROM users;",
        )?;

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
            id: "v1.0.0-test-migration".to_string(),
            version: Version::new(1, 0, 0),
            description: "Test migration".to_string(),
            checksum: "abc123".to_string(),
            cypher: "CREATE NODE TABLE test ();".to_string(),
        };

        let history = MigrationHistory::new(&migration)?;

        assert_eq!(history.migration_id, "v1.0.0-test-migration");
        assert_eq!(history.version, "1.0.0");
        assert_eq!(history.description, "Test migration");
        assert_eq!(history.checksum, "abc123");
        assert!(!history.applied_at.is_empty());

        // Verify timestamp format (should be RFC3339)
        assert!(chrono::DateTime::parse_from_rfc3339(&history.applied_at).is_ok());

        Ok(())
    }

    #[test]
    fn test_migration_discovery_sorting() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create migrations in non-sorted order
        create_test_migration_file(
            temp_dir.path(),
            "v2.1.0-second.cypher",
            "CREATE NODE TABLE second ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0-first.cypher",
            "CREATE NODE TABLE first ();",
        )?;

        create_test_migration_file(
            temp_dir.path(),
            "v2.0.0-middle.cypher",
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
    fn test_migration_discovery_ignores_non_migration_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create migration files
        create_test_migration_file(
            temp_dir.path(),
            "v1.0.0-valid.cypher",
            "CREATE NODE TABLE valid ();",
        )?;

        // Create non-migration files that should be ignored
        write(temp_dir.path().join("README.md"), "This is a readme")?;
        write(temp_dir.path().join("invalid.txt"), "Not a migration")?;

        // Create subdirectory with files (should be ignored)
        let subdir = temp_dir.path().join("subdir");
        create_dir(&subdir)?;
        write(subdir.join("v2.0.0-in-subdir.cypher"), "Should be ignored")?;

        let database = Arc::new(
            Database::new(":memory:", SystemConfig::default())
                .map_err(|e| eyre!("Failed to create test database: {}", e))?,
        );

        let runner = MigrationRunner::new(temp_dir.path().to_path_buf(), database);
        let migrations = runner.discover_migrations()?;

        // Should only find the one valid migration
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version.to_string(), "1.0.0");
        assert_eq!(migrations[0].description, "Valid");

        Ok(())
    }

    #[test]
    fn test_migration_discovery_empty_directory() -> Result<()> {
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
    fn test_migration_discovery_nonexistent_directory() -> Result<()> {
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
}
