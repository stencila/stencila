use std::collections::BTreeMap;

use inflector::Inflector;
use itertools::Itertools;

use crate::kuzu_types::{Column, DatabaseSchema, DerivedProperty, NodeTable, RelationshipInfo};

/// Generates Rust `DatabaseNode` implementations from a DatabaseSchema
pub fn generate_node_types(
    schema: &DatabaseSchema,
    primary_keys: &BTreeMap<String, String>,
    node_relationships: &BTreeMap<String, Vec<RelationshipInfo>>,
    union_types: &[&str],
    schemas: &crate::schemas::Schemas,
    skip_types: &[&str],
) -> String {
    let mut result = r"// Generated file, do not edit. See the Rust `schema-gen` crate.

use kernel_kuzu::{kuzu::{LogicalType, Value}, ToKuzu};
use codec_text_trait::to_text;
use schema::*;

use super::{embeddings_property, embeddings_type, DatabaseNode};

"
    .to_string();

    // Helper functions
    result.push_str(&generate_primary_key_function(primary_keys));
    result.push_str("\n\n");
    result.push_str(&generate_relations_function());
    result.push_str("\n\n");

    // Individual node implementations
    for table in &schema.node_tables {
        let relations = node_relationships
            .get(&table.name)
            .cloned()
            .unwrap_or_default();
        result.push_str(&generate_database_node_impl(
            table,
            primary_keys,
            &relations,
        ));
        result.push_str("\n\n");
    }

    // Union type implementations
    for union_type in union_types {
        if let Some(union_impl) = generate_union_impl(union_type, schemas, skip_types) {
            result.push_str(&union_impl);
            result.push_str("\n\n");
        }
    }
    result
}

fn generate_primary_key_function(primary_keys: &BTreeMap<String, String>) -> String {
    let match_arms = primary_keys
        .iter()
        .map(|(node_type, key)| format!("        NodeType::{node_type} => \"{key}\","))
        .join("\n");

    format!(
        "pub(super) fn primary_key(node_type: &NodeType) -> &'static str {{\n    match node_type {{\n{match_arms}\n        _ => \"nodeId\"\n    }}\n}}",
    )
}

fn generate_relations_function() -> String {
    r#"fn relations<'lt, I, D>(iter: I) -> Vec<(NodeType, Value)>
where
    I: Iterator<Item = &'lt D>,
    D: DatabaseNode + 'lt,
{
    iter.flat_map(|item| (!matches!(item.node_type(), NodeType::Unknown)).then_some((item.node_type(), item.primary_key())))
        .collect()
}"#.to_string()
}

fn generate_database_node_impl(
    table: &NodeTable,
    primary_keys: &BTreeMap<String, String>,
    relationships: &[RelationshipInfo],
) -> String {
    let name = &table.name;

    let primary_key = primary_keys
        .get(name)
        .map(|key| format!("self.{key}.to_kuzu_value()"))
        .unwrap_or_else(|| "self.node_id().to_kuzu_value()".to_string());

    let node_table_props = generate_node_table_properties(table);
    let rel_table_props = generate_rel_table_properties(relationships);

    format!(
        r#"impl DatabaseNode for {name} {{
    fn node_type(&self) -> NodeType {{
        NodeType::{name}
    }}

    fn node_id(&self) -> NodeId {{
        {name}::node_id(self)
    }}
    
    fn primary_key(&self) -> Value {{
        {primary_key}
    }}
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        vec![
            {node_table_props}
        ]
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        vec![
            {rel_table_props}
        ]
    }}
}}"#
    )
}

fn generate_node_table_properties(table: &NodeTable) -> String {
    let mut props = Vec::new();

    // Regular columns
    for column in &table.columns {
        props.push(generate_column_property(column));
    }

    // Derived properties
    for derived in &table.derived_properties {
        props.push(generate_derived_property(derived));
    }

    // Embeddings
    if table.has_embeddings {
        props.push("(embeddings_property(), embeddings_type(), Null.to_kuzu_value())".to_string());
    }

    props.join(",\n            ")
}

