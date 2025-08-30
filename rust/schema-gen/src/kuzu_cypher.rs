use std::collections::{HashMap, HashSet};

use common::{eyre::Result, itertools::Itertools, serde_json, tokio::fs::read_to_string};

use crate::kuzu_types::{
    Cardinality, Column, DataType, DatabaseSchema, DerivedProperty, FromToPair, Index, NodeTable,
    RelationshipTable,
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

/// Represents a schema migration operation
#[derive(Debug, Clone, PartialEq, Eq)]
enum MigrationOperation {
    /// Create a new node table
    CreateNodeTable(NodeTable),
    /// Create a new relationship table
    CreateRelationshipTable(RelationshipTable),
    /// Drop an existing table
    DropTable { name: String },
    /// Add a column to an existing table
    AddColumn { table: String, column: Column },
    /// Drop a column from an existing table
    DropColumn { table: String, column_name: String },
    /// Change a column's data type (implemented as add new, copy data, drop old, rename)
    ChangeColumnType {
        table: String,
        column_name: String,
        old_type: DataType,
        new_type: DataType,
    },
    /// Add a derived property to a table
    AddDerivedProperty {
        table: String,
        property: DerivedProperty,
    },
    /// Remove a derived property from a table
    RemoveDerivedProperty {
        table: String,
        property_name: String,
    },
    /// Create a new index
    CreateIndex(Index),
    /// Drop an existing index
    DropIndex { table: String, name: String },
    /// Add embeddings to a table
    AddEmbeddings { table: String },
    /// Remove embeddings from a table
    RemoveEmbeddings { table: String },
}

/// Compares two database schemas and generates migration operations
fn generate_migration_operations(
    old_schema: &DatabaseSchema,
    new_schema: &DatabaseSchema,
) -> Vec<MigrationOperation> {
    let mut operations = Vec::new();

    // Create maps for efficient lookups
    let old_tables: HashMap<String, &NodeTable> = old_schema
        .node_tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();
    let new_tables: HashMap<String, &NodeTable> = new_schema
        .node_tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    let old_rel_tables: HashMap<String, &RelationshipTable> = old_schema
        .relationship_tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();
    let new_rel_tables: HashMap<String, &RelationshipTable> = new_schema
        .relationship_tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    // Find new node tables
    for (name, new_table) in &new_tables {
        if !old_tables.contains_key(name) {
            operations.push(MigrationOperation::CreateNodeTable((*new_table).clone()));
        }
    }

    // Find removed node tables
    for name in old_tables.keys() {
        if !new_tables.contains_key(name) {
            operations.push(MigrationOperation::DropTable {
                name: name.clone(),
            });
        }
    }

    // Compare existing node tables
    for (name, new_table) in &new_tables {
        if let Some(old_table) = old_tables.get(name) {
            operations.extend(compare_node_tables(old_table, new_table));
        }
    }

    // Find new relationship tables
    for (name, new_table) in &new_rel_tables {
        if !old_rel_tables.contains_key(name) {
            operations.push(MigrationOperation::CreateRelationshipTable(
                (*new_table).clone(),
            ));
        }
    }

    // Find removed relationship tables
    for name in old_rel_tables.keys() {
        if !new_rel_tables.contains_key(name) {
            operations.push(MigrationOperation::DropTable {
                name: name.clone(),
            });
        }
    }

    // Compare indices
    operations.extend(compare_indices(&old_schema.indices, &new_schema.indices));

    operations
}

/// Compare two node tables and generate operations for differences
fn compare_node_tables(old_table: &NodeTable, new_table: &NodeTable) -> Vec<MigrationOperation> {
    let mut operations = Vec::new();
    let table_name = &new_table.name;

    // Compare columns
    let old_columns: HashMap<String, &Column> = old_table
        .columns
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();
    let new_columns: HashMap<String, &Column> = new_table
        .columns
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    // Find new columns
    for (name, new_column) in &new_columns {
        if !old_columns.contains_key(name) {
            operations.push(MigrationOperation::AddColumn {
                table: table_name.clone(),
                column: (*new_column).clone(),
            });
        }
    }

    // Find removed columns
    for name in old_columns.keys() {
        if !new_columns.contains_key(name) {
            operations.push(MigrationOperation::DropColumn {
                table: table_name.clone(),
                column_name: name.clone(),
            });
        }
    }

    // Find changed column types
    for (name, new_column) in &new_columns {
        if let Some(old_column) = old_columns.get(name) {
            if old_column.data_type != new_column.data_type {
                operations.push(MigrationOperation::ChangeColumnType {
                    table: table_name.clone(),
                    column_name: name.clone(),
                    old_type: old_column.data_type.clone(),
                    new_type: new_column.data_type.clone(),
                });
            }
        }
    }

    // Compare derived properties
    let old_props: HashMap<String, &DerivedProperty> = old_table
        .derived_properties
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();
    let new_props: HashMap<String, &DerivedProperty> = new_table
        .derived_properties
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    // Find new derived properties
    for (name, new_prop) in &new_props {
        if !old_props.contains_key(name) {
            operations.push(MigrationOperation::AddDerivedProperty {
                table: table_name.clone(),
                property: (*new_prop).clone(),
            });
        }
    }

    // Find removed derived properties
    for name in old_props.keys() {
        if !new_props.contains_key(name) {
            operations.push(MigrationOperation::RemoveDerivedProperty {
                table: table_name.clone(),
                property_name: name.clone(),
            });
        }
    }

    // Compare embeddings
    if old_table.has_embeddings != new_table.has_embeddings {
        if new_table.has_embeddings {
            operations.push(MigrationOperation::AddEmbeddings {
                table: table_name.clone(),
            });
        } else {
            operations.push(MigrationOperation::RemoveEmbeddings {
                table: table_name.clone(),
            });
        }
    }

    operations
}

/// Compare indices and generate operations for differences
fn compare_indices(old_indices: &[Index], new_indices: &[Index]) -> Vec<MigrationOperation> {
    let mut operations = Vec::new();

    // Create sets for comparison (indices are compared by their properties)
    let old_set: HashSet<&Index> = old_indices.iter().collect();
    let new_set: HashSet<&Index> = new_indices.iter().collect();

    // Find new indices
    for index in new_set.difference(&old_set) {
        operations.push(MigrationOperation::CreateIndex((*index).clone()));
    }

    // Find removed indices
    for index in old_set.difference(&new_set) {
        let (table, name) = match index {
            Index::FullTextSearch { table, name, .. } => (table, name),
            Index::Vector { table, name, .. } => (table, name),
        };
        operations.push(MigrationOperation::DropIndex {
            table: table.clone(),
            name: name.clone(),
        });
    }

    operations
}

/// Generate Cypher DDL from migration operations
fn generate_migration_cypher(operations: &[MigrationOperation]) -> String {
    let mut cypher_statements = Vec::new();

    for operation in operations {
        match operation {
            MigrationOperation::CreateNodeTable(table) => {
                cypher_statements.push(generate_node_table(table));
            }
            MigrationOperation::CreateRelationshipTable(table) => {
                cypher_statements.push(generate_relationship_table(table));
            }
            MigrationOperation::DropTable { name } => {
                cypher_statements.push(format!("DROP TABLE `{}`;", name));
            }
            MigrationOperation::AddColumn { table, column } => {
                let default_value = if column.nullable {
                    "DEFAULT NULL"
                } else {
                    ""
                };
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` ADD COLUMN `{}` {} {};",
                    table,
                    column.name,
                    generate_data_type(&column.data_type),
                    default_value
                ));
            }
            MigrationOperation::DropColumn { table, column_name } => {
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` DROP COLUMN `{}`;",
                    table, column_name
                ));
            }
            MigrationOperation::ChangeColumnType {
                table,
                column_name,
                new_type,
                ..
            } => {
                // Multi-step column type change
                let temp_name = format!("{}_new", column_name);
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` ADD COLUMN `{}` {};",
                    table,
                    temp_name,
                    generate_data_type(new_type)
                ));
                cypher_statements.push(format!(
                    "-- TODO: Copy and transform data from `{}` to `{}`",
                    column_name, temp_name
                ));
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` DROP COLUMN `{}`;",
                    table, column_name
                ));
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` RENAME COLUMN `{}` TO `{}`;",
                    table, temp_name, column_name
                ));
            }
            MigrationOperation::AddDerivedProperty { table, property } => {
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` ADD COLUMN `{}` STRING DEFAULT NULL;",
                    table, property.name
                ));
            }
            MigrationOperation::RemoveDerivedProperty {
                table,
                property_name,
            } => {
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` DROP COLUMN `{}`;",
                    table, property_name
                ));
            }
            MigrationOperation::CreateIndex(index) => {
                cypher_statements.push(generate_index(index));
            }
            MigrationOperation::DropIndex { table: _, name } => {
                cypher_statements.push(format!("-- TODO: Drop index '{}'", name));
            }
            MigrationOperation::AddEmbeddings { table } => {
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` ADD COLUMN `embeddings` FLOAT[384] DEFAULT NULL;",
                    table
                ));
            }
            MigrationOperation::RemoveEmbeddings { table } => {
                cypher_statements.push(format!(
                    "ALTER TABLE `{}` DROP COLUMN `embeddings`;",
                    table
                ));
            }
        }
    }

    cypher_statements.join("\n\n")
}

