//! Generation of reference documentation from Stencila Schema

use std::{
    collections::{BTreeSet, HashMap},
    fs::read_dir,
    path::{Path, PathBuf},
};

use codecs::{CodecSupport, Format};
use common::{
    eyre::{bail, Context as _, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    strum::IntoEnumIterator,
    tokio::fs::{create_dir_all, remove_dir_all, remove_file},
};
use schema::{shortcuts::*, Article, Block, Inline, Node, NodeType, NoteType, TableCell};
use status::Status;

use crate::{
    schema::{Category, Items, ProptestLevel, Schema, Type},
    schemas::Schemas,
};

/// Markdown files that should not be generated
///
/// These files are manually written so should not be removed during cleanup.
const HANDWRITTEN_FILES: &[&str] = &["README.md"];

impl Schemas {
    /// Generate a Markdown file for each schema
    ///
    /// This function takes a dog-food approach to generating documentation for
    /// each Stencila Schema type. That is, for each type, a Stencila `Article` is
    /// constructed and then exported as Markdown. This makes this crate sort of recursive
    /// in that it both generates the Rust types in the [`Schemas::rust`] function,
    /// and then uses those to construct the `Article`.
    #[allow(clippy::print_stderr)]
    pub async fn docs_types(&self) -> Result<()> {
        eprintln!("Generating documentation for types");

        // The top level destination for documentation
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference/schema");

        // Ensure it is present
        if !dest.exists() {
            create_dir_all(&dest).await?;
        }

        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        // Ensure it is empty
        for file in read_dir(&dest)?.flatten() {
            let path = file.path();

            let Some(name) = path.file_name() else {
                continue;
            };

            if HANDWRITTEN_FILES.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }

            if path.is_file() {
                remove_file(&path).await?
            } else {
                remove_dir_all(path).await?
            }
        }

        // Ensure category folders are present
        for category in Category::iter() {
            create_dir_all(dest.join(category.to_string())).await?;
        }

        // Create a map of each schema title to it's documentation URL
        // This is necessary because we usually only have access to the string title of a related schema
        // and do not know the category that it is nested within
        let urls: HashMap<String, String> = self
            .schemas
            .iter()
            .map(|(title, schema)| {
                (
                    title.clone(),
                    format!(
                        "https://github.com/stencila/stencila/blob/main/docs/reference/schema/{category}/{slug}.md",
                        category = schema.category,
                        slug = title.to_kebab_case()
                    ),
                )
            })
            .collect();

        // Create a map of the children of each schema
        let mut children: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (title, schema) in &self.schemas {
            for parent in &schema.extends {
                children
                    .entry(parent.to_string())
                    .and_modify(|children| {
                        children.insert(title.to_string());
                    })
                    .or_insert_with(|| BTreeSet::from([title.to_string()]));
            }
        }

        let context = Context { urls, children };

        // Create a file for each schema
        let futures = self
            .schemas
            .values()
            .map(|schema| docs_file(&dest, schema, &context));
        try_join_all(futures).await?;

        // Create an index, grouped by category
        // TODO

        Ok(())
    }
}

struct Context {
    urls: HashMap<String, String>,
    children: HashMap<String, BTreeSet<String>>,
}

/// Generate a documentation file for a schema
async fn docs_file(dest: &Path, schema: &Schema, context: &Context) -> Result<String> {
    let Some(title) = &schema.title else {
        bail!("Schema has no title");
    };

    let path = dest.join(format!(
        "{category}/{title}.md",
        category = schema.category,
        title = title.to_kebab_case()
    ));
    if path.exists() {
        return Ok(title.to_string());
    }

    let article = if schema.is_object() {
        docs_object(title, schema, context)
    } else if schema.is_union() {
        docs_any_of(title, schema, context)
    } else {
        docs_primitive(title, schema)
    };

    codecs::to_path(&Node::Article(article), &path, None).await?;

    Ok(title.to_string())
}

/// Generate documentation for an object schema with `properties`
fn docs_object(title: &str, schema: &Schema, context: &Context) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut properties(title, schema, context));
    content.append(&mut related(title, schema, context));
    if !schema.r#abstract {
        content.append(&mut formats(title, schema));
    }
    content.append(&mut bindings(title, schema));
    if schema.proptest.is_some() {
        content.append(&mut proptests_object(title, schema));
    }
    content.append(&mut source(title));

    Article {
        content,
        ..Default::default()
    }
}

/// Generate documentation file for an `anyOf` root schema
fn docs_any_of(title: &str, schema: &Schema, context: &Context) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut members(title, schema, context));
    content.append(&mut bindings(title, schema));
    if schema.proptest.is_some() {
        content.append(&mut proptests_anyof(title, schema));
    }
    content.append(&mut source(title));

    Article {
        content,
        ..Default::default()
    }
}

