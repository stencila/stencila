use std::path::Path;

use stencila_node_media::extract_media;
use tokio::fs::{create_dir_all, write};

use stencila_codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_pandoc::{pandoc_availability, pandoc_to_format, root_to_pandoc};

use crate::{decode::decode, encode::encode};

mod decode;
mod encode;

/// A codec for LaTeX
pub struct LatexCodec;

#[async_trait]
impl Codec for LatexCodec {
    fn name(&self) -> &str {
        "latex"
    }

    fn availability(&self) -> CodecAvailability {
        pandoc_availability()
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Latex | Format::Tex => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Latex | Format::Tex => CodecSupport::LowLoss,
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
        latex: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        decode(latex, options).await
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let mut options = options.unwrap_or_default();
        let format = options.format.clone().unwrap_or(Format::Latex);

        if options.tool.is_none() {
            if let Some(media) = options.extract_media.as_ref() {
                let mut copy = node.clone();
                extract_media(&mut copy, media)?;
                encode(&copy, options)
            } else {
                encode(node, options)
            }
        } else if matches!(options.tool.as_deref(), Some("pandoc")) {
            options.tool_args.push("--listings".into());
            if options.standalone.unwrap_or_default() {
                options.tool_args.push("--standalone".into());
            }
            let options = Some(options);

            let (pandoc, info) = root_to_pandoc(node, format, &options)?;
            let output = pandoc_to_format(&pandoc, None, "latex", &options).await?;

            Ok((output, info))
        } else {
            bail!("Tool is not supported")
        }
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.embed_media.unwrap_or_default() {
            bail!("Embedded media are not supported with LaTeX");
        }
        if options.standalone.is_none() {
            options.standalone = Some(true);
        }
        if options.extract_media.is_none() {
            options.extract_media = Some(path.with_extension("media"));
        }
        options.to_path = Some(path.to_path_buf());

        let (md, info) = self.to_string(node, Some(options)).await?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        write(&path, md).await?;

        Ok(info)
    }
}
