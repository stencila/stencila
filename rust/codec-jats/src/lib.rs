use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    Codec, DecodeOptions, EncodeOptions,
};

/// A codec for JATS XML
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn formats(&self) -> Vec<Format> {
        vec![Format::Jats]
    }

    async fn from_str(&self, _str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        todo!()
    }

    async fn to_string(&self, _node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        todo!()
    }
}