/// Generate documentation for a primitive schema
fn docs_primitive(title: &str, schema: &Schema) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut formats(title, schema));
    content.append(&mut bindings(title, schema));
    content.append(&mut source(title));

    Article {
        content,
        ..Default::default()
    }
}

/// Generate introductory headers and paragraphs for a schema
fn intro(title: &str, schema: &Schema) -> Vec<Block> {
    let mut blocks = vec![h1([t(title.to_title_case())])];

    if let Some(description) = &schema.description {
        blocks.push(p([stg([t(description.trim())])]));
    }

    if let Some(comment) = &schema.comment {
        blocks.push(p([t(comment)]));
    }

    if let Some(id) = schema.jid.clone() {
        let id = if let Some(name) = id.clone().strip_prefix("schema:") {
            lnk([ci(id)], format!("https://schema.org/{name}"))
        } else {
            ci(id)
        };
        blocks.push(p([stg([ci("@id")]), t(": "), id]));
    }

    if !matches!(schema.status, Status::Stable) {
        blocks.push(p([t(if matches!(schema.status, Status::Experimental) {
            "This type is marked as experimental and is likely to change."
        } else {
            "This type is marked as unstable and is subject to change."
        })]))
    }

    blocks
}

/// Generate a "Properties" section for a schema
fn properties(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut rows = vec![tr([
        th([t("Name")]),
        th([t("Aliases")]),
        th([ci("@id")]),
        th([t("Type")]),
        th([t("Description")]),
        th([t("Inherited from")]),
    ])];

    for (name, property) in &schema.properties {
        if name == "type" {
            continue;
        }

        #[allow(unstable_name_collisions)]
        let mut aliases = property
            .aliases
            .iter()
            .map(ci)
            .intersperse(t(", "))
            .collect_vec();
        if aliases.is_empty() {
            aliases.push(t("-"));
        };

        let id = property.jid.clone().unwrap_or_default();
        let id = if id.starts_with("schema:") {
            lnk([ci(&id)], id.replace("schema:", "https://schema.org/"))
        } else {
            ci(id)
        };

        fn type_link(title: &str, context: &Context) -> Inline {
            let url = context.urls.get(title).cloned().unwrap_or_default();
            lnk([ci(title)], url)
        }
        fn schema_type(schema: &Schema, context: &Context) -> Vec<Inline> {
            if let Some(r#type) = &schema.r#type {
                if matches!(r#type, Type::Array) {
                    let mut items = match &schema.items {
                        Some(Items::Type(r#type)) => {
                            vec![type_link(&r#type.r#type.to_string(), context)]
                        }
                        Some(Items::Ref(r#ref)) => vec![type_link(&r#ref.r#ref, context)],
                        Some(Items::AnyOf(any_of)) => {
                            let mut inner = schema_type(
                                &Schema {
                                    any_of: Some(any_of.any_of.clone()),
                                    ..Default::default()
                                },
                                context,
                            );
                            inner.insert(0, t("("));
                            inner.push(t(")"));
                            inner
                        }
                        _ => vec![t("?")],
                    };
                    items.push(t("*"));
                    items
                } else {
                    vec![type_link(&r#type.to_string(), context)]
                }
            } else if let Some(r#ref) = &schema.r#ref {
                vec![type_link(r#ref, context)]
            } else if let Some(any_of) = &schema.any_of {
                any_of.iter().fold(Vec::new(), |mut inlines, schema| {
                    if !inlines.is_empty() {
                        inlines.push(t(" | "));
                    }
                    inlines.append(&mut schema_type(schema, context));
                    inlines
                })
            } else {
                vec![t("")]
            }
        }
        let r#type = schema_type(property, context);

        let description = property
            .description
            .clone()
            .unwrap_or_default()
            .trim()
            .replace('\n', " ");

        let from = if property.defined_on != title {
            let from = property.defined_on.as_str().to_pascal_case();
            let url = context.urls.get(&from).cloned().unwrap_or_default();
            lnk([ci(from)], url)
        } else {
            t("-")
        };

        rows.push(tr([
            td([ci(name)]),
            td(aliases),
            td([id]),
            td(r#type),
            td([t(description)]),
            td([from]),
        ]));
    }

    vec![
        h2([t("Properties")]),
        p([t("The "), ci(title), t(" type has these properties:")]),
        tbl(rows),
    ]
}

/// Generate a "Members" section for a schema
fn members(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut items = Vec::new();
    for schema in schema.any_of.as_ref().expect("should have an anyOf") {
        if let Some(title) = &schema.r#ref {
            let url = context.urls.get(title).cloned().unwrap_or_default();
            items.push(li([lnk([ci(title)], url)]));
        } else if let Some(value) = &schema.r#const {
            items.push(li([ci(value.to_string())]));
        } else {
            continue;
        }
    }

    vec![
        h2([t("Members")]),
        p([t("The "), ci(title), t(" type has these members:")]),
        ul(items),
    ]
}

/// Generate a "Formats" section for a schema
fn formats(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([t("Format")]),
        th([t("Encoding")]),
        th([t("Decoding")]),
        th([t("Status")]),
        th([t("Notes")]),
    ])];

    let node_type = NodeType::try_from(title).ok();
    for format in Format::iter() {
        let Ok(codec) = codecs::get(None, Some(&format), None) else {
            continue;
        };

        let name = format.name();
        let name = td([lnk(
            [t(name)],
            format!(
                "https://github.com/stencila/stencila/blob/main/docs/reference/formats/{format}.md"
            ),
        )]);

        fn codec_support(support: CodecSupport) -> TableCell {
            match support {
                CodecSupport::None => td([]),
                support => td([t(format!(
                    "{icon} {desc}",
                    icon = match support {
                        CodecSupport::NoLoss => "🟢",
                        CodecSupport::LowLoss => "🔷",
                        CodecSupport::HighLoss => "⚠️",
                        CodecSupport::None => "",
                    },
                    desc = support.to_string().to_sentence_case()
                ))]),
            }
        }

        let encoding = codec_support(
            node_type
                .as_ref()
                .map(|node_type| codec.supports_to_type(*node_type))
                .unwrap_or_default(),
        );

        let decoding = codec_support(
            node_type
                .as_ref()
                .map(|node_type| codec.supports_from_type(*node_type))
                .unwrap_or_default(),
        );

        let status = td([t(format!(
            "{icon} {desc}",
            icon = codec.status().emoji(),
            desc = codec.status().to_string().to_sentence_case()
        ))]);

        let notes = td(Schemas::docs_format_notes(schema, format));

        rows.push(tr([name, encoding, decoding, status, notes]));
    }

    vec![
        h2([t("Formats")]),
        p([
            t("The "),
            ci(title),
            t(" type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:"),
        ]),
        tbl(rows),
    ]
}

/// Generate a "testing" section for an object type schema
fn proptests_anyof(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([t("Variant")]),
        th([t("Complexity")]),
        th([t("Description")]),
        th([t("Strategy")]),
    ])];

    for variant_schema in schema.any_of.iter().flatten() {
        let Some(proptest) = &variant_schema.proptest else {
            continue;
        };
        let Some(variant_name) = &variant_schema.r#ref else {
            continue;
        };

        for level in ProptestLevel::iter() {
            let Some(options) = proptest.get(&level) else {
                continue;
            };

            let description = options
                .description
                .clone()
                .unwrap_or_else(|| String::from("Generate an arbitrary value of type."));

            let mut strategy = if options.skip {
                vec![t("-")]
            } else if let Some(strategy) = &options.strategy {
                vec![ci(strategy)]
            } else if let Some(value) = &options.value {
                vec![ci(value)]
            } else {
                vec![t("Default for level")]
            };
            if let Some(filter) = &options.filter {
                strategy.append(&mut vec![t(" with filter "), ci(filter)]);
            }

            let row = vec![
                if matches!(level, ProptestLevel::Min) {
                    td([ci(variant_name)])
                } else {
                    td([])
                },
                td([t(format!(
                    "{}{}",
                    level.to_string().to_title_case(),
                    if matches!(level, ProptestLevel::Max) {
                        ""
                    } else {
                        "+"
                    }
                ))]),
                td([t(description)]),
                td(strategy),
            ];

            rows.push(tr(row));
        }
    }

    if rows.len() == 1 {
        return Vec::new();
    }

    vec![
        h2([t("Testing")]),
        p([
            t("During property-based (a.k.a generative) testing, the variants of the "),
            ci(title),
            t(" type are generated using the following strategies"),
            nte(
                NoteType::Footnote,
                [p([
                    t("See the "),
                    ci("proptest"),
                    t(" "),
                    lnk([t("book")], "https://proptest-rs.github.io/proptest/"),
                    t(" and the "),
                    lnk([ci("proptest.rs")], "https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs"),
                    t(" module for details."),
                ])],
            ),
            t(" for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level."),

        ]),
        tbl(rows)
    ]
}

/// Generate a "testing" section for a union type schema
fn proptests_object(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([t("Property")]),
        th([t("Complexity")]),
        th([t("Description")]),
        th([t("Strategy")]),
    ])];

    for (property_name, property_schema) in &schema.properties {
        let Some(proptest) = &property_schema.proptest else {
            continue;
        };

        for level in ProptestLevel::iter() {
            let Some(options) = proptest.get(&level) else {
                continue;
            };

            let description = options
                .description
                .clone()
                .unwrap_or_else(|| String::from("Generate an arbitrary value of type."));

            let mut strategy = if let Some(strategy) = &options.strategy {
                vec![ci(strategy)]
            } else if let Some(value) = &options.value {
                vec![ci(value)]
            } else if let Some(regex) = &options.regex {
                vec![t("Regex "), ci(regex)]
            } else {
                vec![t("Default for level")]
            };
            if let Some(filter) = &options.filter {
                strategy.append(&mut vec![t(" with filter "), ci(filter)]);
            }

            let row = vec![
                if matches!(level, ProptestLevel::Min) {
                    td([ci(property_name.clone())])
                } else {
                    td([])
                },
                td([t(format!(
                    "{}{}",
                    level.to_string().to_title_case(),
                    if matches!(level, ProptestLevel::Max) {
                        ""
                    } else {
                        "+"
                    }
                ))]),
                td([t(description)]),
                td(strategy),
            ];

            rows.push(tr(row));
        }
    }

    if rows.len() == 1 {
        return Vec::new();
    }

    vec![
        h2([t("Testing")]),
        p([
            t("During property-based (a.k.a generative) testing, the properties of the "),
            ci(title),
            t(" type are generated using the following strategies"),
            nte(
                NoteType::Footnote,
                [p([
                    t("See the "),
                    ci("proptest"),
                    t(" "),
                    lnk([t("book")], "https://proptest-rs.github.io/proptest/"),
                    t(" and the "),
                    lnk([ci("proptest.rs")], "https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs"),
                    t(" module for details."),
                ])],
            ),
            t(" for each complexity level. Any optional properties that are not in this table are set to "),
            ci("None"),
            t("."),
        ]),
        tbl(rows),
    ]
}

