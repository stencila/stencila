//! Generation of Rust types from Stencila Schema

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
    tokio::fs::{create_dir_all, remove_file, write},
};

use crate::schemas::{Items, Schema, Schemas, Type, Value};

/// Modules that should not be generated
const NO_GENERATE_MODULE: &[&str] = &[
    "Array",
    "Boolean",
    "Integer",
    "Null",
    "Number",
    "Object",
    "String",
    "TextValue",
    "UnsignedInteger",
];

/// Types that should not derive `Read` because there are manual implementations
const NO_DERIVE_READ: &[&str] = &["Null", "Primitive", "TextValue", "Node"];

/// Types that should not derive `Write` because there are manual implementations
const NO_DERIVE_WRITE: &[&str] = &["Null", "Primitive", "TextValue"];

/// Types that should not derive the `Strip` trait because there are manual implementations
const NO_DERIVE_STRIP: &[&str] = &[
    "Call",
    "CallArgument",
    "CodeChunk",
    "CodeExpression",
    "For",
    "If",
    "IfClause",
    "Include",
];

/// Types that should not derive the `ToHtml` trait because there are manual implementations
const NO_DERIVE_TO_HTML: &[&str] = &["Paragraph"];

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

const KEYWORDS: &[&str; 52] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "union", "use",
    "where", "while", // STRICT, 2018
    "async", "await", "dyn", // RESERVED, 2015
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized",
    "virtual", "yield", "try",
];

fn handle_keyword(input: &String) -> String {
    if KEYWORDS.contains(&input.as_str()) {
        format!("r#{input}")
    } else {
        input.to_string()
    }
}

impl Schemas {
    /// Generate Rust modules for each schema
    pub async fn rust(&self) -> Result<()> {
        eprintln!("Generating Rust types");

        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../schema/src");
        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        let types = dest.join("types");
        if types.exists() {
            for file in read_dir(&types)?.flatten() {
                let path = file.path();

                if NO_GENERATE_MODULE.contains(
                    &path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .strip_suffix(".rs")
                        .unwrap()
                        .to_pascal_case()
                        .as_str(),
                ) {
                    continue;
                }

                remove_file(&path).await?
            }
        } else {
            create_dir_all(&types).await?;
        }

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
            .map(handle_keyword)
            .collect_vec();
        let mods = modules
            .iter()
            .map(|module| format!("mod {module};\n"))
            .join("");
        let uses = modules
            .iter()
            .map(|module| format!("pub use {module}::*;\n"))
            .join("");
        write(
            path,
            format!("// Generated file. Do not edit; see `schema-gen` crate.\n\n{mods}\n{uses}"),
        )
        .await?;

        Ok(())
    }

