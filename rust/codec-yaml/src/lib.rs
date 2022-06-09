use codec::{
    common::{eyre::Result, serde_yaml},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use node_coerce::coerce;

/// A codec for YAML
pub struct YamlCodec {}

impl CodecTrait for YamlCodec {
    fn spec() -> Codec {
        Codec {
            status: "stable".to_string(),
            formats: vec_string!["yaml"],
            root_types: vec_string!["*"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        coerce(serde_yaml::from_str(str)?, None)
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        Ok(serde_yaml::to_string(node)?)
    }
}
