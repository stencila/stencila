//! The collection of schemas in the Stencila Schema

use std::{fs::read_dir, path::PathBuf};

use common::{
    eyre::{Context, Result},
    futures::future::try_join_all,
    indexmap::IndexMap,
    itertools::Itertools,
};

use crate::schema::{Category, Schema, Value};

pub struct Schemas {
    pub schemas: IndexMap<String, Schema>,
}

impl Schemas {
    /// Read all the `schema/*.yaml` files into a map of [`Schema`] objects
    pub async fn read() -> Result<Schemas> {
        let schemas = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema");
        let schemas = schemas
            .canonicalize()
            .context(format!("can not find directory `{}`", schemas.display()))?;

        let yamls = read_dir(&schemas)
            .context(format!("unable to read directory `{}`", schemas.display()))?
            .flatten()
            .filter_map(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| (ext.to_string_lossy() == "yaml").then_some(entry.path()))
            });

        let futures = yamls.map(|path| async { Schema::read(path).await });
        let schemas = try_join_all(futures).await?.into_iter().collect();

        Ok(Schemas { schemas })
    }

    /// Check and normalize schemas
    pub fn check(&mut self) -> Result<()> {
        for (name, schema) in self.schemas.iter_mut() {
            schema
                .normalize(name, false)
                .context(format!("invalid schema `{name}`"))?;
        }

        Ok(())
    }

    /// Apply the `extend` keyword to each schema that has one
    pub fn extend(&mut self) -> Result<()> {
        let mut schemas = self.schemas.clone();
        for (name, schema) in &self.schemas {
            schema.extend(name, &mut schemas)?;
        }
        self.schemas = schemas;

        Ok(())
    }

    /// Expand the schema with synthetic types based on ancestor types
    ///
    /// Only does this for the union types that are referred to elsewhere in the
    /// schema, not for every base type.
    pub fn expand(&mut self) -> Result<()> {
        // Node union type
        // Order is important for deserialization correctness and performance since
        // serde attempts to deserialize in the order in the enum. We put primitives
        // first (for fast deserialization for kernel outputs) except for `Object` which is
        // last so it does not "consume" entity types (which are also objects).
        let mut any_of = [
            "Null",
            "Boolean",
            "Integer",
            "UnsignedInteger",
            "Number",
            "String",
            "Cord",
            "Array",
        ]
        .iter()
        .map(|name| Schema {
            r#ref: Some(name.to_string()),
            ..Default::default()
        })
        .collect_vec();

        let mut entities = self
            .schemas
            .iter()
            .filter_map(|(name, schema)| {
                (schema.r#type.is_none() && schema.any_of.is_none() && !schema.r#abstract)
                    .then_some(Schema {
                        r#ref: Some(name.to_string()),
                        ..Default::default()
                    })
            })
            .collect_vec();
        entities.sort_by(|a, b| a.r#ref.cmp(&b.r#ref));
        any_of.append(&mut entities);

        any_of.push(Schema {
            r#ref: Some("Object".to_string()),
            ..Default::default()
        });

        let title = "Node".to_string();
        self.schemas.insert(
            title.clone(),
            Schema {
                title: Some(title),
                description: Some(
                    "Union type for all types in this schema, including primitives and entities"
                        .to_string(),
                ),
                default: Some(Value::Null),
                any_of: Some(any_of),
                ..Default::default()
            },
        );

        // Union types for descendants of...
        for (base, category) in [
            ("Thing", Category::Other),
            ("CreativeWork", Category::Works),
            ("SuggestionInline", Category::Edits),
            ("SuggestionBlock", Category::Edits),
        ] {
            let mut any_of = Vec::new();
            for (name, schema) in &self.schemas {
                fn is_descendent(
                    schemas: &IndexMap<String, Schema>,
                    base: &str,
                    nest: &Schema,
                ) -> bool {
                    if nest.extends.contains(&base.to_string()) {
                        return true;
                    }
                    nest.extends
                        .iter()
                        .any(|extend| is_descendent(schemas, base, &schemas[extend]))
                }

                if is_descendent(&self.schemas, base, schema) {
                    any_of.push(Schema {
                        r#ref: Some(name.to_string()),
                        ..Default::default()
                    });
                }
            }
            any_of.sort_by(|a, b| a.r#ref.cmp(&b.r#ref));

            let title = format!("{base}Type");
            self.schemas.insert(
                title.clone(),
                Schema {
                    title: Some(title),
                    description: Some(format!(
                        "Union type for all types that are descended from `{base}`"
                    )),
                    any_of: Some(any_of),
                    category,
                    ..Default::default()
                },
            );
        }

        Ok(())
    }
}
