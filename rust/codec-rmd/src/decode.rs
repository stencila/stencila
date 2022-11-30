use codec::{common::eyre::Result, stencila_schema::*, CodecTrait, DecodeOptions};
use codec_md::MdCodec;

const LANGUAGES: &[&str] = &["r", "py", "python", "js", "javascript"];

/// Decode a R Markdown document to a `Node`
pub fn decode(input: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let mut node = MdCodec::from_str(input, options)?;
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
                code,
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
                        code: code.to_string(),
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
            InlineContent::CodeFragment(CodeFragment { code, .. }) => {
                for lang in LANGUAGES {
                    if let Some(code) = code.strip_prefix(&[lang, " "].concat()) {
                        *inline = InlineContent::CodeExpression(CodeExpression {
                            programming_language: lang.to_string(),
                            code: code.to_string(),
                            ..Default::default()
                        });
                        break;
                    }
                }
            }
            // Recursively transform other inlines
            InlineContent::Strikeout(Strikeout { content, .. })
            | InlineContent::Emphasis(Emphasis { content, .. })
            | InlineContent::Subscript(Subscript { content, .. })
            | InlineContent::Superscript(Superscript { content, .. })
            | InlineContent::Strong(Strong { content, .. })
            | InlineContent::Underline(Underline { content, .. }) => transform_inlines(content),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::{insta::assert_json_snapshot, snapshot_fixtures_content};

    #[test]
    fn decode_rmd_articles() {
        snapshot_fixtures_content("articles/*.Rmd", |content| {
            assert_json_snapshot!(decode(content, None).unwrap());
        });
    }

    #[test]
    fn decode_rmd_fragments() {
        snapshot_fixtures_content("fragments/rmd/*.Rmd", |content| {
            assert_json_snapshot!(decode(content, None).unwrap());
        });
    }
}
