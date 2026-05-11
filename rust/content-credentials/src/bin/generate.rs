//! Generate JSON Schema and reference documentation for Stencila provenance.
//!
//! Creates:
//! 1. `json/stencila-provenance-assertion-v1.schema.json` - JSON Schema with documentation URLs
//! 2. `site/docs/content-credentials/provenance-assertion/**/*.md` - Reference documentation pages
//! 3. `site/docs/content-credentials/provenance-assertion/_nav.yaml` - Reference navigation file

#![allow(clippy::print_stdout)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result};
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value, json};
use stencila_content_credentials::schema::{PROVENANCE_SCHEMA, ProvenanceAssertion};

const SCHEMA_FILENAME: &str = "stencila-provenance-assertion-v1.schema.json";
const DOCS_DIR: &str = "content-credentials";
const REFERENCE_DOCS_DIR: &str = "provenance-assertion";
const DOCS_BASE_URL: &str = "https://stencila.io/docs/content-credentials/provenance-assertion";
const GITHUB_BASE_URL: &str = "https://github.com/stencila/stencila/blob/main";

/// A generated documentation page.
#[derive(Debug)]
struct DocPage {
    /// Page slug relative to the docs directory, without `.md`.
    slug: String,

    /// Human readable page title.
    title: String,

    /// Complete Markdown page content.
    content: String,
}

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_dir = manifest_dir.join("../..").canonicalize()?;

    let schema = generate_schema()?;
    let pages = generate_doc_pages(&schema)?;

    write_docs(&repo_dir, &pages)?;

    let schema = add_doc_urls(schema, &pages);
    write_schema(&repo_dir, &schema)?;

    Ok(())
}

/// Generate the base JSON Schema from the Rust wire schema.
fn generate_schema() -> Result<Value> {
    let settings = SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<ProvenanceAssertion>();
    let mut schema = serde_json::to_value(schema)?;

    if let Some(obj) = schema.as_object_mut() {
        obj.insert("$id".to_string(), json!(PROVENANCE_SCHEMA));
        obj.insert(
            "title".to_string(),
            json!("Stencila Provenance Assertion v1"),
        );
    }

    Ok(schema)
}

/// Generate Markdown pages from the schema.
fn generate_doc_pages(schema: &Value) -> Result<Vec<DocPage>> {
    let definitions = definitions(schema)?;
    let definition_names = ordered_definition_names(schema, definitions);
    let type_links = type_links(definitions);
    let mut pages = Vec::with_capacity(definitions.len() + 1);

    pages.push(generate_index_page(
        schema,
        definitions,
        &definition_names,
        &type_links,
    )?);

    for name in &definition_names {
        let definition = definitions
            .get(name)
            .ok_or_else(|| eyre::eyre!("schema definition not found: {name}"))?;
        pages.push(generate_definition_page(name, definition, &type_links)?);
    }

    Ok(pages)
}

/// Definition names ordered by their first use from the assertion payload.
fn ordered_definition_names(schema: &Value, definitions: &Map<String, Value>) -> Vec<String> {
    let mut names = Vec::with_capacity(definitions.len());
    let mut seen = BTreeSet::new();

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for property in properties.values() {
            collect_definition_names(property, definitions, &mut seen, &mut names);
        }
    }

    for name in definitions.keys() {
        if seen.insert(name.to_string()) {
            names.push(name.to_string());
        }
    }

    names
}

/// Recursively collect referenced definitions in first-use order.
fn collect_definition_names(
    value: &Value,
    definitions: &Map<String, Value>,
    seen: &mut BTreeSet<String>,
    names: &mut Vec<String>,
) {
    if let Some(name) = value
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#/definitions/"))
    {
        if seen.insert(name.to_string()) {
            names.push(name.to_string());

            if let Some(definition) = definitions.get(name)
                && let Some(properties) = definition.get("properties").and_then(Value::as_object)
            {
                for property in properties.values() {
                    collect_definition_names(property, definitions, seen, names);
                }
            }
        }

        return;
    }

    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(items) = value.get(key).and_then(Value::as_array) {
            for item in items {
                collect_definition_names(item, definitions, seen, names);
            }
        }
    }

    if let Some(items) = value.get("items") {
        collect_definition_names(items, definitions, seen, names);
    }

    if let Some(additional) = value.get("additionalProperties") {
        collect_definition_names(additional, definitions, seen, names);
    }
}

