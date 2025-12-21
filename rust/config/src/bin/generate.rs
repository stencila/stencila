//! Generate JSON Schema and documentation for Stencila project configuration
//!
//! This binary generates:
//! 1. `json/stencila-config.schema.json` - JSON Schema with documentation URLs
//! 2. `site/docs/config/*.md` - Documentation extracted from Rust source

use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use schemars::generate::SchemaSettings;
use serde_json::{Value, json};
use stencila_config::Config;
use syn::{Attribute, Expr, Fields, Item, Lit, Meta, Type, parse_file};

/// Base URL for documentation
const DOCS_BASE_URL: &str = "https://stencila.io/docs/config";

#[allow(clippy::print_stdout)]
fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Generate JSON Schema
    let schema_value = generate_schema()?;
    write_schema(&manifest_dir, &schema_value)?;

    // Generate documentation
    let lib_path = manifest_dir.join("src/lib.rs");
    let docs = parse_lib_rs(&lib_path)?;
    write_docs(&manifest_dir, &docs)?;

    Ok(())
}

/// Generate the JSON Schema with documentation URLs
fn generate_schema() -> Result<Value> {
    let settings = SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<Config>();

    let mut schema_value = serde_json::to_value(schema)?;

    if let Some(obj) = schema_value.as_object_mut() {
        // Add metadata fields
        obj.insert(
            "$id".to_string(),
            json!("https://stencila.org/stencila-config.schema.json"),
        );
        obj.insert("title".to_string(), json!("Stencila Config"));
        obj.insert(
            "description".to_string(),
            json!("Configuration for Stencila workspaces"),
        );
        // Add documentation URLs to descriptions (SchemaStore format)
        // See: https://github.com/SchemaStore/schemastore/blob/master/CONTRIBUTING.md
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            add_doc_url(properties, "site", "site");
            add_doc_url(properties, "remotes", "remotes");
        }
        if let Some(definitions) = obj.get_mut("definitions").and_then(|d| d.as_object_mut()) {
            add_doc_url(definitions, "SiteConfig", "site");
            add_doc_url(definitions, "RouteTarget", "site#routes");
            add_doc_url(definitions, "RouteRedirect", "site#routes-redirect");
            add_doc_url(definitions, "RouteSpread", "site#routes-spread");
            add_doc_url(definitions, "RemoteValue", "remotes");
            add_doc_url(definitions, "RemoteTarget", "remotes");
            add_doc_url(definitions, "RemoteWatch", "remotes#watch");
            add_doc_url(definitions, "RemoteSpread", "remotes#spread");
        }
    }

    Ok(schema_value)
}

/// Update description to SchemaStore format: "<first line>\n<url>"
fn add_doc_url(map: &mut serde_json::Map<String, Value>, key: &str, doc_path: &str) {
    if let Some(item) = map.get_mut(key).and_then(|v| v.as_object_mut())
        && let Some(desc) = item.get("description").and_then(|v| v.as_str())
    {
        let first_line = desc.lines().next().unwrap_or(desc);
        let url = format!("{DOCS_BASE_URL}/{doc_path}");
        item.insert(
            "description".to_string(),
            json!(format!("{first_line}\n{url}")),
        );
    }
}

/// Write the JSON Schema to file
#[allow(clippy::print_stdout)]
fn write_schema(manifest_dir: &Path, schema_value: &Value) -> Result<()> {
    let output_dir = manifest_dir.join("../../json");
    fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join("stencila-config.schema.json");
    let json = serde_json::to_string_pretty(schema_value)?;
    fs::write(&output_path, json)?;

    let display = output_path.display();
    println!("Generated: {display}");
    Ok(())
}

/// Documentation extracted from a struct or enum
#[derive(Debug, Default)]
struct TypeDocs {
    /// Documentation for the type itself
    doc: String,
    /// Documentation for each field/variant
    fields: Vec<FieldDocs>,
    /// Example(s) extracted from doc comment
    example: Option<String>,
}

