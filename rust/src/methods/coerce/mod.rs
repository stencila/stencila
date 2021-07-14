use super::decode::{date::decode_date_maybe, person::decode_person_maybe};
use eyre::{bail, Result};
use inflector::Inflector;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, sync::Mutex};
use stencila_schema::{self, Node, Object, Primitive};

/// Coerce a JSON value to the Stencila JSON Schema
///
/// This function is intended to be used prior to deserializing
/// generic data formats e.g JSON, YAML to to a `Node`.
/// For example, coercing an object to the schema avoids `serde` treating
/// it as the lowest common denominator type i.e. an `Entity` unless all of
/// its properties match the schema.
///
/// Examples of places where this might be necessary:
/// - when decoding JSON, YAML, etc documents
/// - when deserializing the result from delegating a method
///   to a peer or plugin
/// - when decoding the YAML header of a Markdown document
pub fn coerce(value: JsonValue) -> Result<Node> {
    if let JsonValue::Object(object) = &value {
        if object.contains_key("type") {
            if let JsonValue::String(type_) = &object["type"].clone() {
                let mut value = value;
                coerce_to_type(&mut value, type_);
                let node = serde_json::from_value(value)?;
                return Ok(node);
            }
        }
    }

    Ok(match coerce_to_primitive(value) {
        Primitive::Null => Node::Null,
        Primitive::Boolean(node) => Node::Boolean(node),
        Primitive::Integer(node) => Node::Integer(node),
        Primitive::Number(node) => Node::Number(node),
        Primitive::String(node) => Node::String(node),
        Primitive::Array(node) => Node::Array(node),
        Primitive::Object(node) => Node::Object(node),
    })
}

/// A JSON Schema object
///
/// Only implements properties of JSON Schema used in coercion or
/// validation in the Stencila Schema (e.g. exclude `patternProperties`)
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonSchema {
    #[serde(rename = "type")]
    type_: Option<String>,

    // Parsing to a type
    parser: Option<String>,

    // String schema
    // https://json-schema.org/understanding-json-schema/reference/string.html
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
    format: Option<String>,

    // Integer and number schema
    // https://json-schema.org/understanding-json-schema/reference/numeric.html
    minimum: Option<f64>,
    exclusive_minimum: Option<f64>,
    maximum: Option<f64>,
    exclusive_maximum: Option<f64>,

    // Object schema
    // https://json-schema.org/understanding-json-schema/reference/object.html
    title: Option<String>,
    properties: Option<HashMap<String, JsonSchema>>,
    property_aliases: Option<HashMap<String, String>>,
    required: Option<Vec<String>>,

    // Array schema
    // https://json-schema.org/understanding-json-schema/reference/array.html
    items: Option<Box<JsonSchema>>,
    min_items: Option<usize>,
    max_items: Option<usize>,

    // Default value, if missing
    default: Option<JsonValue>,

    // allOf / anyOf coercion
    any_of: Option<Vec<JsonSchema>>,
    all_of: Option<Vec<JsonSchema>>,

    // Reference to one of the other schemas
    #[serde(
        rename = "$ref",
        default,
        deserialize_with = "JsonSchema::deserialize_ref"
    )]
    ref_: Option<String>,
}

static SCHEMAS: Lazy<Mutex<HashMap<String, JsonSchema>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl JsonSchema {
    /// Custom de-serialization of the `$ref` property to leave only the type name
    fn deserialize_ref<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(Some(string.replace(".schema.json", "")))
    }

    /// Get a schema by id
    ///
    /// This lazily deserializes schemas as they are needed therby avoiding an expensive
    /// deserialization of all schemas as once.
    fn get(id: &str) -> Result<JsonSchema> {
        let mut schemas = SCHEMAS.lock().expect("Unable to lock");
        if let Some(schema) = schemas.get(id) {
            Ok(schema.clone())
        } else if let Some(schema) = stencila_schema::SCHEMAS.iter().find_map(|(name, schema)| {
            if *name == id {
                Some(
                    serde_json::from_str::<JsonSchema>(schema)
                        .expect("Unable to deserialize schema"),
                )
            } else {
                None
            }
        }) {
            schemas.insert(id.to_string(), schema.clone());
            Ok(schema)
        } else {
            bail!("Could not find schema for type '{}'", id)
        }
    }
}

