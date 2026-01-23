//! Generate documentation for Stencila project configuration
//!
//! Creates:
//! 1. `json/stencila-config.schema.json` - JSON Schema with documentation URLs
//! 2. `site/docs/config/**/*.md` - Documentation pages
//! 3. `site/docs/config/**/_nav.yaml` - Navigation files

#![allow(clippy::print_stdout)]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value, json};
use stencila_config::Config;

/// Base URL for documentation
const DOCS_BASE_URL: &str = "https://stencila.io/docs/config";

/// Base URL for GitHub source links
const GITHUB_BASE_URL: &str = "https://github.com/stencila/stencila/blob/main";

/// A documentation page to be generated
#[derive(Debug, Clone)]
struct DocPage {
    /// File path relative to output directory (e.g., "site/layout/header.md")
    path: String,
    /// Page title
    title: String,
    /// Page description (first line)
    description: String,
    /// Full page content
    content: String,
    /// Whether this is an index page for a directory
    is_index: bool,
}

/// Classification of a schema property for doc generation
#[derive(Debug)]
enum PropertyType {
    /// Object with properties that has nested documentable children - gets a directory with index.md
    ObjectWithChildren { def_name: String },
    /// Object with properties but no nested documentable children - gets its own .md file
    ObjectStandalone { def_name: String },
    /// Map with additionalProperties pointing to simple type
    Map,
    /// Map with additionalProperties pointing to a tagged enum - directory with page per variant
    MapWithTaggedEnum {
        tag_field: String,
        variants: Vec<Value>,
    },
    /// Internally-tagged oneOf enum - directory with page per variant
    TaggedEnum {
        tag_field: String,
        variants: Vec<Value>,
    },
    /// Simple type - documented inline
    Simple,
}

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join("../../site/docs/config").canonicalize()?;
    fs::create_dir_all(&output_dir)?;

    // Generate JSON Schema
    let schema = generate_schema_base()?;
    let definitions = get_definitions(&schema)?;

    // Discover and generate documentation pages
    let pages = discover_pages(&schema, definitions)?;

    // Write all pages
    for page in &pages {
        write_page(&output_dir, page)?;
    }

    // Write navigation files
    write_nav_files(&output_dir, &pages)?;

    // Update index.md
    update_index_md(&output_dir, &schema, &pages)?;

    // Add documentation URLs to schema and write
    let schema_with_urls = add_doc_urls(schema, &pages)?;
    write_schema(&manifest_dir, &schema_with_urls)?;

    Ok(())
}

/// Generate the base JSON Schema
fn generate_schema_base() -> Result<Value> {
    let settings = SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<Config>();

    let mut schema_value = serde_json::to_value(schema)?;

    if let Some(obj) = schema_value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            json!("https://stencila.org/stencila-config.schema.json"),
        );
        obj.insert("title".to_string(), json!("Stencila Config"));
        obj.insert(
            "description".to_string(),
            json!("Configuration for Stencila workspaces"),
        );
    }

    Ok(schema_value)
}

/// Get the definitions object from the schema
fn get_definitions(schema: &Value) -> Result<&Map<String, Value>> {
    schema
        .get("definitions")
        .and_then(|d| d.as_object())
        .ok_or_else(|| eyre!("No definitions in schema"))
}

/// Discover all documentation pages from the schema
fn discover_pages(schema: &Value, definitions: &Map<String, Value>) -> Result<Vec<DocPage>> {
    let mut pages = Vec::new();

    let root_props = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .ok_or_else(|| eyre!("No root properties"))?;

    for (prop_name, prop) in root_props {
        discover_property_pages(prop_name, prop, "", definitions, &mut pages)?;
    }

    Ok(pages)
}

