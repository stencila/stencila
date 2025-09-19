use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

mod decode;

/// A codec for decoding DOIs into Stencila [`Node`]
///
/// This codec is used for fetching metadata for an [`Node`] having
/// a DOI. It is used to supplement other codecs, such as `codec-arxiv`,
/// `codec-openrxiv`, and `codec-pmcoa` by providing standardized metadata
/// for properties such as authors and references, which may not be well
/// supported by those codecs.
///
/// CSL-JSON is used because it is most widely supported across registries
/// such as DataCite and Crossref.
pub struct DoiCodec;

#[async_trait]
impl Codec for DoiCodec {
    fn name(&self) -> &str {
        "doi"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        StructuringOptions::none()
    }
}

impl DoiCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_doi(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some(doi) = decode::extract_doi(identifier) else {
            bail!("Not a recognized DOI")
        };

        let (node, info) = decode::decode_doi(&doi, options).await?;
        let structuring_options = Self.structuring_options(&Format::Csl);

        Ok((node, info, structuring_options))
    }
}