/// Coerce a JSON value to a `Primitive` node.
fn coerce_to_primitive(value: JsonValue) -> Primitive {
    match value {
        JsonValue::Null => Primitive::Null,
        JsonValue::Bool(value) => Primitive::Boolean(value),
        JsonValue::Number(value) => {
            if value.is_i64() {
                Primitive::Integer(value.as_i64().unwrap())
            } else {
                Primitive::Number(value.as_f64().unwrap())
            }
        }
        JsonValue::String(value) => Primitive::String(value),
        JsonValue::Array(value) => {
            Primitive::Array(value.into_iter().map(coerce_to_primitive).collect())
        }
        JsonValue::Object(value) => Primitive::Object(
            value
                .into_iter()
                .map(|(key, value)| (key, coerce_to_primitive(value)))
                .collect::<Object>(),
        ),
    }
}

/// Coerce a JSON value to a JSON Schema
fn coerce_to_schema(value: &mut JsonValue, schema: &JsonSchema) {
    // Usually (always) `$ref` will be on it's own (as part of a
    // large `anyOf` or `allOf`) so for efficiency handle, and
    // return from that, first
    if let Some(schema) = &schema.ref_ {
        return coerce_to_type(value, &schema);
    }

    if let Some(type_) = &schema.type_ {
        // For these primitive schema types can just return early
        match type_.as_str() {
            "null" => {
                return coerce_to_null(value);
            }
            "boolean" => {
                return coerce_to_boolean(value);
            }
            "integer" => {
                return coerce_to_integer_schema(value, &schema);
            }
            "number" => {
                return coerce_to_number_schema(value, &schema);
            }
            "string" => {
                return coerce_to_string_schema(value, &schema);
            }
            _ => {}
        }
    }

    // Apply any parser first so that subsequent coercions can
    // also be applied (e.g. properties)
    if let Some(parser) = &schema.parser {
        if value.is_string() {
            coerce_using_parser(
                value,
                value.clone().as_str().expect("Should be a string"),
                parser,
            )
        }
    }

    // Assume that either `object` or `array` type, but not both
    if schema.properties.is_some() {
        coerce_to_object_schema(value, &schema);
    } else if schema.items.is_some() {
        coerce_to_array_schema(value, schema)
    }

    // Assume that either `allOf` or `anyOf` conditions, but not both
    if let Some(any_of) = schema.any_of.as_ref() {
        coerce_to_any_of(value, any_of)
    } else if let Some(all_of) = schema.all_of.as_ref() {
        coerce_to_all_of(value, all_of)
    }
}

/// Validate a JSON value against a JSON Schema
///
/// This is optimized for the Stencila Schema in that it only check
/// against one type of keyword per schema e.g. `properties` or `anyOf`, not both.
fn valid_for_schema(value: &JsonValue, schema: &JsonSchema) -> bool {
    let type_ = schema.type_.as_deref().unwrap_or_default();
    if type_ == "null" {
        matches!(value, JsonValue::Null)
    } else if type_ == "boolean" {
        matches!(value, JsonValue::Bool(_))
    } else if type_ == "integer" {
        valid_for_integer_schema(value, &schema)
    } else if type_ == "number" {
        valid_for_number_schema(value, &schema)
    } else if type_ == "string" {
        valid_for_string_schema(value, &schema)
    } else if schema.properties.is_some() {
        valid_for_object_schema(value, &schema)
    } else if schema.items.is_some() {
        valid_for_array_schema(value, schema)
    } else if let Some(schema) = &schema.ref_ {
        valid_for_type(value, &schema)
    } else if let Some(any_of) = schema.any_of.as_ref() {
        valid_for_any_of(value, any_of)
    } else if let Some(all_of) = schema.all_of.as_ref() {
        valid_for_all_of(value, all_of)
    } else {
        true
    }
}