/// Documentation for a single field
#[derive(Debug)]
struct FieldDocs {
    /// Field name
    name: String,
    /// Field documentation
    doc: String,
    /// Field type as a string
    type_str: String,
    /// Whether the field is optional
    optional: bool,
    /// Regex pattern from #[schemars(regex(pattern = "..."))]
    pattern: Option<String>,
    /// Example(s) extracted from doc comment
    example: Option<String>,
}

/// All documentation extracted from lib.rs
#[derive(Debug, Default)]
struct ExtractedDocs {
    /// Config struct docs
    config: TypeDocs,
    /// SiteConfig struct docs
    site_config: TypeDocs,
    /// RouteTarget enum docs
    route_target: TypeDocs,
    /// RouteRedirect struct docs
    route_redirect: TypeDocs,
    /// RouteSpread struct docs
    route_spread: TypeDocs,
    /// RemoteValue enum docs
    remote_value: TypeDocs,
    /// RemoteTarget enum docs
    remote_target: TypeDocs,
    /// RemoteWatch struct docs
    remote_watch: TypeDocs,
    /// RemoteSpread struct docs
    remote_spread: TypeDocs,
    /// SpreadMode enum docs
    spread_mode: TypeDocs,
}

/// Parse lib.rs and extract documentation
fn parse_lib_rs(path: &PathBuf) -> Result<ExtractedDocs> {
    let content = fs::read_to_string(path)?;
    let file = parse_file(&content).map_err(|e| eyre!("Failed to parse lib.rs: {e}"))?;

    let mut docs = ExtractedDocs::default();

    for item in file.items {
        match item {
            Item::Struct(s) => {
                let name = s.ident.to_string();
                let type_docs = extract_struct_docs(&s.attrs, &s.fields, &name);

                match name.as_str() {
                    "Config" => docs.config = type_docs,
                    "SiteConfig" => docs.site_config = type_docs,
                    "RouteRedirect" => docs.route_redirect = type_docs,
                    "RouteSpread" => docs.route_spread = type_docs,
                    "RemoteWatch" => docs.remote_watch = type_docs,
                    "RemoteSpread" => docs.remote_spread = type_docs,
                    _ => {}
                }
            }
            Item::Enum(e) => {
                let name = e.ident.to_string();
                let type_docs = extract_enum_docs(&e.attrs, &e.variants, &name);

                match name.as_str() {
                    "RouteTarget" => docs.route_target = type_docs,
                    "RemoteValue" => docs.remote_value = type_docs,
                    "RemoteTarget" => docs.remote_target = type_docs,
                    "SpreadMode" => docs.spread_mode = type_docs,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(docs)
}

/// Extract documentation from struct attributes and fields
fn extract_struct_docs(attrs: &[Attribute], fields: &Fields, _name: &str) -> TypeDocs {
    let raw_doc = extract_doc_comments(attrs);
    let (doc, example) = split_doc_and_example(&raw_doc);

    let fields = match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let field_name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
                let raw_field_doc = extract_doc_comments(&f.attrs);
                let (field_doc, field_example) = split_doc_and_example(&raw_field_doc);
                let (type_str, optional) = type_to_string(&f.ty);
                let pattern = extract_schemars_pattern(&f.attrs);

                FieldDocs {
                    name: field_name,
                    doc: field_doc,
                    type_str,
                    optional,
                    pattern,
                    example: field_example,
                }
            })
            .collect(),
        _ => vec![],
    };

    TypeDocs {
        doc,
        fields,
        example,
    }
}

/// Extract documentation from enum attributes and variants
fn extract_enum_docs(
    attrs: &[Attribute],
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    _name: &str,
) -> TypeDocs {
    let raw_doc = extract_doc_comments(attrs);
    let (doc, example) = split_doc_and_example(&raw_doc);

    let fields = variants
        .iter()
        .map(|v| {
            let variant_name = v.ident.to_string();
            let raw_variant_doc = extract_doc_comments(&v.attrs);
            let (variant_doc, variant_example) = split_doc_and_example(&raw_variant_doc);

            FieldDocs {
                name: variant_name,
                doc: variant_doc,
                type_str: String::new(),
                optional: false,
                pattern: None,
                example: variant_example,
            }
        })
        .collect();

    TypeDocs {
        doc,
        fields,
        example,
    }
}

/// Extract doc comments from attributes
fn extract_doc_comments(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let Meta::NameValue(meta) = &attr.meta
                && let Expr::Lit(expr_lit) = &meta.value
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                return Some(lit_str.value());
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Split doc comment into description and example
///
/// Assumes "Example:" or "Examples:" section is always last in the doc comment.
/// Returns (description, example) where example includes everything after the marker.
fn split_doc_and_example(doc: &str) -> (String, Option<String>) {
    // Find "Example:" or "Examples:" marker
    let marker_pos = doc.find("\nExample:").or_else(|| doc.find("\nExamples:"));

    if let Some(pos) = marker_pos {
        let description = doc[..pos].trim().to_string();
        let example = doc[pos + 1..].trim().to_string(); // +1 to skip the leading newline
        (description, Some(example))
    } else {
        (doc.to_string(), None)
    }
}

/// Extract regex pattern from #[schemars(regex(pattern = "..."))]
fn extract_schemars_pattern(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("schemars")
            && let Ok(nested) = attr.parse_args::<syn::ExprCall>()
            && let Expr::Path(path) = nested.func.as_ref()
            && path.path.is_ident("regex")
        {
            for arg in nested.args.iter() {
                if let Expr::Assign(assign) = arg
                    && let Expr::Path(left) = assign.left.as_ref()
                    && left.path.is_ident("pattern")
                    && let Expr::Lit(lit) = assign.right.as_ref()
                    && let Lit::Str(s) = &lit.lit
                {
                    return Some(s.value());
                }
            }
        }
    }
    None
}

/// Convert a syn::Type to a readable string
fn type_to_string(ty: &Type) -> (String, bool) {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<_> = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            let type_name = segments.join("::");

            // Check if it's Option<T>
            if type_name == "Option" {
                if let Some(segment) = type_path.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    let (inner_str, _) = type_to_string(inner);
                    return (inner_str, true);
                }
                return ("unknown".to_string(), true);
            }

            // Simplify common types
            let simplified = match type_name.as_str() {
                "String" => "string",
                "bool" => "boolean",
                "u16" | "u32" | "u64" | "i16" | "i32" | "i64" => "integer",
                "f32" | "f64" => "number",
                "Url" => "string (URL)",
                "ConfigRelativePath" => "string (path)",
                _ => &type_name,
            };

            (simplified.to_string(), false)
        }
        Type::Reference(type_ref) => type_to_string(&type_ref.elem),
        _ => ("unknown".to_string(), false),
    }
}

