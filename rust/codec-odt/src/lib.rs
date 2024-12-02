use std::path::Path;

use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, NodeType,
};
use codec_pandoc::{pandoc_from_format, pandoc_to_format, root_from_pandoc, root_to_pandoc};

/// A codec for Open Document Format
pub struct OdtCodec;

const PANDOC_FORMAT: &str = "odt";

#[async_trait]
impl Codec for OdtCodec {
    fn name(&self) -> &str {
        "odt"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Odt => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Odt => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let pandoc = pandoc_from_format(
            "",
            Some(path),
            PANDOC_FORMAT,
            options
                .map(|options| options.passthrough_args)
                .unwrap_or_default(),
        )
        .await?;
        root_from_pandoc(pandoc)
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let (pandoc, info) = root_to_pandoc(node)?;
        pandoc_to_format(
            &pandoc,
            Some(path),
            PANDOC_FORMAT,
            options
                .map(|options| options.passthrough_args)
                .unwrap_or_default(),
        )
        .await?;
        Ok(info)
    }
}
