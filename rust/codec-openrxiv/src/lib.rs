use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, StructuringOperation, StructuringOptions,
    async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};

mod decode;

/// A codec for decoding https://bioRxiv.org and https://medRxiv.org preprints
pub struct OpenRxivCodec;

#[async_trait]
impl Codec for OpenRxivCodec {
    fn name(&self) -> &str {
        "openrxiv"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Meca => CodecSupport::LowLoss,
            Format::Pdf => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn structuring_options(&self, format: &Format) -> StructuringOptions {
        use StructuringOperation::*;
        match format {
            Format::Meca => StructuringOptions::new(
                [NormalizeCitations, TableImagesToRows, MathImagesToTex],
                [],
            ),
            Format::Pdf => StructuringOptions::all(),
            _ => StructuringOptions::default(),
        }
    }
}

impl OpenRxivCodec {
    pub fn supports_identifier(identifier: &str) -> bool {
        decode::extract_openrxiv_id(identifier).is_some()
    }

    pub async fn from_identifier(
        identifier: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo, StructuringOptions)> {
        let Some((openrxiv_id, server)) = decode::extract_openrxiv_id(identifier) else {
            bail!("Not a recognized arXiv id")
        };

        let (node, info, format) =
            decode::decode_openrxiv_id(&openrxiv_id, server.as_deref(), options).await?;
        let structuring_options = Self.structuring_options(&format);

        Ok((node, info, structuring_options))
    }
}
