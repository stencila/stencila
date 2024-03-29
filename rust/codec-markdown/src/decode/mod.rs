use std::collections::HashMap;

use markdown::{mdast, to_mdast, unist::Position, ParseOptions};

use codec::{
    common::eyre::{bail, eyre, Result},
    schema::{
        walk::{VisitorMut, WalkControl},
        Article, Block, Inline, Node, NodeId, NodeType,
    },
    DecodeOptions, Losses, Mapping,
};

mod blocks;
mod inlines;
mod shared;

/// Decode a Markdown string to a Stencila Schema [`Node`]
pub(super) fn decode(md: &str, _options: Option<DecodeOptions>) -> Result<(Node, Losses, Mapping)> {
    let mut options = ParseOptions::gfm();
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

    let mdast = to_mdast(md, &options).map_err(|error| eyre!(error))?;

    let mut context = Context::default();
    let Some(mut node) = md_to_node(mdast, &mut context) else {
        bail!("No node decoded from Markdown")
    };

    // TODO: parse frontmatter into Article

    if !context.footnotes.is_empty() {
        context.visit(&mut node);
    }

    Ok((node, context.losses, context.mapping))
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

    /// Map the position of a node in the source
    fn map(&mut self, position: Option<Position>, node_type: NodeType, node_id: NodeId) {
        if let Some(position) = position {
            self.mapping.add(
                position.start.offset,
                position.end.offset,
                node_type,
                node_id,
                None,
            );
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
            context.map(position, node.node_type(), node.node_id());
            Node::Article(node)
        }
        _ => {
            context.lost("Node");
            return None;
        }
    })
}
