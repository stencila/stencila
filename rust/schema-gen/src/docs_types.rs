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
    strum::IntoEnumIterator,
    tokio::fs::{create_dir_all, remove_dir_all, remove_file},
};
use schema::{shortcuts::*, Article, Block, Inline, Node, NodeType, TableCell};
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

            if HANDWRITTEN_FILES.contains(&path.file_name().unwrap().to_string_lossy().as_ref()) {
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
    let mut blocks = vec![h1([text(title.to_title_case())])];

    if let Some(description) = &schema.description {
        blocks.push(p([strong([text(description)])]));
    }

    if let Some(comment) = &schema.comment {
        blocks.push(p([text(comment)]));
    }

    if let Some(id) = schema.jid.clone() {
        let id = if let Some(name) = id.clone().strip_prefix("schema:") {
            link([cf(id)], format!("https://schema.org/{name}"))
        } else {
            cf(id)
        };
        blocks.push(p([strong([cf("@id")]), text(": "), id]));
    }

    if !matches!(schema.status, Status::Stable) {
        blocks.push(p([text(
            if matches!(schema.status, Status::Experimental) {
                "This type is marked as experimental and is likely to change."
            } else {
                "This type is marked as unstable and is subject to change."
            },
        )]))
    }

    blocks
}

/// Generate a "Properties" section for a schema
fn properties(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut rows = vec![tr([
        th([text("Name")]),
        th([cf("@id")]),
        th([text("Type")]),
        th([text("Description")]),
        th([text("Inherited from")]),
    ])];

    for (name, property) in &schema.properties {
        if name == "type" {
            continue;
        }

        let id = property.jid.clone().unwrap_or_default();
        let id = if id.starts_with("schema:") {
            link([cf(&id)], id.replace("schema:", "https://schema.org/"))
        } else {
            cf(id)
        };

        fn type_link(title: &str, context: &Context) -> Inline {
            let url = context.urls.get(title).cloned().unwrap_or_default();
            link([cf(title)], url)
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
                            inner.insert(0, text("("));
                            inner.push(text(")"));
                            inner
                        }
                        _ => vec![text("?")],
                    };
                    items.push(text("*"));
                    items
                } else {
                    vec![type_link(&r#type.to_string(), context)]
                }
            } else if let Some(r#ref) = &schema.r#ref {
                vec![type_link(r#ref, context)]
            } else if let Some(any_of) = &schema.any_of {
                any_of.iter().fold(Vec::new(), |mut inlines, schema| {
                    if !inlines.is_empty() {
                        inlines.push(text(" | "));
                    }
                    inlines.append(&mut schema_type(schema, context));
                    inlines
                })
            } else {
                vec![text("")]
            }
        }
        let r#type = schema_type(property, context);

        let description = property.description.clone().unwrap_or_default();

        let from = property.defined_on.as_str().to_pascal_case();
        let url = context.urls.get(&from).cloned().unwrap_or_default();
        let from = link([cf(from)], url);

        rows.push(tr([
            th([text(name)]),
            th([id]),
            th(r#type),
            th([text(description)]),
            th([from]),
        ]));
    }

    vec![
        h2([text("Properties")]),
        p([text("The "), cf(title), text(" type has these properties:")]),
        table(rows),
    ]
}

/// Generate a "Members" section for a schema
fn members(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut items = Vec::new();
    for schema in schema.any_of.as_ref().expect("should have an anyOf") {
        if let Some(title) = &schema.r#ref {
            let url = context.urls.get(title).cloned().unwrap_or_default();
            items.push(li([link([cf(title)], url)]));
        } else if let Some(value) = &schema.r#const {
            items.push(li([cf(value.to_string())]));
        } else {
            continue;
        }
    }

    vec![
        h2([text("Members")]),
        p([text("The "), cf(title), text(" type has these members:")]),
        ul(items),
    ]
}

