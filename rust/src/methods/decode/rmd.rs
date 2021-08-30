use super::md;
use eyre::Result;
use stencila_schema::{
    BlockContent, CodeBlock, CodeChunk, CodeExpression, CodeFragment, Delete, Emphasis,
    InlineContent, Node, NontextualAnnotation, Paragraph, Strong, Subscript, Superscript,
};

const LANGUAGES: &[&str] = &["r", "py", "python", "js", "javascript"];

/// Decode a R Markdown document to a `Node`
pub fn decode(input: &str) -> Result<Node> {
    let mut node = md::decode(input)?;
    if let Node::Article(article) = &mut node {
        if let Some(content) = &mut article.content {
            transform_blocks(content)
        }
    }
    Ok(node)
}

fn transform_blocks(blocks: &mut Vec<BlockContent>) {
    for block in blocks {
        match block {
            // Code blocks with curly braced language are transformed to code chunks
            BlockContent::CodeBlock(CodeBlock {
                programming_language,
                text,
                ..
            }) => {
                let lang = programming_language
                    .clone()
                    .map(|boxed| *boxed)
                    .unwrap_or_else(|| "".to_string());
                if lang.starts_with('{') && lang.ends_with('}') {
                    let lang = lang[1..(lang.len() - 1)].to_string();
                    *block = BlockContent::CodeChunk(CodeChunk {
                        programming_language: lang,
                        text: text.to_string(),
                        ..Default::default()
                    })
                }
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
            // Code fragments prefixed with a language code get transformed to a code expression
            InlineContent::CodeFragment(CodeFragment { text, .. }) => {
                for lang in LANGUAGES {
                    if let Some(text) = text.strip_prefix(&[lang, " "].concat()) {
                        *inline = InlineContent::CodeExpression(CodeExpression {
                            programming_language: lang.to_string(),
                            text: text.to_string(),
                            ..Default::default()
                        });
                        break;
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[ignore]
    #[test]
    fn rmd_articles() {
        snapshot_content("articles/*.Rmd", |_path, content| {
            assert_json_snapshot!(decode(&content).unwrap());
        });
    }

    #[test]
    fn rmd_fragments() {
        snapshot_content("fragments/rmd/*.Rmd", |_path, content| {
            assert_json_snapshot!(decode(&content).unwrap());
        });
    }
}
