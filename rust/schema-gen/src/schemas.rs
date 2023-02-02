use std::{collections::HashMap, fs::read_dir, path::PathBuf};

use common::{
    eyre::{bail, eyre, Context, Result},
    futures::future::try_join_all,
    indexmap::IndexMap,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    serde_yaml,
    strum::AsRefStr,
    tokio::fs::read_to_string, defaults::Defaults,
};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum Type {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Value {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Object(IndexMap<String, Value>),
    Array(Vec<Value>),
    #[serde(rename = "null")]
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Items {
    // This should be `One(Box<Schema>)` but serde have difficulty resolving
    // the non-list variants given that the properties are optional
    Ref(ItemsRef),
    Type(ItemsType),
    AnyOf(ItemsAnyOf),
    List(Vec<Schema>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ItemsRef {
    #[serde(rename = "$ref")]
    pub r#ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ItemsType {
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ItemsAnyOf {
    #[serde(rename = "anyOf")]
    pub r#any_of: Vec<Schema>,
}

/// A schema object
#[derive(Debug, Defaults, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[skip_serializing_none]
pub struct Schema {
    // JSONSchema7 properties
    // See https://tools.ietf.org/html/draft-handrews-json-schema-validation-01
    //
    // This was originally derived from the TypesScript `JSONSchema7` type
    // at https://github.com/DefinitelyTyped/DefinitelyTyped/blob/277204df4cec750287e82926ed13ac32fbe86828/types/json-schema/index.d.ts#L617
    #[serde(rename = "$id")]
    pub id: Option<String>,
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    #[serde(rename = "$defs")]
    pub defs: Option<IndexMap<String, Schema>>,

    pub r#type: Option<Type>,
    pub r#enum: Option<Vec<Value>>,
    pub r#const: Option<Value>,

    pub multiple_of: Option<f64>,
    pub maximum: Option<f64>,
    pub exclusive_maximum: Option<f64>,
    pub minimum: Option<f64>,
    pub exclusive_minimum: Option<f64>,

    pub max_length: Option<f64>,
    pub min_length: Option<f64>,
    pub pattern: Option<String>,

    pub items: Option<Items>,
    pub additional_items: Option<Box<Schema>>,
    pub max_items: Option<f64>,
    pub min_items: Option<f64>,
    pub unique_items: Option<bool>,
    pub contains: Option<Box<Schema>>,

    pub max_properties: Option<f64>,
    pub min_properties: Option<f64>,
    pub required: Option<Vec<String>>,
    pub properties: Option<IndexMap<String, Schema>>,
    pub pattern_properties: Option<IndexMap<String, Schema>>,
    pub additional_properties: Option<Box<Schema>>,
    pub dependencies: Option<IndexMap<String, Schema>>,
    pub property_names: Option<Box<Schema>>,

    pub r#if: Option<Box<Schema>>,
    pub then: Option<Box<Schema>>,
    pub r#else: Option<Box<Schema>>,

    pub all_of: Option<Vec<Schema>>,
    pub any_of: Option<Vec<Schema>>,
    pub one_of: Option<Vec<Schema>>,
    pub not: Option<Box<Schema>>,

    pub format: Option<String>,

    pub content_media_type: Option<String>,
    pub content_encoding: Option<String>,

    pub definitions: Option<IndexMap<String, Schema>>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub default: Option<Value>,
    pub read_only: Option<bool>,
    pub write_only: Option<bool>,
    pub examples: Option<Value>,

    // Stencila extensions
    /// The JSON-LD id for the schema
    #[serde(rename = "@id")]
    pub jid: Option<String>,

    /// The status of the schema
    #[def = "\"stable\".to_string()"]
    #[serde(default)]
    pub status: String,

    /// The title of the schema that this schema extends
    pub extends: Option<String>,

    /// Whether the schema is only an abstract base for other schemas
    /// 
    /// Types are usually not generated for abstract schemas.
    #[serde(default)]
    pub r#abstract: bool,

    /// Stencila derived properties
    /// Whether this is a property schema and is required (is in the `required` keyword)
    #[serde(skip)]
    pub is_required: bool,

    /// Whether the `extend()` method has been run on this schema yet
    #[serde(skip)]
    pub is_extended: bool,
}

impl Schema {
    /// Read a `schema/*.yaml` file into a [`Schema`] object
    async fn read(file: PathBuf) -> Result<(String, Schema)> {
        let yaml = read_to_string(&file)
            .await
            .context(format!("unable to read file `{}`", file.display()))?;

        let schema = serde_yaml::from_str(&yaml)
            .context(format!("unable to deserialize file `{}`", file.display()))?;

        let title = file
            .file_name()
            .and_then(|name| {
                name.to_string_lossy()
                    .strip_suffix(".yaml")
                    .map(String::from)
            })
            .expect("all files to have a prefix");

        Ok((title, schema))
    }

    /// Check and normalize the schema
    ///
    /// This performs normalization on fields tat make subsequent steps, as well as
    /// code generation easier.
    fn normalize(&mut self, name: &str, is_prop: bool) -> Result<()> {
        if !is_prop {
            let Some(title) = &mut self.title else {
                bail!("schema does not have a title")
            };
            if title != name {
                bail!("title is not the same as the name of file")
            }
        }

        let Some(description) = &mut self.description else {
            bail!("schema does not have a description")
        };
        *description = description.replace('\n', " ");

        let required = self.required.iter().flatten().collect_vec();
        if let Some(properties) = &mut self.properties {
            for (name, property) in properties.iter_mut() {
                property.normalize(name, true)?;
                if required.contains(&name) {
                    property.is_required = true;
                }
            }
        }

        Ok(())
    }

    /// Extend the schema by inheriting properties of it's parent
    fn extend(&self, name: &str, schemas: &mut HashMap<String, Schema>) -> Result<()> {
        let Some(extends) = &self.extends else {
            return Ok(())
        };

        let parent = schemas
            .get(extends)
            .ok_or_else(|| eyre!("no schema matching `extends` keyword: {}", extends))?
            .clone();

        if !parent.is_extended {
            parent.extend(extends, schemas)?;
        }

        let mut extended = self.clone();

        let mut properties = IndexMap::new();
        for (name, property) in parent.properties.into_iter().flatten() {
            properties.insert(name, property);
        }
        for (name, property) in extended.properties.into_iter().flatten() {
            properties.insert(name, property);
        }

        extended.properties = if properties.is_empty() {
            None
        } else {
            Some(properties)
        };
        extended.is_extended = true;

        schemas.insert(name.to_string(), extended);

        Ok(())
    }
}

pub struct Schemas {
    pub schemas: HashMap<String, Schema>,
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
        let title = "Node".to_string();
        let mut any_of = Vec::new();
        for (name, schema) in &self.schemas {
            if !schema.r#abstract && schema.any_of.is_none() {
                any_of.push(Schema {
                    r#ref: Some(name.to_string()),
                    ..Default::default()
                })
            }
        }
        any_of.sort_by(|a, b| a.r#ref.cmp(&b.r#ref));
        self.schemas.insert(
            title.clone(),
            Schema {
                title: Some(title),
                description: Some(
                    "Union type for all types in this schema, including primitives and entities"
                        .to_string(),
                ),
                any_of: Some(any_of),
                ..Default::default()
            },
        );

        // Union types for descendants of...
        for base in ["Thing", "CreativeWork"] {
            let mut any_of = Vec::new();
            for (name, schema) in &self.schemas {
                let mut is_descendent = false;
                let mut parent = Some(schema);
                while let Some(extends) = parent.and_then(|parent| parent.extends.as_ref()) {
                    if extends == base {
                        is_descendent = true;
                        break;
                    }
                    parent = self.schemas.get(extends);
                }

                if is_descendent {
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
                    ..Default::default()
                },
            );
        }

        Ok(())
    }
}
