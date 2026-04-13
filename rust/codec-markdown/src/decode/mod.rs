use std::{collections::HashMap, ops::Range, sync::LazyLock};

use markdown::{
    ParseOptions,
    mdast::{self, Root},
    to_mdast,
    unist::Position,
};
use regex::Regex;

use stencila_codec::{
    DecodeInfo, DecodeOptions, Losses, Mapping,
    eyre::{Result, eyre},
    stencila_format::Format,
    stencila_schema::{
        Agent, Article, Block, Chat, CodeBlock, CodeChunk, Comment, CommentOptions, Inline, Node,
        NodeId, NodeType, Null, Prompt, Skill, VisitorMut, WalkControl, Workflow,
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

    let node_type = options.as_ref().and_then(|options| options.node_type);

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

    // Extract comment definitions before MDAST parsing
    let (md, comment_definitions) = if matches!(format, Format::Smd) {
        extract_comment_definitions(&md)
    } else {
        (md, HashMap::new())
    };

    // Parse Markdown to a MDAST root node and get its children
    let (children, position) =
        match to_mdast(&md, &parse_options(&format)).map_err(|error| eyre!(error))? {
            mdast::Node::Root(mdast::Root { children, position }) => (children, position),
            _ => (Vec::new(), None),
        };

    // Transform MDAST to blocks
    let mut context = Context::new(format);
    context.comment_definitions = comment_definitions;
    let content = mds_to_blocks(children, &mut context);

    // Decode frontmatter (which may have a `type`, but defaults to `Article`)
    let mut node = if let Some(yaml) = context.yaml.take() {
        match decode_frontmatter(&yaml, node_type).0 {
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
            Node::Agent(rest) => {
                let mut agent = Agent {
                    frontmatter: Some(yaml),
                    ..rest
                };
                if !content.is_empty() {
                    agent.content = Some(content);
                }
                Node::Agent(agent)
            }
            Node::Workflow(rest) => {
                let pipeline = extract_dot_pipeline(&content);
                let mut workflow = Workflow {
                    frontmatter: Some(yaml),
                    ..rest
                };
                if !content.is_empty() {
                    workflow.content = Some(content);
                }
                if workflow.pipeline.is_none() {
                    workflow.pipeline = pipeline;
                }
                Node::Workflow(workflow)
            }
            Node::Skill(rest) => Node::Skill(Skill {
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

    // Build Comment nodes from extracted definitions and assign to article
    if !context.comment_definitions.is_empty()
        && let Node::Article(ref mut article) = node
    {
        let mut top_level: Vec<Comment> = Vec::new();
        let mut replies: Vec<(String, Comment)> = Vec::new();

        // Sort keys for deterministic ordering
        let mut ids: Vec<String> = context.comment_definitions.keys().cloned().collect();
        ids.sort();

        for id in ids {
            let content_md = context.comment_definitions.remove(&id).unwrap_or_default();
            let content = decode_blocks(&content_md, &mut context);
            let is_reply = id.contains('.');

            let comment = Comment {
                id: Some(id.clone()),
                content,
                options: Box::new(CommentOptions {
                    start_location: if !is_reply {
                        Some(format!("#comment-{id}-start"))
                    } else {
                        None
                    },
                    end_location: if !is_reply {
                        Some(format!("#comment-{id}-end"))
                    } else {
                        None
                    },
                    ..Default::default()
                }),
                ..Default::default()
            };

            if is_reply {
                let parent_id = id.rsplitn(2, '.').last().unwrap_or(&id).to_string();
                replies.push((parent_id, comment));
            } else {
                top_level.push(comment);
            }
        }

        // Nest replies into their parent comments
        for (parent_id, reply) in replies {
            nest_reply(&mut top_level, &parent_id, reply);
        }

        if !top_level.is_empty() {
            article.options.comments = Some(top_level);
        }
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
    static FOOTNOTE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[\^\w+\]").expect("invalid regex"));

    let captures = FOOTNOTE_REGEX.captures_iter(md);

    let mut md = md.to_string();
    for capture in captures {
        md.push_str("\n\n");
        md.push_str(&capture[0]);
        md.push_str(":\n\n");
    }

    match to_mdast(&md, &parse_options(&context.format)) {
        Ok(mdast::Node::Root(Root { children, .. })) => mds_to_blocks(children, context),
        _ => vec![],
    }
}

/// Decode a string to inlines
fn decode_inlines(md: &str, context: &mut Context) -> Vec<Inline> {
    match to_mdast(md, &parse_options(&context.format)) {
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

/// Extract the raw DOT source from the first ```dot code block/chunk in the content blocks.
fn extract_dot_pipeline(blocks: &[Block]) -> Option<String> {
    for block in blocks {
        let (lang, code) = match block {
            Block::CodeBlock(CodeBlock {
                programming_language: Some(lang),
                code,
                ..
            }) => (lang.as_str(), code),
            Block::CodeChunk(CodeChunk {
                programming_language: Some(lang),
                code,
                ..
            }) => (lang.as_str(), code),
            _ => continue,
        };
        if lang == "dot" {
            let source = code.to_string();
            if !source.is_empty() {
                return Some(source);
            }
        }
    }
    None
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
fn parse_options(format: &Format) -> ParseOptions {
    // Use GitHub Flavoured Markdown with the following differences
    let mut options = ParseOptions::gfm();

    // Enable frontmatter
    options.constructs.frontmatter = true;

    // Enable inline and block math
    options.constructs.math_text = true;
    options.constructs.math_flow = true;

    // Enable inline code so that backtick code spans take precedence
    // over dollar-sign math (e.g. `$code$` should be code, not math).
    // Custom handling of code attributes (e.g. `code`{python}), MyST roles,
    // and QMD code expressions is done in post-processing in `mds_to_inlines`.
    options.constructs.code_text = true;

    // Do not enable indented code blocks for Stencila Markdown to avoid
    // conflict with indentation within fenced divs
    options.constructs.code_indented = !matches!(format, Format::Smd);

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

    /// Comment definitions extracted during preprocessing
    comment_definitions: HashMap<String, String>,

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
        if let Inline::Note(note) = inline
            && let Some(id) = note.id.take()
            && let Some(content) = self.footnotes.remove(&id)
        {
            note.content = content;
        }

        WalkControl::Continue
    }
}

/// Extract comment definitions from markdown text
///
/// Finds `[>>id]: content` blocks (similar to footnote definitions) and
/// removes them from the markdown, returning the cleaned text and a map
/// of comment id to raw markdown content.
fn extract_comment_definitions(input: &str) -> (String, HashMap<String, String>) {
    static COMMENT_DEF_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\[>>([a-zA-Z0-9._-]+)\]:\s*(.*)$").expect("invalid regex"));

    let mut cleaned = String::new();
    let mut defs: HashMap<String, String> = HashMap::new();
    let mut current_id: Option<String> = None;
    let mut current_content = String::new();
    let mut in_fenced_code = false;

    for line in input.lines() {
        // Track fenced code blocks to avoid matching inside them
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fenced_code = !in_fenced_code;
        }

        if !in_fenced_code {
            if let Some(caps) = COMMENT_DEF_REGEX.captures(line) {
                // Flush previous definition
                if let Some(id) = current_id.take() {
                    defs.insert(id, current_content.trim().to_string());
                    current_content.clear();
                }
                current_id = Some(caps[1].to_string());
                let first_line = &caps[2];
                if !first_line.is_empty() {
                    current_content.push_str(first_line);
                    current_content.push('\n');
                }
                continue;
            }

            if current_id.is_some() {
                if line.starts_with("    ") {
                    // Continuation line (4-space indent)
                    current_content.push_str(&line[4..]);
                    current_content.push('\n');
                    continue;
                } else if line.trim().is_empty() {
                    // Blank line within definition (paragraph separator)
                    current_content.push('\n');
                    continue;
                } else {
                    // Non-continuation line: flush the definition
                    if let Some(id) = current_id.take() {
                        defs.insert(id, current_content.trim().to_string());
                        current_content.clear();
                    }
                }
            }
        }

        cleaned.push_str(line);
        cleaned.push('\n');
    }

    // Flush final definition
    if let Some(id) = current_id {
        defs.insert(id, current_content.trim().to_string());
    }

    (cleaned, defs)
}

/// Nest a reply comment into its parent's `comments` array
fn nest_reply(comments: &mut [Comment], parent_id: &str, reply: Comment) {
    for comment in comments.iter_mut() {
        if comment.id.as_deref() == Some(parent_id) {
            let replies = comment.options.comments.get_or_insert_with(Vec::new);
            replies.push(reply);
            return;
        }
        // Search nested replies
        if let Some(ref mut nested) = comment.options.comments {
            nest_reply(nested, parent_id, reply.clone());
        }
    }
}