/// Generate a default value for a JSON Schema
fn default_for_schema(schema: &JsonSchema) -> JsonValue {
    if let Some(default) = &schema.default {
        default.clone()
    } else {
        if let Some(type_) = &schema.type_ {
            return match type_.as_str() {
                "null" => JsonValue::Null,
                "boolean" => json!(false),
                "integer" => json!(0i64),
                "number" => json!(0f64),
                "string" => json!(""),
                "array" => json!([]),
                "object" => json!({}),
                _ => {
                    tracing::warn!("Unhandled JSON Schema type '{}'", type_);
                    JsonValue::Null
                }
            };
        }

        if let Some(any_of) = &schema.any_of {
            if let Some(schema) = any_of.first() {
                return default_for_schema(schema);
            }
        }

        if let Some(all_of) = &schema.all_of {
            if let Some(schema) = all_of.first() {
                return default_for_schema(schema);
            }
        }

        tracing::warn!("Unable to create default value for schema");
        JsonValue::Null
    }
}

/// Coerce a JSON value to a type (i.e. a schema with that name)
fn coerce_to_type(value: &mut JsonValue, type_: &str) {
    if let Ok(schema) = JsonSchema::get(type_) {
        coerce_to_schema(value, &schema);
    } else {
        tracing::warn!("Unable to find JSON Schema for type '{}'", type_);
        *value = JsonValue::Null;
    }
}

/// Validate a JSON value against a type (i.e. a schema with that name)
fn valid_for_type(value: &JsonValue, type_: &str) -> bool {
    if let Ok(schema) = JsonSchema::get(type_) {
        valid_for_schema(value, &schema)
    } else {
        tracing::warn!("Unable to find JSON Schema for type '{}'", type_);
        false
    }
}

/// Coerce a JSON string using a named parser
fn coerce_using_parser(value: &mut JsonValue, text: &str, parser: &str) {
    if let Some(parsed) = match parser {
        // Separated items
        "ssi" | "csi" | "scsi" => {
            // Space separated items
            static SSI_REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("\\s+").expect("Unable to create regex"));
            // Comma separated items
            static CSI_REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("\\s*(,|(and)|&)\\s*").expect("Unable to create regex"));
            // Semicolon separated items
            static SCSI_REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("\\s*(;|(and)|&)\\s*").expect("Unable to create regex"));

            let regex = match parser {
                "ssi" => &SSI_REGEX,
                "csi" => &CSI_REGEX,
                "scsi" => &SCSI_REGEX,
                _ => unreachable!(),
            };

            let items = regex
                .split(text)
                .map(|str| JsonValue::String(str.to_string()))
                .collect::<Vec<JsonValue>>();

            Some(JsonValue::Array(items))
        }

        "date" => decode_date_maybe(text)
            .map(|date| serde_json::to_value(date).expect("Should be able to convert to value")),

        "person" => decode_person_maybe(text).map(|person| {
            serde_json::to_value(person).expect("Should be able to convert to value")
        }),

        _ => None,
    } {
        *value = parsed;
    }
}

/// Coerce a JSON value to a `Null`
fn coerce_to_null(value: &mut JsonValue) {
    if !matches!(value, JsonValue::Null) {
        *value = JsonValue::Null
    }
}

/// Coerce a JSON value to a `Bool`
fn coerce_to_boolean(value: &mut JsonValue) {
    let boolean = match value {
        JsonValue::Null => false,
        JsonValue::Bool(_) => return,
        JsonValue::Number(number) => number.as_f64().expect("Should be a float") > 0f64,
        JsonValue::String(string) => !(string.to_lowercase() == "false" || string == "0"),
        JsonValue::Array(_) => true,
        JsonValue::Object(_) => true,
    };
    *value = JsonValue::Bool(boolean)
}

