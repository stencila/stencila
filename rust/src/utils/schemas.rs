//! Functions for consistently generating JSON Schemata from
//! internal Rust `struct`s.
//!
//! Not to be confused with the `stencila-schema` crate which
//! provides Rust `struct`s generated from Stencila's JSON Schema ;)

use eyre::Result;
use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::{Schema, SchemaObject},
    JsonSchema, Map,
};
use serde_json::{json, Value as JsonValue};

/// Create a `schemars` JSON Schema generator
///
/// Having a shared generator allow for consistent settings
/// for how JSON Schemas are produced across modules.
pub fn generator() -> SchemaGenerator {
    let settings = SchemaSettings::draft2019_09().with(|settings| {
        settings.option_add_null_type = false;
        settings.inline_subschemas = true;
    });
    settings.into_generator()
}

/// Generate the JSON Schema for a property with a specified TypeScript type.
pub fn typescript(typescript_type: &str, required: bool) -> Schema {
    let mut extensions = Map::new();
    extensions.insert("tsType".to_string(), json!(typescript_type));
    extensions.insert("isRequired".to_string(), json!(required));
    Schema::Object(SchemaObject {
        extensions,
        ..Default::default()
    })
}

// Modify `$id`, `title` and `description` for compatibility with TypeScript
// and UI form generation. Also apply the `isRequired` override.
// See https://github.com/stencila/stencila/pull/929#issuecomment-842623228
fn transform(value: JsonValue) -> JsonValue {
    if let JsonValue::Object(object) = value {
        let mut modified = serde_json::Map::<String, JsonValue>::new();

        // Copy over modified child properties
        for (key, child) in &object {
            modified.insert(key.clone(), transform(child.clone()));
        }

        // For `type:object` schemas, including sub-schemas..
        if let Some(value) = object.get("type") {
            if value == &serde_json::to_value("object").unwrap() {
                // Put any `title` into `$id`
                if let Some(title) = object.get("title") {
                    modified.insert("$id".into(), title.clone());
                }
                // Parse any `description` and if multi-line, put
                // the first "paragraph" into the `title`
                if let Some(JsonValue::String(description)) = object.get("description") {
                    let paras = description.split("\n\n").collect::<Vec<&str>>();
                    if paras.len() > 1 {
                        modified.insert("title".into(), JsonValue::String(paras[0].into()));
                        modified.insert(
                            "description".into(),
                            JsonValue::String(paras[1..].join("\n\n")),
                        );
                    }
                }
                // Check if any properties declare themselves `isRequired`
                if let Some(JsonValue::Object(properties)) = object.get("properties") {
                    for (name, subschema) in properties {
                        if let Some(JsonValue::Bool(is_required)) = subschema.get("isRequired") {
                            let name = JsonValue::String(name.clone());
                            if let Some(JsonValue::Array(required)) = modified.get_mut("required") {
                                if *is_required {
                                    required.push(name)
                                } else {
                                    required.retain(|prop| *prop != name)
                                }
                            } else if *is_required {
                                modified.insert("required".into(), JsonValue::Array(vec![name]));
                            }
                        }
                    }
                }
            }
        }
        JsonValue::Object(modified)
    } else {
        value
    }
}

/// Generate a JSON Schema for a type using the generator
pub fn generate<Type>() -> Result<JsonValue>
where
    Type: JsonSchema,
{
    let schema = generator().into_root_schema_for::<Type>();
    let schema = serde_json::to_value(schema)?;
    let schema = transform(schema);
    Ok(schema)
}