/// Generate the root reference page for the assertion payload.
fn generate_index_page(
    schema: &Value,
    definitions: &Map<String, Value>,
    definition_names: &[String],
    type_links: &BTreeMap<String, String>,
) -> Result<DocPage> {
    let title = schema
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Stencila Provenance Assertion");
    let description = get_description(schema);
    let properties = properties(schema)?;
    let required = required_fields(schema);

    let mut content = frontmatter(title, &first_paragraph(&description));
    content.push_str("# ");
    content.push_str(title);
    content.push_str("\n\n");
    content.push_str("This reference is generated from the Rust wire schema used for the `org.stencila.provenance` C2PA assertion. It documents the public JSON payload shape, including why each field exists and how records relate to Stencila authorship and provenance concepts.\n\n");

    content.push_str("## Payload Fields\n\n");
    content.push_str(&fields_table(properties, &required, type_links));
    content.push('\n');
    content.push_str(&field_sections(properties, &required, type_links));

    content.push_str("\n## Record Types\n\n");
    content.push_str("| Record | Description |\n");
    content.push_str("|--------|-------------|\n");
    for name in definition_names {
        let definition = definitions
            .get(name)
            .ok_or_else(|| eyre::eyre!("schema definition not found: {name}"))?;
        let slug = type_links
            .get(name)
            .map(String::as_str)
            .unwrap_or("index.md")
            .trim_end_matches(".md");
        content.push_str("| [`");
        content.push_str(name);
        content.push_str("`](");
        content.push_str(slug);
        content.push_str(") | ");
        content.push_str(&escape_table_cell(&first_sentence(&get_description(
            definition,
        ))));
        content.push_str(" |\n");
    }

    content.push_str(&source_footer());

    Ok(DocPage {
        slug: "index".to_string(),
        title: title.to_string(),
        content,
    })
}

/// Generate a reference page for a schema definition.
fn generate_definition_page(
    name: &str,
    definition: &Value,
    type_links: &BTreeMap<String, String>,
) -> Result<DocPage> {
    let title = type_name_to_title(name);
    let description = get_description(definition);
    let properties = properties(definition)?;
    let required = required_fields(definition);

    let mut content = frontmatter(&title, &first_paragraph(&description));
    content.push_str("# ");
    content.push_str(&title);
    content.push_str("\n\n");
    content.push_str(&description);
    content.push_str("\n\n");

    if properties.is_empty() {
        content.push_str("This record has no public fields.\n");
    } else {
        content.push_str("## Fields\n\n");
        content.push_str(&fields_table(properties, &required, type_links));
        content.push('\n');
        content.push_str(&field_sections(properties, &required, type_links));
    }

    content.push_str(&source_footer());

    Ok(DocPage {
        slug: slug_for_type(name),
        title,
        content,
    })
}

/// Render a compact field table.
fn fields_table(
    properties: &Map<String, Value>,
    required: &BTreeSet<String>,
    type_links: &BTreeMap<String, String>,
) -> String {
    let mut table = String::from("| Field | Type | Required | Description |\n");
    table.push_str("|-------|------|----------|-------------|\n");

    for (name, property) in properties {
        table.push_str("| [`");
        table.push_str(name);
        table.push_str("`](#");
        table.push_str(&slugify(name));
        table.push_str(") | ");
        table.push_str(&escape_table_cell(&type_string(property, type_links)));
        table.push_str(" | ");
        table.push_str(if required.contains(name) { "Yes" } else { "No" });
        table.push_str(" | ");
        table.push_str(&escape_table_cell(&first_sentence(&get_description(
            property,
        ))));
        table.push_str(" |\n");
    }

    table
}

