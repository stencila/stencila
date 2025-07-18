use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions,
    common::{async_trait::async_trait, eyre::Result, serde_json},
    format::Format,
    schema::Node,
    status::Status,
};

use crate::csl::CslItem;

mod csl;

/// A codec for decoding CSL-JSON into a Stencila [`Node`]
pub struct CslCodec;

#[async_trait]
impl Codec for CslCodec {
    fn name(&self) -> &str {
        "csl"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Csl => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let csl_item: CslItem = serde_json::from_str(str)?;
        let article = csl_item.to_article()?;
        Ok((Node::Article(article), DecodeInfo::default()))
    }
}
