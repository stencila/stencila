use stencila_codec::{
    Codec, EncodeInfo, EncodeOptions, async_trait, eyre::Result, stencila_format::Format,
    stencila_schema::Node,
};

use stencila_codec_text_trait::TextCodec as _;

pub use stencila_codec_text_trait::to_text;

/// A codec for plain text
pub struct TextCodec;

#[async_trait]
impl Codec for TextCodec {
    fn name(&self) -> &str {
        "text"
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Text)
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let mut text = node.to_text();

        if text.ends_with("\n\n") {
            text.pop();
        }

        Ok((text, EncodeInfo::none()))
    }
}
