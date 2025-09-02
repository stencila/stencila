use stencila_codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Node, NodeType},
    stencila_status::Status,
};

mod blocks;
mod coarse;
mod inlines;
mod meta;
mod nodes;
mod pandoc;
mod shared;

// Exports for derived crates
pub use coarse::*;
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

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
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
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let pandoc = serde_json::from_str(str)?;

        root_from_pandoc(pandoc, Format::Pandoc, &options)
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let (pandoc, info) = root_to_pandoc(node, Format::Pandoc, &options)?;

        let json = match options.and_then(|options| options.compact) {
            Some(true) | None => serde_json::to_string(&pandoc)?,
            Some(false) => serde_json::to_string_pretty(&pandoc)?,
        };

        Ok((json, info))
    }
}

pub fn pandoc_availability() -> CodecAvailability {
    match stencila_tools::is_installed("pandoc") {
        Ok(true) => CodecAvailability::Available,
        _ => CodecAvailability::Installable("pandoc".into()),
    }
}