/// Recursively discover pages for a property
fn discover_property_pages(
    prop_name: &str,
    prop: &Value,
    parent_path: &str,
    definitions: &Map<String, Value>,
    pages: &mut Vec<DocPage>,
) -> Result<()> {
    let current_path = if parent_path.is_empty() {
        prop_name.to_string()
    } else {
        format!("{parent_path}/{prop_name}")
    };

    match classify_property(prop, definitions) {
        PropertyType::ObjectWithChildren { def_name } => {
            let def = definitions
                .get(&def_name)
                .ok_or_else(|| eyre!("Definition not found: {}", def_name))?;

            // Check if this object has child properties that need their own pages
            let child_props = def
                .get("properties")
                .and_then(|p| p.as_object())
                .cloned()
                .unwrap_or_default();

            let mut children_with_pages = Vec::new();
            for (child_name, child_prop) in &child_props {
                if should_have_own_page(child_prop, definitions) {
                    children_with_pages.push(child_name.clone());
                }
            }

            // Build type links for sibling pages (def_name -> relative path)
            let mut sibling_type_links: HashMap<String, String> = HashMap::new();
            for (child_name, child_prop) in &child_props {
                if should_have_own_page(child_prop, definitions)
                    && let Some((child_def_name, _)) =
                        resolve_to_definition(child_prop, definitions)
                {
                    // Link to sibling page: ./child-name.md or ./child-name/
                    let has_children = has_documentable_children(child_prop, definitions);
                    let link = if has_children {
                        format!("./{child_name}/")
                    } else {
                        format!("./{child_name}.md")
                    };
                    sibling_type_links.insert(child_def_name, link);
                }
            }

            // Generate content for this page
            let desc = get_description(def);
            let title = generate_page_title(&current_path, true);
            let content =
                generate_object_page_content(&title, def, definitions, &children_with_pages);

            pages.push(DocPage {
                path: format!("{current_path}/index.md"),
                title,
                description: first_line(&desc),
                content,
                is_index: true,
            });

            // Recurse into children that need pages, passing sibling type links
            for (child_name, child_prop) in &child_props {
                if should_have_own_page(child_prop, definitions) {
                    discover_property_pages_with_links(
                        child_name,
                        child_prop,
                        &current_path,
                        definitions,
                        pages,
                        &sibling_type_links,
                    )?;
                }
            }
        }

        PropertyType::ObjectStandalone { def_name } => {
            let def = definitions
                .get(&def_name)
                .ok_or_else(|| eyre!("Definition not found: {}", def_name))?;

            // Get property-level description (from field doc in parent type)
            let prop_desc = get_prop_description(prop);

            // Determine which properties to exclude based on page path
            // The `responsive` field only applies to sidebar regions
            let exclude_props: &[&str] = if is_non_sidebar_region(&current_path) {
                &["responsive"]
            } else {
                &[]
            };

            // Generate content for this standalone page (no sibling links in this context)
            let title = generate_page_title(&current_path, false);
            let content = generate_standalone_object_page_content(
                &title,
                &def_name,
                def,
                definitions,
                &HashMap::new(),
                Some(&prop_desc),
                exclude_props,
            );

            // Use property description for page description
            let desc = if prop_desc.is_empty() {
                get_description(def)
            } else {
                prop_desc
            };

            pages.push(DocPage {
                path: format!("{current_path}.md"),
                title,
                description: first_line(&desc),
                content,
                is_index: false,
            });
        }

        PropertyType::Map => {
            let desc = get_prop_description(prop);
            let title = generate_page_title(&current_path, false);
            let content = generate_map_page_content(&title, prop, definitions);

            pages.push(DocPage {
                path: format!("{current_path}.md"),
                title,
                description: first_line(&desc),
                content,
                is_index: false,
            });
        }

        PropertyType::MapWithTaggedEnum {
            tag_field,
            variants,
        } => {
            // Create index page for the map with a summary table of all variants
            let desc = get_prop_description(prop);
            let title = generate_page_title(&current_path, true);
            let content = generate_map_enum_index_content(&title, &variants, &tag_field, &desc);

            pages.push(DocPage {
                path: format!("{current_path}/index.md"),
                title,
                description: first_line(&desc),
                content,
                is_index: true,
            });

            // Create page for each variant
            for variant in &variants {
                if let Some(variant_name) = get_variant_name(variant, &tag_field) {
                    let variant_title = generate_variant_title(&current_path, &variant_name);
                    let variant_page =
                        generate_variant_page(&variant_title, variant, &tag_field, definitions)?;
                    let variant_path = format!("{current_path}/{variant_name}");
                    pages.push(DocPage {
                        path: format!("{variant_path}.md"),
                        title: variant_title,
                        description: variant_page.0,
                        content: variant_page.1,
                        is_index: false,
                    });
                }
            }
        }

        PropertyType::TaggedEnum {
            tag_field,
            variants,
        } => {
            // Create index page for the enum
            let desc = get_prop_description(prop);
            let title = generate_page_title(&current_path, true);
            let content = generate_enum_index_content(&title, prop_name, &variants, &tag_field);

            pages.push(DocPage {
                path: format!("{current_path}/index.md"),
                title,
                description: first_line(&desc),
                content,
                is_index: true,
            });

            // Create page for each variant
            for variant in &variants {
                if let Some(variant_name) = get_variant_name(variant, &tag_field) {
                    let variant_title = generate_variant_title(&current_path, &variant_name);
                    let variant_page =
                        generate_variant_page(&variant_title, variant, &tag_field, definitions)?;
                    let variant_path = format!("{current_path}/{variant_name}");
                    pages.push(DocPage {
                        path: format!("{variant_path}.md"),
                        title: variant_title,
                        description: variant_page.0,
                        content: variant_page.1,
                        is_index: false,
                    });
                }
            }
        }

        PropertyType::Simple => {
            // Simple properties are documented inline in their parent
        }
    }

    Ok(())
}

