use indexmap::IndexMap;
use serde_json::Value;

use stencila_codec::stencila_schema::{Primitive, PropertyValue, PropertyValueOrString};

/// A types used for the `ids` property of all OpenAlex entity types
pub(crate) type Ids = IndexMap<String, Value>;

/// Get an id as a string
pub(crate) fn ids_get_maybe(ids: &Ids, name: &str) -> Option<String> {
    ids.get(name).map(|value| match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    })
}

/// Convert [`Ids`] to a Stencila `identifiers` property
pub(crate) fn ids_to_identifiers(ids: Ids) -> Option<Vec<PropertyValueOrString>> {
    if ids.is_empty() {
        return None;
    }

    Some(
        ids.into_iter()
            .map(|(key, value)| id_to_identifier(key, value))
            .collect(),
    )
}

/// Convert a single id to a Stencila `identifiers` property
pub(crate) fn id_to_identifiers(name: &str, value: String) -> Option<Vec<PropertyValueOrString>> {
    Some(vec![id_to_identifier(name.into(), Value::String(value))])
}

/// Convert a id name/value pair to a Stencila [PropertyValueOrString]
fn id_to_identifier(name: String, value: Value) -> PropertyValueOrString {
    let value: Primitive = match value {
        Value::String(value) => {
            if value.starts_with("http://") || value.starts_with("https://") {
                return PropertyValueOrString::String(value);
            } else {
                Primitive::String(value)
            }
        }
        value => serde_json::from_value(value).unwrap_or_default(),
    };

    PropertyValueOrString::PropertyValue(PropertyValue {
        property_id: Some(name),
        value,
        ..Default::default()
    })
}
