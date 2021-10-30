//! A codec for plain text
//!
//! This codec is intentionally lossy but is useful for when a
//! plain text representation of a node is needed.

use codec_trait::{
    eyre::Result,
    stencila_schema::{Node, Null},
    Codec, EncodeOptions,
};

mod to_txt;
use node_coerce::coerce;
pub use to_txt::ToTxt;

pub struct TxtCodec {}

impl Codec for TxtCodec {
    /// Decode plain text to a `Node`
    ///
    /// Attempts to decode as a JSON5 string first, falling back
    /// to unquoting a string, falling back to just returning a
    /// string node.
    ///
    /// Uses JSON5 over JSON, as a more permissive standard (e.g. strings can
    /// be single quoted) and thus able to deal with representations of values
    /// that are not strictly JSON (e.g. a Python dict repr).
    ///
    /// This function is useful in contexts where some text may or may not
    /// represent a boolean, number or some other JSON object, for example
    /// the output of a Jupyter code cell where a `dict` has a representation
    /// which can be parsed as a JSON object.
    fn from_str(str: &str) -> Result<Node> {
        match str.trim() {
            "true" | "TRUE" | "True" => return Ok(Node::Boolean(true)),
            "false" | "FALSE" | "False" => return Ok(Node::Boolean(false)),
            "null" | "NULL" | "Null" => return Ok(Node::Null(Null {})),
            _ => (),
        };

        if let Ok(node) = json5::from_str(str) {
            return coerce(node, None);
        }

        Ok(Node::String(str.to_string()))
    }

    /// Encode a `Node` to plain text
    ///
    /// The only structure added is placing two newlines after each `BlockContent`
    /// node. e.g. paragraphs, code blocks
    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        Ok(node.to_txt().trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use codec_trait::{eyre::bail, stencila_schema::Primitive};
    use test_utils::assert_debug_eq;

    #[test]
    fn booleans() -> Result<()> {
        assert!(matches!(TxtCodec::from_str("true")?, Node::Boolean(true)));
        assert!(matches!(TxtCodec::from_str("TRUE")?, Node::Boolean(true)));
        assert!(matches!(
            TxtCodec::from_str("  True  ")?,
            Node::Boolean(true)
        ));
        Ok(())
    }

    #[test]
    fn nulls() -> Result<()> {
        assert!(matches!(TxtCodec::from_str("null")?, Node::Null(Null {})));
        assert!(matches!(TxtCodec::from_str("NULL")?, Node::Null(Null {})));
        assert!(matches!(
            TxtCodec::from_str("  Null  ")?,
            Node::Null(Null {})
        ));
        Ok(())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn numbers() -> Result<()> {
        assert!(matches!(TxtCodec::from_str("1")?, Node::Integer(1)));
        match TxtCodec::from_str("1.23")? {
            Node::Number(value) => assert_eq!(value, 1.23),
            _ => bail!("Wrong type {:?}",),
        }
        Ok(())
    }

    #[test]
    fn objects_arrays() -> Result<()> {
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), Primitive::Integer(1));
        assert_debug_eq!(TxtCodec::from_str(r#"{"a": 1}"#)?, Primitive::Object(map));
        assert_debug_eq!(
            TxtCodec::from_str(r#"[1]"#)?,
            Node::Array(vec![Primitive::Integer(1)])
        );
        Ok(())
    }

    #[test]
    fn strings() -> Result<()> {
        match TxtCodec::from_str("not valid json")? {
            Node::String(value) => assert_eq!(value, "not valid json"),
            _ => bail!("Wrong type {:?}",),
        }
        Ok(())
    }
}
