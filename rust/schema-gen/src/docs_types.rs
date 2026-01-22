//! Generation of reference documentation from Stencila Schema

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use eyre::{Context as _, Result, bail};
use futures::future::try_join_all;
use inflector::Inflector;
use strum::IntoEnumIterator;
use tokio::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write};

use stencila_codecs::{EncodeOptions, Format};
use stencila_schema::{
    Article, ArticleOptions, Block, Inline, Node, Table, shortcuts::*,
};

use crate::{
    schema::{Category, Items, ProptestLevel, Schema, Status, Type},
    schemas::Schemas,
};

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
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../site/docs/schema");

        // Clean and recreate directory, but preserve the hand-authored index.md
        if dest.exists() {
            let mut entries = read_dir(&dest).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.file_name().and_then(|name| name.to_str()) == Some("index.md") {
                    continue;
                }
                if path.is_dir() {
                    remove_dir_all(&path).await?;
                } else {
                    remove_file(&path).await?;
                }
            }
        } else {
            create_dir_all(&dest).await?;
        }

        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        // Create a map of each schema title to its relative documentation URL
        let urls: HashMap<String, String> = self
            .schemas
            .keys()
            .map(|title| (title.clone(), format!("./{}.md", title.to_kebab_case())))
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

        // Generate _nav.yaml grouped by category
        let nav_content = generate_nav_yaml(&self.schemas);
        tokio::fs::write(dest.join("_nav.yaml"), nav_content).await?;
        eprintln!("Wrote navigation file");

        // Generate index.md with overview
        generate_index(&dest, &self.schemas).await?;
        eprintln!("Wrote index file");

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

    let path = dest.join(format!("{}.md", title.to_kebab_case()));
    if path.exists() {
        return Ok(title.to_string());
    }

    let content = if schema.is_object() {
        docs_object(title, schema, context)
    } else if schema.is_union() {
        docs_any_of(title, schema, context)
    } else {
        docs_primitive(title, schema)
    };

    let title = title.to_title_case();
    let title_inlines = Some(vec![t(title.clone())]);

    let description = schema.description.clone();

    let frontmatter = serde_yaml::to_string(&serde_json::json!({
        "title": title,
        "description": description
    }))
    .map(|yaml| yaml.trim_end().to_string())
    .ok();

    let article = Article {
        title: title_inlines,
        frontmatter,
        content,
        options: Box::new(ArticleOptions {
            description,
            ..Default::default()
        }),
        ..Default::default()
    };

    stencila_codecs::to_path(&Node::Article(article), &path, None).await?;

    Ok(title)
}

/// Generate documentation for an object schema with `properties`
fn docs_object(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut content = intro(title, schema);
    content.append(&mut properties(title, schema, context));
    content.append(&mut related(title, schema, context));
    content.append(&mut bindings(title, schema));
    if schema.proptest.is_some() {
        content.append(&mut proptests_object(title, schema));
    }
    content.append(&mut source(title));

    content
}

/// Generate documentation file for an `anyOf` root schema
fn docs_any_of(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut content = intro(title, schema);
    content.append(&mut members(title, schema, context));
    content.append(&mut bindings(title, schema));
    if schema.proptest.is_some() {
        content.append(&mut proptests_anyof(title, schema));
    }
    content.append(&mut source(title));

    content
}

/// Generate documentation for a primitive schema
fn docs_primitive(title: &str, schema: &Schema) -> Vec<Block> {
    let mut content = intro(title, schema);
    content.append(&mut bindings(title, schema));
    content.append(&mut source(title));

    content
}

/// Generate introductory headers and paragraphs for a schema
fn intro(_title: &str, schema: &Schema) -> Vec<Block> {
    let mut blocks = Vec::new();

    if let Some(comment) = &schema.comment {
        blocks.push(p([t(comment)]));
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
        th([t("Description")]),
        th([t("Type")]),
        th([t("Inherited from")]),
    ])];

    for (name, property) in &schema.properties {
        if name == "type" {
            continue;
        }

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
            td([t(description)]),
            td(r#type),
            td([from]),
        ]));
    }

    vec![
        h1([t("Properties")]),
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
        h1([t("Members")]),
        p([t("The "), ci(title), t(" type has these members:")]),
        ul(items),
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
        h1([t("Testing")]),
        p([
            t("During property-based (a.k.a generative) testing, the variants of the "),
            ci(title),
            t(
                " type are generated using the following strategies. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.",
            ),
        ]),
        Block::Table(Table {
            rows,
            notes: Some(vec![p([
                t("See the "),
                ci("proptest"),
                t(" "),
                lnk([t("book")], "https://proptest-rs.github.io/proptest/"),
                t(" and Stencila Schema's "),
                lnk(
                    [ci("proptest.rs")],
                    "https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs",
                ),
                t(" module for details on the proptest generation strategies listed."),
            ])]),
            ..Default::default()
        }),
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
        h1([t("Testing")]),
        p([
            t("During property-based (a.k.a generative) testing, the properties of the "),
            ci(title),
            t(" type are generated using the following strategies."),
        ]),
        Block::Table(Table {
            rows,
            notes: Some(vec![p([
                t("See the "),
                ci("proptest"),
                t(" "),
                lnk([t("book")], "https://proptest-rs.github.io/proptest/"),
                t(" and Stencila Schema's "),
                lnk(
                    [ci("proptest.rs")],
                    "https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs",
                ),
                t(" module for details on proptest generation strategies listed."),
            ])]),
            ..Default::default()
        }),
    ]
}

