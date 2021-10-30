//! A codec for JSON

use codec_trait::{eyre::Result, stencila_schema::Node, Codec};

pub struct JsonCodec {}

impl Codec for JsonCodec {
    fn from_str(str: &str) -> Result<Node> {
        Ok(serde_json::from_str(str)?)
    }

    fn to_string(node: &Node) -> Result<String> {
        Ok(serde_json::to_string(node)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn from_str() {
        assert!(matches!(
            JsonCodec::from_str("true").unwrap(),
            Node::Boolean(true)
        ));

        assert!(matches!(
            JsonCodec::from_str("42").unwrap(),
            Node::Integer(42)
        ));

        #[allow(clippy::float_cmp)]
        if let Node::Number(num) = JsonCodec::from_str("1.23").unwrap() {
            assert_eq!(num, 1.23_f64)
        }

        assert!(matches!(
            JsonCodec::from_str("[1, 2, 3]").unwrap(),
            Node::Array(..)
        ));
    }

    #[test]
    fn to_str() {
        assert_eq!(
            JsonCodec::to_string(&Node::Boolean(true)).unwrap(),
            "true".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Integer(42)).unwrap(),
            "42".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Number(1.23)).unwrap(),
            "1.23".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Array(Vec::new())).unwrap(),
            "[]".to_string()
        );

        assert_eq!(
            JsonCodec::to_string(&Node::Object(BTreeMap::new())).unwrap(),
            "{}".to_string()
        );
    }
}
