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
