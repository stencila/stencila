use std::path::Path;

use codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, NodeType,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
    },
    format::Format,
    schema::Node,
    status::Status,
};
use codec_latex_trait::{latex_to_png, to_latex};
use images::mogrify_image;

/// A codec for PNG
pub struct PngCodec;

#[async_trait]
impl Codec for PngCodec {
    fn name(&self) -> &str {
        "png"
    }

    fn status(&self) -> Status {
        Status::Alpha
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Png => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::HighLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = options.unwrap_or_default();
        let tool = options.tool.clone().unwrap_or_default();

        let info = if tool == "latex" || tool.is_empty() {
            let (latex, info) = to_latex(node, Format::Latex, false, true);
            latex_to_png(&latex, path).await?;
            info
        } else {
            bail!("Tool `{tool}` is not supported for encoding to PNG")
        };

        if !options.tool_args.is_empty() {
            mogrify_image(path, &options.tool_args).await?;
        }

        Ok(info)
    }
}
