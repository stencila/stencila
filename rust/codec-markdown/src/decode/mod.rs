use std::{collections::HashMap, ops::Range};

use markdown::{
    mdast::{self, Root},
    to_mdast,
    unist::Position,
    ParseOptions,
};

use codec::{
    common::{
        eyre::{eyre, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        serde_json::{self, json},
        serde_yaml, tracing,
    },
    format::Format,
    schema::{
        Article, Block, Chat, Inline, Node, NodeId, NodeType, Null, Prompt, VisitorMut, WalkControl,
    },
    DecodeInfo, DecodeOptions, Losses, Mapping,
};

use self::{blocks::mds_to_blocks, inlines::mds_to_inlines};

mod blocks;
mod check;
mod inlines;
mod shared;

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
        Format::Myst => myst_to_md(content),
        Format::Qmd => qmd_to_md(content),
        _ => preprocess_md(content),
    };

    // Parse Markdown to a MDAST root node and get its children
    let (children, position) =
        match to_mdast(&md, &parse_options()).map_err(|error| eyre!(error))? {
            mdast::Node::Root(mdast::Root { children, position }) => (children, position),
            _ => (Vec::new(), None),
        };

    // Transform MDAST to blocks
    let mut context = Context::new(format);
    let content = blocks::mds_to_blocks(children, &mut context);

    // Decode frontmatter (which may have a `type`, but defaults to `Article`)
    let frontmatter = context.frontmatter();
    let mut node = if let Some(Node::Article(rest)) = frontmatter {
        Node::Article(Article { content, ..rest })
    } else if let Some(Node::Prompt(rest)) = frontmatter {
        Node::Prompt(Prompt { content, ..rest })
    } else if let Some(Node::Chat(rest)) = frontmatter {
        Node::Chat(Chat { content, ..rest })
    } else {
        Node::Article(Article::new(content))
    };

    // Link footnotes to the `Note` inlines in the content
    if !context.footnotes.is_empty() {
        context.visit(&mut node);
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
///
/// See issue #2438 for why this is necessary.
fn preprocess_md(input: &str) -> String {
    let mut output = String::new();

    let mut empty_line_needed = false;
    for line in input.lines() {
        if empty_line_needed && !line.is_empty() {
            output.push('\n');
        }

        output.push_str(line);
        output.push('\n');

        empty_line_needed = line.starts_with(":::")
            && (line.trim_end().ends_with(":::") || line.trim_end().ends_with(">>>"));
    }

    output
}

/// Convert MyST colon-fenced directives to backtick-fenced directives
///
/// This conversion allows for more straightforward decoding in subsequent
/// decoding steps because all MyST directives become code blocks.
fn myst_to_md(myst: &str) -> String {
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

/// Convert QMD to Markdown parsable by the main parser
///
/// Ensures all lines starting with `:::` are surrounded by a blank line.
fn qmd_to_md(input: &str) -> String {
    let mut output = String::new();

    let mut empty_line_needed = false;
    for line in input.lines() {
        if empty_line_needed && !line.is_empty() {
            output.push('\n');
        }

        let colons = line.starts_with(":::");

        if colons {
            if !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
            empty_line_needed = true;
        } else {
            empty_line_needed = false;
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

/// Markdown parsing options
fn parse_options() -> ParseOptions {
    let mut options = ParseOptions::gfm();
    options.constructs.frontmatter = true;

    // Do not parse inline code since we have a custom parser for that
    options.constructs.code_text = false;

    // Do not parse inline math since we have a custom parser for that
    // to avoid clashes with dollars inside code
    options.math_text_single_dollar = false;
    options.constructs.math_text = false;

    // Do not parse GFM single strikethrough since we use that for subscripts
    options.constructs.gfm_strikethrough = false;

    // Enable block math
    options.constructs.math_flow = true;

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

    /// Parse any YAML frontmatter
    fn frontmatter(&mut self) -> Option<Node> {
        let yaml = self.yaml.as_ref()?;

        // Deserialize YAML to a value, and add `type` properties if necessary
        let mut value = match serde_yaml::from_str(yaml) {
            Ok(serde_json::Value::Object(mut value)) => {
                if let Some(typ) = value.get("type").and_then(|typ| typ.as_str()) {
                    // Ensure that `content` is present for types that require it, so that
                    // `serde_json::from_value` succeeds
                    if matches!(typ, "Article" | "Prompt" | "Chat")
                        && value.get("content").is_none()
                    {
                        value.insert("content".to_string(), json!([]));
                    }
                } else {
                    value.insert("type".into(), json!("Article"));
                    value.insert("content".into(), json!([]));
                }

                if let Some(model) = value
                    .get_mut("model")
                    .and_then(|model: &mut serde_json::Value| model.as_object_mut())
                {
                    // Ensure that `model` has `type: InstructionModel`
                    model.insert("type".into(), json!("InstructionModel"));
                }

                if let Some(config) = value
                    .get_mut("config")
                    .and_then(|config: &mut serde_json::Value| config.as_object_mut())
                {
                    // Ensure that `config` has `type: Config`
                    config.insert("type".into(), json!("Config"));
                }

                json!(value)
            }
            Ok(_) => {
                tracing::debug!("YAML frontmatter is not an object, will be ignored");
                return None;
            }
            Err(error) => {
                tracing::warn!(
                    "Error while parsing YAML frontmatter, will be ignored: {}",
                    error
                );
                return None;
            }
        };

        // Parse title and abstract as Markdown (need to do here before deserializing to node
        // and remove from value so does not cause an error when deserializing)
        let (title, abs) = if let Some(object) = value.as_object_mut() {
            let title = object
                .remove("title")
                .and_then(|value| value.as_str().map(String::from))
                .map(|title| decode_inlines(&title, self));
            let abs = object
                .remove("abstract")
                .and_then(|value| value.as_str().map(String::from))
                .map(|abs| decode_blocks(&abs, self));
            (title, abs)
        } else {
            (None, None)
        };

        // Deserialize to a `Node`: note that `type` is ensured to be present
        let Ok(mut node) = serde_json::from_value::<Node>(value) else {
            tracing::warn!("Error while parsing YAML frontmatter, will be ignored",);
            return None;
        };

        // Set title and abstract for node types that have them
        match &mut node {
            Node::Article(article) => {
                article.title = title;
                article.r#abstract = abs;
            }
            Node::Prompt(prompt) => {
                prompt.options.title = title;
                prompt.options.r#abstract = abs;
            }
            Node::Chat(chat) => {
                chat.title = title;
                chat.options.r#abstract = abs;
            }
            _ => {}
        }

        Some(node)
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