/// Recursively discover pages for a property, with sibling type links for cross-referencing
fn discover_property_pages_with_links(
    prop_name: &str,
    prop: &Value,
    parent_path: &str,
    definitions: &Map<String, Value>,
    pages: &mut Vec<DocPage>,
    sibling_type_links: &HashMap<String, String>,
) -> Result<()> {
    let current_path = if parent_path.is_empty() {
        prop_name.to_string()
    } else {
        format!("{parent_path}/{prop_name}")
    };

    // Only ObjectStandalone uses the sibling type links; other cases delegate to the regular function
    if let PropertyType::ObjectStandalone { def_name } = classify_property(prop, definitions) {
        let def = definitions
            .get(&def_name)
            .ok_or_else(|| eyre!("Definition not found: {}", def_name))?;

        // Get property-level description (from field doc in parent type)
        let prop_desc = get_prop_description(prop);

        // Determine which properties to exclude based on page path
        // The `responsive` field only applies to sidebar regions
        let exclude_props: &[&str] = if is_non_sidebar_region(&current_path) {
            &["responsive"]
        } else {
            &[]
        };

        // Generate content with sibling type links
        let title = generate_page_title(&current_path, false);
        let content = generate_standalone_object_page_content(
            &title,
            &def_name,
            def,
            definitions,
            sibling_type_links,
            Some(&prop_desc),
            exclude_props,
        );

        // Use property description for page description
        let desc = if prop_desc.is_empty() {
            get_description(def)
        } else {
            prop_desc
        };

        pages.push(DocPage {
            path: format!("{current_path}.md"),
            title,
            description: first_line(&desc),
            content,
            is_index: false,
        });
    } else {
        // Delegate to the regular function for other property types
        discover_property_pages(prop_name, prop, parent_path, definitions, pages)?;
    }

    Ok(())
}

/// Classify a property's schema type for page generation
fn classify_property(prop: &Value, definitions: &Map<String, Value>) -> PropertyType {
    // Check for map (additionalProperties) first, then check if value type is a tagged enum
    if let Some(additional) = prop.get("additionalProperties") {
        // Check if the value type is a tagged enum
        if let Some((_, value_def)) = resolve_to_definition(additional, definitions)
            && let Some((tag_field, variants)) = is_internally_tagged_enum(value_def, definitions)
        {
            return PropertyType::MapWithTaggedEnum {
                tag_field,
                variants,
            };
        }
        return PropertyType::Map;
    }

    // Resolve any $ref or anyOf
    let resolved = resolve_to_definition(prop, definitions);

    if let Some((def_name, def)) = resolved {
        // Check for internally-tagged oneOf enum
        if let Some((tag_field, variants)) = is_internally_tagged_enum(def, definitions) {
            return PropertyType::TaggedEnum {
                tag_field,
                variants,
            };
        }

        // Check for object with properties
        if def.get("properties").is_some() {
            // Check if this object has nested documentable children
            if has_documentable_children(prop, definitions) {
                return PropertyType::ObjectWithChildren { def_name };
            }
            // Object without nested documentable children - standalone page if significant
            if is_significant_object(def) {
                return PropertyType::ObjectStandalone { def_name };
            }
        }
    }

    PropertyType::Simple
}

/// Check if an object definition is "significant" enough to warrant its own page
///
/// Returns true if the object has at least 2 properties (not counting internal fields)
fn is_significant_object(def: &Value) -> bool {
    if let Some(props) = def.get("properties").and_then(|p| p.as_object()) {
        props.len() >= 2
    } else {
        false
    }
}

/// Check if a property should have its own documentation page
///
/// Returns true if the property:
/// - Has documentable children (tagged enums, etc.)
/// - OR references a significant object type
fn should_have_own_page(prop: &Value, definitions: &Map<String, Value>) -> bool {
    // Check for documentable children first
    if has_documentable_children(prop, definitions) {
        return true;
    }

    // Check if it references a significant object
    if let Some((_, def)) = resolve_to_definition(prop, definitions)
        && def.get("properties").is_some()
        && is_significant_object(def)
    {
        return true;
    }

    false
}

/// Check if a property has children that need their own documentation pages
///
/// Returns true if:
/// - The property is a tagged enum (each variant gets its own page)
/// - The property is a map with tagged enum values (each variant gets its own page)
/// - The property references an object type that has properties with documentable children
fn has_documentable_children(prop: &Value, definitions: &Map<String, Value>) -> bool {
    // Check for map with tagged enum value type
    if let Some(additional) = prop.get("additionalProperties")
        && let Some((_, value_def)) = resolve_to_definition(additional, definitions)
        && is_internally_tagged_enum(value_def, definitions).is_some()
    {
        return true;
    }

    // Check if this property references a definition
    if let Some((_, def)) = resolve_to_definition(prop, definitions) {
        // Check if it's a tagged enum directly
        if is_internally_tagged_enum(def, definitions).is_some() {
            return true;
        }

        // Check if it's an object with properties that have documentable children
        if let Some(props) = def.get("properties").and_then(|p| p.as_object()) {
            for (_, child_prop) in props {
                if has_documentable_children(child_prop, definitions) {
                    return true;
                }
            }
        }
    }

    false
}

