use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
    },
    format::Format,
    schema::Node,
    status::Status,
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

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Html => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode_html::decode_arxiv_html("", str, options).await
    }
}

impl ArxivCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_arxiv_id(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let Some(arxiv_id) = decode::extract_arxiv_id(identifier) else {
            bail!("Not a recognized arXiv id")
        };

        decode::decode_arxiv_id(&arxiv_id, options).await
    }
}
