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

/// Comment to place at top of a files to indicate it is generated
const GENERATED_COMMENT: &str = "// Generated file; do not edit. See `schema-gen` crate.";

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
const NO_READ: &[&str] = &["Null", "Primitive", "TextValue", "Node"];

/// Types that should not derive `Write` because there are manual implementations
const NO_WRITE: &[&str] = &["Null", "Primitive", "TextValue"];

/// Types that should not derive the `Strip` trait because there are manual implementations
const NO_STRIP: &[&str] = &[
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
const NO_TO_HTML: &[&str] = &["Paragraph"];

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

/// If the input is a Rust keyword then escape it so it is valid Rust code
fn escape_keyword(input: &str) -> String {
    if KEYWORDS.contains(&input) {
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

        // Create an types.rs which export types from all modules (including those
        // that are not generated)
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
            .sorted()
            // Escape after sorting to ignore any `r#` prefix in escaped name
            .map(|name| escape_keyword(&name))
            .collect_vec();
        let mods = modules
            .iter()
            .map(|module| format!("mod {module};"))
            .join("\n");
        let uses = modules
            .iter()
            .map(|module| format!("pub use {module}::*;"))
            .join("\n");
        write(
            dest.join("types.rs"),
            format!(
                r"{GENERATED_COMMENT}

{mods}

{uses}
"
            ),
        )
        .await?;

        Ok(())
    }

    /// Generate a Rust module for a schema
    async fn rust_module(dest: &Path, schema: &Schema) -> Result<()> {
        let Some(title) = &schema.title else {
            bail!("Schema has no title");
        };

        if NO_GENERATE_MODULE.contains(&title.as_str()) || schema.r#abstract {
            return Ok(());
        }

        if schema.any_of.is_some() {
            Self::rust_any_of(dest, schema).await?;
        } else if schema.r#type.is_none() {
            Self::rust_object(dest, title, schema).await?;
        }

        Ok(())
    }

    /// Generate a Rust type for a schema
    ///
    /// Returns the name of the type and whether:
    ///  - it is an array
    ///  - it is a type (rather than an enum variant)
    #[async_recursion]
    async fn rust_type(dest: &Path, schema: &Schema) -> Result<(String, bool, bool)> {
        let result = if let Some(r#type) = &schema.r#type {
            match r#type {
                Type::Array => {
                    let items = match &schema.items {
                        Some(Items::Ref(inner)) => inner.r#ref.to_string(),
                        Some(Items::Type(inner)) => inner.r#type.to_pascal_case(),
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

    /// Generate a Rust `struct` for an object schema with `properties`
    ///
    /// Returns the name of the generated `struct`.
    async fn rust_object(dest: &Path, title: &String, schema: &Schema) -> Result<String> {
        let module = title.to_snake_case();

        let path = dest.join(format!("{}.rs", module));
        if path.exists() {
            return Ok(title.to_string());
        }

        let description = schema
            .description
            .as_ref()
            .unwrap_or(title)
            .trim_end_matches('\n')
            .replace('\n', "\n/// ")
            .to_string();

        let mut derive_traits =
            "Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize".to_string();
        let title = title.as_str();
        if !NO_STRIP.contains(&title) {
            derive_traits += ", Strip";
        }
        if !NO_READ.contains(&title) {
            derive_traits += ", Read";
        }
        if !NO_WRITE.contains(&title) {
            derive_traits += ", Write";
        }
        if !NO_TO_HTML.contains(&title) {
            derive_traits += ", ToHtml";
        }
        derive_traits += ", ToText";

        let mut fields = Vec::new();
        let mut used_types = HashSet::new();
        for (name, property) in schema.properties.iter().flatten() {
            let description = property
                .description
                .as_ref()
                .unwrap_or(name)
                .trim_end_matches('\n')
                .replace('\n', "\n    /// ")
                .to_string();

            let mut attrs = Vec::new();

            // Rewrite name as necessary for Rust compatibility
            let name = name.to_snake_case();
            let name = escape_keyword(&name);

            // Determine Rust type for the property
            let (mut typ, is_vec) = if name == "r#type" {
                (format!(r#"MustBe!("{title}")"#), false)
            } else {
                let (typ, is_vec, ..) = Self::rust_type(dest, property).await?;
                used_types.insert(typ.clone());
                (typ, is_vec)
            };

            // Does the field have a default?
            let mut default = property.default.as_ref().map(Self::rust_value);

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
                attrs.push(format!("#[default = {default}]"));
            }

            // Generate the code for the field
            let attrs = match attrs.is_empty() {
                true => String::new(),
                false => attrs.join("\n    ") + "\n    ",
            };
            let code = format!("/// {description}\n    {attrs}pub {name}: {typ},");

            fields.push((property.is_required, property.is_core, name, typ, code));
        }

        let mut uses = used_types
            .iter()
            .filter(|used_type| *used_type != title)
            .sorted()
            .map(|used_type| {
                let module = used_type.to_snake_case();
                let module = escape_keyword(&module);
                format!("use super::{module}::{used_type};")
            })
            .join("\n");
        if !uses.is_empty() {
            uses.push_str("\n\n");
        }

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
}}
"#
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

        write(
            path,
            &format!(
                r#"{GENERATED_COMMENT}

use crate::prelude::*;

{uses}/// {description}
#[skip_serializing_none]
#[derive({derive_traits})]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct {title} {{
    {core_fields}
}}{options}
impl {title} {{{new}}}
"#
            ),
        )
        .await?;

        Ok(title.to_string())
    }

    /// Generate a Rust `enum` for an `anyOf` root schema or property schema.
    ///
    /// Returns the name of the generated enum.
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
                .as_ref()
                .unwrap_or(title)
                .trim_end_matches('\n')
                .replace('\n', "\n    /// ")
                .to_string()
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
                let module = escape_keyword(&module);
                is_type.then_some(format!("use super::{module}::{name};",))
            })
            .join("\n");
        if !uses.is_empty() {
            uses.push_str("\n\n");
        }

        let default = schema.default.as_ref().map(Self::rust_value);

        let mut unit_variants = true;
        let variants = alternatives
            .into_iter()
            .map(|(variant, is_type)| {
                let default = default
                    .as_ref()
                    .and_then(|default| {
                        (default == &variant).then_some("#[default]\n    ".to_string())
                    })
                    .unwrap_or_default();

                if is_type {
                    unit_variants = false;
                    format!("{default}{variant}({variant}),")
                } else {
                    format!("{default}{variant},")
                }
            })
            .join("\n    ");

        let mut derive_traits =
            "Debug, Clone, PartialEq, Display, Serialize, Deserialize".to_string();
        let title = name.as_str();
        if default.is_some() {
            derive_traits += ", SmartDefault";
        };
        if !NO_STRIP.contains(&title) {
            derive_traits += ", Strip";
        }
        if !NO_READ.contains(&title) {
            derive_traits += ", Read";
        }
        if !NO_WRITE.contains(&title) {
            derive_traits += ", Write";
        }
        if !NO_TO_HTML.contains(&title) {
            derive_traits += ", ToHtml";
        }
        derive_traits += ", ToText";

        let serde_tagged = match unit_variants {
            false => "untagged, ",
            true => "",
        };

        let rust = format!(
            r#"{GENERATED_COMMENT}

use crate::prelude::*;

{uses}/// {description}
#[derive({derive_traits})]
#[serde({serde_tagged}crate = "common::serde")]
pub enum {name} {{
    {variants}
}}
"#
        );
        write(path, rust).await?;

        Ok(name)
    }

    /// Generate a Rust `type` for an array of a type
    ///
    /// Returns the name of the generated type which will be the plural
    /// of the type of the array items.
    async fn rust_array_of(dest: &Path, item_type: &str) -> Result<String> {
        let name = item_type.to_plural();

        let path = dest.join(format!("{}.rs", name.to_snake_case()));
        if path.exists() {
            return Ok(name);
        }

        let module = item_type.to_snake_case();
        let rust = format!(
            r#"{GENERATED_COMMENT}

use super::{module}::{item_type};

pub type {name} = Vec<{item_type}>;
"#
        );
        write(path, rust).await?;

        Ok(name)
    }

    /// Generate a Rust representation of a JSON schema value
    ///
    /// Returns a literal to the type of value.
    fn rust_value(value: &Value) -> String {
        match value {
            Value::Null => "Null".to_string(),
            Value::Boolean(inner) => inner.to_string(),
            Value::Integer(inner) => inner.to_string(),
            Value::Number(inner) => inner.to_string(),
            Value::String(inner) => inner.to_string(),
            _ => "Unhandled value type".to_string(),
        }
    }
}
