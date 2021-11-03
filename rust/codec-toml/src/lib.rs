use codec::{
    eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait, DecodeOptions,
    EncodeOptions,
};
use node_coerce::coerce;

/// A codec for TOML
///
/// Intended primarily for representing document metadata.
/// TOML is not recommended for large complex documents and encoding
/// may fail with the error "values must be emitted before tables".
pub struct TomlCodec {}

impl CodecTrait for TomlCodec {
    fn spec() -> Codec {
        Codec {
            formats: vec_string!["toml"],
            root_types: vec_string!["*"],
            from_string: true,
            from_path: true,
            to_string: true,
            to_path: true,
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        coerce(toml::from_str(str)?, None)
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        Ok(toml::to_string::<Node>(node)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::stencila_schema::Person;
    use test_utils::{assert_json_eq, pretty_assertions::assert_eq};

    #[test]
    fn to_from() {
        let sarah = Node::Person(Person {
            given_names: Some(vec!["Sarah".to_string()]),
            ..Default::default()
        });

        let toml = "type = \"Person\"\ngivenNames = [\"Sarah\"]\n".to_string();

        assert_json_eq!(TomlCodec::from_str(&toml, None).unwrap(), sarah);
        assert_eq!(TomlCodec::to_string(&sarah, None).unwrap(), toml);
    }
}
