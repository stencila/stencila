use codec::{
    common::{
        eyre::{bail, Result},
        regex::Regex,
    },
    stencila_schema::{Article, BlockContent, CodeChunk, Node},
    DecodeOptions,
};

use formats::Format;

pub fn decode(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let options = options.unwrap_or_default();
    let lang = match options.format {
        Some(format) => format.to_lowercase(),
        None => bail!("A format option (the programming language of the script) is required"),
    };

    let lang = formats::match_name(&lang);
    use Format::*;

    // Define single line comment regexes for each language
    let single_line_regex = Regex::new(match lang {
        JavaScript => r"^//\s*(.*)$",
        SQL => r"^--\s*(.*)$",
        Bash | Python | R | Shell | Zsh => r"^#\s*(.*)$",
        _ => bail!("Unhandled programming language `{}`", lang),
    })
    .expect("Regex should compile");

    // Define multi-line block comment regexes (begin, mid, end)
    let multi_line_regexes = match lang {
        JavaScript | SQL => Some((r"^/\*+\s*(.*)$", r"^\s*\*?\s*(.*)$", r"^\s*(.*?)\*+/$")),
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
    let md_blocks =
        |md: &str| -> Vec<BlockContent> { codec_md::decode_fragment(md, Some(lang.to_string())) };
    let code_chunk = |code: &str| -> BlockContent {
        BlockContent::CodeChunk(CodeChunk {
            programming_language: lang.to_string().to_lowercase(),
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
