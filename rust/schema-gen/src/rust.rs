use std::{
    collections::HashSet,
    fs::read_dir,
    path::{Path, PathBuf},
};

use common::{
    async_recursion::async_recursion,
    eyre::{bail, Context, Report, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    tokio::fs::{create_dir_all, remove_dir_all, write},
};

use crate::schemas::{Items, Schema, Schemas, Type, Value};

/// Properties that need to be boxed to avoid recursive types
const BOX_PROPERTIES: &[&str] = &[
    "*.is_part_of",
    "ArrayValidator.contains",
    "ArrayValidator.items_validator",
    "CallArgument.default",
    "CallArgument.value",
    "CodeExpression.output",
    "Comment.parent_item",
    "ConstantValidator.value",
    "ImageObject.thumbnail",
    "ListItem.item",
    "Organization.logo",
    "Organization.parent_organization",
    "Parameter.default",
    "Parameter.value",
    "Variable.value",
];

impl Schemas {
    /// Generate Rust modules for each schema
    pub async fn rust(&self) -> Result<()> {
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../schema/src");
        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        let types = dest.join("types");
        if types.exists() {
            remove_dir_all(&types).await?;
        }
        create_dir_all(&types).await?;

        let futures = self
            .schemas
            .values()
            .map(|schema| Self::rust_module(&types, schema));
        try_join_all(futures).await?;

        let modules = read_dir(&types)
            .context(format!("unable to read directory `{}`", types.display()))?
            .flatten()
            .map(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .strip_suffix(".rs")
                    .unwrap()
                    .to_string()
            })
            .collect_vec();

        let path = dest.join("types.rs");
        let modules = modules
            .iter()
            .filter(|module| !module.is_empty())
            .sorted()
            .collect_vec();
        let mods = modules
            .iter()
            .map(|module| format!("mod {module};\n"))
            .join("");
        let uses = modules
            .iter()
            .map(|module| format!("pub use {module}::*;\n"))
            .join("");
        write(path, format!("{mods}\n\n{uses}")).await?;

        Ok(())
    }

    /// Generate a Rust module for a schema
    async fn rust_module(dest: &Path, schema: &Schema) -> Result<()> {
        let Some(title) = &schema.title else {
            bail!("Schema has no title");
        };

        let rust = if schema.any_of.is_some() {
            Self::rust_any_of(dest, schema).await?
        } else if schema.r#type.is_none() {
            Self::rust_struct(dest, title, schema).await?
        } else if let Some(r#type) = &schema.r#type {
            match r#type {
                Type::Null => NULL_RUST,
                Type::Boolean => "pub type Boolean = bool;\n",
                Type::Integer => "pub type Integer = i64;\n",
                Type::Number => "pub type Number = f64;\n",
                Type::String => "pub type String = std::string::String;\n",
                Type::Array => "pub type Array = Vec<super::primitive::Primitive>;\n",
                Type::Object => "pub type Object = std::collections::HashMap<String, super::primitive::Primitive>;\n",
            }.to_string()
        } else {
            return Ok(());
        };

        let module = title.to_snake_case();
        let module = match module.as_str() {
            "if" => "if_".to_string(),
            "for" => "for_".to_string(),
            _ => module,
        };

        let path = dest.join(format!("{module}.rs"));
        if !path.exists() {
            write(path, rust).await?;
        }

        Ok(())
    }

    /// Generate a Rust struct for a schema
    async fn rust_struct(dest: &Path, title: &String, schema: &Schema) -> Result<String> {
        let description = schema.description.as_ref().unwrap_or(title);

        let mut fields = Vec::new();
        let mut used_types = HashSet::new();
        for (name, property) in schema.properties.iter().flatten() {
            let description = property.description.as_ref().unwrap_or(name);

            let name = name.to_snake_case();
            let name = match name.as_str() {
                "type" => "r#type".to_string(),
                _ => name,
            };

            let r#type = if name == "r#type" {
                format!(r#"MustBe!("{title}")"#)
            } else {
                let (mut property_type, is_vec, ..) = Self::rust_type(dest, property).await?;
                used_types.insert(property_type.clone());

                if is_vec {
                    property_type = format!("Vec<{property_type}>");
                };

                if BOX_PROPERTIES.contains(&format!("{title}.{name}").as_str())
                    || BOX_PROPERTIES.contains(&format!("*.{name}").as_str())
                {
                    property_type = format!("Box<{property_type}>");
                }

                if !property.is_required {
                    property_type = format!("Option<{property_type}>");
                }

                property_type
            };

            fields.push(format!("/// {description}\n    {name}: {type},"));
        }
        let fields = fields.join("\n\n    ");

        let uses = used_types
            .iter()
            .filter(|used_type| *used_type != title)
            .map(|used_type| {
                format!(
                    "use super::{module}::{used_type};",
                    module = used_type.to_snake_case()
                )
            })
            .join("\n");

        let rust = format!(
            r#"//! Generated file, do not edit

use crate::prelude::*;

{uses}

/// {description}
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct {title} {{
    {fields}
}}
"#
        );
        Ok(rust)
    }

    /// Generate a Rust type for a schema
    #[async_recursion]
    async fn rust_type(dest: &Path, schema: &Schema) -> Result<(String, bool, bool)> {
        let result = if let Some(r#type) = &schema.r#type {
            match r#type {
                Type::Array => {
                    let items = match &schema.items {
                        Some(Items::Ref(inner)) => inner.r#ref.to_string(),
                        Some(Items::Type(inner)) => inner.r#type.to_class_case(),
                        Some(Items::AnyOf(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.any_of.clone()),
                                ..Default::default()
                            };
                            Self::rust_type(dest, &schema).await?.0
                        }
                        Some(Items::List(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            };
                            Self::rust_type(dest, &schema).await?.0
                        }
                        None => "Unhandled".to_string(),
                    };
                    (items, true, true)
                }
                _ => (r#type.as_ref().to_string(), false, true),
            }
        } else if let Some(r#ref) = &schema.r#ref {
            (r#ref.to_string(), false, true)
        } else if schema.any_of.is_some() {
            (Self::rust_any_of(dest, schema).await?, false, true)
        } else if let Some(title) = &schema.title {
            (title.to_string(), false, true)
        } else if let Some(r#const) = &schema.r#const {
            let typ = match r#const {
                Value::String(inner) => inner.to_string(),
                _ => "Unhandled".to_string(),
            };
            (typ, false, false)
        } else {
            ("Unhandled".to_string(), false, true)
        };
        Ok(result)
    }

    /// Generate a Rust enum for an `anyOf` property
    async fn rust_any_of(dest: &Path, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Schema has no anyOf");
        };

        let (alternatives, are_types): (Vec<_>, Vec<_>) =
            try_join_all(any_of.iter().map(|schema| async {
                let (typ, is_array, is_type) = Self::rust_type(dest, schema).await?;
                let typ = if is_array {
                    Self::rust_array_of(dest, &typ).await?
                } else {
                    typ
                };
                Ok::<_, Report>((typ, is_type))
            }))
            .await?
            .into_iter()
            .unzip();

        let name = schema
            .title
            .clone()
            .unwrap_or_else(|| alternatives.join("Or"));

        let path = dest.join(format!("{}.rs", name.to_snake_case()));
        if path.exists() {
            return Ok(name);
        }

        let description = schema.description.clone().unwrap_or_else(|| {
            alternatives
                .iter()
                .map(|variant| format!("[`{variant}`]"))
                .join(" or ")
        });

        let alternatives = alternatives
            .into_iter()
            .zip(are_types.into_iter())
            .collect_vec();

        let uses = alternatives
            .iter()
            .filter_map(|(name, is_type)| {
                let module = name.to_snake_case();
                let module = match module.as_str() {
                    "if" => "if_".to_string(),
                    "for" => "for_".to_string(),
                    _ => module,
                };
                is_type.then_some(format!("use super::{module}::{name};",))
            })
            .join("\n");

        let variants = alternatives
            .iter()
            .map(|(variant, is_type)| {
                if *is_type {
                    format!("{variant}({variant}),")
                } else {
                    format!("{variant},")
                }
            })
            .join("\n    ");

        let default = alternatives
            .get(0)
            .map(|(variant, is_type)| {
                if *is_type {
                    format!(r#"#[def = "{variant}({variant}::default())"]"#)
                } else {
                    format!(r#"#[def = "{variant}"]"#)
                }
            })
            .unwrap_or_default();

        let rust = format!(
            r#"//! Generated file, do not edit
    
use crate::prelude::*;

{uses}

/// {description}
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
{default}
pub enum {name} {{
    {variants}
}}
    "#
        );
        write(path, rust).await?;

        Ok(name)
    }

    /// Generate a Rust `type` for an array of a type
    async fn rust_array_of(dest: &Path, typ: &str) -> Result<String> {
        let name = typ.to_plural();

        let path = dest.join(format!("{}.rs", name.to_snake_case()));
        if path.exists() {
            return Ok(name);
        }

        let module = typ.to_snake_case();
        let rust = format!(
            r#"//! Generated file, do not edit

use super::{module}::{typ};

pub type {name} = Vec<{typ}>;
"#
        );
        write(path, rust).await?;

        Ok(name)
    }
}

const NULL_RUST: &str = r#"use std::fmt;

use crate::prelude::*;

/// A null value
///
/// This is a struct, rather than a unit variant of `Primitive`, so that
/// it can be treated the same way as other variants when dispatching to
/// trait methods.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Null;

impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

impl Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for Null {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.is_null() {
            true => Ok(Null {}),
            false => Err(serde::de::Error::custom("Expected a null value")),
        }
    }
}
"#;
