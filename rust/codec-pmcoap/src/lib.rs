use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType,
};
use decode::decode_id;

mod decode;

/// A codec for decoding PubMed Central Open Access Package
///
/// See https://pmc.ncbi.nlm.nih.gov/tools/oa-service/ and
/// https://pmc.ncbi.nlm.nih.gov/tools/openftlist/
pub struct PmcOapCodec;

#[async_trait]
impl Codec for PmcOapCodec {
    fn name(&self) -> &str {
        "pmcoap"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::PmcOap => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_str(
        &self,
        id: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode_id(id, options).await
    }
}
