use std::{collections::BTreeMap, path::PathBuf};

use common::{
    eyre::{bail, OptionExt, Result},
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
    ///     CREATE NODE TABLE Paragraph(...)
    ///
    /// with table properties for primitive properties (e.g. strings, numbers).
    ///
    /// For entity type properties we create relationship tables which have `TO`
    /// and `FROM` pairs for every possible parent-child combination. e.g.
    ///
    ///     CREATE REL TABLE content(FROM Article TO Paragraph, FROM Article TO CodeBlock)
    #[allow(clippy::print_stderr)]
    pub async fn kuzu(&self) -> Result<()> {
        eprintln!("Generating Kuzu Schema");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-db/src");

        let mut nodes = Vec::new();
        let mut one_to_many = BTreeMap::new();
        let mut one_to_one = BTreeMap::new();
        for (title, schema) in &self.schemas {
            if !schema.is_object() || schema.r#abstract || title.starts_with("Config") {
                continue;
            }

            let mut properties = Vec::new();
            for (name, property) in &schema.properties {
                if name == "type" || name == "id" || name == "config" {
                    continue;
                }

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
                                Items::Type(items_type) => match items_type {
                                    ItemsType { r#type } => match r#type {
                                        Type::Null => "NULL[]",
                                        Type::Boolean => "BOOLEAN[]",
                                        Type::Integer => "INT64[]",
                                        Type::Number => "DOUBLE[]",
                                        Type::String => "STRING[]",
                                        _ => bail!("Unhandled items type: {type}"),
                                    },
                                },
                                Items::Ref(ItemsRef { r#ref: ref_type }) => {
                                    if ref_type == "Node" || ref_type == "ThingType" {
                                        continue;
                                    }

                                    let schema = self
                                        .schemas
                                        .get(ref_type)
                                        .ok_or_eyre("schema should exist")?;

                                    let mut pairs = if ref_type == "Block"
                                        || ref_type == "Inline"
                                        || ref_type == "Author"
                                    {
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
                                                if [
                                                    "Null",
                                                    "Boolean",
                                                    "Integer",
                                                    "UnsignedInteger",
                                                    "Number",
                                                ]
                                                .contains(&variant)
                                                {
                                                    None
                                                } else {
                                                    Some(format!("FROM `{title}` TO `{variant}`"))
                                                }
                                            })
                                            .collect_vec()
                                    } else if schema.any_of.is_some() {
                                        properties.push(format!("`{name}` STRING"));
                                        continue;
                                    } else {
                                        vec![format!("FROM `{title}` TO `{ref_type}`")]
                                    };

                                    one_to_many
                                        .entry(name)
                                        .and_modify(|existing: &mut Vec<String>| {
                                            existing.append(&mut pairs)
                                        })
                                        .or_insert(pairs);

                                    continue;
                                }
                                _ => "STRING",
                            }
                        }
                        Type::Object => "STRING",
                    };
                    properties.push(format!("`{name}` {data_type}"));
                    continue;
                }

                if let Some(ref_type) = &property.r#ref {
                    let r#type = if ref_type == "UnsignedInteger" {
                        "UINT64"
                    } else if ref_type == "Cord" {
                        "STRING"
                    } else if ref_type == "Date" {
                        "DATE"
                    } else if ref_type == "DateTime" || ref_type == "Timestamp" {
                        "TIMESTAMP"
                    } else {
                        let schema = self
                            .schemas
                            .get(ref_type)
                            .ok_or_eyre("schema should exist")?;

                        if schema.any_of.is_some() {
                            "STRING"
                        } else {
                            if ref_type == "Node" || ref_type == "ThingType" {
                                continue;
                            }

                            let mut pairs = if ref_type == "Block"
                                || ref_type == "Inline"
                                || ref_type == "Author"
                            {
                                let variants = self
                                    .schemas
                                    .get(ref_type)
                                    .ok_or_eyre("schema should exist")?
                                    .any_of
                                    .as_ref()
                                    .ok_or_eyre("any_of should be some")?;
                                variants
                                    .iter()
                                    .map(|schema| {
                                        format!(
                                            "FROM `{title}` TO `{}`",
                                            schema.r#ref.as_ref().expect("should exit")
                                        )
                                    })
                                    .collect_vec()
                            } else {
                                vec![format!("FROM `{title}` TO `{ref_type}`")]
                            };

                            one_to_one
                                .entry(name)
                                .and_modify(|existing: &mut Vec<String>| {
                                    existing.append(&mut pairs)
                                })
                                .or_insert(pairs);
                            continue;
                        }
                    };
                    properties.push(format!("`{name}` {type}"));
                }
            }

            properties.push("`nodeId` STRING PRIMARY KEY".to_string());

            nodes.push(format!(
                "CREATE NODE TABLE IF NOT EXISTS `{title}` (\n  {}\n);",
                properties.join(",\n  ")
            ));
        }

        let nodes = nodes.join("\n\n");

        let one_to_one = one_to_one
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  ONE_ONE\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        let one_to_many = one_to_many
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  `index` UINT32,\n  ONE_MANY\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        write(
            dir.join("schema.kuzu"),
            format!(
                "// Generated file; do not edit. See the Rust `schema-gen` crate.

{nodes}

{one_to_one}

{one_to_many}
"
            ),
        )
        .await?;

        Ok(())
    }
}
