use codec::{
    common::{eyre::Result, serde_json},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use node_coerce::coerce;

/// A codec for JSON5
pub struct Json5Codec {}

impl CodecTrait for Json5Codec {
    fn spec() -> Codec {
        Codec {
            status: "stable".to_string(),
            formats: vec_string!["json5"],
            root_types: vec_string!["*"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        coerce(json5::from_str(str)?, None)
    }

    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        // At the time of writing, the `json5` crate actually produces plain
        // old JSON, and does not offer pretty printing (so we use `serde_json` for that).
        let compact = options.map_or_else(|| true, |options| options.compact);
        let json = match compact {
            true => json5::to_string::<Node>(node)?,
            false => serde_json::to_string_pretty::<Node>(node)?,
        };
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::stencila_schema::{Entity, Paragraph};
    use test_utils::assert_json_eq;

    #[test]
    fn from_str() {
        assert!(matches!(
            Json5Codec::from_str("{type: 'Entity'}", None).unwrap(),
            Node::Entity(..)
        ));

        assert_json_eq!(
            Json5Codec::from_str("{type: 'Paragraph'}", None).unwrap(),
            Node::Paragraph(Paragraph {
                content: vec![],
                ..Default::default()
            })
        );
    }

    #[test]
    fn to_str() {
        assert_eq!(
            Json5Codec::to_string(&Node::Entity(Entity::default()), None).unwrap(),
            "{\"type\":\"Entity\"}".to_string()
        );
        assert_eq!(
            Json5Codec::to_string(
                &Node::Paragraph(Paragraph::default()),
                Some(EncodeOptions {
                    compact: false,
                    ..Default::default()
                })
            )
            .unwrap(),
            "{\n  \"type\": \"Paragraph\",\n  \"content\": []\n}".to_string()
        );
    }
}
