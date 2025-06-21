use codec::{common::eyre::Result, schema::Node, EncodeInfo, EncodeOptions};
use codec_markdown_trait::{MarkdownCodec as _, MarkdownEncodeContext};

/// Encode a Stencila Schema [`Node`] to a Markdown string
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    let options = options.unwrap_or_default();

    let mut context = MarkdownEncodeContext::new(options.format, options.render);

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
