use stencila_codec::{
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_latex_trait::to_latex;
use stencila_codec_pandoc::{pandoc_availability, pandoc_to_format, root_to_pandoc};

mod decode;

/// A codec for LaTeX
pub struct LatexCodec;

const PANDOC_FORMAT: &str = "latex";

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
        decode::decode(latex, options).await
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let mut options = options.unwrap_or_default();
        let format = options.format.clone().unwrap_or(Format::Latex);
        let tool = options.tool.clone().unwrap_or_default();
        let standalone = options.standalone.unwrap_or_default();
        let render = options.render.unwrap_or_default();
        let highlight = options.highlight.unwrap_or_default();
        let reproducible = options.reproducible.unwrap_or_default();

        if tool.is_empty() {
            Ok(to_latex(
                node,
                format,
                standalone,
                render,
                highlight,
                reproducible,
                None,
            ))
        } else if tool == "pandoc" {
            options.tool_args.push("--listings".into());
            if standalone {
                options.tool_args.push("--standalone".into());
            }
            let options = Some(options);

            let (pandoc, info) = root_to_pandoc(node, format, &options)?;
            let output = pandoc_to_format(&pandoc, None, PANDOC_FORMAT, &options).await?;

            Ok((output, info))
        } else {
            bail!("Tool `{tool}` is not supported for encoding to {format}")
        }
    }
}