/// Write documentation markdown files
fn write_docs(manifest_dir: &Path, docs: &ExtractedDocs) -> Result<()> {
    let output_dir = manifest_dir.join("../../site/docs/config");
    fs::create_dir_all(&output_dir)?;

    // Generate index.md
    write_index_md(&output_dir, docs)?;

    // Generate site.md
    write_site_md(&output_dir, docs)?;

    // Generate routes.md
    write_routes_md(&output_dir, docs)?;

    // Generate remotes.md
    write_remotes_md(&output_dir, docs)?;

    Ok(())
}

/// Write the index.md file
#[allow(clippy::print_stdout)]
fn write_index_md(output_dir: &Path, docs: &ExtractedDocs) -> Result<()> {
    let mut content = r#"---
title: Configuration Reference
description: Reference documentation for stencila.toml configuration files
---

# Configuration Reference

Stencila uses `stencila.toml` files for project configuration. This reference documents all available configuration options.

## Configuration Files

- `stencila.toml` - Main configuration file
- `stencila.local.toml` - Local overrides (typically gitignored)

## Sections

| Section | Description |
|---------|-------------|
"#
    .to_string();

    // Extract first line of each section's doc
    let site_desc = docs
        .site_config
        .doc
        .lines()
        .next()
        .unwrap_or("Site configuration");
    let routes_desc = first_sentence(
        docs.site_config
            .fields
            .iter()
            .find(|f| f.name == "routes")
            .map(|f| f.doc.as_str())
            .unwrap_or("Custom routes for serving content"),
    );
    let remotes_desc = first_sentence(
        docs.config
            .fields
            .iter()
            .find(|f| f.name == "remotes")
            .map(|f| f.doc.as_str())
            .unwrap_or("Remote synchronization configuration"),
    );

    content.push_str(&format!("| [`[site]`](site) | {site_desc} |\n"));
    content.push_str(&format!(
        "| [`[site.routes]`](site#routes) | {routes_desc} |\n"
    ));
    content.push_str(&format!("| [`[remotes]`](remotes) | {remotes_desc} |\n"));

    // Add examples from routes and remotes field doc comments
    content.push_str("\n## Examples\n\n");

    if let Some(routes_field) = docs.site_config.fields.iter().find(|f| f.name == "routes")
        && let Some(example) = &routes_field.example
    {
        content.push_str("### Routes\n\n");
        content.push_str(&format_doc(example));
        content.push_str("\n\n");
    }

    if let Some(remotes_field) = docs.config.fields.iter().find(|f| f.name == "remotes")
        && let Some(example) = &remotes_field.example
    {
        content.push_str("### Remotes\n\n");
        content.push_str(&format_doc(example));
        content.push('\n');
    }

    let path = output_dir.join("index.md");
    fs::write(&path, content)?;
    let display = path.display();
    println!("Generated: {display}");

    Ok(())
}

