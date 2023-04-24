use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, DecodeOptions, EncodeOptions,
};

/// A codec for JATS XML
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Jats]
    }

    async fn from_str(&self, _str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        todo!()
    }

    async fn to_string(&self, _node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        todo!()
    }
}
