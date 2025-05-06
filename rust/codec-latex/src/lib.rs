use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
};
use codec_latex_trait::{LatexCodec as _, LatexEncodeContext};
use codec_pandoc::{pandoc_availability, pandoc_to_format, root_to_pandoc};

mod decode;

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
        let options = options.unwrap_or_default();
        let format = options.format.unwrap_or(Format::Latex);
        let tool = options.tool.unwrap_or_default();

        if tool == "" {
            let mut context = LatexEncodeContext::new(format);
            node.to_latex(&mut context);

            let mut output = context.content;
            if output.ends_with("\n\n") {
                output.pop();
            }

            let info = EncodeInfo {
                losses: context.losses,
                mapping: context.mapping,
            };

            Ok((output, info))
        } else if tool == "pandoc" {
            let (pandoc, info) = root_to_pandoc(node, format)?;

            let mut args = options.tool_args;
            args.append(&mut vec!["--listings".into(), "--standalone".into()]);

            let output = pandoc_to_format(&pandoc, None, PANDOC_FORMAT, args).await?;

            Ok((output, info))
        } else {
            bail!("Tool `{tool}` is not supported for encoding to {format}",)
        }
    }
}
