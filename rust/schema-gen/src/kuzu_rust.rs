use std::collections::BTreeMap;

use common::{inflector::Inflector, itertools::Itertools};

use crate::kuzu_types::{Column, DatabaseSchema, DerivedProperty, NodeTable, RelationInfo};

/// Generates Rust `DatabaseNode` implementations from a DatabaseSchema
pub fn generate_node_types(
    schema: &DatabaseSchema,
    primary_keys: &BTreeMap<String, String>,
    node_relations: &BTreeMap<String, Vec<RelationInfo>>,
    union_types: &[&str],
    schemas: &crate::schemas::Schemas,
    skip_types: &[&str],
) -> String {
    let mut parts = Vec::new();

    // Header
    parts.push("// Generated file, do not edit. See the Rust `schema-gen` crate.".to_string());
    parts.push("".to_string());
    parts.push("use kernel_kuzu::{kuzu::{LogicalType, Value}, ToKuzu};".to_string());
    parts.push("use codec_text_trait::to_text;".to_string());
    parts.push("use schema::*;".to_string());
    parts.push("".to_string());
    parts.push("use super::{embeddings_property, embeddings_type, DatabaseNode};".to_string());
    parts.push("".to_string());

    // Primary key function
    parts.push(generate_primary_key_function(primary_keys));
    parts.push("".to_string());

    // Relations helper function
    parts.push(generate_relations_function());
    parts.push("".to_string());

    // Individual node implementations
    for table in &schema.node_tables {
        let relations = node_relations.get(&table.name).cloned().unwrap_or_default();
        parts.push(generate_database_node_impl(table, primary_keys, &relations));
        parts.push("".to_string());
    }

    // Union type implementations
    for union_type in union_types {
        if let Some(union_impl) = generate_union_impl(union_type, schemas, skip_types) {
            parts.push(union_impl);
            parts.push("".to_string());
        }
    }

    let mut result = parts.join("\n");
    result.push('\n');
    result
}

fn generate_primary_key_function(primary_keys: &BTreeMap<String, String>) -> String {
    let match_arms = primary_keys
        .iter()
        .map(|(node_type, key)| format!("        NodeType::{} => \"{}\",", node_type, key))
        .join("\n");

    format!(
        "pub(super) fn primary_key(node_type: &NodeType) -> &'static str {{\n    match node_type {{\n{}\n        _ => \"nodeId\"\n    }}\n}}",
        match_arms
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
    relations: &[RelationInfo],
) -> String {
    let name = &table.name;

    let primary_key = primary_keys
        .get(name)
        .map(|key| format!("self.{}.to_kuzu_value()", key))
        .unwrap_or_else(|| "self.node_id().to_kuzu_value()".to_string());

    let node_table_props = generate_node_table_properties(table);
    let rel_table_props = generate_rel_table_properties(relations);

    format!(
        r#"impl DatabaseNode for {} {{
    fn node_type(&self) -> NodeType {{
        NodeType::{}
    }}

    fn node_id(&self) -> NodeId {{
        {}::node_id(self)
    }}
    
    fn primary_key(&self) -> Value {{
        {}
    }}
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        vec![
            {}
        ]
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        vec![
            {}
        ]
    }}
}}"#,
        name, name, name, primary_key, node_table_props, rel_table_props
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
        field = format!("options.{}", field);
    }

    format!(
        "(NodeProperty::{}, self.{}.to_kuzu_type(), self.{}.to_kuzu_value())",
        property, field, field
    )
}

fn generate_derived_property(derived: &DerivedProperty) -> String {
    let property = derived.name.to_pascal_case();
    format!(
        "(NodeProperty::{}, LogicalType::String, {}.to_kuzu_value())",
        property, derived.derivation
    )
}

fn generate_rel_table_properties(relations: &[RelationInfo]) -> String {
    relations
        .iter()
        .map(|rel| {
            let property = rel.name.to_pascal_case();
            let collect = generate_relation_collect(rel);
            format!("(NodeProperty::{}, {})", property, collect)
        })
        .join(",\n            ")
}

fn generate_relation_collect(rel: &RelationInfo) -> String {
    if rel.ref_type == "AuthorRoleAuthor" {
        let mut field = rel.name.to_snake_case();
        if field == "abstract" {
            field = "r#abstract".to_string();
        }
        if rel.on_options {
            field = format!("options.{}", field);
        }
        format!(
            "vec![(self.{}.node_type(), self.{}.primary_key())]",
            field, field
        )
    } else {
        let mut field = rel.name.to_snake_case();
        if field == "abstract" {
            field = "r#abstract".to_string();
        }
        if rel.on_options {
            field = format!("options.{}", field);
        }

        let mut collect = format!("self.{}", field);

        if rel.is_option || rel.is_array {
            collect += ".iter()";
        }

        if rel.is_box {
            collect += ".map(|boxed| boxed.as_ref())";
        }

        if rel.is_option && rel.is_array {
            collect += ".flatten()";
        }

        format!("relations({})", collect)
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
        .map(|variant| {
            format!(
                "            {}::{}(node) => node.node_type()",
                union_type, variant
            )
        })
        .join(",\n");

    let node_id_arms = variants
        .iter()
        .map(|variant| {
            format!(
                "            {}::{}(node) => node.node_id()",
                union_type, variant
            )
        })
        .join(",\n");

    let primary_key_arms = variants
        .iter()
        .map(|variant| {
            format!(
                "            {}::{}(node) => node.primary_key()",
                union_type, variant
            )
        })
        .join(",\n");

    let node_table_arms = variants
        .iter()
        .map(|variant| {
            format!(
                "            {}::{}(node) => node.node_table()",
                union_type, variant
            )
        })
        .join(",\n");

    let rel_tables_arms = variants
        .iter()
        .map(|variant| {
            format!(
                "            {}::{}(node) => node.rel_tables()",
                union_type, variant
            )
        })
        .join(",\n");

    format!(
        r#"#[allow(unreachable_patterns)]
impl DatabaseNode for {} {{
    fn node_type(&self) -> NodeType {{
        match self {{
{},
            _ => NodeType::Unknown
        }}
    }}

    fn node_id(&self) -> NodeId {{
        match self {{
{},
            _ => NodeId::null()
        }}
    }}

    fn primary_key(&self) -> Value {{
        match self {{
{},
            _ => Value::Null(LogicalType::Any)
        }}
    }}

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        match self {{
{},
            _ => Vec::new()
        }}
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        match self {{
{},
            _ => Vec::new()
        }}
    }}
}}"#,
        union_type,
        node_type_arms,
        node_id_arms,
        primary_key_arms,
        node_table_arms,
        rel_tables_arms
    )
    .into()
}

fn get_union_variants(
    union_type: &str,
    schemas: &crate::schemas::Schemas,
    skip_types: &[&str],
) -> Vec<String> {
    if let Some(schema) = schemas.schemas.get(union_type) {
        if let Some(any_of) = &schema.any_of {
            return any_of
                .iter()
                .filter_map(|schema| {
                    let variant = schema.r#ref.as_deref()?;
                    (!skip_types.contains(&variant)).then_some(variant.to_string())
                })
                .collect();
        }
    }
    Vec::new()
}
