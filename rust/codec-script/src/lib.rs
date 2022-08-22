use codec::{
    common::{
        eyre::{bail, Result},
        regex::Regex,
    },
    stencila_schema::{Article, BlockContent, CodeChunk, Node},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_md::ToMd;
use common::itertools::Itertools;

// A codec for programming language scripts
pub struct ScriptCodec;

impl CodecTrait for ScriptCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["bash", "js", "py", "r", "sh", "zsh"],
            root_types: vec_string!["Article"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        let options = options.unwrap_or_default();
        let lang = match options.format {
            Some(format) => format.to_lowercase(),
            None => bail!("A format option (the programming language of the script) is required"),
        };

        // Define single line comment regexes for each langage
        let single_line_regex = Regex::new(match lang.as_str() {
            "js" => r"^//\s*(.*)$",
            "bash" | "py" | "r" | "sh" | "zsh" => r"^#\s*(.*)$",
            _ => bail!("Unhandled programming language `{}`", lang),
        })
        .expect("Regex should compile");

        // Define multi-line block comment regexes (begin, mid, end)
        let multi_line_regexes = match lang.as_str() {
            "js" => Some((r"^/\*+\s*(.*)$", r"^\s*\*?\s*(.*)$", r"^\s*(.*?)\*+/$")),
            _ => None,
        }
        .map(|regexes| {
            (
                Regex::new(regexes.0).expect("Regex should compile"),
                Regex::new(regexes.1).expect("Regex should compile"),
                Regex::new(regexes.2).expect("Regex should compile"),
            )
        });

        // Split into lines and classify each as either Markdown or code
        // using the regexes
        let mut blocks = Vec::new();
        let md_blocks = |md: &str| -> Vec<BlockContent> {
            codec_md::decode_fragment(md, Some(lang.to_string()))
        };
        let code_block = |code: &str| -> BlockContent {
            BlockContent::CodeChunk(CodeChunk {
                programming_language: lang.clone(),
                text: code.trim().to_string(),
                ..Default::default()
            })
        };
        let mut in_multiline = false;
        let mut md = String::new();
        let mut code = String::new();
        for line in str.lines() {
            if let Some((start_regex, mid_regex, end_regex)) = &multi_line_regexes {
                if in_multiline {
                    let line_md = if let Some(captures) = end_regex.captures(line) {
                        in_multiline = false;
                        captures[1].to_string()
                    } else if let Some(captures) = mid_regex.captures(line) {
                        captures[1].to_string()
                    } else {
                        line.to_string()
                    };
                    md.push_str(&line_md);
                    md.push('\n');
                    continue;
                } else if let Some(captures) = start_regex.captures(line) {
                    if !code.is_empty() {
                        blocks.push(code_block(&code));
                        code.clear();
                    }

                    in_multiline = true;
                    md.push_str(&captures[1]);
                    md.push('\n');
                    continue;
                } else {
                    in_multiline = false;
                }
            }

            // Either add the line to Markdown or to code and if switching between them then
            // add to blocks and clear the buffer.
            if let Some(captures) = single_line_regex.captures(line) {
                if !code.is_empty() {
                    blocks.push(code_block(&code));
                    code.clear();
                }

                md.push_str(&captures[1]);
                md.push('\n');
            } else {
                if !md.is_empty() {
                    blocks.append(&mut md_blocks(&md));
                    md.clear();
                }

                code.push_str(line);
                code.push('\n');
            }
        }

        // Any remaining code to add?
        if !code.is_empty() {
            blocks.push(code_block(&code));
        }

        // Any remaining Markdown to add?
        if !md.is_empty() {
            blocks.append(&mut md_blocks(&md));
        }

        Ok(Node::Article(Article {
            content: Some(blocks),
            ..Default::default()
        }))
    }

    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let options = options.unwrap_or_default();

        let blocks = match node {
            Node::Article(Article { content, .. }) => match content {
                Some(blocks) => blocks,
                None => return Ok(String::new()),
            },
            _ => bail!("Unhandled node type `{}`", node.as_ref()),
        };

        let lang = match options.format {
            Some(format) => format.to_lowercase(),
            None => bail!("A format option (the programming language of the script) is required"),
        };
        let comment_start = match lang.as_str() {
            "bash" | "py" | "r" | "sh" | "zsh" => "# ",
            "js" => "// ",
            _ => bail!("Unhandled programming language `{}`", lang),
        };

        // Iterate over blocks, adding `CodeChunk`s as code, and everything else, as Markdown comments
        let mut script = String::new();
        let mut comment_blocks = Vec::new();
        let blocks_to_comment = |blocks: &Vec<&BlockContent>| -> String {
            blocks
                .iter()
                .map(|block| block.to_md().trim_end().to_string())
                .join("\n\n")
                .lines()
                .map(|line| [comment_start, line].concat())
                .join("\n")
        };
        for block in blocks {
            if let BlockContent::CodeChunk(CodeChunk { text, .. }) = block {
                if !comment_blocks.is_empty() {
                    script.push_str(&blocks_to_comment(&comment_blocks));
                    script.push_str("\n\n");

                    comment_blocks.clear();
                }
                script.push_str(text);

                if text.ends_with('\n') {
                    script.push('\n');
                } else {
                    script.push_str("\n\n");
                }
            } else {
                comment_blocks.push(block)
            }
        }

        if !comment_blocks.is_empty() {
            script.push_str(&blocks_to_comment(&comment_blocks))
        }

        Ok(script.trim_end().to_string() + "\n")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use test_snaps::{
        insta::{assert_json_snapshot, assert_snapshot},
        snapshot_fixtures_path_content,
    };

    #[test]
    fn decode_and_encode_articles() {
        snapshot_fixtures_path_content("articles/scripts/*", |path: &Path, content| {
            let format = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let article = ScriptCodec::from_str(
                content,
                Some(DecodeOptions {
                    format: Some(format.clone()),
                }),
            )
            .unwrap();
            assert_json_snapshot!(article);

            let script = ScriptCodec::to_string(
                &article,
                Some(EncodeOptions {
                    format: Some(format),
                    ..Default::default()
                }),
            )
            .unwrap();
            assert_snapshot!(script);
        });
    }
}