/// Resolve a property to its definition name and value
///
/// Follows through $ref, anyOf wrappers, and allOf to find the underlying object type.
fn resolve_to_definition<'a>(
    prop: &'a Value,
    definitions: &'a Map<String, Value>,
) -> Option<(String, &'a Value)> {
    // Direct $ref
    if let Some(ref_name) = extract_ref_name(prop)
        && let Some(def) = definitions.get(&ref_name)
    {
        // If the definition is an anyOf, follow through to find object type
        if let Some(any_of) = def.get("anyOf").and_then(|a| a.as_array()) {
            for variant in any_of {
                // Skip null
                if variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                    continue;
                }
                // Skip primitives
                if variant.get("type").is_some() {
                    continue;
                }
                // Follow nested $ref to find object with properties
                if let Some(inner_ref_name) = extract_ref_name(variant)
                    && let Some(inner_def) = definitions.get(&inner_ref_name)
                    && inner_def.get("properties").is_some()
                {
                    return Some((inner_ref_name, inner_def));
                }
            }
        }
        return Some((ref_name, def));
    }

    // anyOf with null (optional type)
    if let Some(any_of) = prop.get("anyOf").and_then(|a| a.as_array()) {
        for variant in any_of {
            if variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                continue;
            }
            if let Some(ref_name) = extract_ref_name(variant)
                && let Some(def) = definitions.get(&ref_name)
            {
                // If the definition is an anyOf, follow through to find object type
                if let Some(inner_any_of) = def.get("anyOf").and_then(|a| a.as_array()) {
                    for inner_variant in inner_any_of {
                        if inner_variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                            continue;
                        }
                        if inner_variant.get("type").is_some() {
                            continue;
                        }
                        if let Some(inner_ref_name) = extract_ref_name(inner_variant)
                            && let Some(inner_def) = definitions.get(&inner_ref_name)
                            && inner_def.get("properties").is_some()
                        {
                            return Some((inner_ref_name, inner_def));
                        }
                    }
                }
                return Some((ref_name, def));
            }
        }
    }

    None
}

/// Check if a definition is an internally-tagged oneOf enum
fn is_internally_tagged_enum(
    def: &Value,
    _definitions: &Map<String, Value>,
) -> Option<(String, Vec<Value>)> {
    let one_of = def.get("oneOf")?.as_array()?;

    if one_of.is_empty() {
        return None;
    }

    // Find common discriminator field across variants
    let first = one_of.first()?;
    let props = first.get("properties")?.as_object()?;

    // Look for a property with const value (the tag field)
    for (field_name, field_def) in props {
        if field_def.get("const").is_some() {
            // Verify all variants have this field with const
            let all_tagged = one_of.iter().all(|v| {
                v.get("properties")
                    .and_then(|p| p.get(field_name))
                    .and_then(|f| f.get("const"))
                    .is_some()
            });
            if all_tagged {
                return Some((field_name.clone(), one_of.clone()));
            }
        }
    }

    None
}

/// Extract a type name from $ref
fn extract_ref_name(value: &Value) -> Option<String> {
    if let Some(ref_path) = value.get("$ref").and_then(|r| r.as_str()) {
        return Some(ref_path.trim_start_matches("#/definitions/").to_string());
    }

    if let Some(all_of) = value.get("allOf").and_then(|a| a.as_array()) {
        for item in all_of {
            if let Some(ref_path) = item.get("$ref").and_then(|r| r.as_str()) {
                return Some(ref_path.trim_start_matches("#/definitions/").to_string());
            }
        }
    }

    None
}

/// Get the variant name from a oneOf variant
fn get_variant_name(variant: &Value, tag_field: &str) -> Option<String> {
    variant
        .get("properties")?
        .get(tag_field)?
        .get("const")?
        .as_str()
        .map(String::from)
}

/// Get description from a definition
fn get_description(def: &Value) -> String {
    def.get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string()
}

/// Get description from a property
fn get_prop_description(prop: &Value) -> String {
    // Try direct description
    if let Some(desc) = prop.get("description").and_then(|d| d.as_str()) {
        return desc.to_string();
    }

    // Try anyOf variants
    if let Some(any_of) = prop.get("anyOf").and_then(|a| a.as_array()) {
        for variant in any_of {
            if variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                continue;
            }
            if let Some(desc) = variant.get("description").and_then(|d| d.as_str()) {
                return desc.to_string();
            }
        }
    }

    String::new()
}

/// Generate content for an object type page
fn generate_object_page_content(
    title: &str,
    def: &Value,
    definitions: &Map<String, Value>,
    children_with_pages: &[String],
) -> String {
    let desc = get_description(def);
    let first = first_line(&desc);

    let mut content = format!("---\ntitle: {title}\ndescription: {first}\n---\n\n{desc}\n\n");

    // Document all properties
    if let Some(props) = def.get("properties").and_then(|p| p.as_object()) {
        for (prop_name, prop) in props {
            // Properties with their own pages get linked type
            if children_with_pages.contains(prop_name) {
                content.push_str(&format_property_with_page_link(
                    prop_name,
                    prop,
                    definitions,
                ));
            } else {
                content.push_str(&format_property(prop_name, prop, definitions));
            }
        }
    }

    content
}

/// Format a property that has its own documentation page
fn format_property_with_page_link(
    prop_name: &str,
    prop: &Value,
    definitions: &Map<String, Value>,
) -> String {
    let desc = get_prop_description(prop);
    let first = first_line(&desc);

    // Get the type name and link
    let (type_name, link) = if has_documentable_children(prop, definitions) {
        // Has children - directory link
        if let Some((def_name, _)) = resolve_to_definition(prop, definitions) {
            (def_name, format!("{prop_name}/"))
        } else {
            (prop_name.to_string(), format!("{prop_name}/"))
        }
    } else {
        // Standalone page - .md link
        if let Some((def_name, _)) = resolve_to_definition(prop, definitions) {
            (def_name, format!("{prop_name}.md"))
        } else {
            (prop_name.to_string(), format!("{prop_name}.md"))
        }
    };

    let optional = is_optional(prop);
    let optional_str = if optional { " (optional)" } else { "" };

    let mut md = format!("# `{prop_name}`\n\n");
    md.push_str(&format!(
        "**Type:** [`{type_name}`]({link}){optional_str}\n\n"
    ));
    md.push_str(&first);
    md.push_str("\n\n");

    md
}

