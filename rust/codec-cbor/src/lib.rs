use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Node, NodeType},
};
use stencila_node_media::{embed_media, extract_media};

pub mod r#trait;
use r#trait::CborCodec as _;

/// A codec for CBOR
pub struct CborCodec;

#[async_trait]
impl Codec for CborCodec {
    fn name(&self) -> &str {
        "cbor"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cbor => CodecSupport::NoLoss,
            Format::CborZstd => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cbor => CodecSupport::NoLoss,
            Format::CborZstd => CodecSupport::NoLoss,
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
        let bytes = if let Some(Format::CborZstd) = options.and_then(|options| options.format) {
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
        let bytes = if let Some(media) = options
            .as_ref()
            .and_then(|opts| opts.extract_media.as_ref())
        {
            let mut copy = node.clone();
            extract_media(
                &mut copy,
                options.as_ref().and_then(|opts| opts.to_path.as_deref()),
                media,
            )?;
            copy.to_cbor()?
        } else if options
            .as_ref()
            .and_then(|opts| opts.embed_media)
            .unwrap_or_default()
        {
            let mut copy = node.clone();
            embed_media(
                &mut copy,
                options.as_ref().and_then(|opts| opts.from_path.as_deref()),
            )?;
            copy.to_cbor()?
        } else {
            node.to_cbor()?
        };

        let bytes = if let Some(Format::CborZstd) = options.and_then(|options| options.format) {
            zstd::encode_all(&bytes[..], 0)?
        } else {
            bytes
        };

        Ok((bytes, EncodeInfo::none()))
    }
}