/// Render detailed sections for each field.
fn field_sections(
    properties: &Map<String, Value>,
    required: &BTreeSet<String>,
    type_links: &BTreeMap<String, String>,
) -> String {
    let mut sections = String::new();

    for (name, property) in properties {
        sections.push_str("### `");
        sections.push_str(name);
        sections.push_str("`\n\n");
        sections.push_str(&get_description(property));
        sections.push_str("\n\n");
        sections.push_str(&field_metadata(
            property,
            required.contains(name),
            type_links,
        ));
        sections.push_str("\n\n");
    }

    sections
}

/// Render compact metadata for a field.
fn field_metadata(
    property: &Value,
    required: bool,
    type_links: &BTreeMap<String, String>,
) -> String {
    let mut metadata = vec![
        format!("**Type:** {}", type_string(property, type_links)),
        format!("**Required:** {}", if required { "Yes" } else { "No" }),
    ];

    if is_nullable(property) {
        metadata.push("**Nullable:** Yes".to_string());
    }

    if let Some(default) = property.get("default") {
        metadata.push(format!("**Default:** `{}`", default));
    }

    if let Some(values) = enum_values(property) {
        metadata.push(format!("**Allowed values:** {}", values.join(", ")));
    }

    metadata.join(" | ")
}

/// Add documentation links to the schema before writing the published JSON file.
fn add_doc_urls(mut schema: Value, pages: &[DocPage]) -> Value {
    let page_urls: BTreeMap<&str, String> = pages
        .iter()
        .map(|page| {
            let url = if page.slug == "index" {
                DOCS_BASE_URL.to_string()
            } else {
                format!("{DOCS_BASE_URL}/{}", page.slug)
            };
            (page.title.as_str(), url)
        })
        .collect();

    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        add_property_doc_urls(properties, DOCS_BASE_URL);
    }

    if let Some(definitions) = schema.get_mut("definitions").and_then(Value::as_object_mut) {
        for (name, definition) in definitions {
            let title = type_name_to_title(name);
            let base_url = page_urls
                .get(title.as_str())
                .cloned()
                .unwrap_or_else(|| format!("{DOCS_BASE_URL}/{}", slug_for_type(name)));

            append_doc_url(definition, &base_url);

            if let Some(properties) = definition
                .get_mut("properties")
                .and_then(Value::as_object_mut)
            {
                add_property_doc_urls(properties, &base_url);
            }
        }
    }

    schema
}

/// Add documentation URLs to a set of properties.
fn add_property_doc_urls(properties: &mut Map<String, Value>, base_url: &str) {
    for (name, property) in properties {
        append_doc_url(property, &format!("{base_url}#{}", slugify(name)));
    }
}

/// Append a URL to a schema description, preserving the generated Rust docs.
fn append_doc_url(value: &mut Value, url: &str) {
    if let Some(obj) = value.as_object_mut()
        && let Some(description) = obj.get("description").and_then(Value::as_str)
    {
        let description = description.trim_end();
        obj.insert(
            "description".to_string(),
            json!(format!("{description}\n\nDocumentation: {url}")),
        );
    }
}

/// Write documentation pages and navigation.
fn write_docs(repo_dir: &Path, pages: &[DocPage]) -> Result<()> {
    let output_dir = repo_dir
        .join("site/docs")
        .join(DOCS_DIR)
        .join(REFERENCE_DOCS_DIR);
    fs::create_dir_all(&output_dir)?;

    for page in pages {
        let path = output_dir.join(format!("{}.md", page.slug));
        fs::write(&path, &page.content)?;
        println!("Generated: {}", path.display());
    }

    let nav_path = output_dir.join("_nav.yaml");
    fs::write(&nav_path, nav_yaml(pages))?;
    println!("Generated: {}", nav_path.display());

    Ok(())
}