/// Coerce a JSON value to an integer schema
///
/// Respects `minimum` and `maximum` keywords only.
fn coerce_to_integer_schema(value: &mut JsonValue, schema: &JsonSchema) {
    let mut integer = match value {
        JsonValue::Null => 0,
        JsonValue::Bool(value) => match value {
            true => 1,
            false => 0,
        },
        JsonValue::Number(number) => match number.is_f64() {
            true => number.as_f64().expect("Should be a float") as i64,
            false => number.as_i64().expect("Should be an integer"),
        },
        JsonValue::String(string) => string.parse::<f64>().unwrap_or(0f64) as i64,
        JsonValue::Array(_) => 0,
        JsonValue::Object(_) => 0,
    };

    if let Some(minimum) = schema.minimum {
        if integer < minimum as i64 {
            integer = minimum as i64
        }
    }
    if let Some(maximum) = schema.maximum {
        if integer > maximum as i64 {
            integer = maximum as i64
        }
    }

    *value = json!(integer)
}

/// Validate a JSON value against an integer schema
///
/// Respects `minimum` and `maximum` keywords only.
fn valid_for_integer_schema(value: &JsonValue, schema: &JsonSchema) -> bool {
    let integer = match value {
        JsonValue::Number(number) => match number.is_f64() {
            true => {
                return false;
            }
            false => number.as_i64().expect("Should be an integer") as i64,
        },
        _ => return false,
    };

    if let Some(minimum) = schema.minimum {
        if integer < minimum as i64 {
            return false;
        }
    }

    if let Some(maximum) = schema.maximum {
        if integer > maximum as i64 {
            return false;
        }
    }
    true
}

/// Coerce a JSON value to a number schema
///
/// Respects `minimum` and `maximum` keywords only.
fn coerce_to_number_schema(value: &mut JsonValue, schema: &JsonSchema) {
    let mut float = match value {
        JsonValue::Null => 0.,
        JsonValue::Bool(value) => match value {
            true => 1.,
            false => 0.,
        },
        JsonValue::Number(number) => number.as_f64().expect("Should be a float") as f64,
        JsonValue::String(string) => string.parse::<f64>().unwrap_or(0f64),
        JsonValue::Array(_) => 0.,
        JsonValue::Object(_) => 0.,
    };

    if let Some(minimum) = schema.minimum {
        if float < minimum {
            float = minimum
        }
    }

    if let Some(maximum) = schema.maximum {
        if float > maximum {
            float = maximum
        }
    }

    *value = json!(float)
}

/// Validate a JSON value against a number schema
///
/// Respects `minimum` and `maximum` keywords only.
fn valid_for_number_schema(value: &JsonValue, schema: &JsonSchema) -> bool {
    let number = match value {
        JsonValue::Number(number) => match number.is_f64() {
            true => number.as_f64().expect("Should be a float") as f64,
            false => return false,
        },
        _ => return false,
    };

    if let Some(minimum) = schema.minimum {
        if number < minimum {
            return false;
        }
    }

    if let Some(maximum) = schema.maximum {
        if number > maximum {
            return false;
        }
    }

    true
}

/// Coerce a JSON value to a string schema
///
/// Respects `minLength` (via right padding) and `maxLength` keywords
/// (via truncation) but ignores `format` of `pattern` keywords.
fn coerce_to_string_schema(value: &mut JsonValue, schema: &JsonSchema) {
    match value {
        JsonValue::Null => *value = JsonValue::String("".to_string()),
        JsonValue::String(_) => (),
        _ => *value = JsonValue::String(value.to_string()),
    }

    // There is some code repetition below but avoid slowing down the
    // hot path (e.g. cloning string) in which neither these keywords apply

    if let Some(min_length) = schema.min_length {
        let string = value.as_str().expect("Should be a string");
        if string.len() < min_length {
            *value = JsonValue::String(format!("{:width$}", string, width = min_length))
        }
    }

    if let Some(max_length) = schema.max_length {
        let string = value.as_str().expect("Should be a string");
        if string.len() > max_length {
            *value = JsonValue::String(string[..max_length].to_string())
        }
    }
}

