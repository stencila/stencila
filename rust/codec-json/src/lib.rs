use codec::{
    common::{eyre::Result, serde_json},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use node_coerce::coerce;

/// A codec for JSON
pub struct JsonCodec {}

impl CodecTrait for JsonCodec {
    fn spec() -> Codec {
        Codec {
            status: "stable".to_string(),
            formats: vec_string!["json"],
            root_types: vec_string!["*"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        coerce(serde_json::from_str(str)?, None)
    }

    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let compact = options.map_or_else(|| false, |options| options.compact);
        let json = match compact {
            true => serde_json::to_string::<Node>(node)?,
            false => serde_json::to_string_pretty::<Node>(node)?,
        };
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::stencila_schema::{Number, Paragraph, Primitive};
    use std::collections::BTreeMap;
    use test_utils::assert_json_eq;

    #[test]
    fn from_str() {
        assert!(matches!(
            JsonCodec::from_str("true", None).unwrap(),
            Node::Boolean(true)
        ));

        assert!(matches!(
            JsonCodec::from_str("42", None).unwrap(),
            Node::Integer(42)
        ));

        #[allow(clippy::float_cmp)]
        if let Node::Number(num) = JsonCodec::from_str("1.23", None).unwrap() {
            assert_eq!(num, Number(1.23))
        }

        assert!(matches!(
            JsonCodec::from_str("[1, 2, 3]", None).unwrap(),
            Node::Array(..)
        ));

        assert!(matches!(
            JsonCodec::from_str("{}", None).unwrap(),
            Node::Object(..)
        ));

        assert!(matches!(
            JsonCodec::from_str("{\"type\": \"Entity\"}", None).unwrap(),
            Node::Entity(..)
        ));

        assert_json_eq!(
            JsonCodec::from_str("{\"type\": \"Paragraph\"}", None).unwrap(),
            Node::Paragraph(Paragraph {
                content: vec![],
                ..Default::default()
            })
        );
    }

    #[test]
    fn to_str() {
        assert_eq!(
            JsonCodec::to_string(&Node::Boolean(true), None).unwrap(),
            "true".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Integer(42), None).unwrap(),
            "42".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Number(Number(1.23)), None).unwrap(),
            "1.23".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Array(Vec::new()), None).unwrap(),
            "[]".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Object(BTreeMap::new()), None).unwrap(),
            "{}".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(
                &Node::Array(vec![Primitive::Integer(42)]),
                Some(EncodeOptions {
                    compact: false,
                    ..Default::default()
                })
            )
            .unwrap(),
            "[\n  42\n]".to_string()
        );
    }
}