/// Write the published JSON Schema.
fn write_schema(repo_dir: &Path, schema: &Value) -> Result<()> {
    let output_dir = repo_dir.join("json");
    fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join(SCHEMA_FILENAME);
    let json = serde_json::to_string_pretty(schema)?;
    fs::write(&output_path, json)?;

    println!("Generated: {}", output_path.display());
    Ok(())
}

/// Build a simple generated navigation file.
fn nav_yaml(pages: &[DocPage]) -> String {
    let mut nav = String::from("# Auto-generated navigation\n\nitems:\n");

    for page in pages {
        nav.push_str("  - \"");
        nav.push_str(&page.slug);
        nav.push_str("\"\n");
    }

    nav
}

/// Frontmatter for a generated page.
fn frontmatter(title: &str, description: &str) -> String {
    format!(
        "---\ntitle: {}\ndescription: {}\n---\n\n",
        yaml_string(title),
        yaml_string(description)
    )
}

/// Footer linking generated docs back to their sources.
fn source_footer() -> String {
    format!(
        "\n---\n\nThis documentation was generated from [`schema.rs`]({GITHUB_BASE_URL}/rust/content-credentials/src/schema.rs) by [`generate.rs`]({GITHUB_BASE_URL}/rust/content-credentials/src/bin/generate.rs).\n"
    )
}

/// Get top-level definitions from a schema.
fn definitions(schema: &Value) -> Result<&Map<String, Value>> {
    schema
        .get("definitions")
        .and_then(Value::as_object)
        .ok_or_eyre("schema has no definitions")
}

/// Get object properties from a schema object.
fn properties(value: &Value) -> Result<&Map<String, Value>> {
    value
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_eyre("schema object has no properties")
}

/// Required field names for a schema object.
fn required_fields(value: &Value) -> BTreeSet<String> {
    value
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

/// A stable map of schema definition names to relative Markdown links.
fn type_links(definitions: &Map<String, Value>) -> BTreeMap<String, String> {
    definitions
        .keys()
        .map(|name| (name.to_string(), slug_for_type(name)))
        .collect()
}

/// Human-readable type string for a JSON Schema fragment.
fn type_string(value: &Value, type_links: &BTreeMap<String, String>) -> String {
    if let Some(ref_name) = ref_name(value) {
        return linked_type(&ref_name, type_links);
    }

    if let Some(items) = value.get("items") {
        let item_type = type_string(items, type_links);
        return format!("array<{item_type}>");
    }

    if let Some(types) = value.get("type").and_then(Value::as_array) {
        let types: Vec<String> = types
            .iter()
            .filter_map(Value::as_str)
            .filter(|name| *name != "null")
            .map(format_json_type)
            .collect();

        if !types.is_empty() {
            return types.join(" | ");
        }
    }

    if let Some(type_name) = value.get("type").and_then(Value::as_str) {
        if type_name == "array"
            && let Some(items) = value.get("items")
        {
            let item_type = type_string(items, type_links);
            return format!("{item_type}*");
        }

        if type_name == "object"
            && let Some(additional) = value.get("additionalProperties")
        {
            let item_type = type_string(additional, type_links);
            return format!("object map of {item_type}");
        }

        return format_json_type(type_name);
    }

    if let Some(any_of) = value.get("anyOf").and_then(Value::as_array) {
        let types: Vec<String> = any_of
            .iter()
            .filter(|item| !is_null_schema(item))
            .map(|item| type_string(item, type_links))
            .collect();

        if !types.is_empty() {
            return types.join(" | ");
        }
    }

    if let Some(all_of) = value.get("allOf").and_then(Value::as_array)
        && let Some(first) = all_of.first()
    {
        return type_string(first, type_links);
    }

    "any JSON value".to_string()
}

/// Format a JSON Schema primitive type.
fn format_json_type(type_name: &str) -> String {
    format!("`{type_name}`")
}

/// Link to a record page when the referenced type is documented.
fn linked_type(ref_name: &str, type_links: &BTreeMap<String, String>) -> String {
    if let Some(link) = type_links.get(ref_name) {
        format!("[`{ref_name}`]({link})")
    } else {
        format!("`{ref_name}`")
    }
}

/// Find a referenced definition name in a schema fragment.
fn ref_name(value: &Value) -> Option<String> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        return reference
            .strip_prefix("#/definitions/")
            .map(ToString::to_string);
    }

    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(items) = value.get(key).and_then(Value::as_array) {
            for item in items {
                if let Some(name) = ref_name(item) {
                    return Some(name);
                }
            }
        }
    }

    None
}

