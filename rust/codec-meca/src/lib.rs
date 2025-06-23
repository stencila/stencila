use std::path::Path;

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType,
};
use decode::decode_path;

mod decode;

/// A codec for decoding MECA
///
/// See https://www.niso.org/standards-committees/meca
pub struct MecaCodec;

#[async_trait]
impl Codec for MecaCodec {
    fn name(&self) -> &str {
        "meca"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Meca => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        decode_path(path, options).await
    }
}