/// Generate a "Related" section for a schema
fn related(title: &str, schema: &Schema, context: &Context) -> Vec<Block> {
    let mut parents = vec![t("Parents: ")];
    if schema.extends.is_empty() {
        parents.push(t("None"));
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
        h1([t("Related")]),
        p([t("The "), ci(title), t(" type is related to these types:")]),
        ul([li(parents), li(children)]),
    ]
}

/// Generate a "Bindings" section for a schema
fn bindings(title: &str, schema: &Schema) -> Vec<Block> {
    vec![
        h1([t("Bindings")]),
        p([t("The "), ci(title), t(" type is represented in:")]),
        ul([
            li([lnk(
                [t("JSON-LD")],
                format!("https://stencila.org/{title}.jsonld"),
            )]),
            li([lnk(
                [t("JSON Schema")],
                format!("https://stencila.org/{title}.schema.json"),
            )]),
            li([
                t("Python "),
                t(if schema.is_object() {
                    "class "
                } else {
                    "type "
                }),
                lnk(
                    [ci(title)],
                    format!(
                        "https://github.com/stencila/stencila/blob/main/python/python/stencila/types/{module}.py",
                        module = title.to_snake_case()
                    ),
                ),
            ]),
            li([
                t("Rust "),
                t(if schema.is_object() {
                    "struct "
                } else {
                    "type "
                }),
                lnk(
                    [ci(title)],
                    format!(
                        "https://github.com/stencila/stencila/blob/main/rust/schema/src/types/{module}.rs",
                        module = title.to_snake_case()
                    ),
                ),
            ]),
            li([
                t("TypeScript "),
                t(if schema.is_object() {
                    "class "
                } else {
                    "type "
                }),
                lnk(
                    [ci(title)],
                    format!(
                        "https://github.com/stencila/stencila/blob/main/ts/src/types/{module}.ts",
                        module = title.to_pascal_case()
                    ),
                ),
            ]),
        ]),
    ]
}

/// Generate a "Source" section for a schema
fn source(title: &str) -> Vec<Block> {
    vec![
        h1([t("Source")]),
        p([
            t("This documentation was generated from "),
            lnk(
                [ci(format!("{title}.yaml"))],
                format!("https://github.com/stencila/stencila/blob/main/schema/{title}.yaml"),
            ),
            t(" by "),
            lnk(
                [ci("docs_types.rs")],
                "https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs",
            ),
            t("."),
        ]),
    ]
}

/// Generate _nav.yaml content for schema documentation
fn generate_nav_yaml(schemas: &std::collections::BTreeMap<String, Schema>) -> String {
    let mut output = String::new();
    output.push_str("# Auto-generated navigation for schema docs\n");
    output.push_str("# Regenerate with: cargo run -p stencila-schema-gen\n\n");
    output.push_str("items:\n");

    // Group schemas by category
    let mut by_category: HashMap<Category, Vec<&str>> = HashMap::new();
    for (title, schema) in schemas {
        by_category
            .entry(schema.category)
            .or_default()
            .push(title.as_str());
    }

    // Write each category as a nav group (iterate in enum order)
    for category in Category::iter() {
        if let Some(titles) = by_category.get(&category) {
            output.push_str(&format!(
                "  - label: \"{}\"\n",
                category.to_string().to_title_case()
            ));
            output.push_str("    children:\n");
            for title in titles {
                output.push_str(&format!("      - \"{}\"\n", title.to_kebab_case()));
            }
            output.push('\n');
        }
    }

    output
}

/// Generate index.md for schema documentation
async fn generate_index(
    dest: &Path,
    schemas: &std::collections::BTreeMap<String, Schema>,
) -> Result<()> {
    // Group by category for the index
    let mut by_category: HashMap<Category, Vec<(&str, &Schema)>> = HashMap::new();
    for (title, schema) in schemas {
        by_category
            .entry(schema.category)
            .or_default()
            .push((title.as_str(), schema));
    }

    let mut content = Vec::new();

    for category in Category::iter() {
        if let Some(types) = by_category.get(&category) {
            content.push(h1([t(category.to_string().to_title_case())]));

            let items: Vec<_> = types
                .iter()
                .map(|(title, schema)| {
                    let desc = schema.description.clone().unwrap_or_default();
                    li([
                        lnk([ci(*title)], format!("./{}.md", title.to_kebab_case())),
                        t(format!(" - {desc}")),
                    ])
                })
                .collect();
            content.push(ul(items));
        }
    }

    let article = Article {
        content,
        ..Default::default()
    };

    let md = stencila_codecs::to_string(
        &Node::Article(article),
        Some(EncodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        }),
    )
    .await?;

    let index_path = dest.join("index.md");
    let mut existing = read_to_string(&index_path).await?;

    let mut marker_end = None;
    let mut offset = 0;
    for line in existing.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed == "***" {
            marker_end = Some(offset + line.len());
            break;
        }
        offset += line.len();
    }

    let Some(marker_end) = marker_end else {
        bail!("Index file does not contain thematic break marker `***`");
    };

    let generated = ["\n\n", md.trim(), "\n"].concat();
    existing.replace_range(marker_end.., &generated);

    write(&index_path, existing).await?;

    Ok(())
}