/// Write the site.md file
#[allow(clippy::print_stdout)]
fn write_site_md(output_dir: &Path, docs: &ExtractedDocs) -> Result<()> {
    let description = first_line(&docs.site_config.doc);

    let mut content = format!(
        r#"---
title: Site Configuration
description: {description}
---

# Site Configuration

"#
    );

    // Add description from SiteConfig doc comment
    content.push_str(&format_doc(&docs.site_config.doc));
    content.push_str("\n\n## Properties\n\n");

    for field in &docs.site_config.fields {
        let name = &field.name;
        let type_str = &field.type_str;
        let optional_str = if field.optional { " (optional)" } else { "" };

        content.push_str(&format!("### `{name}`\n\n"));
        content.push_str(&format!("**Type:** `{type_str}`{optional_str}\n"));

        if let Some(pattern) = &field.pattern {
            content.push_str(&format!("**Pattern:** `{pattern}`\n"));
        }

        content.push('\n');
        content.push_str(&format_doc(&field.doc));
        content.push_str("\n\n");
    }

    // Add example from SiteConfig struct doc comment if available
    if let Some(example) = &docs.site_config.example {
        content.push_str("## ");
        content.push_str(&format_doc(example));
        content.push('\n');
    }

    let path = output_dir.join("site.md");
    fs::write(&path, content)?;
    let display = path.display();
    println!("Generated: {display}");

    Ok(())
}