/// Whether a schema fragment admits JSON null.
fn is_nullable(value: &Value) -> bool {
    if is_null_schema(value) {
        return true;
    }

    if let Some(types) = value.get("type").and_then(Value::as_array)
        && types.iter().any(|item| item.as_str() == Some("null"))
    {
        return true;
    }

    for key in ["anyOf", "oneOf"] {
        if let Some(items) = value.get(key).and_then(Value::as_array)
            && items.iter().any(is_null_schema)
        {
            return true;
        }
    }

    false
}

/// Whether a schema fragment is exactly a null schema.
fn is_null_schema(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("null")
}

/// Render enum values if the schema constrains a field to fixed strings.
fn enum_values(value: &Value) -> Option<Vec<String>> {
    let values = value.get("enum").and_then(Value::as_array)?;
    let values: Vec<String> = values
        .iter()
        .filter(|item| item.as_str() != Some("null"))
        .map(|item| {
            if let Some(string) = item.as_str() {
                format!("`{string}`")
            } else {
                format!("`{item}`")
            }
        })
        .collect();

    (!values.is_empty()).then_some(values)
}

/// Extract a schema description.
fn get_description(value: &Value) -> String {
    value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

/// First paragraph of a description.
fn first_paragraph(text: &str) -> String {
    text.split("\n\n").next().unwrap_or(text).trim().to_string()
}

/// First sentence or paragraph for tables.
fn first_sentence(text: &str) -> String {
    let paragraph = first_paragraph(text);

    if let Some((sentence, _)) = paragraph.split_once(". ") {
        format!("{sentence}.")
    } else {
        paragraph
    }
}

/// Convert a Rust type name into a page title.
fn type_name_to_title(name: &str) -> String {
    let mut words = Vec::new();
    let mut word = String::new();

    for character in name.strip_suffix("Record").unwrap_or(name).chars() {
        if character.is_ascii_uppercase() && !word.is_empty() {
            words.push(title_word(&word));
            word = String::new();
        }
        word.push(character);
    }

    if !word.is_empty() {
        words.push(title_word(&word));
    }

    format!("{} Record", words.join(" "))
}

/// Normalize common Stencila and web acronyms in generated titles.
fn title_word(word: &str) -> String {
    match word {
        "Ai" => "AI".to_string(),
        "Io" => "IO".to_string(),
        "Id" => "ID".to_string(),
        word => word.to_string(),
    }
}

/// Slug for a schema definition.
fn slug_for_type(name: &str) -> String {
    slugify(&type_name_to_title(name))
}

/// Slugify a title, type, or field name for local links.
fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    let mut previous_was_lower_or_digit = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase()
                && previous_was_lower_or_digit
                && !slug.is_empty()
                && !previous_dash
            {
                slug.push('-');
            }
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
            previous_was_lower_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
            previous_was_lower_or_digit = false;
        }
    }

    slug.trim_matches('-').to_string()
}

/// Escape a value used as a YAML string.
fn yaml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Escape characters that would break a Markdown table.
fn escape_table_cell(value: &str) -> String {
    value.replace('\n', " ").replace('|', "\\|")
}
