use codec::{
    common::{eyre::Result, once_cell::sync::Lazy, regex::Regex},
    format::Format,
    schema::{Article, Block, CodeChunk, Node, RawBlock, Section},
    DecodeInfo,
};

/// Decode LaTeX into an [`Article`] with only [`RawBlock`]s and executable block types
pub(super) fn coarse(latex: &str) -> Result<(Node, DecodeInfo)> {
    Ok((
        Node::Article(Article::new(latex_to_blocks(latex))),
        DecodeInfo::none(),
    ))
}

/// Decode LaTeX into a vector of "coarse" [`Block`]s
fn latex_to_blocks(latex: &str) -> Vec<Block> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?sx)
            \\expr\{(?P<expr>[^}]*)\}

          | \\begin\{chunk\} \s*
              (?:\[(?P<chunk_opts>[^\]]*?)\])? \s* 
              (?P<chunk>.*?)
            \\end\{chunk\}

          | \\begin\{island\}
              (?P<island>.*?)
            \\end\{island\}
        ",
        )
        .expect("invalid regex")
    });

    let mut blocks = Vec::new();
    let mut cursor = 0;

    for captures in RE.captures_iter(latex) {
        let m = captures.get(0).expect("always present");
        if m.start() > cursor {
            blocks.push(Block::RawBlock(RawBlock::new(
                Format::Latex.to_string(),
                latex[cursor..m.start()].into(),
            )));
        }

        if let Some(mat) = captures.name("expr").or(captures.name("chunk")) {
            let code = mat.as_str().into();

            let mut programming_language = None;
            let mut is_echoed = None;
            let mut is_hidden = None;
            if let Some(options) = captures.name("chunk_opts") {
                for option in options
                    .as_str()
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    if option == "hide" {
                        is_hidden = Some(true);
                    } else if option == "echo" {
                        is_echoed = Some(true);
                    } else if programming_language.is_none() {
                        programming_language = Some(option.to_string());
                    }
                }
            }

            blocks.push(Block::CodeChunk(CodeChunk {
                programming_language,
                is_hidden,
                is_echoed,
                code,
                ..Default::default()
            }));
        } else if let Some(mat) = captures.name("island") {
            blocks.push(Block::Section(Section::new(latex_to_blocks(mat.as_str()))));
        }

        cursor = m.end();
    }

    if cursor < latex.len() {
        blocks.push(Block::RawBlock(RawBlock::new(
            Format::Latex.to_string(),
            latex[cursor..].into(),
        )));
    }

    blocks
}
