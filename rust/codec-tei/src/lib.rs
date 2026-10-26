//! Decode the full-text TEI produced by GROBID and ScienceBeam.
//!
//! # Page coordinates
//!
//! Coordinates are preserved verbatim as `PropertyValue` identifiers with
//! `propertyId: "coords"`. People, affiliations, references, figures and tables
//! carry these in their own `identifiers`. An optional `name` identifies a part,
//! such as `persName`, `orgName`, `graphic` or `table`.
//!
//! Content nodes without an `identifiers` property (including headings and
//! paragraphs) retain their `xml:id`, or receive a deterministic, collision-free
//! ID when they have coordinates. Their coordinate identifiers are stored in the
//! article's `identifiers`, with `name` set to the content node's ID. For example,
//! a paragraph with `id: "p0"` points to the article identifier
//! `{"type":"PropertyValue","propertyId":"coords","name":"p0","value":"1,10,20,30,40"}`.
//! If a TEI paragraph is split around embedded blocks, each resulting paragraph
//! gets its own ID and points to the original element's coordinates.
//!
//! Each coordinate string is a semicolon-separated list of `page,x,y,width,height`
//! boxes in PDF points measured from the top left of the page. The node comparison
//! crate ignores coordinate identifiers by default, so they do not affect alignment
//! or create differences against formats without page coordinates.

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
