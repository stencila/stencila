use codec::{
    common::{
        eyre::{bail, Result},
        regex::Regex,
        serde_json,
    },
    stencila_schema::{Article, BlockContent, CodeChunk, For, If, InlineContent, Node, Parameter},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_md::ToMd;
use common::itertools::Itertools;
use node_pointer::{walk, Visitor};

// A codec for programming language scripts
pub struct ScriptCodec;

impl CodecTrait for ScriptCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["bash", "js", "py", "r", "sh", "sql", "zsh"],
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

        // Define single line comment regexes for each language
        let single_line_regex = Regex::new(match lang.as_str() {
            "js" => r"^//\s*(.*)$",
            "sql" => r"^--\s*(.*)$",
            "bash" | "py" | "r" | "sh" | "zsh" => r"^#\s*(.*)$",
            _ => bail!("Unhandled programming language `{}`", lang),
        })
        .expect("Regex should compile");

        // Define multi-line block comment regexes (begin, mid, end)
        let multi_line_regexes = match lang.as_str() {
            "js" | "sql" => Some((r"^/\*+\s*(.*)$", r"^\s*\*?\s*(.*)$", r"^\s*(.*?)\*+/$")),
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
        let code_chunk = |code: &str| -> BlockContent {
            BlockContent::CodeChunk(CodeChunk {
                programming_language: lang.clone(),
                text: code.trim().to_string(),
                ..Default::default()
            })
        };
        let mut in_multiline = false;
        let mut md = String::new();
        let mut code = String::new();
        let mut skip = false;
        for line in str.lines() {
            if skip {
                skip = false;
                continue;
            }

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
                    if !code.trim().is_empty() {
                        blocks.push(code_chunk(&code));
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
                let content = captures[1].to_string();
                if content.starts_with("@ignore") {
                    continue;
                } else if content.starts_with("@skip") {
                    skip = true;
                    continue;
                }

                if !code.trim().is_empty() {
                    blocks.push(code_chunk(&code));
                    code.clear();
                }

                md.push_str(&content);
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
        if !code.trim().is_empty() {
            blocks.push(code_chunk(&code));
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
        let mut options = options.unwrap_or_default();
        if options.max_width.is_none() {
            options.max_width = Some(100);
        }

        // Determine language and language-specific variables
        let lang = match &options.format {
            Some(format) => format.to_lowercase(),
            None => bail!("A format option (the programming language of the script) is required"),
        };
        let lang = lang.as_str();

        let comment_start = match lang {
            "bash" | "py" | "r" | "sh" | "zsh" => "# ",
            "js" => "// ",
            "sql" => "-- ",
            _ => bail!(
                "No comment start defined for programming language `{}`",
                lang
            ),
        };

        let indentation = match lang {
            "py" => "    ",
            _ => "  ",
        };

        let (for_supported, if_supported) = match lang {
            "js" | "py" | "r" | "sh" | "bash" | "zsh" => (true, true),
            _ => (false, false),
        };

        let empty_block = match lang {
            "py" => "pass\n",
            "bash" | "sh" | "zsh" => "true\n",
            _ => "", // Not necessary
        };

        let for_var = match lang {
            "js" => ("for$index", "const for$index = $expr;\n\n"),
            "py" => ("for$index", "for$index = $expr\n\n"),
            "r" => ("for$index", "for$index = $expr;\n\n"),
            "bash" | "sh" | "zsh" => ("for$index", "for$index=$($expr)\n\n"),
            _ => ("", ""), // Not supported
        };
        let for_start = match lang {
            "js" => "for ($symbol of $expr) {\n\n",
            "py" => "for $symbol in $expr:\n\n",
            "r" => "for ($symbol in $expr) {\n\n",
            "bash" | "sh" | "zsh" => "for $symbol in $expr; do\n\n",
            _ => "", // Not supported
        };
        let for_end = match lang {
            "js" | "r" => "\n}\n\n",
            "py" => "\n\n",
            "bash" | "sh" | "zsh" => "\ndone\n\n",
            _ => "", // Not necessary or not supported
        };
        let for_otherwise_start = match lang {
            "js" => "if ($expr.length == 0) {\n\n",
            "py" => "if len($expr) == 0:\n\n",
            "r" => "if (length($expr) == 0) {\n\n",
            "bash" | "sh" | "zsh" => "if [ $expr ]; then\n\n",
            _ => "", // Not supported
        };
        let for_otherwise_end = match lang {
            "js" | "r" => "\n}\n\n",
            "py" => "\n\n",
            "bash" | "sh" | "zsh" => "\nfi\n\n",
            _ => "", // Not necessary or not supported
        };

        let if_start = match lang {
            "js" => "if ($expr) {\n\n",
            "py" => "if $expr:\n\n",
            "r" => "if ($expr) {\n\n",
            "bash" | "sh" | "zsh" => "if [ $expr ]; then\n\n",
            _ => "", // Not supported
        };
        let if_alternative = match lang {
            "js" | "r" => "\n\n} else if ($expr) {\n\n",
            "py" => "\nelif $expr:\n\n",
            "bash" | "sh" | "zsh" => "\nelif [ $expr ]; then\n\n",
            _ => "", // Not supported
        };
        let if_otherwise = match lang {
            "js" | "r" => "\n} else {\n\n",
            "py" => "\nelse:\n\n",
            "bash" | "sh" | "zsh" => "\nelse\n\n",
            _ => "", // Not supported
        };
        let if_end = match lang {
            "js" | "r" => "\n}\n\n",
            "py" => "\n\n",
            "bash" | "sh" | "zsh" => "\nfi\n\n",
            _ => "", // Not supported
        };

        let params_prelude = match lang {
            "js" => "// @skip\nconst $param = (type, index, def) => (type === 'string' ? String : JSON.parse)(process.argv[2 + index] || def)\n\n",
            "py" =>"# @skip\ndef __param__(type, index, default): import sys, json; return (str if type == 'string' else json.loads)(sys.argv[1 + index] if len(sys.argv) > index + 1 else default)\n\n",
            "r" =>"# @skip\nparam__ <- function(type, index, def) { argv <- commandArgs(trailingOnly=TRUE); ifelse(type == 'string', identity, jsonlite::fromJSON)(ifelse(length(argv) > index + 1, argv[1 + index], def)) }\n\n",
            _ => "", // Not supported
        };
        let param_template = match lang {
            "bash" | "sh" | "zsh" => "$name=${1:-$default}\n\n",
            "js" => "let $name = $param('$type', $index, $default);\n\n",
            "py" => "$name = __param__('$type', $index, $default)\n\n",
            "r" => "$name = param__('$type', $index, $default)\n\n",
            _ => "", // Not supported
        };

        // Get blocks, returning early if none
        let blocks = match node {
            Node::Article(Article { content, .. }) => match content {
                Some(blocks) => blocks,
                None => return Ok(String::new()),
            },
            _ => bail!("Unhandled node type `{}`", node.as_ref()),
        };

        let mut script = String::new();
        let mut code = String::new();

        let blocks_to_comment = |blocks: &Vec<&BlockContent>| -> String {
            blocks
                .iter()
                .map(|block| block.to_md(&options).trim_end().to_string())
                .join("\n\n")
                .lines()
                .map(|line| [comment_start, line].concat())
                .join("\n")
        };

        let blocks_to_code = |blocks: &Vec<BlockContent>| -> String {
            let mut inner = Self::to_string(
                &Node::Article(Article {
                    content: Some(blocks.clone()),
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(lang.to_string()),
                    ..options.clone()
                }),
            )
            .unwrap_or_default()
            .trim()
            .to_string();

            if !empty_block.is_empty() {
                if inner.is_empty() {
                    inner = empty_block.to_string();
                } else if !inner.lines().any(|line| !line.starts_with(comment_start)) {
                    inner.push_str("\n\n");
                    inner.push_str(empty_block);
                }
            }

            inner
        };

        let indent = |content: &str| -> String {
            content
                .lines()
                .map(|line| [indentation, line].concat())
                .join("\n")
        };

        // Iterate over blocks, adding `For`s, `If`s, `CodeChunk`s, `Parameter`s etc as code, and
        // everything else, as Markdown comments
        let mut comment_blocks = Vec::new();
        let mut params_preluded = false;
        let mut for_index = 0;
        for block in blocks {
            if matches!(block, BlockContent::For(..)) && for_supported
                || matches!(block, BlockContent::If(..)) && if_supported
                || matches!(block, BlockContent::CodeChunk(..))
            {
                if !comment_blocks.is_empty() {
                    script.push_str(&blocks_to_comment(&comment_blocks));
                    script.push_str("\n\n");

                    comment_blocks.clear();
                }

                if !code.is_empty() {
                    script.push_str(&code);
                    code.clear();
                }

                if let BlockContent::For(For {
                    symbol,
                    expression,
                    content,
                    otherwise,
                    ..
                }) = block
                {
                    for_index += 1;

                    let expr = if otherwise.is_some() && !for_var.0.is_empty() {
                        let index = for_index.to_string();
                        let assign = for_var
                            .1
                            .replace("$index", &index)
                            .replace("$expr", &expression.text);
                        code.push_str(&assign);
                        for_var.0.replace("$index", &index)
                    } else {
                        expression.text.clone()
                    };

                    code.push_str(&for_start.replace("$symbol", symbol).replace("$expr", &expr));
                    code.push_str(&indent(&blocks_to_code(content)));
                    code.push('\n');
                    code.push_str(for_end);

                    if let Some(otherwise) = otherwise {
                        code.push_str(&for_otherwise_start.replace("$expr", &expr));
                        code.push_str(&indent(&blocks_to_code(otherwise)));
                        code.push('\n');
                        code.push_str(for_otherwise_end);
                    }
                } else if let BlockContent::If(If {
                    condition,
                    content,
                    alternatives,
                    otherwise,
                    ..
                }) = block
                {
                    code.push_str(&if_start.replace("$expr", &condition.text));
                    code.push_str(&indent(&blocks_to_code(content)));
                    code.push('\n');

                    for alternative in alternatives.iter().flatten() {
                        let If {
                            condition, content, ..
                        } = alternative;
                        code.push_str(&if_alternative.replace("$expr", &condition.text));
                        code.push_str(&indent(&blocks_to_code(content)));
                        code.push('\n');
                    }

                    if let Some(otherwise) = otherwise {
                        code.push_str(if_otherwise);
                        code.push_str(&indent(&blocks_to_code(otherwise)));
                        code.push('\n');
                    }

                    code.push_str(if_end);
                } else if let BlockContent::CodeChunk(CodeChunk { text, .. }) = block {
                    script.push_str(text);

                    if text.ends_with('\n') {
                        script.push('\n');
                    } else {
                        script.push_str("\n\n");
                    }
                }
            } else {
                if !code.is_empty() {
                    script.push_str(&code);
                    code.clear();
                }

                comment_blocks.push(block);

                // If supported, get parameters and add a code section to instantiate them
                if !param_template.is_empty() {
                    let mut params = ParameterGetter::default();
                    walk(block, &mut params);
                    if !params.params.is_empty() {
                        if !params_preluded {
                            code += params_prelude;
                            params_preluded = true;
                        }

                        for (index, param) in params.params.into_iter().enumerate() {
                            let typ = param
                                .validator
                                .map(|validator| {
                                    validator
                                        .as_ref()
                                        .as_ref()
                                        .strip_suffix("Validator")
                                        .unwrap_or_default()
                                        .to_string()
                                        .to_lowercase()
                                })
                                .unwrap_or_else(|| "string".to_string());
                            let default = param
                                .default
                                .unwrap_or_else(|| Box::new(Node::String(String::new())));
                            let param_line = param_template
                                .replace("$name", &param.name)
                                .replace("$type", &typ)
                                .replace("$index", &index.to_string())
                                .replace(
                                    "$default",
                                    &serde_json::to_string(&default).unwrap_or_default(),
                                );
                            code += &[comment_start, "@skip\n", &param_line].concat();
                        }
                    }
                }
            }
        }

        if !comment_blocks.is_empty() {
            script.push_str(&blocks_to_comment(&comment_blocks));
            script.push_str("\n\n");
        }

        if !code.is_empty() {
            script.push_str(&code)
        }

        Ok(script.trim_end().to_string() + "\n")
    }
}

#[derive(Default)]
struct ParameterGetter {
    params: Vec<Parameter>,
}

impl Visitor for ParameterGetter {
    fn visit_inline(
        &mut self,
        _address: &node_pointer::Address,
        node: &codec::stencila_schema::InlineContent,
    ) -> bool {
        if let InlineContent::Parameter(param) = node {
            self.params.push(param.clone());
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use test_snaps::{
        insta::{assert_json_snapshot, assert_snapshot},
        snapshot_fixtures_path_content,
    };

    use super::*;

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
