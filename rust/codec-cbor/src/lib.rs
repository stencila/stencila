use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};

pub mod r#trait;
use r#trait::CborCodec as _;

/// A codec for CBOR
pub struct CborCodec;

#[async_trait]
impl Codec for CborCodec {
    fn name(&self) -> &str {
        "cbor"
    }

    fn status(&self) -> Status {
        Status::Stable
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cbor => CodecSupport::NoLoss,
            Format::CborZst => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cbor => CodecSupport::NoLoss,
            Format::CborZst => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_bytes(&self) -> bool {
        true
    }

    fn supports_to_bytes(&self) -> bool {
        true
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn from_bytes(
        &self,
        bytes: &[u8],
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let bytes = if let Some(Format::CborZst) = options.and_then(|options| options.format) {
            zstd::decode_all(bytes)?
        } else {
            bytes.to_vec()
        };

        let node = Node::from_cbor(&bytes)?;

        Ok((node, DecodeInfo::none()))
    }

    async fn to_bytes(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(Vec<u8>, EncodeInfo)> {
        let bytes = node.to_cbor()?;

        let bytes = if let Some(Format::CborZst) = options.and_then(|options| options.format) {
            zstd::encode_all(&bytes[..], 0)?
        } else {
            bytes
        };

        Ok((bytes, EncodeInfo::none()))
    }
}