/// Validate a JSON value to a string schema
///
/// Respects `minLength` (via right padding) and `maxLength` keywords
/// (via truncation) but ignores `format` of `pattern` keywords.
fn valid_for_string_schema(value: &JsonValue, schema: &JsonSchema) -> bool {
    let string = match value {
        JsonValue::String(string) => string,
        _ => return false,
    };

    if let Some(min_length) = schema.min_length {
        if string.len() < min_length {
            return false;
        }
    }

    if let Some(max_length) = schema.max_length {
        if string.len() > max_length {
            return false;
        }
    }

    true
}

/// Coerce a JSON value to an object schema
fn coerce_to_object_schema(value: &mut JsonValue, schema: &JsonSchema) {
    // If the value is not an object, make it one
    if !matches!(value, JsonValue::Object(_)) {
        *value = json!({});
    }
    let object = value.as_object_mut().expect("Should always be an object");

    let properties = if let Some(properties) = &schema.properties {
        properties
    } else {
        return;
    };

    // Coerce each of the object's existing properties
    let mut rename: Vec<(String, String)> = Vec::new();
    let mut remove: Vec<String> = Vec::new();
    for (key, value) in object.iter_mut() {
        // Camel case the property name to match the convention used in
        // the schema.
        let mut name = key.to_camel_case();

        // Get the schema for this property, either directly, or as an alias
        let mut subschema = properties.get(&name);
        if subschema.is_none() {
            if let Some(aliases) = &schema.property_aliases {
                if let Some(aliased_name) = aliases.get(&name) {
                    name = aliased_name.clone();
                    subschema = properties.get(aliased_name)
                }
            }
        }
        if name != *key {
            rename.push((key.clone(), name))
        }

        if let Some(subschema) = subschema {
            // Coerce to the subschema
            coerce_to_schema(value, subschema);
        } else {
            // Add to list of properties to be removed
            if key != "type" {
                remove.push(key.clone());
            }
        }
    }

    // Rename and remove properties
    for (old, new) in rename {
        if let Some(value) = object.remove(&old) {
            object.insert(new, value);
        }
    }
    for key in remove {
        object.remove(&key);
    }

    // Add any properties in `required` that are not in object
    if let Some(required) = &schema.required {
        for key in required {
            if !object.contains_key(key) {
                if let Some(subschema) = properties.get(key) {
                    let default = default_for_schema(subschema);
                    object.insert(key.to_string(), default);
                }
            }
        }
    }

    if let Some(title) = &schema.title {
        object.insert("type".to_string(), JsonValue::String(title.clone()));
    }
}

/// Validate a JSON value against an object schema
fn valid_for_object_schema(value: &JsonValue, _schema: &JsonSchema) -> bool {
    if !matches!(value, JsonValue::Object(_)) {
        return false;
    }

    // TODO Check against `properties` and `required`

    true
}

/// Coerce a JSON value to an array schema
///
/// Wraps the value into an array if necessary and then attempts to
/// coerce those all items in the array.
fn coerce_to_array_schema(value: &mut JsonValue, schema: &JsonSchema) {
    // If the value is not an array, wrap the it into one
    if !matches!(value, JsonValue::Array(_)) {
        *value = json!([value.clone()]);
    }
    let array = value.as_array_mut().expect("Should always be an array");

    if let Some(items) = &schema.items {
        for item in array {
            coerce_to_schema(item, items)
        }
    }
}

/// Validate a JSON value against an array schema
fn valid_for_array_schema(value: &JsonValue, schema: &JsonSchema) -> bool {
    let array = match value {
        JsonValue::Array(array) => array,
        _ => {
            return false;
        }
    };

    if let Some(items) = &schema.items {
        for item in array {
            if !valid_for_schema(item, items) {
                return false;
            }
        }
    }

    true
}

/// Coerce a JSON value to any of a set of schemas
///
/// If the value is an object with a `type` property that matches one of the schemas,
/// then it will be coerced to that schema. Otherwise, the value will be coerced
/// to the first schema.
fn coerce_to_any_of(value: &mut JsonValue, schemas: &[JsonSchema]) {
    if let JsonValue::Object(object) = &value {
        if object.contains_key("type") {
            if let JsonValue::String(value_type) = &object["type"].clone() {
                for schema in schemas {
                    if let Some(ref_type) = &schema.ref_ {
                        if ref_type == value_type {
                            return coerce_to_type(value, value_type);
                        }
                    }
                }
            }
        }
    }

    if valid_for_any_of(value, schemas) {
        return;
    }

    if let Some(schema) = schemas.first() {
        coerce_to_schema(value, schema)
    }
}

