use codec::{eyre::Result, stencila_schema::*, CodecTrait, EncodeOptions};
use codec_md::MdCodec;

/// Encode a `Node` to R Markdown
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let mut node = node.clone();
    if let Node::Article(article) = &mut node {
        if let Some(content) = &mut article.content {
            transform_blocks(content)
        }
    }
    MdCodec::to_string(&node, options)
}

fn transform_blocks(blocks: &mut Vec<BlockContent>) {
    for block in blocks {
        match block {
            // Code chunks are transformed to code blocks with curly braced language
            BlockContent::CodeChunk(CodeChunk {
                programming_language,
                text,
                ..
            }) => {
                *block = BlockContent::CodeBlock(CodeBlock {
                    programming_language: Some(Box::new(["{", programming_language, "}"].concat())),
                    text: text.to_string(),
                    ..Default::default()
                })
            }
            // Transform the inline content of other block types
            BlockContent::Paragraph(Paragraph { content, .. }) => transform_inlines(content),
            _ => (),
        }
    }
}

fn transform_inlines(inlines: &mut Vec<InlineContent>) {
    for inline in inlines {
        match inline {
            // Code expressions are transformed to code fragments prefixed with the language
            InlineContent::CodeExpression(CodeExpression {
                programming_language,
                text,
                ..
            }) => {
                *inline = InlineContent::CodeFragment(CodeFragment {
                    text: [programming_language, " ", text].concat(),
                    ..Default::default()
                })
            }
            // Recursively transform other inlines
            InlineContent::Delete(Delete { content, .. })
            | InlineContent::Emphasis(Emphasis { content, .. })
            | InlineContent::Subscript(Subscript { content, .. })
            | InlineContent::Superscript(Superscript { content, .. })
            | InlineContent::Strong(Strong { content, .. })
            | InlineContent::NontextualAnnotation(NontextualAnnotation { content, .. }) => {
                transform_inlines(content)
            }
            _ => (),
        }
    }
}
