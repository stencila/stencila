use codec::{
    common::{async_trait::async_trait, eyre::Result, serde_json},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};

mod blocks;
mod inlines;
mod nodes;
mod pandoc;
mod shared;

// Exports for derived crates
pub use nodes::*;
pub use pandoc::*;

/// A codec for Pandoc AST JSON
///
/// This codec is responsible for encoding a Stencila node to Pandoc AST
/// and and decoding Pandoc AST to a Stencila node.
///
/// It is seldom used by itself, but rather acts as a utility for other codecs
/// that delegate encoding/decoding to Pandoc (e.g. DOCX, LaTeX).
pub struct PandocCodec;

#[async_trait]
impl Codec for PandocCodec {
    fn name(&self) -> &str {
        "pandoc"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pandoc => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Pandoc => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let pandoc = serde_json::from_str(str)?;

        root_from_pandoc(pandoc)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let (pandoc, info) = root_to_pandoc(node)?;

        let json = match options.and_then(|options| options.compact) {
            Some(true) | None => serde_json::to_string(&pandoc)?,
            Some(false) => serde_json::to_string_pretty(&pandoc)?,
        };

        Ok((json, info))
    }
}
