use stencila_codec::{EncodeInfo, EncodeOptions, eyre::Result, stencila_schema::Node};
use stencila_codec_markdown_trait::{
    MarkdownCodec as _, MarkdownEncodeContext, MarkdownEncodeMode,
};

/// Encode a Stencila Schema [`Node`] to a Markdown string
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let options = options.unwrap_or_default();

    let mode = if options.render.unwrap_or_default() {
        MarkdownEncodeMode::Render
    } else {
        MarkdownEncodeMode::Normal
    };

    let mut context = MarkdownEncodeContext::new(options.format, Some(mode));

    node.to_markdown(&mut context);
    if context.content.ends_with("\n\n") {
        context.content.pop();
    }

    Ok((
        context.content,
        EncodeInfo {
            losses: context.losses,
            mapping: context.mapping,
        },
    ))
}
