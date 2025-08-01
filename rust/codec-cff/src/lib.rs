use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions,
    common::{async_trait::async_trait, eyre::Result, serde_json},
    format::Format,
    schema::Node,
    status::Status,
};

/// A codec for the Citation File Format (CFF)
///
/// Only supports decoding from CFF. Primarily used for fetching
/// metadata about creative works hosted on GitHub.
///
/// See:
/// - https://citation-file-format.github.io/
/// - https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-citation-files
pub struct CffCodec;

#[async_trait]
impl Codec for CffCodec {
    fn name(&self) -> &str {
        "cff"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Cff => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let node = todo!();

        Ok((node, DecodeInfo::default()))
    }
}