    /// Generate a Rust module for a schema
    async fn rust_module(dest: &Path, schema: &Schema) -> Result<()> {
        let Some(title) = &schema.title else {
            bail!("Schema has no title");
        };

        let rust = if NO_GENERATE_MODULE.contains(&title.as_str()) || schema.r#abstract {
            return Ok(());
        } else if schema.any_of.is_some() {
            Self::rust_any_of(dest, schema).await?
        } else if schema.r#type.is_none() {
            Self::rust_struct(dest, title, schema).await?
        } else {
            return Ok(());
        };

        let module = title.to_snake_case();

        let path = dest.join(format!("{module}.rs"));
        if !path.exists() {
            write(
                path,
                &format!("// Generated file. Do not edit; see `schema-gen` crate.\n\n{rust}"),
            )
            .await?;
        }

        Ok(())
    }

    /// Generate a Rust struct for a schema
    async fn rust_struct(dest: &Path, title: &String, schema: &Schema) -> Result<String> {
        let description = schema.description.as_ref().unwrap_or(title);

        let mut derive_traits =
            "Debug, Defaults, Clone, PartialEq, Serialize, Deserialize".to_string();
        let title = title.as_str();
        if !NO_DERIVE_STRIP.contains(&title) {
            derive_traits += ", Strip";
        }
        if !NO_DERIVE_READ.contains(&title) {
            derive_traits += ", Read";
        }
        if !NO_DERIVE_WRITE.contains(&title) {
            derive_traits += ", Write";
        }
        if !NO_DERIVE_TO_HTML.contains(&title) {
            derive_traits += ", ToHtml";
        }

        let mut fields = Vec::new();
        let mut used_types = HashSet::new();
        for (name, property) in schema.properties.iter().flatten() {
            let description = property.description.as_ref().unwrap_or(name);

            let mut attrs = Vec::new();

            // Rewrite name as necessary for Rust compatibility
            let name = name.to_snake_case();
            let name = handle_keyword(&name);

            // Determine Rust type for the property
            let (mut typ, is_vec) = if name == "r#type" {
                (format!(r#"MustBe!("{title}")"#), false)
            } else {
                let (typ, is_vec, ..) = Self::rust_type(dest, property).await?;
                used_types.insert(typ.clone());
                (typ, is_vec)
            };

            // Does the field have a default?
            let mut default = property.default.as_ref().map(|default| {
                let mut default = Self::rust_value(default);
                if default == "Null" {
                    used_types.insert(default);
                    default = "Node::Null(Null{})".to_string();
                }
                default
            });

            // Wrap type and defaults in generic types as necessary

            if is_vec {
                typ = format!("Vec<{typ}>");
            };

            if BOX_PROPERTIES.contains(&format!("{title}.{name}").as_str())
                || BOX_PROPERTIES.contains(&format!("*.{name}").as_str())
            {
                typ = format!("Box<{typ}>");
                default = default.map(|default| format!("Box::new({default})"));
            }

            if !property.is_required {
                typ = format!("Option<{typ}>");
                default = default.map(|default| format!("Some({default})"));
            }

            if let Some(default) = default {
                attrs.push(format!("#[def = \"{default}\"]"));
            }

            // Generate the code for the field
            let attrs = match attrs.is_empty() {
                true => String::new(),
                false => attrs.join("\n    ") + "\n    ",
            };
            let code = format!("/// {description}\n    {attrs}pub {name}: {typ},");

            fields.push((property.is_required, property.is_core, name, typ, code));
        }

        let uses = used_types
            .iter()
            .filter(|used_type| *used_type != title)
            .sorted()
            .map(|used_type| {
                let module = used_type.to_snake_case();
                let module = handle_keyword(&module);
                format!("use super::{module}::{used_type};")
            })
            .join("\n");

        let optional_fields = fields
            .iter()
            .filter_map(|(is_required, is_core, .., field)| {
                (!is_required && !is_core).then_some(field)
            })
            .join("\n\n    ");

        let options = if optional_fields.is_empty() {
            String::new()
        } else {
            format!(
                r#"

#[skip_serializing_none]
#[derive({derive_traits})]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct {title}Options {{
    {optional_fields}
}}"#
            )
        };

        let mut core_fields = fields
            .iter()
            .filter_map(|(is_required, is_core, .., field)| {
                (*is_required || *is_core).then_some(field)
            })
            .join("\n\n    ");
        if !options.is_empty() {
            core_fields += &format!(
                "

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<{title}Options>,"
            );
        }

        let required_fields = fields
            .iter()
            .filter_map(|(is_required, _is_core, name, typ, ..)| {
                (*is_required && name != "r#type").then_some((name, typ))
            })
            .collect_vec();

        let new = if required_fields.len() < 5 {
            let params = required_fields
                .iter()
                .map(|(name, typ)| format!("{name}: {typ}"))
                .join(", ");

            let args = required_fields
                .iter()
                .map(|(name, ..)| name.to_string())
                .join(",\n            ");

            let defaults = if required_fields.len() < fields.len() {
                format!(
                    "{comma_newline}..Default::default()",
                    comma_newline = if required_fields.is_empty() {
                        ""
                    } else {
                        ",\n            "
                    }
                )
            } else {
                String::new()
            };

            format!(
                r#"
    pub fn new({params}) -> Self {{
        Self {{
            {args}{defaults}
        }}
    }}
"#
            )
        } else {
            String::new()
        };

        let implem = format!(
            r#"
impl {title} {{{new}}}"#,
        );

        let rust = format!(
            r#"use crate::prelude::*;

{uses}

/// {description}
#[skip_serializing_none]
#[derive({derive_traits})]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct {title} {{
    {core_fields}
}}{options}
{implem}
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
            (Self::rust_value(r#const), false, false)
        } else {
            ("Unhandled".to_string(), false, true)
        };
        Ok(result)
    }

    /// Generate a Rust enum for an `anyOf` root schema or property schema
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

        let description = if let Some(title) = &schema.title {
            schema
                .description
                .clone()
                .unwrap_or(title.clone())
                // Any_of comes from prop and adds four spaces to schemas
                .replace(r#"    ///"#, r#"///"#)
        } else {
            alternatives
                .iter()
                .map(|variant| format!("[`{variant}`]"))
                .join(" or ")
        };

        let alternatives = alternatives
            .into_iter()
            .zip(are_types.into_iter())
            .collect_vec();

        let mut uses = alternatives
            .iter()
            .sorted()
            .filter_map(|(name, is_type)| {
                let module = name.to_snake_case();
                let module = handle_keyword(&module);
                is_type.then_some(format!("use super::{module}::{name};",))
            })
            .join("\n");
        if !uses.is_empty() {
            uses.push_str("\n\n");
        }

        let mut unit_variants = true;
        let variants = alternatives
            .iter()
            .map(|(variant, is_type)| {
                if *is_type {
                    unit_variants = false;
                    format!("{variant}({variant}),")
                } else {
                    format!("{variant},")
                }
            })
            .join("\n    ");

        let default = match &schema.default {
            Some(default) => {
                let default = Self::rust_value(default);
                if alternatives.iter().any(|(.., is_type)| *is_type) {
                    format!(r#"#[def = "{default}({default}::default())"]"#)
                } else {
                    format!(r#"#[def = "{default}"]"#)
                }
            }
            None => String::new(),
        };

        let mut derive_traits =
            "Debug, Clone, PartialEq, Display, Serialize, Deserialize".to_string();
        let title = name.as_str();
        if !default.is_empty() {
            derive_traits += ", Defaults";
        };
        if !NO_DERIVE_STRIP.contains(&title) {
            derive_traits += ", Strip";
        }
        if !NO_DERIVE_READ.contains(&title) {
            derive_traits += ", Read";
        }
        if !NO_DERIVE_WRITE.contains(&title) {
            derive_traits += ", Write";
        }
        if !NO_DERIVE_TO_HTML.contains(&title) {
            derive_traits += ", ToHtml";
        }

        let serde_tagged = match unit_variants {
            false => "untagged, ",
            true => "",
        };

        let rust = format!(
            r#"use crate::prelude::*;

{uses}/// {description}
#[derive({derive_traits})]
#[serde({serde_tagged}crate = "common::serde")]
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
            r#"use super::{module}::{typ};

pub type {name} = Vec<{typ}>;
"#
        );
        write(path, rust).await?;

        Ok(name)
    }

    /// Generate a Rust representation of a JSON schema value
    fn rust_value(value: &Value) -> String {
        match value {
            Value::Null => "Null()".to_string(),
            Value::Boolean(inner) => inner.to_string(),
            Value::Integer(inner) => inner.to_string(),
            Value::Number(inner) => inner.to_string(),
            Value::String(inner) => inner.to_string(),
            _ => "Unhandled value type".to_string(),
        }
    }
}