/// Generate content for a standalone object page (e.g., region pages)
///
/// If `prop_desc` is provided (field-level description from parent type),
/// it's used as the primary description. Otherwise falls back to type description.
///
/// `exclude_props` allows filtering out properties that don't apply to this specific page
/// (e.g., `responsive` only applies to sidebar regions, not header/footer/top/bottom).
fn generate_standalone_object_page_content(
    title: &str,
    def_name: &str,
    def: &Value,
    definitions: &Map<String, Value>,
    type_links: &HashMap<String, String>,
    prop_desc: Option<&str>,
    exclude_props: &[&str],
) -> String {
    // Use property-level description if available, otherwise fall back to type description
    let desc = prop_desc
        .filter(|d| !d.is_empty())
        .map(String::from)
        .unwrap_or_else(|| get_description(def));
    let first = first_line(&desc);

    let mut content = format!("---\ntitle: {title}\ndescription: {first}\n---\n\n{desc}\n\n");

    // Add type info
    content.push_str(&format!("**Type:** `{def_name}`\n\n"));

    // Document properties with type links for sibling pages
    if let Some(props) = def.get("properties").and_then(|p| p.as_object()) {
        for (name, prop) in props {
            // Skip excluded properties
            if exclude_props.contains(&name.as_str()) {
                continue;
            }
            content.push_str(&format_property_with_links(
                name,
                prop,
                definitions,
                type_links,
            ));
        }
    }

    content
}

/// Generate content for a map type page
fn generate_map_page_content(
    title: &str,
    prop: &Value,
    definitions: &Map<String, Value>,
) -> String {
    let desc = get_prop_description(prop);
    let first = first_line(&desc);

    let mut content = format!("---\ntitle: {title}\ndescription: {first}\n---\n\n{desc}\n\n");

    // Document the value type if it references a definition
    if let Some(additional) = prop.get("additionalProperties")
        && let Some(ref_name) = extract_ref_name(additional)
        && let Some(def) = definitions.get(&ref_name)
    {
        content.push_str(&format!("## {} Entry\n\n", type_name_to_title(&ref_name)));

        let def_desc = get_description(def);
        content.push_str(&def_desc);
        content.push_str("\n\n");

        // Document anyOf variants
        if let Some(any_of) = def.get("anyOf").and_then(|a| a.as_array()) {
            for variant in any_of {
                if variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                    continue;
                }
                let variant_desc = variant
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                if !variant_desc.is_empty() {
                    content.push_str(&format!("### {}\n\n", first_line(variant_desc)));
                    // Output description without the first line (already used as heading)
                    let rest = rest_of_lines(variant_desc);
                    if !rest.is_empty() {
                        content.push_str(&rest);
                        content.push_str("\n\n");
                    }
                }
            }
        }

        // Document properties if it's an object
        if let Some(props) = def.get("properties").and_then(|p| p.as_object()) {
            for (prop_name, prop) in props {
                content.push_str(&format_property(prop_name, prop, definitions));
            }
        }
    }

    content
}

/// Generate content for a tagged enum index page
fn generate_enum_index_content(
    title: &str,
    prop_name: &str,
    variants: &[Value],
    tag_field: &str,
) -> String {
    let mut content = format!(
        "---\ntitle: {title}\ndescription: Available {} types\n---\n\nAvailable {} types:\n\n",
        prop_name, prop_name
    );

    content.push_str("| Type | Description |\n|------|-------------|\n");

    for variant in variants {
        if let Some(name) = get_variant_name(variant, tag_field) {
            let desc = variant
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let first = first_line(desc);
            content.push_str(&format!("| [`{name}`]({name}.md) | {first} |\n"));
        }
    }

    content.push('\n');
    content
}

/// Generate content for a map with tagged enum values index page
fn generate_map_enum_index_content(
    title: &str,
    variants: &[Value],
    tag_field: &str,
    map_desc: &str,
) -> String {
    let first = first_line(map_desc);

    let mut content = format!(
        "---\ntitle: {title}\ndescription: {first}\n---\n\n{map_desc}\n\n## Available Types\n\n"
    );

    content.push_str("| Type | Description |\n|------|-------------|\n");

    for variant in variants {
        if let Some(name) = get_variant_name(variant, tag_field) {
            let desc = variant
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let first = first_line(desc);
            content.push_str(&format!("| [`{name}`]({name}.md) | {first} |\n"));
        }
    }

    content.push('\n');
    content
}

