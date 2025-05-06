use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecAvailability, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
    NodeType,
};
use codec_latex_trait::{LatexCodec as _, LatexEncodeContext};
use codec_pandoc::{
    pandoc_availability, pandoc_from_format, pandoc_to_format, root_from_pandoc, root_to_pandoc,
};

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
        input: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let options = options.unwrap_or_default();

        if options.coarse.unwrap_or_default() {
            decode::coarse(input)
        } else {
            let pandoc =
                pandoc_from_format(input, None, PANDOC_FORMAT, options.passthrough_args).await?;
            root_from_pandoc(pandoc, Format::Latex)
        }
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let options = options.unwrap_or_default();
        let format = options.format.unwrap_or(Format::Latex);

        if let Some("--builtin") = options.passthrough_args.first().map(|arg| arg.as_str()) {
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
        } else {
            let (pandoc, info) = root_to_pandoc(node, format)?;

            let mut args = options.passthrough_args;
            args.push("--listings".into());

            let output = pandoc_to_format(&pandoc, None, PANDOC_FORMAT, args).await?;

            Ok((output, info))
        }
    }
}
