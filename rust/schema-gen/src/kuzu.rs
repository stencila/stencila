use std::path::PathBuf;

use common::{
    eyre::Result,
    serde_json,
    tokio::fs::{self, remove_file, write},
};

use crate::{kuzu_builder::KuzuSchemaBuilder, kuzu_cypher, kuzu_rust, schemas::Schemas};

impl Schemas {
    /// Generate Kuzu database schema and Rust ORM code from Stencila Schema
    ///
    /// This function transforms the Stencila Schema type definitions into a
    /// Kuzu graph database schema with corresponding Rust code for data access.
    /// The generation process uses an abstraction layer to separate schema
    /// analysis from code generation, making the system more maintainable and
    /// extensible.
    #[allow(clippy::print_stderr)]
    pub async fn kuzu(&self) -> Result<()> {
        eprintln!("Generating Kuzu Schema and Migrations");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-db");
        let schemas_dir = dir.join("schemas");

        // Build the database schema
        let mut builder = KuzuSchemaBuilder::new(self);
        let schema = builder.build()?;

        // Generate migration if we have a previous schema and it's different
        let mut migrations_needed = false;
        let migrations_file = dir.join("migrations").join("v99.99.99.cypher");
        if let Some(previous_schema) = find_latest_schema(&schemas_dir).await?
            && let Some(migration_cypher) =
                kuzu_cypher::generate_migration(&previous_schema, &schema)
            {
                migrations_needed = true;
                write(&migrations_file, migration_cypher).await?;
            }

        // Ensure there is not an unneeded migrations file
        if !migrations_needed && migrations_file.exists() {
            remove_file(migrations_file).await?
        }

        // Write current schema as JSON
        let json = serde_json::to_string_pretty(&schema)?;
        write(schemas_dir.join("v99.99.99.json"), json).await?;

        // Write current schema as Cypher DDL
        let cypher = kuzu_cypher::generate_schema(&schema);
        write(schemas_dir.join("current.cypher"), cypher).await?;

        // Generate Rust code
        let primary_keys = builder.get_primary_keys();
        let node_relations = builder.get_node_relationships();
        let union_types = ["Node", "Block", "Inline", "Author", "AuthorRoleAuthor"];
        let skip_types = builder.get_skip_types();
        let rust = kuzu_rust::generate_node_types(
            &schema,
            &primary_keys,
            node_relations,
            &union_types,
            self,
            skip_types,
        );
        write(dir.join("src").join("node_types.rs"), rust).await?;

        Ok(())
    }
}

/// Find the latest schema file in the schemas directory
async fn find_latest_schema(
    schemas_dir: &PathBuf,
) -> Result<Option<crate::kuzu_types::DatabaseSchema>> {
    use semver::Version;

    let mut entries = fs::read_dir(schemas_dir).await?;
    let mut latest_version: Option<Version> = None;
    let mut latest_file: Option<String> = None;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str())
            && filename.starts_with("v") && filename.ends_with(".json") {
                // Extract version from filename like "v2.6.0.json"
                if let Some(version_str) = filename
                    .strip_prefix("v")
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    // Skip the temporary development version
                    if version_str == "99.99.99" {
                        continue;
                    }

                    if let Ok(version) = Version::parse(version_str)
                        && latest_version.as_ref().is_none_or(|v| version > *v) {
                            latest_version = Some(version);
                            latest_file = Some(path.to_string_lossy().to_string());
                        }
                }
            }
    }

    if let Some(file_path) = latest_file {
        kuzu_cypher::load_previous_schema(&file_path).await
    } else {
        Ok(None)
    }
}