/// Generate a "Related" section for a schema
fn related(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut parents = vec![t("Parents: ")];
    if schema.extends.is_empty() {
        parents.push(t("none"));
    } else {
        for parent in &schema.extends {
            parents.push(lnk(
                [ci(parent)],
                context.urls.get(parent).expect("Missing URL for parent"),
            ));
        }
    }

    let mut children = vec![t("Children: ")];
    if let Some(title) = context.children.get(title) {
        for (index, child) in title.iter().enumerate() {
            if index != 0 {
                children.push(t(", "))
            }
            children.push(lnk(
                [ci(child)],
                context.urls.get(child).expect("Missing URL for child"),
            ));
        }
    } else {
        children.push(t("none"));
    }

    vec![
        h2([t("Related")]),
        p([t("The "), ci(title), t(" type is related to these types:")]),
        ul([li(parents), li(children)]),
    ]
}

/// Generate a "Bindings" section for a schema
fn bindings(title: &str, schema: &Schema) -> Vec<Block> {
    vec![
        h2([t("Bindings")]),
        p([
            t("The "),
            ci(title),
            t(" type is represented in these bindings:"),
        ]),
        ul([
            li([lnk(
                [t("JSON-LD")],
                format!("https://stencila.org/{title}.jsonld"),
            )]),
            li([lnk(
                [t("JSON Schema")],
                format!("https://stencila.org/{title}.schema.json"),
            )]),
            li([t("Python "), t(if schema.is_object() { "class "} else {"type "}), lnk(
                [ci(title)],
                format!("https://github.com/stencila/stencila/blob/main/python/python/stencila/types/{module}.py", module = title.to_snake_case()),
            )]),
            li([t("Rust "), t(if schema.is_object() { "struct "} else {"type "}), lnk(
                [ci(title)],
                format!("https://github.com/stencila/stencila/blob/main/rust/schema/src/types/{module}.rs", module = title.to_snake_case()),
            )]),
            li([t("TypeScript "), t(if schema.is_object() { "class "} else {"type "}), lnk(
                [ci(title)],
                format!("https://github.com/stencila/stencila/blob/main/ts/src/types/{module}.ts", module = title.to_pascal_case()),
            )]),
        ]),
    ]
}

/// Generate a "Source" section for a schema
fn source(title: &str) -> Vec<Block> {
    vec![
        h2([t("Source")]),
        p([
            t("This documentation was generated from "),
            lnk(
                [ci(format!("{title}.yaml"))],
                format!("https://github.com/stencila/stencila/blob/main/schema/{title}.yaml"),
            ),
            t(" by "),
            lnk(
                [ci("docs_type.rs")],
                "https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs",
            ),
            t("."),
        ]),
    ]
}