/// Write the routes.md file
///
/// Note: Routes are now part of site configuration, so this reads from site_config
#[allow(clippy::print_stdout)]
fn write_routes_md(output_dir: &Path, docs: &ExtractedDocs) -> Result<()> {
    let routes_field = docs.site_config.fields.iter().find(|f| f.name == "routes");
    let description = routes_field
        .map(|f| first_line(&f.doc))
        .unwrap_or_else(|| "Custom routes for serving content".to_string());

    let mut content = format!(
        r#"---
title: Routes Configuration
description: {description}
---

# Routes Configuration

Routes are configured under the `[site.routes]` section.

"#
    );

    // Add description from SiteConfig.routes field
    if let Some(routes_field) = routes_field {
        content.push_str(&format_doc(&routes_field.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Route Types\n\n");

    // Document each variant from RouteTarget
    for variant in &docs.route_target.fields {
        let name = &variant.name;
        content.push_str(&format!("### {name}\n\n"));
        content.push_str(&format_doc(&variant.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Redirect Properties\n\n");
    for field in &docs.route_redirect.fields {
        let name = &field.name;
        let type_str = &field.type_str;
        let optional_str = if field.optional { " (optional)" } else { "" };

        content.push_str(&format!("### `{name}`\n\n"));
        content.push_str(&format!("**Type:** `{type_str}`{optional_str}\n\n"));
        content.push_str(&format_doc(&field.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Spread Properties\n\n");
    for field in &docs.route_spread.fields {
        let name = &field.name;
        let type_str = &field.type_str;
        let optional_str = if field.optional { " (optional)" } else { "" };

        content.push_str(&format!("### `{name}`\n\n"));
        content.push_str(&format!("**Type:** `{type_str}`{optional_str}\n\n"));
        content.push_str(&format_doc(&field.doc));
        content.push_str("\n\n");
    }

    // Add example from routes field doc comment
    if let Some(routes_field) = docs.site_config.fields.iter().find(|f| f.name == "routes")
        && let Some(example) = &routes_field.example
    {
        content.push_str("## ");
        content.push_str(&format_doc(example));
        content.push('\n');
    }

    let path = output_dir.join("routes.md");
    fs::write(&path, content)?;
    let display = path.display();
    println!("Generated: {display}");

    Ok(())
}

/// Write the remotes.md file
#[allow(clippy::print_stdout)]
fn write_remotes_md(output_dir: &Path, docs: &ExtractedDocs) -> Result<()> {
    let remotes_field = docs.config.fields.iter().find(|f| f.name == "remotes");
    let description = remotes_field
        .map(|f| first_line(&f.doc))
        .unwrap_or_else(|| "Remote synchronization configuration".to_string());

    let mut content = format!(
        r#"---
title: Remotes Configuration
description: {description}
---

# Remotes Configuration

"#
    );

    // Add description from Config.remotes field
    if let Some(remotes_field) = remotes_field {
        content.push_str(&format_doc(&remotes_field.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Remote Types\n\n");

    // Document each variant from RemoteTarget
    for variant in &docs.remote_target.fields {
        let name = &variant.name;
        content.push_str(&format!("### {name}\n\n"));
        content.push_str(&format_doc(&variant.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Watch Properties\n\n");
    for field in &docs.remote_watch.fields {
        let name = &field.name;
        let type_str = &field.type_str;
        let optional_str = if field.optional { " (optional)" } else { "" };

        content.push_str(&format!("### `{name}`\n\n"));
        content.push_str(&format!("**Type:** `{type_str}`{optional_str}\n\n"));
        if let Some(pattern) = &field.pattern {
            content.push_str(&format!("**Pattern:** `{pattern}`\n\n"));
        }
        content.push_str(&format_doc(&field.doc));
        content.push_str("\n\n");
    }

    content.push_str("## Spread Properties\n\n");
    for field in &docs.remote_spread.fields {
        let name = &field.name;
        let type_str = &field.type_str;
        let optional_str = if field.optional { " (optional)" } else { "" };

        content.push_str(&format!("### `{name}`\n\n"));
        content.push_str(&format!("**Type:** `{type_str}`{optional_str}\n\n"));
        content.push_str(&format_doc(&field.doc));
        content.push_str("\n\n");
    }

    // Add example from remotes field doc comment
    if let Some(remotes_field) = docs.config.fields.iter().find(|f| f.name == "remotes")
        && let Some(example) = &remotes_field.example
    {
        content.push_str("## ");
        content.push_str(&format_doc(example));
        content.push('\n');
    }

    let path = output_dir.join("remotes.md");
    fs::write(&path, content)?;
    let display = path.display();
    println!("Generated: {display}");

    Ok(())
}

/// Format a doc string for markdown output
fn format_doc(doc: &str) -> String {
    // Trim leading/trailing whitespace from each line and rejoin
    doc.lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get the first line from a doc string (for descriptions)
fn first_line(doc: &str) -> String {
    doc.lines().next().unwrap_or(doc).trim().to_string()
}

/// Get the first sentence from a doc string (for table summaries)
fn first_sentence(doc: &str) -> String {
    // Join lines with spaces to handle multi-line doc comments
    let single_line = doc
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    single_line
        .split('.')
        .next()
        .unwrap_or(&single_line)
        .trim()
        .to_string()
}
