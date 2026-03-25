//! Codec for encoding Stencila documents to AT Protocol JSON.
//!
//! This codec encodes Stencila [`Article`](stencila_codec::stencila_schema::Article) nodes
//! into AT Protocol-compatible JSON using OXA block types and richtext facets.
//! It is an encode-only codec; decoding is not supported.

use stencila_codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, Losses, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

/// Block-level encoding for AT Protocol JSON.
pub mod blocks;
mod encode;
/// Inline-tree-to-facet flattening for AT Protocol richtext.
pub mod facets;
/// AT Protocol Namespaced Identifier (NSID) constants.
pub mod nsids;

/// A codec for encoding Stencila documents to AT Protocol JSON.
pub struct AtProtoCodec;

#[async_trait]
impl Codec for AtProtoCodec {
    fn name(&self) -> &str {
        "atproto"
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::AtProtoJson => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let mut losses = Losses::none();
        let value = encode::encode_article(node, &mut losses)?;

        let json = match options.and_then(|options| options.compact) {
            Some(true) => serde_json::to_string(&value)?,
            Some(false) | None => serde_json::to_string_pretty(&value)?,
        };

        Ok((
            json,
            EncodeInfo {
                losses,
                ..EncodeInfo::none()
            },
        ))
    }
}
