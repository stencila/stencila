use codec_json5::Json5Codec;
use codec_trait::Codec;
use eyre::Result;
use stencila_schema::{Node, Null};

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
pub fn decode(txt: &str) -> Result<Node> {
    Ok(decode_fragment(txt))
}

/// Decode plain text to a `Node`, infallibly.
///
/// This function simply exists for when you want a `Node`, rather
/// than a `Result<Node>` which is the return signature of all decode functions.
pub fn decode_fragment(txt: &str) -> Node {
    match txt.trim() {
        "true" | "TRUE" | "True" => return Node::Boolean(true),
        "false" | "FALSE" | "False" => return Node::Boolean(false),
        "null" | "NULL" | "Null" => return Node::Null(Null {}),
        _ => (),
    };

    if let Ok(node) = Json5Codec::from_str(txt) {
        return node;
    }

    Node::String(txt.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_json_eq;
    use eyre::bail;
    use serde_json::json;
    use stencila_schema::Null;

    #[test]
    fn booleans() -> Result<()> {
        assert!(matches!(decode("true")?, Node::Boolean(true)));
        assert!(matches!(decode("TRUE")?, Node::Boolean(true)));
        assert!(matches!(decode("  True  ")?, Node::Boolean(true)));
        assert_json_eq!(decode("tRUe")?, json!("tRUe"));
        Ok(())
    }

    #[test]
    fn nulls() -> Result<()> {
        assert!(matches!(decode("null")?, Node::Null(Null {})));
        assert!(matches!(decode("NULL")?, Node::Null(Null {})));
        assert!(matches!(decode("  Null  ")?, Node::Null(Null {})));
        assert_json_eq!(decode("nUll")?, json!("nUll"));
        Ok(())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn numbers() -> Result<()> {
        assert!(matches!(decode("1")?, Node::Integer(1)));
        match decode("1.23")? {
            Node::Number(value) => assert_eq!(value, 1.23),
            _ => bail!("Wrong type {:?}",),
        }
        Ok(())
    }

    #[test]
    fn objects_arrays() -> Result<()> {
        assert_json_eq!(decode(r#"{"a": 1}"#)?, json!({"a": 1}));
        assert_json_eq!(decode(r#"[1, 2, 3]"#)?, json!([1, 2, 3]));
        Ok(())
    }

    #[test]
    fn strings() -> Result<()> {
        match decode("not valid json")? {
            Node::String(value) => assert_eq!(value, "not valid json"),
            _ => bail!("Wrong type {:?}",),
        }
        Ok(())
    }
}
