use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, StructuringOptions, async_trait,
    eyre::Result, stencila_format::Format, stencila_schema::Node,
};

mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;

#[cfg(test)]
mod tests;

/// A codec for JATS
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Jats)
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Jats)
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        use stencila_codec::StructuringOperation::*;
        StructuringOptions::new([NormalizeCitations], [])
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(str, options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        encode(node, options)
    }
}