/// Generate a description for the migration based on operations
fn generate_migration_description(operations: &[MigrationOperation]) -> String {
    if operations.is_empty() {
        return "no-changes".to_string();
    }

    let mut descriptions = Vec::new();
    let mut has_new_tables = false;
    let mut has_dropped_tables = false;
    let mut has_column_changes = false;
    let mut has_index_changes = false;

    for operation in operations {
        match operation {
            MigrationOperation::CreateNodeTable(_)
            | MigrationOperation::CreateRelationshipTable(_) => {
                has_new_tables = true;
            }
            MigrationOperation::DropTable { .. } => {
                has_dropped_tables = true;
            }
            MigrationOperation::AddColumn { .. }
            | MigrationOperation::DropColumn { .. }
            | MigrationOperation::ChangeColumnType { .. }
            | MigrationOperation::AddDerivedProperty { .. }
            | MigrationOperation::RemoveDerivedProperty { .. }
            | MigrationOperation::AddEmbeddings { .. }
            | MigrationOperation::RemoveEmbeddings { .. } => {
                has_column_changes = true;
            }
            MigrationOperation::CreateIndex(_) | MigrationOperation::DropIndex { .. } => {
                has_index_changes = true;
            }
        }
    }

    if has_new_tables {
        descriptions.push("add-tables");
    }
    if has_dropped_tables {
        descriptions.push("remove-tables");
    }
    if has_column_changes {
        descriptions.push("update-columns");
    }
    if has_index_changes {
        descriptions.push("update-indices");
    }

    if descriptions.is_empty() {
        "schema-updates".to_string()
    } else {
        descriptions.join("-")
    }
}

/// Load the previous schema from a JSON file
pub async fn load_previous_schema(file_path: &str) -> Result<Option<DatabaseSchema>> {
    match read_to_string(file_path).await {
        Ok(content) => {
            let schema: DatabaseSchema = serde_json::from_str(&content)?;
            Ok(Some(schema))
        }
        Err(_) => Ok(None), // File doesn't exist or can't be read
    }
}

/// Generate migration if schemas differ, returning filename and cypher content
pub fn generate_migration(
    old_schema: &DatabaseSchema,
    new_schema: &DatabaseSchema,
    version: &str,
) -> Option<(String, String)> {
    if old_schema != new_schema {
        let operations = generate_migration_operations(old_schema, new_schema);
        if !operations.is_empty() {
            let migration_cypher = generate_migration_cypher(&operations);
            let description = generate_migration_description(&operations);
            let migration_filename = format!("v{}-{}.cypher", version, description);
            return Some((migration_filename, migration_cypher));
        }
    }
    None
}
