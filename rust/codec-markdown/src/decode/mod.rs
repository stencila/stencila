use std::{collections::HashMap, ops::Range};

use markdown::{
    mdast::{self, Root},
    to_mdast,
    unist::Position,
    ParseOptions,
};

use codec::{
    common::{
        eyre::{bail, eyre, Result},
        serde_json, serde_yaml, tracing,
    },
    schema::{Article, Block, Inline, Node, NodeId, NodeType, VisitorMut, WalkControl},
    DecodeInfo, DecodeOptions, Losses, Mapping,
};

use self::{blocks::mds_to_blocks, inlines::mds_to_inlines};

mod blocks;
mod inlines;
mod shared;

/// Decode a Markdown string to a Stencila Schema [`Node`]
pub(super) fn decode(md: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let mdast = to_mdast(md, &parse_options()).map_err(|error| eyre!(error))?;

    let mut context = Context::default();

    let Some(mut node) = md_to_node(mdast, &mut context) else {
        bail!("No node decoded from Markdown")
    };

    if let Some(Node::Article(front)) = context.frontmatter() {
        if let Node::Article(body) = node {
            node = Node::Article(Article {
                content: body.content,
                ..front
            });
        }
    }

    if !context.footnotes.is_empty() {
        context.visit(&mut node);
    }

    Ok((node, context.info()))
}

/// Decode a string to blocks
fn decode_blocks(md: &str) -> Vec<Block> {
    let mut context = Context::default();
    match to_mdast(md, &parse_options()) {
        Ok(mdast::Node::Root(Root { children, .. })) => mds_to_blocks(children, &mut context),
        _ => vec![],
    }
}

/// Decode a string to inlines
fn decode_inlines(md: &str) -> Vec<Inline> {
    let mut context = Context::default();
    match to_mdast(md, &parse_options()) {
        Ok(mdast::Node::Root(Root { children, .. })) => {
            if let Some(mdast::Node::Paragraph(mdast::Paragraph { children, .. })) =
                children.first()
            {
                mds_to_inlines(children.clone(), &mut context)
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Markdown parsing options
fn parse_options() -> ParseOptions {
    let mut options = ParseOptions::gfm();
    options.constructs.frontmatter = true;
    // Do not parse inline code since we have a custom parser for that
    options.constructs.code_text = false;
    // Do not parse GFM single strikethrough
    options.constructs.gfm_strikethrough = false;
    // Enable math
    options.math_text_single_dollar = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    // Do not handle embedded HTML, instead parse manually
    options.constructs.html_text = false;
    options.constructs.html_flow = false;

    options
}

#[derive(Default)]
struct Context {
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
    fn frontmatter(&self) -> Option<Node> {
        let Some(yaml) = &self.yaml else {
            return None;
        };

        // Deserialize YAML to a value, and add `type: Article` if necessary
        let mut value = match serde_yaml::from_str(yaml) {
            Ok(serde_json::Value::Object(mut value)) => {
                if value.get("type").is_none() {
                    value.insert(
                        "type".to_string(),
                        serde_json::Value::String("Article".to_string()),
                    );
                    value.insert("content".to_string(), serde_json::Value::Array(vec![]));
                }
                serde_json::Value::Object(value)
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
                .map(|title| decode_inlines(&title));
            let abs = object
                .remove("abstract")
                .and_then(|value| value.as_str().map(String::from))
                .map(|abs| decode_blocks(&abs));
            (title, abs)
        } else {
            (None, None)
        };

        // Deserialize to a `Node` not that `type` is ensured to be present
        let Ok(mut node) = serde_json::from_value(value) else {
            tracing::warn!("Error while parsing YAML frontmatter, will be ignored",);
            return None;
        };

        // Set title and abstract if node is Article
        if let Node::Article(article) = &mut node {
            article.title = title;
            article.r#abstract = abs;
        }

        Some(node)
    }

    /// Take the decoding info for the context
    pub fn info(self) -> DecodeInfo {
        DecodeInfo {
            losses: self.losses,
            mapping: self.mapping,
        }
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

/// Transform a MDAST node to Stencila Schema node
fn md_to_node(md: mdast::Node, context: &mut Context) -> Option<Node> {
    Some(match md {
        mdast::Node::Root(mdast::Root { children, position }) => {
            let node = Article::new(blocks::mds_to_blocks(children, context));
            context.map_position(&position, node.node_type(), Some(node.node_id()));
            Node::Article(node)
        }
        _ => {
            context.lost("Node");
            return None;
        }
    })
}
