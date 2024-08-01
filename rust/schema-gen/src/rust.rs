//! Generation of Rust types from Stencila Schema

use std::{
    collections::HashSet,
    fs::read_dir,
    path::{Path, PathBuf},
};

use common::{
    eyre::{bail, Context, Report, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    strum::IntoEnumIterator,
    tokio::fs::{create_dir_all, remove_file, write},
};

use crate::{
    schema::{Items, ItemsRef, ProptestLevel, Schema, Type, Value},
    schemas::Schemas,
};

/// Comment to place at top of a files to indicate it is generated
const GENERATED_COMMENT: &str = "// Generated file; do not edit. See `schema-gen` crate.";

/// Modules that should not be generated
const NO_GENERATE_MODULE: &[&str] = &[
    "Array",
    "Boolean",
    "Cord",
    "Integer",
    "Null",
    "Number",
    "Object",
    "String",
    "UnsignedInteger",
];

/// Types that should not derive `ReadNode` because they need special, manual implementations
/// because they are union types with variants of different Automerge types
/// (and so are not easily handled in derive macros)
const NO_READ_NODE: &[&str] = &[
    "Inline",
    "IntegerOrString",
    "Node",
    "Primitive",
    "PropertyValueOrString",
    "StringOrNumber",
    "StringPatchOrPrimitive",
];

/// Unit variant enums for which PartialOrd, Ord should not be implement
/// (usually because it is manually implemented)
const NO_ORD: &[&str] = &["ExecutionStatus"];

/// Properties that are boxed to avoid recursive types or reduce the size of structs
///
/// Note that properties that are not "core" do not need to be boxed because they
/// will be in the `Options` struct for the type and thus are already boxed.
const BOX_PROPERTIES: &[&str] = &[
    "ArrayValidator.contains",
    "ArrayValidator.items_validator",
    "CallArgument.default",
    "CallArgument.value",
    "CodeExpression.output",
    "ConstantValidator.value",
    "InstructionBlock.model",
    "InstructionInline.model",
    "ListItem.item",
    "ModifyOperation.value",
    "Parameter.default",
    "Parameter.value",
    "PublicationIssue.is_part_of",
    "PublicationVolume.is_part_of",
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

        // Export all types from one file
        let nodes = self
            .schemas
            .get("Node")
            .and_then(|schema| schema.any_of.as_ref())
            .expect("should always exist");
        let node_types = nodes
            .iter()
            .filter_map(|schema| schema.r#ref.as_ref())
            .map(|title| format!("    {title}"))
            .join(",\n");

        write(
            dest.join("types.rs"),
            format!(
                r#"{GENERATED_COMMENT}
{mods}

{uses}
"#
            ),
        )
        .await?;

        //  Create an enum with unit variants for each node type
        write(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-type/src/node_type.rs"),
            format!(
                r#"{GENERATED_COMMENT}

use common::{{
    serde::Serialize,
    strum::{{Display, EnumIter, EnumString}},
}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Display, EnumString, EnumIter)]
#[serde(crate = "common::serde")]
#[strum(crate = "common::strum")]
pub enum NodeType {{
{node_types},
}}
"#
            ),
        )
        .await?;

        //  Create an enum with unit variants for each node property
        let node_properties = self
            .schemas
            .iter()
            .flat_map(|(.., schema)| schema.properties.keys())
            .cloned()
            .collect::<HashSet<String>>()
            .iter()
            .sorted()
            .map(|name| {
                let mut name = name.to_pascal_case();
                if name.ends_with("ID") {
                    name.pop();
                    name.push('d');
                }
                format!("    {name}")
            })
            .join(",\n");
        write(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-type/src/node_property.rs"),
            format!(
                r#"{GENERATED_COMMENT}

use common::{{serde::{{Serialize, Deserialize}}, strum::Display}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[strum(serialize_all = "camelCase", crate = "common::strum")]
pub enum NodeProperty {{
{node_properties},
}}
"#
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
            Self::rust_any_of(dest, schema)?;
        } else if schema.r#type.is_none() {
            Self::rust_object(dest, title, schema).await?;
        } else {
            bail!("Schema {title} was not translated to Rust")
        }

        Ok(())
    }

    /// Generate a Rust type for a schema
    ///
    /// Returns the name of the type and whether:
    ///  - it is an array
    ///  - it is a type (rather than an enum variant)
    fn rust_type(dest: &Path, schema: &Schema) -> Result<(String, bool, bool)> {
        let result = if let Some(r#type) = &schema.r#type {
            match r#type {
                Type::Array => {
                    let items = match &schema.items {
                        Some(Items::Ref(inner)) => inner.r#ref.to_string(),
                        Some(Items::Type(inner)) => inner.r#type.to_string().to_pascal_case(),
                        Some(Items::AnyOf(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.any_of.clone()),
                                ..Default::default()
                            };
                            Self::rust_type(dest, &schema)?.0
                        }
                        Some(Items::List(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            };
                            Self::rust_type(dest, &schema)?.0
                        }
                        None => "Unhandled".to_string(),
                    };
                    (items, true, true)
                }
                _ => (r#type.to_string(), false, true),
            }
        } else if let Some(r#ref) = &schema.r#ref {
            (r#ref.to_string(), false, true)
        } else if schema.any_of.is_some() {
            (Self::rust_any_of(dest, schema)?, false, true)
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

        let mut attrs = vec![
            // skip_serializing_none and serde_as have to come before derives
            "#[skip_serializing_none]".to_string(),
            "#[serde_as]".to_string(),
        ];

        // Construct list of traits to derive for the struct
        let mut derives = vec![
            "Debug",
            "SmartDefault",
            "Clone",
            "PartialEq",
            "Serialize",
            "Deserialize",
            "StripNode",
            "WalkNode",
            "WriteNode",
        ];

        if !NO_READ_NODE.contains(&title.as_str()) {
            derives.push("ReadNode");
        }

        let derive_patch = schema
            .patch
            .as_ref()
            .map(|spec| spec.derive)
            .unwrap_or(true);
        if derive_patch {
            derives.push("PatchNode");
        }

        // Codec derives
        {
            if schema.dom.as_ref().map(|spec| spec.derive).unwrap_or(true) {
                derives.push("DomCodec");
            }

            derives.append(&mut vec!["HtmlCodec", "JatsCodec"]);

            if schema
                .markdown
                .as_ref()
                .map(|spec| spec.derive)
                .unwrap_or(true)
            {
                derives.push("MarkdownCodec");
            }

            derives.push("TextCodec");
        }

        attrs.push(format!("#[derive({})]", derives.join(", ")));

        // Add serde related attributes
        attrs.push("#[serde(rename_all = \"camelCase\", crate = \"common::serde\")]".to_string());

        // Add proptest related attributes
        if let Some(proptest) = &schema.proptest {
            attrs.push(String::from(
                r#"#[cfg_attr(feature = "proptest", derive(Arbitrary))]"#,
            ));

            let mut modifiers = None;
            for level in ProptestLevel::iter() {
                if let Some(mods) = proptest.get(&level) {
                    modifiers = Some(mods);
                };

                let Some(mods) = modifiers else { continue };

                let mut args = Vec::new();
                if let Some(filter) = &mods.filter {
                    args.push(format!("filter = r#\"{filter}\"#"));
                }
                if args.is_empty() {
                    continue;
                }

                attrs.push(format!(
                    r#"#[cfg_attr(feature = "proptest-{level}", proptest({args}))]"#,
                    args = args.join(",")
                ));
            }
        }

        // Clone attrs for options before adding display & codec related attrs
        let mut options_attrs = attrs.clone();
        options_attrs.retain(|attr| attr != "PatchNode");

        // Add attributes for displaying name
        attrs.push("#[derive(derive_more::Display)]".to_string());
        attrs.push(format!("#[display(fmt = \"{title}\")]"));

        // Add a #[patch(...)] attribute for main struct if it has authors that is a Vec<Author>
        if derive_patch {
            let mut args = Vec::new();

            if let Some(authors) = schema.properties.get("authors") {
                if let Some(Items::Ref(ItemsRef { r#ref })) = &authors.items {
                    if r#ref == "Author" {
                        let authors_on = if authors.is_required || authors.is_core {
                            r#"authors_on = "self""#
                        } else {
                            r#"authors_on = "options""#
                        }
                        .to_string();
                        args.push(authors_on);

                        if let Some(authors_take) = schema.patch.as_ref().and_then(|options| {
                            options
                                .take_authors
                                .then(|| "authors_take = true".to_string())
                        }) {
                            args.push(authors_take)
                        }
                    }
                }
            }

            if let Some(options) = &schema.patch {
                if let Some(apply_with) = &options.apply_with {
                    args.push(format!(r#"apply_with = "{apply_with}""#));
                }
            }

            if !args.is_empty() {
                attrs.push(format!("#[patch({})]", args.join(", ")));
            }
        }

        // Add #[dom] attribute for main struct if necessary
        if let Some(spec) = &schema.dom {
            if spec.derive {
                if let Some(elem) = &spec.elem {
                    attrs.push(format!(r#"#[dom(elem = "{elem}")]"#));
                }
            }
        }

        // Add #[html] attribute for main struct if necessary
        if let Some(html) = &schema.html {
            let mut args = Vec::new();

            if let Some(elem) = &html.elem {
                args.push(format!("elem = \"{elem}\""));
            }
            if !html.attrs.is_empty() {
                args.push(format!(
                    "attribs({})",
                    html.attrs
                        .iter()
                        .map(|(name, value)| format!(
                            "{name} = \"{value}\"",
                            name = name.replace('-', "__").replace(':', "_")
                        ))
                        .join(", ")
                ));
            }
            if html.special {
                args.push("special".to_string());
            }

            attrs.push(format!("#[html({})]", args.join(", ")));
        }

        // Add #[jats] attribute for main struct if necessary
        if let Some(jats) = &schema.jats {
            let mut args = Vec::new();

            if let Some(elem) = &jats.elem {
                args.push(format!("elem = \"{elem}\""));
            }
            if !jats.attrs.is_empty() {
                args.push(format!(
                    "attribs({})",
                    jats.attrs
                        .iter()
                        .map(|(name, value)| format!(
                            "{name} = \"{value}\"",
                            name = name.replace('-', "__").replace(':', "_")
                        ))
                        .join(", ")
                ));
            }
            if jats.special {
                args.push("special".to_string());
            }

            attrs.push(format!("#[jats({})]", args.join(", ")));
        }

        // Add #[markdown] attribute for main struct if necessary
        if let Some(markdown) = &schema.markdown {
            if markdown.derive {
                let mut args = Vec::new();

                if let Some(template) = &markdown.template {
                    args.push(format!("template = \"{template}\""));
                }
                if let Some(escape) = &markdown.escape {
                    args.push(format!("escape = \"{escape}\""));
                }

                attrs.push(format!("#[markdown({})]", args.join(", ")))
            }
        }

        let attrs = attrs.join("\n");
        let options_attrs = options_attrs.join("\n");

        let mut fields = Vec::new();
        let mut used_types = HashSet::new();
        for (name, property) in schema.properties.iter() {
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

            // Determine Rust type for the property
            let (mut typ, is_vec) = if name == "type" {
                (format!(r#"MustBe!("{title}")"#), false)
            } else {
                let (typ, is_vec, ..) = Self::rust_type(dest, property)?;
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

            // Add #[serde] aliases for field
            if !property.aliases.is_empty() {
                attrs.push(format!(
                    "#[serde({})]",
                    property
                        .aliases
                        .iter()
                        .map(|alias| format!("alias = \"{alias}\""))
                        .join(", ")
                ));
            }

            // Add #[serde] attribute for field if necessary
            if let Some(serde) = &property.serde {
                let mut args = vec!["default".to_string()];

                if let Some(deserialize_with) = &serde.deserialize_with {
                    // `deserializeWith: none` is used in the schema to avoid the
                    // default behavior for arrays below (which is problematic for
                    // arrays of `Node` or `Primitive` (since they can be arrays themselves))
                    if deserialize_with != "none" {
                        args.push(format!("deserialize_with = \"{deserialize_with}\""));
                    }
                }

                attrs.push(format!("#[serde({})]", args.join(", ")))
            } else if property.is_array() {
                if property.is_required {
                    attrs.push("#[serde(deserialize_with = \"one_or_many\")]".to_string())
                } else {
                    attrs.push(
                        "#[serde(default, deserialize_with = \"option_one_or_many\")]".to_string(),
                    )
                }
            }

            if !property.strip.is_empty() {
                attrs.push(format!("#[strip({})]", property.strip.iter().join(", ")));
            }

            // If walk is not specified, defaults to true for `content` property
            let walk = property.walk.unwrap_or_else(|| name == "content");
            if walk {
                attrs.push(String::from("#[walk]"));
            }

            // If patch is not specified, defaults to all formats for `content` property
            if property.patch.is_some() || name == "content" {
                let formats = if let Some(patch) = &property.patch {
                    patch
                        .formats
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .map(|format| format!("format = \"{format}\""))
                        .join(", ")
                } else {
                    "format = \"all\"".to_string()
                };

                attrs.push(format!("#[patch({formats})]"));
            }

            // Add proptest related attributes
            if schema.proptest.is_some() {
                if let Some(proptest) = &property.proptest {
                    let mut modifiers = None;
                    for level in ProptestLevel::iter() {
                        if let Some(mods) = proptest.get(&level) {
                            modifiers = Some(mods);
                        };

                        let Some(mods) = modifiers else { continue };

                        let mut args = Vec::new();
                        if let Some(strategy) = &mods.strategy {
                            args.push(format!("strategy = r#\"{strategy}\"#"));
                        }
                        if let Some(value) = &mods.value {
                            args.push(format!("value = r#\"{value}\"#"));
                        }
                        if let Some(regex) = &mods.regex {
                            args.push(format!("regex = r#\"{regex}\"#"));
                        }
                        if let Some(filter) = &mods.filter {
                            args.push(format!("filter = r#\"{filter}\"#"));
                        }
                        if args.is_empty() {
                            continue;
                        }

                        attrs.push(format!(
                            r#"#[cfg_attr(feature = "proptest-{level}", proptest({args}))]"#,
                            args = args.join(",")
                        ));
                    }
                } else if !property.is_required {
                    // The default is for optional properties to be None at all proptest levels
                    attrs.push(String::from(
                        r#"#[cfg_attr(feature = "proptest", proptest(value = "None"))]"#,
                    ));
                } else {
                    // The default for all other properties is to generate their default at all proptest levels
                    attrs.push(String::from(
                        r#"#[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]"#
                    ));
                }
            }

            // Add #[dom] attribute for field if necessary
            if let (true, Some(dom)) = (
                schema.dom.as_ref().map(|dom| dom.derive).unwrap_or(true),
                &property.dom,
            ) {
                let mut args = Vec::new();

                if dom.skip {
                    args.push("skip".to_string());
                } else if let Some(with) = &dom.with {
                    args.push(format!("with = \"{with}\""));
                } else if let Some(elem) = &dom.elem {
                    args.push(format!("elem = \"{elem}\""));
                } else if let Some(attr) = &dom.attr {
                    args.push(format!("attr = \"{attr}\""));
                }

                attrs.push(format!("#[dom({})]", args.join(", ")))
            }

            // Add #[html] attribute for field if necessary
            if let Some(html) = &property.html {
                let mut args = Vec::new();

                if let Some(attr) = &html.attr {
                    args.push(format!("attr = \"{attr}\""));
                }
                if html.content {
                    args.push("content".to_string());
                }
                if let Some(slot) = &html.slot {
                    args.push(format!("slot = \"{slot}\""));
                }

                attrs.push(format!("#[html({})]", args.join(", ")))
            }

            // Add #[jats] attribute for field if necessary
            if let Some(jats) = &property.jats {
                let mut args = Vec::new();

                if let Some(elem) = &jats.elem {
                    args.push(format!("elem = \"{elem}\""));
                }
                if !jats.attrs.is_empty() {
                    args.push(format!(
                        "attribs({})",
                        jats.attrs
                            .iter()
                            .map(|(name, value)| format!(
                                "{name} = \"{value}\"",
                                name = name.replace('-', "__").replace(':', "_")
                            ))
                            .join(", ")
                    ));
                }
                if let Some(attr) = &jats.attr {
                    args.push(format!("attr = \"{attr}\""));
                }
                if jats.content {
                    args.push("content".to_string());
                }

                attrs.push(format!("#[jats({})]", args.join(", ")))
            }

            let name = escape_keyword(&name);

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

{options_attrs}
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
                r#"

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<{title}Options>,"#
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
    }}"#
            )
        } else {
            String::new()
        };

        let uid_proptest = if schema.proptest.is_some() {
            r#"#[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]"#
        } else {
            ""
        };

        let nick = match &schema.nick {
            Some(nick) => nick.to_lowercase(),
            None => title.to_lowercase(),
        };
        let nick = nick.as_bytes();
        let nick = format!(
            "const NICK: [u8; 3] = [{}, {}, {}];",
            nick[0], nick[1], nick[2]
        );

        write(
            path,
            &format!(
                r#"{GENERATED_COMMENT}

use crate::prelude::*;

{uses}/// {description}
{attrs}
pub struct {title} {{
    {core_fields}

    /// A unique identifier for a node within a document
    {uid_proptest}
    #[serde(skip)]
    pub uid: NodeUid
}}{options}

impl {title} {{
    {nick}
    
    pub fn node_type(&self) -> NodeType {{
        NodeType::{title}
    }}

    pub fn node_id(&self) -> NodeId {{
        NodeId::new(&Self::NICK, &self.uid)
    }}
    {new}
}}
"#
            ),
        )
        .await?;

        Ok(title.to_string())
    }

    /// Generate a Rust `enum` for an `anyOf` root schema or property schema.
    ///
    /// Returns the name of the generated enum.
    fn rust_any_of(dest: &Path, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Schema has no anyOf");
        };

        let variants = any_of
            .iter()
            .map(|schema| {
                let (typ, is_array, is_type) = Self::rust_type(dest, schema)?;
                let variant = if is_array {
                    Self::rust_array_of(dest, &typ)?
                } else {
                    typ
                };
                Ok::<_, Report>((variant, is_type, schema.clone()))
            })
            .collect::<Result<Vec<_>>>()?;

        let name = schema
            .title
            .clone()
            .unwrap_or_else(|| variants.iter().map(|(variant, ..)| variant).join("Or"));

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
            variants
                .iter()
                .map(|(variant, ..)| format!("[`{variant}`]"))
                .join(" or ")
        };

        let mut uses = variants
            .iter()
            .sorted_by(|(a, ..), (b, ..)| a.cmp(b))
            .filter_map(|(variant, is_type, ..)| {
                let module = variant.to_snake_case();
                let module = escape_keyword(&module);
                is_type.then_some(format!("use super::{module}::{variant};",))
            })
            .join("\n");
        if !uses.is_empty() {
            uses.push_str("\n\n");
        }

        let default = schema
            .default
            .as_ref()
            .map(Self::rust_value)
            .or(variants.first().map(|(variant, ..)| variant.clone()));

        let mut unit_variants = true;
        let variants = variants
            .into_iter()
            .map(|(variant, is_type, variant_schema)| {
                let mut attrs = Vec::new();

                // Add default attribute if the variant is the default
                if let Some(default) = &default {
                    if default == &variant {
                        attrs.push(String::from("#[default]"))
                    }
                }

                // Add proptest related attributes
                if let Some(proptest) = &variant_schema.proptest {
                    let mut modifiers = None;
                    for level in ProptestLevel::iter() {
                        if let Some(mods) = proptest.get(&level) {
                            modifiers = Some(mods);
                        };

                        let Some(mods) = modifiers else { continue };

                        let mut args = Vec::new();
                        if mods.skip {
                            args.push(String::from("skip"))
                        }
                        if let Some(weight) = mods.weight {
                            args.push(format!("weight = {weight}"))
                        }
                        if let Some(strategy) = &mods.strategy {
                            args.push(format!("strategy = r#\"{strategy}\"#"));
                        }
                        if let Some(value) = &mods.value {
                            args.push(format!("value = r#\"{value}\"#"));
                        }
                        if let Some(regex) = &mods.regex {
                            args.push(format!("regex = r#\"{regex}\"#"));
                        }
                        if let Some(filter) = &mods.filter {
                            args.push(format!("filter = r#\"{filter}\"#"));
                        }
                        if args.is_empty() {
                            continue;
                        }

                        attrs.push(format!(
                            r#"#[cfg_attr(feature = "proptest-{level}", proptest({args}))]"#,
                            args = args.join(",")
                        ));
                    }
                }

                let mut attrs = attrs.join("\n    ");
                if !attrs.is_empty() {
                    attrs.push_str("\n    ");
                }

                let desc = variant_schema
                    .description
                    .as_ref()
                    .map(|desc| format!("/// {}\n    ", desc.replace('\n', " ")))
                    .unwrap_or_default();

                if is_type {
                    unit_variants = false;
                    format!("{desc}{attrs}{variant}({variant}),")
                } else {
                    format!("{desc}{attrs}{variant},")
                }
            })
            .join("\n\n    ");

        let mut attrs = vec![];

        let mut derives = vec![
            "Debug",
            "strum::Display",
            "Clone",
            "PartialEq",
            "Serialize",
            "Deserialize",
            "StripNode",
            "WalkNode",
            "WriteNode",
        ];

        if default.is_some() {
            derives.push("SmartDefault");
        };

        if unit_variants {
            derives.push("strum::EnumString");
        };

        let title = name.as_str();

        if unit_variants && !NO_ORD.contains(&title) {
            derives.push("Eq, PartialOrd, Ord");
        }

        if !NO_READ_NODE.contains(&title) {
            derives.push("ReadNode");
        }

        if schema
            .patch
            .as_ref()
            .map(|spec| spec.derive)
            .unwrap_or(true)
        {
            derives.push("PatchNode");
        }

        // Codec derives
        {
            if schema.dom.as_ref().map(|spec| spec.derive).unwrap_or(true) {
                derives.push("DomCodec");
            }

            derives.append(&mut vec!["HtmlCodec", "JatsCodec"]);

            if schema
                .markdown
                .as_ref()
                .map(|spec| spec.derive)
                .unwrap_or(true)
            {
                derives.push("MarkdownCodec");
            }

            derives.push("TextCodec");
        }

        attrs.push(format!("#[derive({})]", derives.join(", ")));

        attrs.push(format!(
            "#[serde({}crate = \"common::serde\")]",
            match unit_variants {
                false => "untagged, ",
                true => "",
            }
        ));

        if unit_variants {
            attrs.push(String::from(
                "#[strum(ascii_case_insensitive, crate = \"common::strum\")]",
            ));
        };

        // Add proptest related attributes
        if let Some(proptest) = &schema.proptest {
            attrs.push(String::from(
                r#"#[cfg_attr(feature = "proptest", derive(Arbitrary))]"#,
            ));

            let mut modifiers = None;
            for level in ProptestLevel::iter() {
                if let Some(mods) = proptest.get(&level) {
                    modifiers = Some(mods);
                };

                let Some(mods) = modifiers else { continue };

                let mut args = Vec::new();
                if let Some(filter) = &mods.filter {
                    args.push(format!("filter = r#\"{filter}\"#"));
                }
                if args.is_empty() {
                    continue;
                }

                attrs.push(format!(
                    r#"#[cfg_attr(feature = "proptest-{level}", proptest({args}))]"#,
                    args = args.join(",")
                ));
            }
        }

        let attrs = attrs.join("\n");

        let rust = format!(
            r#"{GENERATED_COMMENT}

use crate::prelude::*;

{uses}/// {description}
{attrs}
pub enum {name} {{
    {variants}
}}
"#
        );
        std::fs::write(path, rust)?;

        Ok(name)
    }

    /// Generate a Rust `type` for an array of a type
    ///
    /// Returns the name of the generated type which will be the plural
    /// of the type of the array items.
    fn rust_array_of(dest: &Path, item_type: &str) -> Result<String> {
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
        std::fs::write(path, rust)?;

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
