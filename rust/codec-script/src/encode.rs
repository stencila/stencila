use codec::{
    common::{
        eyre::{bail, Result},
        serde_json,
    },
    stencila_schema::{Article, BlockContent, CodeChunk, For, If, InlineContent, Node, Parameter},
    EncodeOptions,
};
use codec_md::ToMd;
use common::itertools::Itertools;
use formats::Format;
use node_pointer::{walk, Visitor};

pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let mut options = options.unwrap_or_default();
    if options.max_width.is_none() {
        options.max_width = Some(100);
    }

    // Determine language and language-specific variables
    let lang = match &options.format {
        Some(format) => format.to_lowercase(),
        None => bail!("A format option (the programming language of the script) is required"),
    };
    let lang = formats::match_name(&lang);
    use Format::*;

    let comment_start = match lang {
        Bash | Python | R | Shell | Zsh => "# ",
        JavaScript => "// ",
        SQL => "-- ",
        _ => bail!(
            "No comment start defined for programming language `{}`",
            lang
        ),
    };

    let indentation = match lang {
        Python => "    ",
        _ => "  ",
    };

    let (for_supported, if_supported) = match lang {
        JavaScript | Python | R | Shell | Bash | Zsh => (true, true),
        _ => (false, false),
    };

    let empty_block = match lang {
        Python => "pass\n",
        Bash | Shell | Zsh => "true\n",
        _ => "", // Not necessary
    };

    let for_var = match lang {
        JavaScript => ("for$index", "const for$index = $expr;\n\n"),
        Python => ("for$index", "for$index = $expr\n\n"),
        R => ("for$index", "for$index = $expr\n\n"),
        Bash | Shell | Zsh => ("for$index", "for$index=$($expr)\n\n"),
        _ => ("", ""), // Not supported
    };
    let for_start = match lang {
        JavaScript => "for ($symbol of $expr) {\n\n",
        Python => "for $symbol in $expr:\n\n",
        R => "for ($symbol in $expr) {\n\n",
        Bash | Shell | Zsh => "for $symbol in $expr; do\n\n",
        _ => "", // Not supported
    };
    let for_end = match lang {
        JavaScript | R => "\n}\n\n",
        Python => "\n\n",
        Bash | Shell | Zsh => "\ndone\n\n",
        _ => "", // Not necessary or not supported
    };
    let for_otherwise_start = match lang {
        JavaScript => "if ($expr.length == 0) {\n\n",
        Python => "if len($expr) == 0:\n\n",
        R => "if (length($expr) == 0) {\n\n",
        Bash | Shell | Zsh => "if [ $expr ]; then\n\n",
        _ => "", // Not supported
    };
    let for_otherwise_end = match lang {
        JavaScript | R => "\n}\n\n",
        Python => "\n\n",
        Bash | Shell | Zsh => "\nfi\n\n",
        _ => "", // Not necessary or not supported
    };

    let if_start = match lang {
        JavaScript => "if ($expr) {\n\n",
        Python => "if $expr:\n\n",
        R => "if ($expr) {\n\n",
        Bash | Shell | Zsh => "if [ $expr ]; then\n\n",
        _ => "", // Not supported
    };
    let if_alternative = match lang {
        JavaScript | R => "\n\n} else if ($expr) {\n\n",
        Python => "\nelif $expr:\n\n",
        Bash | Shell | Zsh => "\nelif [ $expr ]; then\n\n",
        _ => "", // Not supported
    };
    let if_otherwise = match lang {
        JavaScript | R => "\n} else {\n\n",
        Python => "\nelse:\n\n",
        Bash | Shell | Zsh => "\nelse\n\n",
        _ => "", // Not supported
    };
    let if_end = match lang {
        JavaScript | R => "\n}\n\n",
        Python => "\n\n",
        Bash | Shell | Zsh => "\nfi\n\n",
        _ => "", // Not supported
    };

    let params_prelude = match lang {
            JavaScript => "// @skip\nconst $param = (type, index, def) => (type === 'string' ? String : JSON.parse)(process.argv[2 + index] || def)\n\n",
            Python =>"# @skip\ndef __param__(type, index, default): import sys, json; return (str if type == 'string' else json.loads)(sys.argv[1 + index] if len(sys.argv) > index + 1 else default)\n\n",
            R =>"# @skip\nparam__ <- function(type, index, def) { argv <- commandArgs(trailingOnly=TRUE); ifelse(type == 'string', identity, jsonlite::fromJSON)(ifelse(length(argv) > index + 1, argv[1 + index], def)) }\n\n",
            _ => "", // Not supported
        };
    let param_template = match lang {
        Bash | Shell | Zsh => "$name=${1:-$default}\n\n",
        JavaScript => "let $name = $param('$type', $index, $default);\n\n",
        Python => "$name = __param__('$type', $index, $default)\n\n",
        R => "$name = param__('$type', $index, $default)\n\n",
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
        let mut inner = encode(
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
