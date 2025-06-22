use std::str::FromStr;

use common::tracing;
use node_url::{NodePosition, NodeUrl};
use schema::{
    Article, Block, ForBlock, IfBlockClause, IncludeBlock, Inline, Link, Node, NodeSet, Paragraph,
    RawBlock, Section, StyledBlock, VisitorMut, WalkControl, get,
};

/// Reconstitute a node from a cache
///
/// Walks over the node an when it encounters a `stencila://<path>` link, replaces it with
/// the node in the cache at the `path`.
pub fn reconstitute(node: &mut Node, cache: Option<Node>) {
    Reconstituter {
        cache,
        ..Default::default()
    }
    .walk(node);

    Janitor.walk(node);
}

/// Reconstitutes nodes from a cache node
#[derive(Default)]
struct Reconstituter {
    /// The cache node that linked nodes are copied from
    cache: Option<Node>,

    /// Stack of blocks collected between (potentially nested) `:begin` and `:end` links
    blocks: Vec<Vec<Block>>,
}

impl VisitorMut for Reconstituter {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        // Create an empty raw block to mark for deletion by `Janitor`
        let delete = || Block::RawBlock(RawBlock::new(String::new(), "".into()));

        // Only reconstitute paragraphs...
        let Block::Paragraph(Paragraph { content, .. }) = block else {
            // If blocks being collected then add to them
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block.clone());
                *block = delete();
            }

            return WalkControl::Continue;
        };

        // ...where the last inline is a link (may be single link, or have other inline nodes,
        // such as an anchor (bookmark) in front of it)
        let Some(Inline::Link(Link { target, .. })) = content.last() else {
            // If blocks being collected then add to them
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block.clone());
                *block = delete();
            }

            return WalkControl::Continue;
        };

        // ...that has a Stencila node URL
        let Some(node_url) = NodeUrl::from_str(target).ok() else {
            return WalkControl::Continue;
        };

        // ...that is for a block node (avoids links for inline nodes, such as
        // code expressions, being turned into a block)
        if !node_url
            .r#type
            .map(|typ| typ.is_block())
            .unwrap_or_default()
        {
            // If blocks are currently being collected then add to them
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block.clone());
                *block = delete();
            }

            return WalkControl::Continue;
        }

        // ...that may be a `begin` or `end` marker
        let mut begin = false;
        let mut mid = false;
        let mut end = false;
        if matches!(node_url.position, Some(NodePosition::Begin)) {
            begin = true;
            self.blocks.push(Vec::new());
        } else if matches!(node_url.position, Some(NodePosition::End)) {
            end = true;
        } else {
            mid = !self.blocks.is_empty();
        };

        // ...with a path as a target (and cache is available)
        let node = if let (Some(node_path), Some(cache)) = (node_url.path, &self.cache) {
            // ...that is getable from the cache
            match get(cache, node_path.clone()) {
                Ok(node_set) => match node_set {
                    NodeSet::One(node) => node,
                    NodeSet::Many(..) => {
                        tracing::error!("Got many nodes at `{node_path}`, expected one");
                        return WalkControl::Continue;
                    }
                },
                Err(error) => {
                    tracing::error!("While getting `{node_path}` from cache: {error}");
                    return WalkControl::Continue;
                }
            }
        } else if let Some(jzb64) = node_url.jzb64 {
            // ...that has a jzb64 field that can be deserialized to a node
            match NodeUrl::from_jzb64::<Node>(&jzb64) {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While decoding `jzb64`: {error}");
                    return WalkControl::Continue;
                }
            }
        } else {
            // If blocks being collected then add to them
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block.clone());
                *block = delete();
            }

            return WalkControl::Continue;
        };

        // ...and is a block node.
        let mut block_node = match Block::try_from(node) {
            Ok(block_node) => block_node,
            Err(error) => {
                tracing::error!("While converting node: {error}");
                return WalkControl::Continue;
            }
        };

        if begin {
            // Mark the begin block for deletion (it will be reconstituted by the `:end` block)
            *block = delete();
        } else if mid {
            // Push the reconstituted block to the current list of blocks and
            // mark for deletion
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block_node);
            }
            *block = delete();
        } else if end {
            // Pop off the collected blocks and assign them to the content of the reconstituted block
            let blocks = self.blocks.pop();
            if let Block::IncludeBlock(node) = &mut block_node {
                node.content = blocks;
            }
            // If there is a list of blocks (ie this is nested)
            // then push there, otherwise overwrite
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(block_node);
                *block = delete();
            } else {
                *block = block_node;
            }
        } else {
            // Just overwrite the block
            *block = block_node;
        }

        // Do not continue walk because do not want to visit the link again
        // in `visit_inline`
        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        // Only reconstitute links...
        let Inline::Link(Link { target, .. }) = inline else {
            return WalkControl::Continue;
        };

        // ...that has a Stencila node URL
        let Some(node_url) = NodeUrl::from_str(target).ok() else {
            return WalkControl::Continue;
        };

        // ...with a path as a target (and cache is available)
        let node = if let (Some(node_path), Some(cache)) = (node_url.path, &self.cache) {
            // ...that is getable from the cache
            match get(cache, node_path.clone()) {
                Ok(node_set) => match node_set {
                    NodeSet::One(node) => node,
                    NodeSet::Many(..) => {
                        tracing::error!("Got many nodes at `{node_path}`, expected one");
                        return WalkControl::Continue;
                    }
                },
                Err(error) => {
                    tracing::error!("While getting `{node_path}` from cache: {error}");
                    return WalkControl::Continue;
                }
            }
        } else if let Some(jzb64) = node_url.jzb64 {
            // ...that has a jzb64 field that can be deserialized to a node
            match NodeUrl::from_jzb64::<Node>(&jzb64) {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While decoding `jzb64`: {error}");
                    return WalkControl::Continue;
                }
            }
        } else {
            return WalkControl::Continue;
        };

        // ...and is an inline node.
        let inline_node = match Inline::try_from(node) {
            Ok(inline_node) => inline_node,
            Err(error) => {
                tracing::error!("While converting node: {error}");
                return WalkControl::Continue;
            }
        };

        *inline = inline_node;

        WalkControl::Continue
    }
}

/// Removes blocks marked for deletion by the `Reconstituter`
struct Janitor;

impl VisitorMut for Janitor {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        // Delete empty raw blocks in any node that has block content
        if let Node::Article(Article { content, .. }) = node {
            content.retain(|block| {
                if let Block::RawBlock(RawBlock {
                    format, content, ..
                }) = block
                {
                    !(format.is_empty() && content.is_empty())
                } else {
                    true
                }
            });
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        // Delete empty raw blocks in any block that has block content
        if let Block::IncludeBlock(IncludeBlock {
            content: Some(content),
            ..
        })
        | Block::ForBlock(ForBlock { content, .. })
        | Block::Section(Section { content, .. })
        | Block::StyledBlock(StyledBlock { content, .. }) = block
        {
            content.retain(|block| {
                if let Block::RawBlock(RawBlock {
                    format, content, ..
                }) = block
                {
                    !(format.is_empty() && content.is_empty())
                } else {
                    true
                }
            });
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &mut IfBlockClause) -> WalkControl {
        // Delete empty raw blocks in content
        clause.content.retain(|block| {
            if let Block::RawBlock(RawBlock {
                format, content, ..
            }) = block
            {
                !(format.is_empty() && content.is_empty())
            } else {
                true
            }
        });

        WalkControl::Continue
    }
}
