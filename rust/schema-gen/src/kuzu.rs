use std::path::PathBuf;

use common::{eyre::Result, serde_json, tokio::fs::write};
use version::STENCILA_VERSION;

use crate::{kuzu_builder::KuzuSchemaBuilder, kuzu_cypher, kuzu_rust, schemas::Schemas};

impl Schemas {
    /// Generate Kuzu database schema and Rust ORM code from Stencila Schema
    ///
    /// This function transforms the Stencila Schema type definitions into a
    /// Kuzu graph database schema with corresponding Rust code for data access.
    /// The generation process uses an abstraction layer to separate schema
    /// analysis from code generation, making the system more maintainable and
    /// extensible.
    ///
    /// ## What is generated:
    ///
    /// **Database Schema (`current.cypher`):**
    /// - Node tables for each Stencila type (Article, Paragraph, etc.)
    /// - Relationship tables connecting related entities
    /// - Full-text search and vector indices for semantic operations
    ///
    /// **Rust ORM (`node_types.rs`):**
    /// - `DatabaseNode` trait implementations for all types
    /// - Methods to extract table data and relationships from Rust structs
    /// - Union type support for polymorphic node handling
    #[allow(clippy::print_stderr)]
    pub async fn kuzu(&self) -> Result<()> {
        eprintln!("Generating Kuzu Schema");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-db");

        // Build the database schema
        let mut builder = KuzuSchemaBuilder::new(self);
        let schema = builder.build()?;

        // Write schema as JSON
        let schema_json = serde_json::to_string_pretty(&schema)?;
        let schema_filename = format!("v{STENCILA_VERSION}.json");
        write(dir.join("schemas").join(schema_filename), schema_json).await?;

        // Generate Cypher DDL
        let cypher = kuzu_cypher::generate_schema(&schema);
        write(dir.join("schemas").join("current.cypher"), cypher).await?;

        // Generate Rust code
        let primary_keys = builder.get_primary_keys();
        let node_relations = builder.get_node_relations();
        let union_types = ["Node", "Block", "Inline", "Author", "AuthorRoleAuthor"];
        let skip_types = builder.get_skip_types();
        let rust = kuzu_rust::generate_node_types(
            &schema,
            &primary_keys,
            node_relations,
            &union_types,
            self,
            &skip_types,
        );
        write(dir.join("src").join("node_types.rs"), rust).await?;

        Ok(())
    }
}
