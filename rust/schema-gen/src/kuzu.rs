use std::{collections::BTreeMap, path::PathBuf};

use common::{
    eyre::{bail, OptionExt, Result},
    inflector::Inflector,
    itertools::Itertools,
    tokio::fs::write,
};

use crate::{
    schema::{Items, ItemsRef, ItemsType, Type},
    schemas::Schemas,
};

impl Schemas {
    /// Generate a Kuzu database schema from Stencila Schema
    ///
    /// For each node type in the Stencila Schema we create a node table e.g
    ///
    ///     CREATE NODE TABLE Paragraph (...)
    ///
    /// with table properties for primitive properties (e.g. strings, numbers).
    ///
    /// For entity type properties we create relationship tables which have `TO`
    /// and `FROM` pairs for every possible parent-child combination. e.g.
    ///
    ///     CREATE REL TABLE content(FROM Article TO Paragraph, FROM Article TO CodeBlock, ..)
    ///
    /// It is currently necessary to create this all in one big schema file which includes
    /// all node and relationship tables. We can can't create these tables on demand
    /// until this is implemented https://github.com/kuzudb/kuzu/issues/5051.
    #[allow(clippy::print_stderr)]
    pub async fn kuzu(&self) -> Result<()> {
        eprintln!("Generating Kuzu Schema");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-db/src");

        // Mostly config or derived properties unlikely to be queried for
        // so avoid bloating db with them
        let skip_props = [
            // General
            "type",
            "id",
            // Articles & other creative works
            "extra",
            "config",
            "text",
            "headings",
            "archive",
            // Others (not excluded by skip_types below)
            "executionInstance",
            "mathml",
            "value",
            // Avoid many paragraph nodes for each table cell with `text`
            // same as the `text` of the table cell itself (most table cells have a single paragraph)
            "TableCell.content",
        ];

        let skip_types = [
            // Object types for which tables are not created
            "Button",
            "CallArgument",
            "CallBlock",
            "Chat",
            "ChatMessage",
            "ChatMessageGroup",
            "CodeLocation",
            "CompilationDigest",
            "CompilationMessage",
            "Datatable",
            "DatatableColumn",
            "Enumeration",
            "ExecutionDependant",
            "ExecutionDependency",
            "ExecutionMessage",
            "ExecutionTag",
            "Form",
            "InstructionBlock",
            "InstructionInline",
            "InstructionMessage",
            "ModelParameters",
            "Prompt",
            "PromptBlock",
            "ProvenanceCount",
            "SuggestionBlock",
            "SuggestionInline",
            "Walkthrough",
            "WalkthroughStep",
            // Types that have equivalent Kuzu data types
            "Null",
            "Boolean",
            "Integer",
            "UnsignedInteger",
            "Number",
            "String",
            "Cord",
            "Text",
            "Array",
            "Object",
            "Date",
            "DateTime",
            "Time",
            "Timestamp",
            "Duration",
            // Union types (and variant object types) not expanded
            "Node",
            "ThingType",
            "CreativeWorkType",
            "Primitive",
            // .. hints
            "Hint",
            "ArrayHint",
            "DatatableHint",
            "DatatableColumnHint",
            "ObjectHint",
            "StringHint",
            // .. validators
            "Validator",
            "ArrayValidator",
            "BooleanValidator",
            "ConstantValidator",
            "DateTimeValidator",
            "DateValidator",
            "DurationValidator",
            "EnumValidator",
            "IntegerValidator",
            "NumberValidator",
            "StringValidator",
            "TimeValidator",
            "TimestampValidator",
            "TupleValidator",
        ];

        // Union types that need to be expanded
        let expand_types = ["Block", "Inline", "Author", "AuthorRoleAuthor"];

        // Node types for which a `text` property should be added with the plain text
        // representation of the node to be used in FTS and semantic search
        let add_text = ["Paragraph", "TableCell"];

        write(
            dir.join("fts_indices.rs"),
            format!(
                r#"// Generated file, do not edit. See the Rust `schema-gen` crate.

pub const FTS_INDICES: &[(&str, &[&str])] = &[
    ("CodeBlock",      &["code"]),
    ("CodeChunk",      &["code"]),
    ("CodeExpression", &["code"]),
    ("CodeInline",     &["code"]),
    ("MathBlock",      &["code"]),
    ("MathInline",     &["code"]),
    ("RawBlock",       &["content"]),
    ("Paragraph",      &["text"]),
    ("TableCell",      &["text"]),
];
"#
            ),
        )
        .await?;

        let mut node_tables = Vec::new();
        let mut one_to_many = BTreeMap::new();
        let mut one_to_one = BTreeMap::new();
        let mut implems = Vec::new();
        for (title, schema) in &self.schemas {
            if !schema.is_object()
                || schema.r#abstract
                || title.starts_with("Config")
                || skip_types.contains(&title.as_str())
            {
                continue;
            }

            let mut properties = Vec::new();
            let mut relations = Vec::new();
            for (name, property) in &schema.properties {
                if skip_props.contains(&name.as_str())
                    || skip_props.contains(&format!("{title}.{name}").as_str())
                {
                    continue;
                }

                let name = name.as_str();
                let on_options = !(property.is_required || property.is_core);
                let is_option = !property.is_required;
                let is_array = property.is_array();

                if let Some(property_type) = &property.r#type {
                    let data_type = match property_type {
                        Type::Null => "NULL",
                        Type::Boolean => "BOOLEAN",
                        Type::Integer => "INT64",
                        Type::Number => "DOUBLE",
                        Type::String => "STRING",
                        Type::Array => {
                            match property
                                .items
                                .as_ref()
                                .ok_or_eyre("type `array` should always have `items` specified")?
                            {
                                Items::Type(items_type) => {
                                    let ItemsType { r#type } = items_type;
                                    match r#type {
                                        Type::Null => "NULL[]",
                                        Type::Boolean => "BOOLEAN[]",
                                        Type::Integer => "INT64[]",
                                        Type::Number => "DOUBLE[]",
                                        Type::String => "STRING[]",
                                        _ => bail!("Unhandled items type: {type}"),
                                    }
                                }
                                Items::Ref(ItemsRef { r#ref: ref_type }) => {
                                    if skip_types.contains(&ref_type.as_str()) {
                                        continue;
                                    }

                                    let schema = self
                                        .schemas
                                        .get(ref_type)
                                        .ok_or_eyre("schema should exist")?;

                                    if schema.is_enumeration() {
                                        properties.push((name, "STRING[]", on_options));
                                        continue;
                                    }

                                    let mut pairs = if expand_types.contains(&ref_type.as_str()) {
                                        let variants = self
                                            .schemas
                                            .get(ref_type)
                                            .ok_or_eyre("schema should exist")?
                                            .any_of
                                            .as_ref()
                                            .ok_or_eyre("any_of should be some")?;
                                        variants
                                            .iter()
                                            .filter_map(|schema| {
                                                let variant =
                                                    schema.r#ref.as_deref().expect("should exit");

                                                (!skip_types.contains(&variant)).then_some(format!(
                                                    "FROM `{title}` TO `{variant}`"
                                                ))
                                            })
                                            .collect_vec()
                                    } else {
                                        vec![format!("FROM `{title}` TO `{ref_type}`")]
                                    };

                                    relations
                                        .push((name, on_options, is_option, is_array, ref_type));

                                    one_to_many
                                        .entry(name)
                                        .and_modify(|existing: &mut Vec<String>| {
                                            existing.append(&mut pairs)
                                        })
                                        .or_insert(pairs);

                                    continue;
                                }
                                _ => {
                                    // Ignore arrays with any_of and lists
                                    continue;
                                }
                            }
                        }
                        Type::Object => "STRING",
                    };

                    properties.push((name, data_type, on_options));
                    continue;
                }

                if let Some(ref_type) = &property.r#ref {
                    let data_type = match ref_type.as_str() {
                        "UnsignedInteger" => "UINT64",
                        "Cord" | "Time" => "STRING",
                        "Date" => "DATE",
                        "DateTime" | "Timestamp" => "TIMESTAMP",
                        "Duration" => "INTERVAL",
                        _ => {
                            if skip_types.contains(&ref_type.as_str()) {
                                continue;
                            }

                            let schema = self
                                .schemas
                                .get(ref_type)
                                .ok_or_eyre("schema should exist")?;

                            if schema.is_enumeration() {
                                properties.push((name, "STRING", on_options));
                                continue;
                            }

                            let mut pairs = if expand_types.contains(&ref_type.as_str()) {
                                let variants = self
                                    .schemas
                                    .get(ref_type)
                                    .ok_or_eyre("schema should exist")?
                                    .any_of
                                    .as_ref()
                                    .ok_or_eyre("any_of should be some")?;
                                variants
                                    .iter()
                                    .filter_map(|schema| {
                                        let variant = schema.r#ref.as_deref().expect("should exit");

                                        (!skip_types.contains(&variant))
                                            .then_some(format!("FROM `{title}` TO `{variant}`"))
                                    })
                                    .collect_vec()
                            } else {
                                vec![format!("FROM `{title}` TO `{ref_type}`")]
                            };

                            relations.push((name, on_options, is_option, is_array, ref_type));

                            one_to_one
                                .entry(name)
                                .and_modify(|existing: &mut Vec<String>| {
                                    existing.append(&mut pairs)
                                })
                                .or_insert(pairs);

                            continue;
                        }
                    };
                    properties.push((name, data_type, on_options));
                }
            }

            if add_text.contains(&title.as_str()) {
                properties.push(("text", "STRING", false));
            }

            node_tables.push(format!(
                "CREATE NODE TABLE IF NOT EXISTS `{title}` ({}\n  `docId` STRING,\n  `nodeId` STRING PRIMARY KEY\n);",
                properties
                    .iter()
                    .map(|(name, data_type, ..)| format!("\n  `{name}` {data_type},"))
                    .join("")
            ));

            let implem_node_table = properties
                .iter()
                .map(|&(name, data_type, on_options)| {
                    if name == "text" {
                        return format!("(NodeProperty::Text, String::to_kuzu_type(), to_text(self).to_kuzu_value())")
                    }

                    let mut property = name.to_pascal_case();
                    if property.ends_with("ID") {
                        property.pop();
                        property.push('d');
                    }

                    fn rust_type(data_type: &str) -> String {
                        if let Some(rest) = data_type.strip_suffix("[]") {
                            return ["Vec::<", &rust_type(rest), ">"].concat()
                        }

                        match data_type {
                            "BOOLEAN" => "bool".to_string(),
                            "INT64" => "i64".to_string(),
                            "UINT64" => "u64".to_string(),
                            "DOUBLE" => "f64".to_string(),
                            "INTERVAL" => "Duration".to_string(),
                            _ => data_type.to_pascal_case(),
                        }
                    }
                    let rust_type = rust_type(data_type);

                    let mut field = name.to_snake_case();
                    if on_options {
                        field = ["options.", &field].concat()
                    };

                    format!("(NodeProperty::{property}, {rust_type}::to_kuzu_type(), self.{field}.to_kuzu_value())")
                })
                .join(",\n            ");

            let implem_rel_tables = relations
                .into_iter()
                .map(|(name, on_options, is_option, is_array, ref_type)| {
                    let property = name.to_pascal_case();

                    let mut field = name.to_snake_case();
                    if field == "abstract" {
                        field = "r#abstract".to_string();
                    }

                    if on_options {
                        field = ["options.", &field].concat()
                    };

                    let collect = if ref_type == "AuthorRoleAuthor" {
                        format!("vec![(self.{field}.node_type(), self.{field}.node_id(), 1)]")
                    } else {
                        let mut collect = format!("self.{field}");

                        if is_option || is_array {
                            collect += ".iter()"
                        }

                        if is_option && is_array {
                            collect += ".flatten()"
                        }

                        format!("relations({collect})")
                    };

                    format!("(NodeProperty::{property}, {collect})")
                })
                .join(",\n            ");

            implems.push(format!(
                r#"impl DatabaseNode for {title} {{
    fn node_type(&self) -> NodeType {{
        NodeType::{title}
    }}

    fn node_id(&self) -> NodeId {{
        {title}::node_id(self)
    }}
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        vec![
            {implem_node_table}
        ]
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {{
        vec![
            {implem_rel_tables}
        ]
    }}
}}
"#
            ))
        }

        let node_tables = node_tables.join("\n\n");

        let one_to_one = one_to_one
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  `position` UINT32,\n  ONE_ONE\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        let one_to_many = one_to_many
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  `position` UINT32,\n  ONE_MANY\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        write(
            dir.join("schema.kuzu"),
            format!(
                "// Generated file, do not edit. See the Rust `schema-gen` crate;

{node_tables}

{one_to_one}

{one_to_many}
"
            ),
        )
        .await?;

        for title in ["Node", "Block", "Inline", "Author"] {
            let variants = self
                .schemas
                .get(title)
                .ok_or_eyre("schema should exist")?
                .any_of
                .as_ref()
                .ok_or_eyre("any_of should be some")?
                .iter()
                .filter_map(|schema| {
                    let variant = schema.r#ref.as_deref().expect("should exit");
                    (!skip_types.contains(&variant)).then_some(variant)
                })
                .collect_vec();

            let node_type = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_type()"))
                .join(",\n            ");

            let node_id = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_id()"))
                .join(",\n            ");

            let node_table = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_table()"))
                .join(",\n            ");

            let rel_tables = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.rel_tables()"))
                .join(",\n            ");

            implems.push(format!(
                r#"#[allow(unreachable_patterns)]
impl DatabaseNode for {title} {{
    fn node_type(&self) -> NodeType {{
        match self {{
            {node_type},
            _ => NodeType::Unknown
        }}
    }}

    fn node_id(&self) -> NodeId {{
        match self {{
            {node_id},
            _ => NodeId::null()
        }}
    }}

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        match self {{
            {node_table},
            _ => Vec::new()
        }}
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {{
        match self {{
            {rel_tables},
            _ => Vec::new()
        }}
    }}
}}
"#
            ));
        }

        let implems = implems.join("\n");

        write(
            dir.join("node_types.rs"),
            format!(
                "// Generated file, do not edit. See the Rust `schema-gen` crate.

use kuzu::{{LogicalType, Value}};

use codec_text_trait::to_text;
use schema::*;

use super::{{DatabaseNode, ToKuzu}};

fn relations<'lt, I, D>(iter: I) -> Vec<(NodeType, NodeId, usize)>
where
    I: Iterator<Item = &'lt D>,
    D: DatabaseNode + 'lt,
{{
    iter.enumerate()
        .flat_map(|(index, item)| (!matches!(item.node_type(), NodeType::Unknown)).then_some((item.node_type(), item.node_id(), index + 1)))
        .collect()
}}

{implems}
"
            ),
        )
        .await?;

        Ok(())
    }
}
