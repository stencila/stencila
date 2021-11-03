//! A codec for YAML

use codec_trait::{eyre::Result, stencila_schema::Node, Codec, DecodeOptions, EncodeOptions};
use node_coerce::coerce;

pub struct YamlCodec {}

impl Codec for YamlCodec {
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        coerce(serde_yaml::from_str(str)?, None)
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        Ok(serde_yaml::to_string(node)?)
    }
}
