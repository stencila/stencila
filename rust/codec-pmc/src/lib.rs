use std::path::Path;

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType, async_trait,
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
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_str(
        &self,
        pmcid: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode::decode_pmcid(pmcid, options).await
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
    ) -> Result<(Node, DecodeInfo)> {
        let Some(pmcid) = decode::extract_pmcid(identifier) else {
            bail!("Not a recognized PubMed Central id")
        };

        decode::decode_pmcid(&pmcid, options).await
    }

    /// Download HTML for a PMCID from PMC website
    ///
    /// Downloads the HTML page for the given PMCID to the specified path.
    /// The `pmcid` can include the "PMC" prefix or just the numeric part.
    pub async fn download_html(pmcid: &str, to_path: &Path) -> Result<()> {
        html::download_html(pmcid, to_path).await
    }
}
