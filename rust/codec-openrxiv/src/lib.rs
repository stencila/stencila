use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
    stencila_status::Status,
};

mod decode;

/// A codec for decoding https://bioRxiv.org and https://medRxiv.org preprints
pub struct OpenRxivCodec;

#[async_trait]
impl Codec for OpenRxivCodec {
    fn name(&self) -> &str {
        "openrxiv"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }
}

impl OpenRxivCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_openrxiv_id(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let Some((openrxiv_id, server)) = decode::extract_openrxiv_id(identifier) else {
            bail!("Not a recognized arXiv id")
        };

        decode::decode_openrxiv_id(&openrxiv_id, server.as_deref(), options).await
    }
}
