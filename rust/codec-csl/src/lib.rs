use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, async_trait, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};

mod date;
mod item;
mod name;
mod ordinary;
mod title;

/// Export `Item` for use by other codecs that use CSL (e.g. Crossref codec)
pub use item::Item;

/// A codec for CSL-JSON (Citation Style Language JSON)
///
/// Only supports decoding from CSL-JSON. Primarily used for fetching
/// metadata about creative works
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/
pub struct CslCodec;

#[async_trait]
impl Codec for CslCodec {
    fn name(&self) -> &str {
        "csl"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let csl_item: Item = serde_json::from_str(str)?;
        let node = csl_item.into();

        Ok((node, DecodeInfo::default()))
    }
}