/// Generate a variant page for a tagged enum
fn generate_variant_page(
    title: &str,
    variant: &Value,
    tag_field: &str,
    definitions: &Map<String, Value>,
) -> Result<(String, String)> {
    let desc = variant
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let first = first_line(desc);

    let mut content = format!("---\ntitle: {title}\ndescription: {first}\n---\n\n{desc}\n\n");

    // Document properties (excluding the tag field)
    if let Some(props) = variant.get("properties").and_then(|p| p.as_object()) {
        let required = variant
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        for (prop_name, prop) in props {
            if prop_name == tag_field {
                continue;
            }

            let prop_desc = prop
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            let type_str = get_type_string(prop);
            let optional_str = if required.contains(&prop_name.as_str()) {
                ""
            } else {
                " (optional)"
            };

            content.push_str(&format!("## `{prop_name}`\n\n"));
            content.push_str(&format!("**Type:** {type_str}{optional_str}\n\n"));
            content.push_str(prop_desc);
            content.push_str("\n\n");

            // If it references a simple enum, inline it as a table
            if let Some(enum_def) = get_simple_enum_from_prop(prop, definitions) {
                content.push_str(&format_simple_enum(&enum_def));
            }
        }
    }

    Ok((first.to_string(), content))
}

/// Format a property for documentation
fn format_property(prop_name: &str, prop: &Value, definitions: &Map<String, Value>) -> String {
    format_property_with_links(prop_name, prop, definitions, &HashMap::new())
}

/// Format a property for documentation with type links
fn format_property_with_links(
    prop_name: &str,
    prop: &Value,
    definitions: &Map<String, Value>,
    type_links: &HashMap<String, String>,
) -> String {
    let desc = prop
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let type_str = get_type_string_with_links(prop, type_links);
    let optional = is_optional(prop);
    let optional_str = if optional { " (optional)" } else { "" };

    let mut md = format!("# `{prop_name}`\n\n");
    md.push_str(&format!("**Type:** {type_str}{optional_str}\n\n"));

    if let Some(pattern) = prop.get("pattern").and_then(|p| p.as_str()) {
        md.push_str(&format!("**Pattern:** `{pattern}`\n\n"));
    }

    md.push_str(desc);
    md.push_str("\n\n");

    // Inline simple enum as table
    if let Some(enum_def) = get_simple_enum_from_prop(prop, definitions) {
        md.push_str(&format_simple_enum(&enum_def));
    }

    md
}

/// Get a human-readable type string
fn get_type_string(prop: &Value) -> String {
    get_type_string_with_links(prop, &HashMap::new())
}

/// Get a human-readable type string with optional links to type pages
fn get_type_string_with_links(prop: &Value, type_links: &HashMap<String, String>) -> String {
    // Helper to format a type name, adding link if available
    let format_type = |name: &str| -> String {
        if let Some(link) = type_links.get(name) {
            format!("[`{name}`]({link})")
        } else {
            format!("`{name}`")
        }
    };

    // Direct type
    if let Some(t) = prop.get("type") {
        if let Some(s) = t.as_str() {
            return format!("`{s}`");
        }
        if let Some(arr) = t.as_array() {
            let types: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|s| *s != "null")
                .map(|s| format!("`{s}`"))
                .collect();
            if !types.is_empty() {
                return types.join(" | ");
            }
        }
    }

    // $ref
    if let Some(ref_name) = extract_ref_name(prop) {
        return format_type(&ref_name);
    }

    // anyOf
    if let Some(any_of) = prop.get("anyOf").and_then(|a| a.as_array()) {
        let types: Vec<String> = any_of
            .iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
            .map(|v| {
                if let Some(ref_name) = extract_ref_name(v) {
                    format_type(&ref_name)
                } else if let Some(t) = v.get("type").and_then(|t| t.as_str()) {
                    format!("`{t}`")
                } else {
                    "`unknown`".to_string()
                }
            })
            .collect();
        if !types.is_empty() {
            return types.join(" | ");
        }
    }

    "`unknown`".to_string()
}

/// Check if a property is optional (nullable)
fn is_optional(prop: &Value) -> bool {
    // Type array with null
    if let Some(types) = prop.get("type").and_then(|t| t.as_array())
        && types.iter().any(|t| t.as_str() == Some("null"))
    {
        return true;
    }

    // anyOf with null
    if let Some(any_of) = prop.get("anyOf").and_then(|a| a.as_array())
        && any_of
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("null"))
    {
        return true;
    }

    false
}

/// Get simple enum from a property, resolving references
fn get_simple_enum_from_prop(prop: &Value, definitions: &Map<String, Value>) -> Option<Value> {
    // Direct check
    if let Some(one_of) = prop.get("oneOf").and_then(|o| o.as_array())
        && one_of.iter().all(|v| v.get("const").is_some())
    {
        return Some(prop.clone());
    }

    // Check referenced type
    if let Some(ref_name) = extract_ref_name(prop)
        && let Some(def) = definitions.get(&ref_name)
        && let Some(one_of) = def.get("oneOf").and_then(|o| o.as_array())
        && one_of.iter().all(|v| v.get("const").is_some())
    {
        return Some(def.clone());
    }

    // Check anyOf variants
    if let Some(any_of) = prop.get("anyOf").and_then(|a| a.as_array()) {
        for variant in any_of {
            if variant.get("type").and_then(|t| t.as_str()) == Some("null") {
                continue;
            }
            if let Some(ref_name) = extract_ref_name(variant)
                && let Some(def) = definitions.get(&ref_name)
                && let Some(one_of) = def.get("oneOf").and_then(|o| o.as_array())
                && one_of.iter().all(|v| v.get("const").is_some())
            {
                return Some(def.clone());
            }
        }
    }

    None
}

