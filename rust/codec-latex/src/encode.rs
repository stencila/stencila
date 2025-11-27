use stencila_codec::{
    EncodeInfo, EncodeOptions, eyre::Result, stencila_format::Format, stencila_schema::Node,
};
use stencila_codec_latex_trait::to_latex;

/// Encode a node to LaTeX
pub(crate) fn encode(node: &Node, options: EncodeOptions) -> Result<(String, EncodeInfo)> {
    let format = options.format.clone().unwrap_or(Format::Latex);
    let standalone = options.standalone.unwrap_or_default();
    let render = options.render.unwrap_or_default();
    let highlight = options.highlight.unwrap_or_default();
    let reproducible = options.reproducible.unwrap_or_default();

    Ok(to_latex(
        node,
        format,
        standalone,
        render,
        highlight,
        reproducible,
    ))
}
