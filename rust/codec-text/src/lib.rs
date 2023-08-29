use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, EncodeOptions, Losses,
};

use codec_text_trait::TextCodec as _;

/// A codec for plain text
pub struct TextCodec;

#[async_trait]
impl Codec for TextCodec {
    fn name(&self) -> &str {
        "text"
    }

    fn status(&self) -> Status {
        Status::Unstable
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Text]
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_from_path(&self) -> bool {
        false
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        Ok(node.to_text())
    }
}
