//! Codec for Tiptap / ProseMirror JSON.
//!
//! The native mapping is intentionally small: document roots, headings,
//! paragraphs, text, and common inline marks such as bold, italic, links, code,
//! strikeout, underline, subscript, and superscript. Other Stencila block and
//! inline nodes are preserved in custom opaque `stencilaBlock` and
//! `stencilaInline` nodes so they can round-trip through Tiptap without being
//! editable as native Tiptap structures. Unsupported native Tiptap nodes and
//! marks are decoded with losses until explicit mappings are added.

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

use nodes::{root_from_tiptap, root_to_tiptap};
use tiptap::TiptapDoc;

mod blocks;
mod inlines;
mod nodes;
mod shared;
mod tiptap;

/// A codec for Tiptap / ProseMirror JSON.
pub struct TiptapCodec;

#[async_trait]
impl Codec for TiptapCodec {
    fn name(&self) -> &str {
        "tiptap"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Tiptap)
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::Tiptap)
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

/// Decode a Tiptap JSON string into a Stencila Schema node.
pub fn decode(content: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let doc: TiptapDoc = serde_json::from_str(content)?;

    root_from_tiptap(doc)
}

/// Encode a Stencila Schema node into a Tiptap JSON string.
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let (doc, info) = root_to_tiptap(node)?;

    let json = match options.and_then(|options| options.compact) {
        Some(false) => serde_json::to_string_pretty(&doc)?,
        Some(true) | None => serde_json::to_string(&doc)?,
    };

    Ok((json, info))
}
