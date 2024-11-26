use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
};
use codec_pandoc::{pandoc_from_format, pandoc_to_format, root_from_pandoc, root_to_pandoc};

/// A codec for LaTeX
pub struct LatexCodec;

const PANDOC_FORMAT: &str = "latex";

#[async_trait]
impl Codec for LatexCodec {
    fn name(&self) -> &str {
        "latex"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Latex => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Latex => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_str(
        &self,
        input: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let pandoc = pandoc_from_format(input, None, PANDOC_FORMAT, &[]).await?;
        root_from_pandoc(pandoc)
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let (pandoc, info) = root_to_pandoc(node)?;
        let output = pandoc_to_format(&pandoc, None, PANDOC_FORMAT, &[]).await?;
        Ok((output, info))
    }
}