/// Format a simple enum as a markdown table
fn format_simple_enum(def: &Value) -> String {
    let Some(one_of) = def.get("oneOf").and_then(|o| o.as_array()) else {
        return String::new();
    };

    let mut md = "| Value | Description |\n|-------|-------------|\n".to_string();

    for variant in one_of {
        let value = variant
            .get("const")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        let desc = variant
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let first = first_line(desc);
        md.push_str(&format!("| `{value}` | {first} |\n"));
    }

    md.push('\n');
    md
}

/// Generate a page title from the path with ancestors and appropriate suffix
///
/// Examples:
/// - "workspace" → "Workspace Config"
/// - "site/layout" → "Site Layout Config"
/// - "site/layout/header" → "Site Layout Header Config"
/// - "site/layout/components" → "Site Layout Components"
/// - "site/layout/components/nav-tree" → "Site Layout NavTree Component"
fn generate_page_title(path: &str, _is_index: bool) -> String {
    let segments: Vec<&str> = path.split('/').collect();

    // Check if this is a component page (under */components/*)
    let is_component =
        segments.contains(&"components") && segments.last().is_some_and(|s| *s != "components");

    // Check if this is the components index
    let is_components_index = segments.last() == Some(&"components");

    // Convert path segments to title parts, excluding "components" for component pages
    let title_parts: Vec<String> = segments
        .iter()
        .filter(|s| !is_component || **s != "components")
        .map(|s| kebab_to_title_spaced(s))
        .collect();

    let base = title_parts.join(" ");

    if is_components_index {
        base
    } else if is_component {
        format!("{base} Component")
    } else {
        format!("{base} Config")
    }
}

/// Generate a variant page title (for components)
fn generate_variant_title(path: &str, variant_name: &str) -> String {
    // Get the parent path (without /components at the end)
    let parent_segments: Vec<&str> = path.split('/').filter(|s| *s != "components").collect();

    let parent_title: Vec<String> = parent_segments
        .iter()
        .map(|s| kebab_to_title_spaced(s))
        .collect();

    let variant_title = kebab_to_title(variant_name);

    format!("{} {} Component", parent_title.join(" "), variant_title)
}

/// Convert type name to page title (for content generation)
fn type_name_to_title(name: &str) -> String {
    let base = name.trim_end_matches("Config").trim_end_matches("Spec");
    format!("{base} Configuration")
}

