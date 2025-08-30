use common::itertools::Itertools;

use crate::kuzu_types::{Index, DatabaseSchema};

/// Generates Cypher DDL from a KuzuSchema
pub struct CypherGenerator;

impl CypherGenerator {
    pub fn generate_schema(schema: &DatabaseSchema) -> String {
        let mut parts = Vec::new();

        // Header comment
        parts.push("// Generated file, do not edit. See the Rust `schema-gen` crate;".to_string());
        parts.push("".to_string());

        // Node tables
        if !schema.node_tables.is_empty() {
            let node_tables = schema
                .node_tables
                .iter()
                .map(|table| table.to_string())
                .join("\n\n");
            parts.push(node_tables);
        }

        // Collect relationship tables by cardinality
        let (one_to_one, one_to_many, many_to_many): (Vec<_>, Vec<_>, Vec<_>) = schema
            .relationship_tables
            .iter()
            .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, table| {
                match table.cardinality {
                    crate::kuzu_types::Cardinality::OneToOne => acc.0.push(table),
                    crate::kuzu_types::Cardinality::OneToMany => acc.1.push(table),
                    crate::kuzu_types::Cardinality::ManyToMany => acc.2.push(table),
                }
                acc
            });

        // One-to-one relationships
        if !one_to_one.is_empty() {
            parts.push("".to_string());
            let tables = one_to_one
                .iter()
                .map(|table| table.to_string())
                .join("\n\n");
            parts.push(tables);
        }

        // One-to-many relationships
        if !one_to_many.is_empty() {
            parts.push("".to_string());
            let tables = one_to_many
                .iter()
                .map(|table| table.to_string())
                .join("\n\n");
            parts.push(tables);
        }

        // Many-to-many relationships
        if !many_to_many.is_empty() {
            parts.push("".to_string());
            let tables = many_to_many
                .iter()
                .map(|table| table.to_string())
                .join("\n\n");
            parts.push(tables);
        }

        // FTS setup and indices
        let fts_indices: Vec<_> = schema
            .indices
            .iter()
            .filter_map(|idx| match idx {
                Index::FullTextSearch { .. } => Some(idx.to_string()),
                _ => None,
            })
            .collect();

        if !fts_indices.is_empty() {
            parts.push("".to_string());
            parts.push("INSTALL FTS;".to_string());
            parts.push("LOAD EXTENSION FTS;".to_string());
            parts.push(fts_indices.join("\n"));
        }

        // Vector setup and indices  
        let vector_indices: Vec<_> = schema
            .indices
            .iter()
            .filter_map(|idx| match idx {
                Index::Vector { .. } => Some(idx.to_string()),
                _ => None,
            })
            .collect();

        if !vector_indices.is_empty() {
            parts.push("".to_string());
            parts.push("INSTALL VECTOR;".to_string());
            parts.push("LOAD EXTENSION VECTOR;".to_string());
            parts.push(vector_indices.join("\n"));
        }

        parts.push("".to_string()); // Final newline
        parts.join("\n")
    }
}