/// Validate a JSON value against any of a set of schemas
fn valid_for_any_of(value: &JsonValue, schemas: &[JsonSchema]) -> bool {
    for schema in schemas {
        if valid_for_schema(value, schema) {
            return true;
        }
    }
    false
}

/// Coerce a JSON value to all of a set of schemas
fn coerce_to_all_of(value: &mut JsonValue, schemas: &[JsonSchema]) {
    for schema in schemas {
        coerce_to_schema(value, schema)
    }
}

/// Validate a JSON value against all of a set of schemas
fn valid_for_all_of(value: &JsonValue, schemas: &[JsonSchema]) -> bool {
    for schema in schemas {
        if !valid_for_schema(value, schema) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_coerce_to_null() {
        let mut value = JsonValue::Bool(true);
        coerce_to_null(&mut value);
        assert!(matches!(value, JsonValue::Null));
    }

    #[test]
    fn test_coerce_to_boolean() {
        let mut value = JsonValue::Null;
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(false)));

        value = json!(true);
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(true)));

        value = json!(-10);
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(false)));

        value = json!(0);
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(false)));

        value = json!(1);
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(true)));

        value = json!("false");
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(false)));

        value = json!("0");
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(false)));

        value = json!("whateva");
        coerce_to_boolean(&mut value);
        assert!(matches!(value, JsonValue::Bool(true)));
    }

    #[test]
    fn test_coerce_to_integer_schema() {
        let mut schema = JsonSchema {
            type_: Some("integer".to_string()),
            ..Default::default()
        };

        let mut value = JsonValue::Null;
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(0));

        value = json!(true);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(1));

        value = json!(false);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(0));

        value = json!(42);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(42));

        value = json!(3.14);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(3));

        value = json!("42");
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(42));

        value = json!("3.14");
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(3));

        value = json!("not a number");
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(0));

        schema.minimum = Some(1.);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(1));

        schema.maximum = Some(-1.);
        coerce_to_integer_schema(&mut value, &schema);
        assert_eq!(value, json!(-1));
    }

    #[test]
    fn test_coerce_to_number_schema() {
        let mut schema = JsonSchema {
            type_: Some("number".to_string()),
            ..Default::default()
        };

        let mut value = JsonValue::Null;
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(0.));

        value = json!(true);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(1.));

        value = json!(false);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(0.));

        value = json!(42);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(42.));

        value = json!(3.14);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(3.14));

        value = json!("42");
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(42.0));

        value = json!("3.14");
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(3.14));

        value = json!("not a number");
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(0.));

        schema.minimum = Some(1.);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(1.));

        schema.maximum = Some(-1.);
        coerce_to_number_schema(&mut value, &schema);
        assert_eq!(value, json!(-1.));
    }

    #[test]
    fn test_coerce_to_string_schema() {
        let mut schema = JsonSchema {
            type_: Some("string".to_string()),
            ..Default::default()
        };

        let mut value = JsonValue::Null;
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("", value);

        value = json!(false);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("false", value);

        value = json!(true);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("true", value);

        value = json!(42);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("42", value);

        value = json!(3.14);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("3.14", value);

        value = json!("foo");
        schema.min_length = Some(10);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("foo       ", value);

        schema.max_length = Some(2);
        coerce_to_string_schema(&mut value, &schema);
        assert_eq!("fo", value);
    }

    #[test]
    fn coerce_yaml_articles() {
        snapshot_content("articles/coerce-*.yaml", |content| {
            let value = serde_yaml::from_str(&content).expect("Unable to deserialize YAML");
            let node = coerce(value).expect("Unable to coerce");
            assert!(matches!(node, Node::Article(_)));
            assert_json_snapshot!(node);
        });
    }
}
