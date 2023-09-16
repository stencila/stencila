//! Generation of reference documentation from Stencila Schema

use std::{
    collections::{BTreeSet, HashMap},
    fs::read_dir,
    path::{Path, PathBuf},
};

use codec::Codec;
use codec_markdown::MarkdownCodec;
use common::{
    eyre::{bail, Context as _, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    strum::IntoEnumIterator,
    tokio::fs::{create_dir_all, remove_dir_all, remove_file},
};
use schema::{shortcuts::*, Article, Block, Node};

use crate::{
    schema::{Category, Schema, Status},
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
    pub async fn docs(&self) -> Result<()> {
        eprintln!("Generating documentation");

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
                        "https://stencila.dev/docs/reference/schema/{category}/{slug}",
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
        docs_any_of(title, schema)
    } else {
        docs_primitive(title, schema)
    };

    let md = MarkdownCodec {};
    md.to_path(&Node::Article(article), &path, None).await?;

    Ok(title.to_string())
}

/// Generate documentation for an object schema with `properties`
fn docs_object(title: &str, schema: &Schema, context: &Context) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut properties(title, schema));
    content.append(&mut related(title, schema, context));
    content.append(&mut bindings(title, schema));
    content.append(&mut source(title));

    Article {
        title: Some(vec![text(title)]),
        content,
        ..Default::default()
    }
}

/// Generate documentation file for an `anyOf` root schema
fn docs_any_of(title: &str, schema: &Schema) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut bindings(title, schema));
    content.append(&mut source(title));

    Article {
        title: Some(vec![text(title)]),
        content,
        ..Default::default()
    }
}

/// Generate documentation for a primitive schema
fn docs_primitive(title: &str, schema: &Schema) -> Article {
    let mut content = intro(title, schema);
    content.append(&mut bindings(title, schema));
    content.append(&mut source(title));

    Article {
        title: Some(vec![text(title)]),
        content,
        ..Default::default()
    }
}

/// Generate introductory headers and paragraphs for a schema
fn intro(title: &str, schema: &Schema) -> Vec<Block> {
    let mut blocks = vec![h1([cf(title)])];

    if let Some(description) = &schema.description {
        blocks.push(p([strong([text(description)])]));
    }

    if let Some(comment) = &schema.comment {
        blocks.push(p([text(comment)]));
    }

    let id = schema.jid.clone().unwrap_or_default();
    if let Some(name) = id.strip_prefix("schema:") {
        blocks.push(p([
            text("This type is an implementation of schema.org "),
            link([cf(name)], format!("https://schema.org/{name}")),
            text("."),
        ]));
    }

    if matches!(schema.status, Status::Experimental | Status::Unstable) {
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
fn properties(title: &str, schema: &Schema) -> Vec<Block> {
    let rows = vec![tr([
        th([text("Name")]),
        th([cf("@id")]),
        th([text("Type")]),
        th([text("Description")]),
        th([text("Inherited from")]),
    ])];
    //schema.properties.iter().map();

    vec![h2([text("Properties")]), table(rows)]
}

/// Generate a "Formats" section for a schema
fn formats(title: &str, schema: &Schema) -> Vec<Block> {
    let blocks = vec![h2([text("Formats")])];

    blocks
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
        p([text("Other types related to this type:")]),
        ul([li(parents), li(children)]),
    ]
}

/// Generate a "Bindings" section for a schema
fn bindings(title: &str, schema: &Schema) -> Vec<Block> {
    vec![
        h2([text("Bindings")]),
        p([text("This type is available as:")]),
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
                format!("https://github.com/stencila/stencila/blob/main/python/stencila/types/{module}.py", module = title.to_snake_case()),
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