/// Convert kebab-case to title case with spaces (e.g., "nav-tree" → "Nav Tree")
fn kebab_to_title_spaced(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert kebab-case to PascalCase (e.g., "nav-tree" → "NavTree")
fn kebab_to_title(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Get the first line of a string
fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

/// Get all lines after the first, trimmed of leading whitespace
fn rest_of_lines(s: &str) -> String {
    let mut lines = s.lines();
    lines.next(); // Skip first line
    lines
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start()
        .to_string()
}

/// Get the primary Rust source file for a documentation page path
fn get_source_file(page_path: &str) -> &'static str {
    if page_path.starts_with("workspace") {
        "workspace.rs"
    } else if page_path.starts_with("remotes") {
        "remotes.rs"
    } else if page_path.starts_with("outputs") {
        "outputs.rs"
    } else if page_path.starts_with("site/reviews") {
        "reviews.rs"
    } else if page_path.starts_with("site/layout/components") {
        "layout/components.rs"
    } else if page_path.starts_with("site/layout/header")
        || page_path.starts_with("site/layout/left-sidebar")
        || page_path.starts_with("site/layout/right-sidebar")
        || page_path.starts_with("site/layout/top")
        || page_path.starts_with("site/layout/bottom")
        || page_path.starts_with("site/layout/footer")
    {
        "layout/regions.rs"
    } else if page_path.starts_with("site/layout") {
        "layout/config.rs"
    } else if page_path.starts_with("site") {
        "site.rs"
    } else {
        "lib.rs"
    }
}

/// Check if a page path is a non-sidebar region (header, top, bottom, footer)
///
/// These regions don't support the `responsive` field, which only applies to sidebars.
fn is_non_sidebar_region(page_path: &str) -> bool {
    page_path.starts_with("site/layout/header")
        || page_path.starts_with("site/layout/top")
        || page_path.starts_with("site/layout/bottom")
        || page_path.starts_with("site/layout/footer")
}

/// Generate the source footer for a documentation page
fn generate_source_footer(page_path: &str) -> String {
    let source_file = get_source_file(page_path);
    format!(
        "\n***\n\nThis documentation was generated from [`{source_file}`]({GITHUB_BASE_URL}/rust/config/src/{source_file}) by [`generate.rs`]({GITHUB_BASE_URL}/rust/config/src/bin/generate.rs).\n"
    )
}

/// Write a documentation page to disk
fn write_page(output_dir: &Path, page: &DocPage) -> Result<()> {
    let path = output_dir.join(&page.path);

    // Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Append source footer to content
    let content_with_footer = format!("{}{}", page.content, generate_source_footer(&page.path));

    fs::write(&path, content_with_footer)?;
    println!("Generated: {}", path.display());

    Ok(())
}

/// Write navigation files for all directories
fn write_nav_files(output_dir: &Path, pages: &[DocPage]) -> Result<()> {
    // Group pages by directory
    let mut dirs: HashMap<String, Vec<String>> = HashMap::new();

    for page in pages {
        let dir = if page.is_index {
            // For index pages, the dir is the path without /index.md
            page.path.trim_end_matches("/index.md").to_string()
        } else {
            // For regular pages, the dir is the parent directory
            Path::new(&page.path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        let filename = if page.is_index {
            // Index pages are represented as directories in nav
            Path::new(&page.path)
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
        } else {
            Path::new(&page.path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        };

        if let Some(filename) = filename {
            // Get the parent of dir for index files
            let nav_dir = if page.is_index {
                Path::new(&dir)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else {
                dir.clone()
            };

            dirs.entry(nav_dir).or_default().push(filename);
        }
    }

    // Write nav files
    for (dir, items) in &dirs {
        let dir_path = if dir.is_empty() {
            output_dir.to_path_buf()
        } else {
            output_dir.join(dir)
        };

        // Deduplicate while preserving definition order
        let mut seen = std::collections::HashSet::new();
        let unique_items: Vec<_> = items
            .iter()
            .filter(|item| seen.insert(item.as_str()))
            .collect();

        let mut content = "# Auto-generated navigation\n\nitems:\n".to_string();
        for item in &unique_items {
            content.push_str(&format!("  - \"{item}\"\n"));
        }

        fs::create_dir_all(&dir_path)?;
        let nav_path = dir_path.join("_nav.yaml");
        fs::write(&nav_path, content)?;
        println!("Generated: {}", nav_path.display());
    }

    Ok(())
}

/// Update the index.md file
fn update_index_md(output_dir: &Path, _schema: &Value, pages: &[DocPage]) -> Result<()> {
    let path = output_dir.join("index.md");

    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut content = if let Some(pos) = existing.find("## Sections") {
        let end = pos + "## Sections".len();
        existing[..end].to_string()
    } else {
        existing.clone()
    };

    content.push_str("\n\n| Section | Description |\n|---------|-------------|\n");

    // Get root-level pages
    for page in pages {
        if !page.path.contains('/') || (page.is_index && page.path.matches('/').count() == 1) {
            let name = Path::new(&page.path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let link_name = if page.is_index {
                page.path.trim_end_matches("/index.md").to_string()
            } else {
                name.clone()
            };

            content.push_str(&format!(
                "| [`[{}]`]({}) | {} |\n",
                link_name, link_name, page.description
            ));
        }
    }

    fs::write(&path, content)?;
    println!("Updated: {}", path.display());

    Ok(())
}

/// Add documentation URLs to schema definitions
fn add_doc_urls(mut schema: Value, pages: &[DocPage]) -> Result<Value> {
    // Build map of definition name to doc path
    let mut type_to_path: HashMap<String, String> = HashMap::new();

    for page in pages {
        // Extract type name from path
        let page_path = page.path.trim_end_matches(".md").trim_end_matches("/index");
        type_to_path.insert(page.title.clone(), page_path.to_string());
    }

    if let Some(obj) = schema.as_object_mut() {
        // Add URLs to root properties
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for (prop_name, prop) in properties {
                add_doc_url_to_prop(prop, prop_name);
            }
        }

        // Add URLs to definitions
        if let Some(definitions) = obj.get_mut("definitions").and_then(|d| d.as_object_mut()) {
            for (def_name, def) in definitions {
                let doc_path = type_to_path
                    .get(&type_name_to_title(def_name))
                    .cloned()
                    .unwrap_or_else(|| def_name.to_lowercase());

                if let Some(desc) = def.get("description").and_then(|d| d.as_str()) {
                    let first = first_line(desc);
                    let url = format!("{DOCS_BASE_URL}/{doc_path}");
                    if let Some(o) = def.as_object_mut() {
                        o.insert("description".to_string(), json!(format!("{first}\n{url}")));
                    }
                }
            }
        }
    }

    Ok(schema)
}

/// Add doc URL to a property
fn add_doc_url_to_prop(prop: &mut Value, prop_name: &str) {
    if let Some(obj) = prop.as_object_mut()
        && let Some(desc) = obj.get("description").and_then(|d| d.as_str())
    {
        let first = first_line(desc);
        let url = format!("{DOCS_BASE_URL}/{prop_name}");
        obj.insert("description".to_string(), json!(format!("{first}\n{url}")));
    }
}

/// Write the JSON Schema to file
fn write_schema(manifest_dir: &Path, schema: &Value) -> Result<()> {
    let output_dir = manifest_dir.join("../../json");
    fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join("stencila-config.schema.json");
    let json = serde_json::to_string_pretty(schema)?;
    fs::write(&output_path, json)?;

    println!("Generated: {}", output_path.display());
    Ok(())
}
