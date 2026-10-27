//! Decode the full-text TEI produced by GROBID and ScienceBeam.
//!
//! # Page coordinates
//!
//! The `coords` of elements, their boxes on the pages of the PDF, are
//! information about the source rather than the content, so they are not kept
//! on the decoded nodes. They are returned in the [`PageMap`] of the
//! [`DecodeInfo`], by the id of each node, with the part of the node that a box
//! is of, such as `persName`, `orgName`, `graphic` or `table`, or `marker` for
//! where a footnote is referenced. A TEI paragraph split around embedded blocks
//! has each of the resulting paragraphs mapped to its boxes. The `xml:id` of an
//! element is kept as the `id` of its node, but no ids are generated.
//!
//! [`PageMap`]: stencila_codec::PageMap
//! [`DecodeInfo`]: stencila_codec::DecodeInfo

use std::io::Read;

use flate2::read::GzDecoder;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, StructuringOptions, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

mod decode;

pub use decode::decode;

#[cfg(test)]
mod tests;

/// A codec for TEI
///
/// Decodes the full-text TEI that GROBID extracts from PDFs, which OpenAlex
/// also serves for many works, to the same Stencila nodes that the JATS codec
/// decodes the same content to. Encoding to TEI is not supported.
pub struct TeiCodec;

#[async_trait]
impl Codec for TeiCodec {
    fn name(&self) -> &str {
        "tei"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::Tei)
    }

    fn supports_from_bytes(&self) -> bool {
        true
    }

    // Not from strings, so that content fetched from a URL, which may be gzip
    // compressed, is decoded as bytes, by `from_bytes`, rather than as text
    fn supports_from_string(&self) -> bool {
        false
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        use stencila_codec::StructuringOperation::*;
        StructuringOptions::new([NormalizeCitations], [])
    }

    async fn from_bytes(
        &self,
        bytes: &[u8],
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        // GROBID TEI is often distributed gzip compressed (OpenAlex serves it
        // that way) so decompress it here rather than requiring a separate step
        let tei = if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut tei = String::new();
            GzDecoder::new(bytes).read_to_string(&mut tei)?;
            tei
        } else {
            String::from_utf8(bytes.to_vec())?
        };

        decode(&tei, options)
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(str, options)
    }
}
