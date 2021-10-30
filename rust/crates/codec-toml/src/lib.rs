//! A codec for TOML
//!
//! Intended primarily for representing document metadata.
//! TOML is not recommended for large complex documents and encoding
//! may fail with the error "values must be emitted before tables".

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, EncodeOptions};
use node_coerce::coerce;

pub struct TomlCodec {}

impl Codec for TomlCodec {
    fn from_str(str: &str) -> Result<Node> {
        coerce(toml::from_str(str)?, None)
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        Ok(toml::to_string::<Node>(node)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec_trait::stencila_schema::Person;
    use test_utils::{assert_debug_eq, pretty_assertions::assert_eq};

    #[test]
    fn to_from() {
        let sarah = Node::Person(Person {
            given_names: Some(vec!["Sarah".to_string()]),
            ..Default::default()
        });

        let toml = "type = \"Person\"\ngivenNames = [\"Sarah\"]\n".to_string();

        assert_debug_eq!(TomlCodec::from_str(&toml).unwrap(), sarah);
        assert_eq!(TomlCodec::to_string(&sarah, None).unwrap(), toml);
    }
}
