use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOperation, StructuringOptions,
    async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

mod decode;
mod decode_html;
mod decode_html_blocks;
mod decode_html_inlines;
mod decode_pdf;
mod decode_src;

/// A codec for decoding https://arXiv.org preprints
pub struct ArxivCodec;

#[async_trait]
impl Codec for ArxivCodec {
    fn name(&self) -> &str {
        "arxiv"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Html => CodecSupport::LowLoss,
            Format::Latex => CodecSupport::LowLoss,
            Format::Pdf => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, format: &Format) -> StructuringOptions {
        match format {
            Format::Html => StructuringOptions::new([StructuringOperation::NormalizeCitations], []),
            Format::Latex => StructuringOptions::none(),
            Format::Pdf => StructuringOptions::all(),
            _ => StructuringOptions::default(),
        }
    }
}

impl ArxivCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_arxiv_id(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some(arxiv_id) = decode::extract_arxiv_id(identifier) else {
            bail!("Not a recognized arXiv id")
        };

        let (node, info, format) = decode::decode_arxiv_id(&arxiv_id, options).await?;
        let structuring_options = Self.structuring_options(&format);

        Ok((node, info, structuring_options))
    }
}
