use common::itertools::Itertools;

use crate::kuzu_types::{
    Cardinality, DataType, DatabaseSchema, FromToPair, Index, NodeTable, RelationshipTable,
};

/// Generates Cypher DDL from a DatabaseSchema
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
            .map(|table| generate_node_table(table))
            .join("\n\n");
        parts.push(node_tables);
    }

    // Collect relationship tables by cardinality
    let (one_to_one, one_to_many, many_to_many): (Vec<_>, Vec<_>, Vec<_>) = schema
        .relationship_tables
        .iter()
        .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, table| {
            match table.cardinality {
                Cardinality::OneToOne => acc.0.push(table),
                Cardinality::OneToMany => acc.1.push(table),
                Cardinality::ManyToMany => acc.2.push(table),
            }
            acc
        });

    // One-to-one relationships
    if !one_to_one.is_empty() {
        parts.push("".to_string());
        let tables = one_to_one
            .iter()
            .map(|table| generate_relationship_table(table))
            .join("\n\n");
        parts.push(tables);
    }

    // One-to-many relationships
    if !one_to_many.is_empty() {
        parts.push("".to_string());
        let tables = one_to_many
            .iter()
            .map(|table| generate_relationship_table(table))
            .join("\n\n");
        parts.push(tables);
    }

    // Many-to-many relationships
    if !many_to_many.is_empty() {
        parts.push("".to_string());
        let tables = many_to_many
            .iter()
            .map(|table| generate_relationship_table(table))
            .join("\n\n");
        parts.push(tables);
    }

    // FTS setup and indices
    let fts_indices: Vec<_> = schema
        .indices
        .iter()
        .filter_map(|idx| match idx {
            Index::FullTextSearch { .. } => Some(generate_index(idx)),
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
            Index::Vector { .. } => Some(generate_index(idx)),
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

fn generate_data_type(data_type: &DataType) -> String {
    match data_type {
        DataType::Null => "NULL".to_string(),
        DataType::Boolean => "BOOLEAN".to_string(),
        DataType::Int64 => "INT64".to_string(),
        DataType::UInt64 => "UINT64".to_string(),
        DataType::Double => "DOUBLE".to_string(),
        DataType::String => "STRING".to_string(),
        DataType::Date => "DATE".to_string(),
        DataType::Timestamp => "TIMESTAMP".to_string(),
        DataType::Interval => "INTERVAL".to_string(),
        DataType::BooleanArray => "BOOLEAN[]".to_string(),
        DataType::Int64Array => "INT64[]".to_string(),
        DataType::DoubleArray => "DOUBLE[]".to_string(),
        DataType::StringArray => "STRING[]".to_string(),
        DataType::FloatArray(dim) => format!("FLOAT[{}]", dim),
    }
}

fn generate_cardinality(cardinality: &Cardinality) -> String {
    match cardinality {
        Cardinality::OneToOne => "ONE_ONE".to_string(),
        Cardinality::OneToMany => "ONE_MANY".to_string(),
        Cardinality::ManyToMany => "MANY_MANY".to_string(),
    }
}

fn generate_from_to_pair(pair: &FromToPair) -> String {
    format!("FROM `{}` TO `{}`", pair.from, pair.to)
}

fn generate_node_table(table: &NodeTable) -> String {
    let mut result = format!("CREATE NODE TABLE IF NOT EXISTS `{}` (", table.name);

    // Regular columns
    for column in &table.columns {
        result.push_str(&format!(
            "\n  `{}` {},",
            column.name,
            generate_data_type(&column.data_type)
        ));
    }

    // Derived properties
    for derived in &table.derived_properties {
        result.push_str(&format!("\n  `{}` STRING,", derived.name));
    }

    // Embeddings
    if table.has_embeddings {
        result.push_str("\n  `embeddings` FLOAT[384],");
    }

    // Primary key or standard node fields
    if let Some(ref primary_key) = table.primary_key {
        result.push_str(&format!("\n  PRIMARY KEY (`{}`)", primary_key));
    } else if table.has_standard_node_fields {
        result.push_str("\n  `docId` STRING,\n  `nodeId` STRING PRIMARY KEY,\n  `nodePath` STRING,\n  `nodeAncestors` STRING,\n  `position` UINT32");
    }

    result.push_str("\n);");
    result
}

fn generate_relationship_table(table: &RelationshipTable) -> String {
    let mut result = format!("CREATE REL TABLE IF NOT EXISTS `{}` (\n  ", table.name);

    let pairs = table
        .pairs
        .iter()
        .map(|pair| generate_from_to_pair(pair))
        .join(",\n  ");
    result.push_str(&pairs);

    result.push_str(&format!(
        ",\n  {}\n);",
        generate_cardinality(&table.cardinality)
    ));
    result
}

fn generate_index(index: &Index) -> String {
    match index {
        Index::FullTextSearch {
            table,
            name,
            properties,
        } => {
            let props = properties.iter().map(|p| format!("'{}'", p)).join(",");
            format!(
                "CALL CREATE_FTS_INDEX('{}', '{}', [{}]);",
                table, name, props
            )
        }
        Index::Vector {
            table,
            name,
            property,
        } => {
            format!(
                "CALL CREATE_VECTOR_INDEX('{}', '{}', '{}');",
                table, name, property
            )
        }
    }
}
