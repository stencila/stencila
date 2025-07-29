use std::{collections::HashMap, ops::Range};

use markdown::{
    ParseOptions,
    mdast::{self, Root},
    to_mdast,
    unist::Position,
};

use codec::{
    DecodeInfo, DecodeOptions, Losses, Mapping,
    common::{
        eyre::{Result, eyre},
        once_cell::sync::Lazy,
        regex::Regex,
    },
    format::Format,
    schema::{
        Article, Block, Chat, Inline, Node, NodeId, NodeType, Null, Prompt, VisitorMut, WalkControl,
    },
};

use self::{blocks::mds_to_blocks, inlines::mds_to_inlines};

mod blocks;
mod check;
mod frontmatter;
mod inlines;
mod shared;

pub use frontmatter::frontmatter as decode_frontmatter;

/// Decode a Markdown string to a Stencila Schema [`Node`]
pub fn decode(content: &str, options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let format = options
        .as_ref()
        .and_then(|options| options.format.clone())
        .unwrap_or(Format::Smd); // Default to Stencila Markdown

    // Check the content and return early if any messages and in strict mode
    let messages = check::check(content, &format);
    if !messages.is_empty() {
        let strict = options
            .and_then(|options| options.strict)
            .unwrap_or_default();
        if strict {
            return Ok((
                Node::Null(Null),
                DecodeInfo {
                    messages,
                    ..Default::default()
                },
            ));
        }
    }

    // Do any necessary pre-processing of Markdown
    let md = match format {
        Format::Myst => preprocess_myst(content),
        _ => preprocess(content),
    };

    // Parse Markdown to a MDAST root node and get its children
    let (children, position) =
        match to_mdast(&md, &parse_options()).map_err(|error| eyre!(error))? {
            mdast::Node::Root(mdast::Root { children, position }) => (children, position),
            _ => (Vec::new(), None),
        };

    // Transform MDAST to blocks
    let mut context = Context::new(format);
    let content = mds_to_blocks(children, &mut context);

    // Decode frontmatter (which may have a `type`, but defaults to `Article`)
    let mut node = if let Some(yaml) = context.yaml.take() {
        match decode_frontmatter(&yaml, None).0 {
            Node::Article(rest) => Node::Article(Article {
                content,
                frontmatter: Some(yaml),
                ..rest
            }),
            Node::Prompt(rest) => Node::Prompt(Prompt {
                content,
                frontmatter: Some(yaml),
                ..rest
            }),
            Node::Chat(rest) => Node::Chat(Chat { content, ..rest }),
            _ => Node::Article(Article {
                frontmatter: Some(yaml),
                content,
                ..Default::default()
            }),
        }
    } else {
        Node::Article(Article::new(content))
    };

    // Link footnotes to the `Note` inlines in the content
    if !context.footnotes.is_empty() {
        context.walk(&mut node);
    }

    // Map the position of the root node
    context.map_position(&position, node.node_type(), node.node_id());

    let info = DecodeInfo {
        messages,
        losses: context.losses,
        mapping: context.mapping,
    };

    Ok((node, info))
}

/// Decode a Markdown string to blocks
///
/// Because this is parsing a standalone fragment of Markdown, and the `to_mdast` function,
/// will ignore any footnote references that do not have a corresponding footnote content,
/// it is necessary to add fake footnote content to the Markdown before parsing.
fn decode_blocks(md: &str, context: &mut Context) -> Vec<Block> {
    static FOOTNOTE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\[\^\w+\]").expect("invalid regex"));

    let captures = FOOTNOTE_REGEX.captures_iter(md);

    let mut md = md.to_string();
    for capture in captures {
        md.push_str("\n\n");
        md.push_str(&capture[0]);
        md.push_str(":\n\n");
    }

    match to_mdast(&md, &parse_options()) {
        Ok(mdast::Node::Root(Root { children, .. })) => mds_to_blocks(children, context),
        _ => vec![],
    }
}