/// Generate a "Formats" section for a schema
fn formats(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([text("Format")]),
        th([text("Encoding")]),
        th([text("Decoding")]),
        th([text("Status")]),
        th([text("Notes")]),
    ])];

    let node_type = NodeType::try_from(title).ok();
    for format in Format::iter() {
        let Ok(codec) = codecs::get(None, Some(format), None) else {
            continue
        };

        let name = format.name();
        let name = td([link(
            [text(name)],
            format!(
                "https://github.com/stencila/stencila/blob/main/docs/reference/formats/{format}.md"
            ),
        )]);

        fn codec_support(support: CodecSupport) -> TableCell {
            match support {
                CodecSupport::None => td([]),
                support => td([text(format!(
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

        let status = td([text(format!(
            "{icon} {desc}",
            icon = codec.status().emoji(),
            desc = codec.status().to_string().to_sentence_case()
        ))]);

        let notes = td(Schemas::docs_format_notes(schema, format));

        rows.push(tr([name, encoding, decoding, status, notes]));
    }

    vec![
        h2([text("Formats")]),
        p([
            text("The "),
            cf(title),
            text(" type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:"),
        ]),
        table(rows),
    ]
}

/// Generate a "testing" section for an object type schema
fn proptests_anyof(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([text("Variant")]),
        th([text("Complexity")]),
        th([text("Description")]),
        th([text("Strategy")]),
    ])];

    for variant_schema in schema.any_of.as_ref().unwrap() {
        let Some(proptest) = &variant_schema.proptest else {
            continue
        };
        let Some(variant_name) = &variant_schema.r#ref else {
            continue
        };

        for level in ProptestLevel::iter() {
            let Some(options) = proptest.get(&level) else {
                continue
            };

            let description = options
                .description
                .clone()
                .unwrap_or_else(|| String::from("Generate an arbitrary value of type."));

            let mut strategy = if options.skip {
                vec![text("-")]
            } else if let Some(strategy) = &options.strategy {
                vec![cf(strategy)]
            } else if let Some(value) = &options.value {
                vec![cf(value)]
            } else {
                vec![text("Default for level")]
            };
            if let Some(filter) = &options.filter {
                strategy.append(&mut vec![text(" with filter "), cf(filter)]);
            }

            let row = vec![
                if matches!(level, ProptestLevel::Min) {
                    td([cf(variant_name)])
                } else {
                    td([])
                },
                td([text(format!(
                    "{}{}",
                    level.to_string().to_title_case(),
                    if matches!(level, ProptestLevel::Max) {
                        ""
                    } else {
                        "+"
                    }
                ))]),
                td([text(description)]),
                td(strategy),
            ];

            rows.push(tr(row));
        }
    }

    vec![
        h2([text("Testing")]),
        p([
            text("During property-based (a.k.a generative) testing, the variants of the "),
            cf(title),
            text(" type are generated using the following strategies for each complexity level (see the "),
            link([cf("proptest"), text(" book")], "https://proptest-rs.github.io/proptest/"),
            text(" for an explanation of the Rust strategy expressions). Any variant not shown in this table is generated using the strategy for the corresponding type."),

        ]),
        table(rows)
    ]
}

/// Generate a "testing" section for a union type schema
fn proptests_object(title: &str, schema: &Schema) -> Vec<Block> {
    let mut rows = vec![tr([
        th([text("Property")]),
        th([text("Complexity")]),
        th([text("Description")]),
        th([text("Strategy")]),
    ])];

    for (property_name, property_schema) in &schema.properties {
        let Some(proptest) = &property_schema.proptest else {
            continue
        };

        for level in ProptestLevel::iter() {
            let Some(options) = proptest.get(&level) else {
                continue
            };

            let description = options
                .description
                .clone()
                .unwrap_or_else(|| String::from("Generate an arbitrary value of type."));

            let mut strategy = if let Some(strategy) = &options.strategy {
                vec![cf(strategy)]
            } else if let Some(value) = &options.value {
                vec![cf(value)]
            } else if let Some(regex) = &options.regex {
                vec![cf(regex)]
            } else {
                vec![text("Default for level")]
            };
            if let Some(filter) = &options.filter {
                strategy.append(&mut vec![text(" with filter "), cf(filter)]);
            }

            let row = vec![
                if matches!(level, ProptestLevel::Min) {
                    td([cf(property_name.clone())])
                } else {
                    td([])
                },
                td([text(format!(
                    "{}{}",
                    level.to_string().to_title_case(),
                    if matches!(level, ProptestLevel::Max) {
                        ""
                    } else {
                        "+"
                    }
                ))]),
                td([text(description)]),
                td(strategy),
            ];

            rows.push(tr(row));
        }
    }

    if rows.len() == 1 {
        return Vec::new();
    }

    vec![
        h2([text("Testing")]),
        p([
            text("During property-based (a.k.a generative) testing, the properties of the "),
            cf(title),
            text(" type are generated using the following strategies for each complexity level (see the "),
            link([cf("proptest"), text(" book")], "https://proptest-rs.github.io/proptest/"),
            text(" for an explanation of the Rust strategy expressions). Any optional properties that are not in this table are set to "),
            cf("None")

        ]),
        table(rows)
    ]
}

/// Generate a "Related" section for a schema
fn related(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut parents = vec![text("Parents: ")];
    if schema.extends.is_empty() {
        parents.push(text("none"));
    } else {
        for parent in &schema.extends {
            parents.push(link(
                [cf(parent)],
                context.urls.get(parent).expect("Missing URL for parent"),
            ));
        }
    }

    let mut children = vec![text("Children: ")];
    if context.children.get(title).is_none() {
        children.push(text("none"));
    } else {
        for (index, child) in context.children[title].iter().enumerate() {
            if index != 0 {
                children.push(text(", "))
            }
            children.push(link(
                [cf(child)],
                context.urls.get(child).expect("Missing URL for child"),
            ));
        }
    }

    vec![
        h2([text("Related")]),
        p([
            text("The "),
            cf(title),
            text(" type is related to these types:"),
        ]),
        ul([li(parents), li(children)]),
    ]
}

/// Generate a "Bindings" section for a schema
fn bindings(title: &str, schema: &Schema) -> Vec<Block> {
    vec![
        h2([text("Bindings")]),
        p([
            text("The "),
            cf(title),
            text(" type is represented in these bindings:"),
        ]),
        ul([
            li([link(
                [text("JSON-LD")],
                format!("https://stencila.dev/{title}.jsonld"),
            )]),
            li([link(
                [text("JSON Schema")],
                format!("https://stencila.dev/{title}.schema.json"),
            )]),
            li([text("Python "), text(if schema.is_object() { "class "} else {"type "}), link(
                [cf(title)],
                format!("https://github.com/stencila/stencila/blob/main/python/python/stencila/types/{module}.py", module = title.to_snake_case()),
            )]),
            li([text("Rust "), text(if schema.is_object() { "struct "} else {"type "}), link(
                [cf(title)],
                format!("https://github.com/stencila/stencila/blob/main/rust/schema/src/types/{module}.rs", module = title.to_snake_case()),
            )]),
            li([text("TypeScript "), text(if schema.is_object() { "class "} else {"type "}), link(
                [cf(title)],
                format!("https://github.com/stencila/stencila/blob/main/typescript/src/types/{module}.ts", module = title.to_pascal_case()),
            )]),
        ]),
    ]
}

/// Generate a "Source" section for a schema
fn source(title: &str) -> Vec<Block> {
    vec![
        h2([text("Source")]),
        p([
            text("This documentation was generated from "),
            link(
                [cf(format!("{title}.yaml"))],
                format!("https://github.com/stencila/stencila/blob/main/schema/{title}.yaml"),
            ),
            text(" by "),
            link(
                [cf("docs.rs")],
                "https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs",
            ),
            text("."),
        ]),
    ]
}
