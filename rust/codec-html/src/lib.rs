use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

mod decode;
mod encode;

// Export sync functions for sibling crates
pub use decode::decode;

/// A codec for HTML
pub struct HtmlCodec;

#[async_trait]
impl Codec for HtmlCodec {
    fn name(&self) -> &str {
        "html"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Html)
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Html)
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode::decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode::encode(node, options)
    }
}
