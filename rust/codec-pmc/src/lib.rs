use std::path::Path;

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOperation, StructuringOptions,
    async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

mod decode;
mod html;
mod tar;

/// A codec for decoding PubMed Central Open Access Packages and HTML pages
///
/// See https://pmc.ncbi.nlm.nih.gov/tools/oa-service/ and
/// https://pmc.ncbi.nlm.nih.gov/tools/openftlist/
pub struct PmcCodec;

#[async_trait]
impl Codec for PmcCodec {
    fn name(&self) -> &str {
        "pmc"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::PmcOa => CodecSupport::LowLoss,
            Format::Html => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, format: &Format) -> StructuringOptions {
        use StructuringOperation::*;
        match format {
            Format::PmcOa => StructuringOptions::new(
                [
                    NormalizeCitations,
                    TableImagesToRows,
                    MathImagesToTex,
                ],
                [],
            ),
            Format::Html => StructuringOptions::new(
                [
                    LinksToCitations,
                    NormalizeCitations,
                ],
                [],
            ),
            _ => StructuringOptions::default(),
        }
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        decode::decode_path(path, options).await
    }
}

impl PmcCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_pmcid(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some(pmcid) = decode::extract_pmcid(identifier) else {
            bail!("Not a recognized PubMed Central id")
        };

        let (node, info, format) = decode::decode_pmcid(&pmcid, options).await?;
        let structuring_options = Self.structuring_options(&format);

        Ok((node, info, structuring_options))
    }
}