/// Decode a string to inlines
fn decode_inlines(md: &str, context: &mut Context) -> Vec<Inline> {
    match to_mdast(md, &parse_options()) {
        Ok(mdast::Node::Root(Root { children, .. })) => {
            if let Some(mdast::Node::Paragraph(mdast::Paragraph { children, .. })) =
                children.first()
            {
                mds_to_inlines(children.clone(), context)
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Preprocess Markdown
pub fn preprocess(input: &str) -> String {
    let mut output = String::new();

    let mut empty_line_needed = false;
    let mut html_tag = None;
    let mut in_math_block = false;
    for line in input.lines() {
        // Wrap certain top level HTML tags in `RawBlock`s
        if line.starts_with("<") && line.ends_with(">") {
            if let Some(tag) = html_tag {
                if line.starts_with(&["</", tag].concat()) {
                    html_tag = None;

                    output.push_str(line);
                    output.push_str("\n``````````\n\n");
                    continue;
                }
            } else if line == "<hr>" {
                output.push_str("***\n\n");
                continue;
            } else {
                if line.starts_with("<div") {
                    html_tag = Some("div")
                } else if line.starts_with("<table") {
                    html_tag = Some("table")
                } else if line.starts_with("<details") {
                    html_tag = Some("details")
                }

                if html_tag.is_some() {
                    output.push_str("``````````html raw\n");
                    output.push_str(line);
                    output.push('\n');
                    continue;
                }
            }
        }

        // If the previous line needs an empty line after it ensure that
        if empty_line_needed && !line.is_empty() {
            output.push('\n');
        }

        let in_special = in_math_block || html_tag.is_some();

        if !in_special && line.starts_with(":::") {
            // Ensure that there is an empty line before this line but
            // not if this is at the start of the document
            if !output.is_empty() && !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
            // Signal that an empty line is required before any following line
            empty_line_needed = true;
        } else {
            empty_line_needed = false;
        }

        // Convert LaTeX style math blocks and inlines. This is done because LLMs often
        // use LaTeX style delimiters when not prompted otherwise. This may be put behind
        // an option if found to interfere with user expectations.
        let trimmed = line.trim();

        let line = if !in_special && trimmed == r"\[" {
            in_math_block = true;
            "$$".to_string()
        } else if in_math_block && trimmed == r"\]" {
            in_math_block = false;
            "$$".to_string()
        } else if !in_special {
            line.replace(r"\(", r"$").replace(r"\)", r"$")
        } else {
            line.to_string()
        };

        output.push_str(&line);
        output.push('\n');
    }

    output
}

/// Convert MyST colon-fenced directives to backtick-fenced directives
///
/// This conversion allows for more straightforward decoding in subsequent
/// decoding steps because all MyST directives become code blocks.
fn preprocess_myst(myst: &str) -> String {
    fn colons_to_backticks(line: &str) -> String {
        let chars = line.chars();

        // Count the number of leading colons
        let mut colons = 0;
        for c in chars {
            if c == ':' {
                colons += 1
            } else {
                break;
            }
        }

        // Replace colons with backticks such that there are more
        // backticks when there are fewer colons (for descending nesting).
        // Assumes no more than 30 colons.
        let backticks = 30usize.saturating_sub(colons);
        let backticks = "`".repeat(backticks);

        [&backticks, &line[colons..]].concat()
    }

    let mut md = String::new();
    let mut depth = 0;
    for line in myst.lines() {
        if line.starts_with(":::")
            && line.contains(":::{")
            && !(line.contains(":::{if}")
                || line.contains(":::{elif}")
                || line.contains(":::{else}")
                || line.contains(":::{for}"))
        {
            depth += 1;
            md.push_str(&colons_to_backticks(line));
        } else if depth > 0 && line.starts_with(":::") {
            depth -= 1;
            md.push_str(&colons_to_backticks(line));
        } else {
            md.push_str(line);
        };
        md.push('\n');
    }

    md
}

/// Markdown parsing options
fn parse_options() -> ParseOptions {
    let mut options = ParseOptions::gfm();
    options.constructs.frontmatter = true;

    // Enable block math
    options.constructs.math_flow = true;

    // Do not parse inline code since we have a custom parser for that
    options.constructs.code_text = false;

    // Do not parse inline math since we have a custom parser for that
    // to avoid clashes with dollars inside code
    options.math_text_single_dollar = false;
    options.constructs.math_text = false;

    // Do not parse GFM single strikethrough since we use that for subscripts
    options.constructs.gfm_strikethrough = false;

    // Do not parse GFM autolinks because this interferes with our parsing
    // of <a> and <img> HTML tags. Instead we implement that separately.
    options.constructs.gfm_autolink_literal = false;

    // Do not handle embedded HTML, instead parse manually
    options.constructs.html_text = false;
    options.constructs.html_flow = false;

    options
}

#[derive(Default)]
struct Context {
    /// The format being decoded
    format: Format,

    /// YAML frontmatter
    yaml: Option<String>,

    /// Footnote content
    footnotes: HashMap<String, Vec<Block>>,

    /// Losses during decoding
    losses: Losses,

    /// Position-to-node mapping
    mapping: Mapping,

    /// Start positions of nodes
    map_stack: Vec<(usize, NodeType, NodeId)>,

    /// Preserve newlines in paragraphs
    ///
    /// By default newlines in paragraphs are converted to a single space.
    /// But in admonitions, for correct parsing, they need to be retained.
    preserve_newlines: bool,
}

impl Context {
    fn new(format: Format) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Store footnote content so that it can be assigned to the footnote itself later
    fn footnote(&mut self, id: String, blocks: Vec<Block>) {
        self.footnotes.insert(id, blocks);
    }

    /// Record the loss of a MDAST type
    fn lost(&mut self, label: &str) {
        self.losses.add(label)
    }

    /// Map the position of a node in the source using an optional (but normally present) `mdast::Position`
    fn map_position(
        &mut self,
        position: &Option<Position>,
        node_type: NodeType,
        node_id: Option<NodeId>,
    ) {
        if let (Some(position), Some(node_id)) = (position, node_id) {
            self.mapping.add(
                position.start.offset,
                position.end.offset,
                node_type,
                node_id,
                None,
                None,
            );
        }
    }

    /// Map the position of a node in the source using a `Range<usize>` span
    fn map_span(&mut self, span: Range<usize>, node_type: NodeType, node_id: Option<NodeId>) {
        if let Some(node_id) = node_id {
            self.mapping
                .add(span.start, span.end, node_type, node_id, None, None);
        }
    }

    /// Record the start position of a node in the source
    fn map_start(&mut self, offset: usize, node_type: NodeType, node_id: NodeId) {
        self.map_stack.push((offset, node_type, node_id));
    }

    /// Map the position of a node in the source using the previously stored start
    fn map_end(&mut self, end: usize) {
        if let Some((start, node_type, node_id)) = self.map_stack.pop() {
            self.mapping.add(start, end, node_type, node_id, None, None);
        }
    }

    /// Replace an entry in the mapping
    fn map_replace(&mut self, node_id: NodeId, new_node_type: NodeType, new_node_id: NodeId) {
        self.mapping.replace(node_id, new_node_type, new_node_id);
    }

    /// Extend an entry in the mapping to the end range of another
    fn map_extend(&mut self, first_node_id: NodeId, second_node_id: NodeId) {
        self.mapping.extend(first_node_id, second_node_id);
    }

    /// Remove an entry for a node id from the mapping
    fn map_remove(&mut self, node_id: NodeId) {
        self.mapping.remove(node_id);
    }
}

impl VisitorMut for Context {
    /// Apply footnote content to inline notes
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::Note(note) = inline {
            if let Some(id) = note.id.take() {
                if let Some(content) = self.footnotes.remove(&id) {
                    note.content = content;
                }
            }
        }

        WalkControl::Continue
    }
}
