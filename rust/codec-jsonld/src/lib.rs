use std::collections::HashMap;

use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        once_cell::sync::Lazy,
        serde_json::{self, json, Map, Value},
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeOptions, EncodeOptions, Losses, Mapping,
};

/// A codec for JSON-LD
pub struct JsonLdCodec;

static CONTEXT_MAPS: Lazy<(HashMap<String, String>, HashMap<String, String>)> = Lazy::new(|| {
    let context: serde_json::Value =
        serde_json::from_str(include_str!("../../../json/context.jsonld"))
            .expect("Should be valid JSON");

    let graph = context["@graph"]
        .as_object()
        .expect("Should be an object")
        .to_owned();

    let decode = graph
        .iter()
        .map(|(key, value)| {
            (
                value.as_str().expect("Should be string").to_string(),
                key.clone(),
            )
        })
        .collect();

    let encode = graph
        .into_iter()
        .map(|(key, value)| (key, value.as_str().expect("Should be string").to_owned()))
        .collect();

    (decode, encode)
});

#[async_trait]
impl Codec for JsonLdCodec {
    fn name(&self) -> &str {
        "jsonld"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::JsonLd => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::JsonLd => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses, Mapping)> {
        let mut value: Value = serde_json::from_str(str)?;

        value.as_object_mut().map(|obj| obj.remove("@context"));

        let node = if let Some(object) = value.as_object() {
            serde_json::from_value(decode_object(object.to_owned()))?
        } else {
            serde_json::from_value(value)?
        };

        Ok((node, Losses::none(), Mapping::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, Losses, Mapping)> {
        let EncodeOptions { compact, .. } = options.unwrap_or_default();

        let value = serde_json::to_value(node)?;

        let Some(object) = value.as_object() else {
            // In the rare case the node does not encode to a JSON object, just return the JSON
            let json = serde_json::to_string_pretty(&value)?;
            return Ok((json, Losses::none(), Mapping::none()));
        };

        let mut encoded = json!({
            "@context": {
                "@vocab": "https://schema.org/",
                "stencila": "https://stencila.org/",
            },
        })
        .as_object()
        .expect("should be object")
        .to_owned();

        for (key, value) in encode_object(object.to_owned())
            .as_object()
            .expect("Should be an object")
        {
            encoded.insert(key.to_string(), value.to_owned());
        }

        let json = match compact {
            Some(true) => serde_json::to_string(&encoded),
            Some(false) | None => serde_json::to_string_pretty(&encoded),
        }?;

        Ok((json, Losses::none(), Mapping::none()))
    }
}

/// Decode a [`serde_json::Value`]
fn decode_value(old: Value) -> Value {
    match old {
        Value::Array(old) => decode_array(old),
        Value::Object(old) => decode_object(old),
        _ => old,
    }
}

/// Encode a [`serde_json::Value`]
fn encode_value(old: Value) -> Value {
    match old {
        Value::Array(old) => encode_array(old),
        Value::Object(old) => encode_object(old),
        _ => old,
    }
}

/// Recursively decode the items of an array
fn decode_array(old: Vec<Value>) -> Value {
    Value::Array(old.into_iter().map(decode_value).collect())
}

/// Recursively encode the items of an array
fn encode_array(old: Vec<Value>) -> Value {
    Value::Array(old.into_iter().map(encode_value).collect())
}

/// Recursively decode the keys and values of an object
fn decode_object(old: Map<String, Value>) -> Value {
    let mut new = serde_json::Map::new();
    for (key, value) in old.into_iter() {
        let (key, value) = if key == "@type" {
            let value = if let Some(id) = value.as_str() {
                let value = if let Some(id) = id.strip_prefix("stencila:") {
                    id.to_string()
                } else if let Some(id) = CONTEXT_MAPS.0.get(&["schema:", id].concat()) {
                    id.to_string()
                } else {
                    id.to_string()
                };
                Value::String(value)
            } else {
                decode_value(value)
            };
            ("type".to_string(), value)
        } else if key == "@id" {
            ("id".to_string(), decode_value(value))
        } else {
            let key = if let Some(id) = key.strip_prefix("stencila:") {
                id.to_string()
            } else if let Some(id) = CONTEXT_MAPS.0.get(&["schema:", &key].concat()) {
                id.to_string()
            } else {
                key
            };
            (key, decode_value(value))
        };

        new.insert(key, value);
    }

    Value::Object(new)
}

/// Recursively encode the keys and values of an object
fn encode_object(old: Map<String, Value>) -> Value {
    let mut new = serde_json::Map::new();
    for (key, value) in old.into_iter() {
        let (key, value) = if key == "type" {
            let value = if let Some(id) = value.as_str().and_then(|value| CONTEXT_MAPS.1.get(value))
            {
                let value = if let Some(id) = id.strip_prefix("schema:") {
                    id.to_string()
                } else {
                    id.to_string()
                };
                Value::String(value)
            } else {
                encode_value(value)
            };
            ("@type".to_string(), value)
        } else if key == "id" {
            ("@id".to_string(), encode_value(value))
        } else if let Some(id) = CONTEXT_MAPS.1.get(&key) {
            let key = if let Some(id) = id.strip_prefix("schema:") {
                id.to_string()
            } else {
                id.to_string()
            };
            (key, encode_value(value))
        } else {
            (key, encode_value(value))
        };

        new.insert(key, value);
    }

    Value::Object(new)
}