fn generate_column_property(column: &Column) -> String {
    let mut property = column.name.to_pascal_case();
    if property.ends_with("ID") {
        property.pop();
        property.push('d');
    }

    let mut field = column.name.to_snake_case();
    if column.on_options {
        field = format!("options.{field}");
    }

    format!("(NodeProperty::{property}, self.{field}.to_kuzu_type(), self.{field}.to_kuzu_value())",)
}

fn generate_derived_property(derived: &DerivedProperty) -> String {
    let property = derived.name.to_pascal_case();
    format!(
        "(NodeProperty::{property}, LogicalType::String, {}.to_kuzu_value())",
        derived.derivation
    )
}

fn generate_rel_table_properties(relationships: &[RelationshipInfo]) -> String {
    relationships
        .iter()
        .map(|rel| {
            let property = rel.name.to_pascal_case();
            let collect = generate_relation_collect(rel);
            format!("(NodeProperty::{property}, {collect})")
        })
        .join(",\n            ")
}

fn generate_relation_collect(relationship: &RelationshipInfo) -> String {
    if relationship.ref_type == "AuthorRoleAuthor" {
        let mut field = relationship.name.to_snake_case();
        if field == "abstract" {
            field = "r#abstract".to_string();
        }
        if relationship.on_options {
            field = format!("options.{field}");
        }
        format!("vec![(self.{field}.node_type(), self.{field}.primary_key())]",)
    } else {
        let mut field = relationship.name.to_snake_case();
        if field == "abstract" {
            field = "r#abstract".to_string();
        }
        if relationship.on_options {
            field = format!("options.{field}");
        }

        let mut collect = format!("self.{field}");

        if relationship.is_option || relationship.is_array {
            collect += ".iter()";
        }

        if relationship.is_box {
            collect += ".map(|boxed| boxed.as_ref())";
        }

        if relationship.is_option && relationship.is_array {
            collect += ".flatten()";
        }

        format!("relations({collect})")
    }
}

fn generate_union_impl(
    union_type: &str,
    schemas: &crate::schemas::Schemas,
    skip_types: &[&str],
) -> Option<String> {
    let variants = get_union_variants(union_type, schemas, skip_types);

    if variants.is_empty() {
        return None;
    }

    let node_type_arms = variants
        .iter()
        .map(|variant| format!("            {union_type}::{variant}(node) => node.node_type()",))
        .join(",\n");

    let node_id_arms = variants
        .iter()
        .map(|variant| format!("            {union_type}::{variant}(node) => node.node_id()",))
        .join(",\n");

    let primary_key_arms = variants
        .iter()
        .map(|variant| format!("            {union_type}::{variant}(node) => node.primary_key()",))
        .join(",\n");

    let node_table_arms = variants
        .iter()
        .map(|variant| format!("            {union_type}::{variant}(node) => node.node_table()",))
        .join(",\n");

    let rel_tables_arms = variants
        .iter()
        .map(|variant| format!("            {union_type}::{variant}(node) => node.rel_tables()",))
        .join(",\n");

    format!(
        r#"#[allow(unreachable_patterns)]
impl DatabaseNode for {union_type} {{
    fn node_type(&self) -> NodeType {{
        match self {{
{node_type_arms},
            _ => NodeType::Unknown
        }}
    }}

    fn node_id(&self) -> NodeId {{
        match self {{
{node_id_arms},
            _ => NodeId::null()
        }}
    }}

    fn primary_key(&self) -> Value {{
        match self {{
{primary_key_arms},
            _ => Value::Null(LogicalType::Any)
        }}
    }}

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        match self {{
{node_table_arms},
            _ => Vec::new()
        }}
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        match self {{
{rel_tables_arms},
            _ => Vec::new()
        }}
    }}
}}"#,
    )
    .into()
}

fn get_union_variants(
    union_type: &str,
    schemas: &crate::schemas::Schemas,
    skip_types: &[&str],
) -> Vec<String> {
    if let Some(schema) = schemas.schemas.get(union_type)
        && let Some(any_of) = &schema.any_of
    {
        return any_of
            .iter()
            .filter_map(|schema| {
                let variant = schema.r#ref.as_deref()?;
                (!skip_types.contains(&variant)).then_some(variant.to_string())
            })
            .collect();
    }
    Vec::new()
}
