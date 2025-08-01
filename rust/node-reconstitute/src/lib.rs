use std::str::FromStr;

use common::tracing;
use node_url::{NodePosition, NodeUrl};
use schema::{
    Article, Block, ForBlock, IfBlock, IfBlockClause, IncludeBlock, Inline, Link, Node, NodePath,
    NodeProperty, NodeSet, NodeSlot, Paragraph, RawBlock, Section, StyledBlock, VisitorMut,
    WalkControl, get,
};

/// Reconstitute a node from a cache
///
/// Walks over the node an when it encounters a `https://stencila.link`, replaces it with
/// the node in the cache at the `path` query param, or with the self-contained `jzb64` query param.
pub fn reconstitute(node: &mut Node, cache: Option<Node>) {
    Reconstituter {
        cache,
        ..Default::default()
    }
    .walk(node);

    Janitor.walk(node);
}

/// A block with its original path information for iteration detection
#[derive(Debug, Clone)]
struct BlockWithPath {
    block: Block,
    path: Option<NodePath>,
}

/// Reconstitutes nodes from a cache node
#[derive(Default)]
struct Reconstituter {
    /// The cache node that linked nodes are copied from
    cache: Option<Node>,

    /// Stack of blocks collected between (potentially nested) `begin` and `end` links
    blocks: Vec<Vec<BlockWithPath>>,
}

impl Reconstituter {
    /// Check if a node path represents a ForBlock iteration
    /// This looks for paths ending with: Property(Iterations) followed by Index(_)
    fn is_iteration_path(path: &NodePath) -> bool {
        let slots: Vec<_> = path.iter().collect();

        // Check if path ends with: iterations/index
        if slots.len() >= 2 {
            if let [
                NodeSlot::Property(NodeProperty::Iterations),
                NodeSlot::Index(_),
            ] = &slots[slots.len() - 2..]
            {
                return true;
            }
        }

        false
    }

    /// Helper method to add blocks to collection if they're being collected
    fn collect_block_if_necessary(&mut self, block: &mut Block) -> WalkControl {
        if let Some(blocks) = self.blocks.last_mut() {
            blocks.push(BlockWithPath {
                block: block.clone(),
                path: None, // No path for non-reconstituted blocks
            });
            *block = Block::RawBlock(RawBlock::new(String::new(), "".into()));
        }
        WalkControl::Continue
    }
}

impl VisitorMut for Reconstituter {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        // Create an empty raw block to mark for deletion by `Janitor`
        let delete = || Block::RawBlock(RawBlock::new(String::new(), "".into()));

        // Only reconstitute paragraphs...
        let Block::Paragraph(Paragraph { content, .. }) = block else {
            return self.collect_block_if_necessary(block);
        };

        // ...that has a link anywhere in it (may be single link, or have other inline nodes,
        // such as an invisible  anchor (bookmark) before or after it)
        let Some(target) = content.iter().find_map(|inline| match inline {
            Inline::Link(Link { target, .. }) => Some(target),
            _ => None,
        }) else {
            return self.collect_block_if_necessary(block);
        };

        // ...that has a Stencila node URL
        let Some(node_url) = NodeUrl::from_str(target).ok() else {
            return self.collect_block_if_necessary(block);
        };

        // ...that is for a block node (avoids links for inline nodes, such as
        // code expressions, being turned into a block)
        if !node_url
            .r#type
            .map(|typ| typ.is_block())
            .unwrap_or_default()
        {
            return self.collect_block_if_necessary(block);
        }

        // Capture path before it gets moved
        let node_path = node_url.path.clone();

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
            return self.collect_block_if_necessary(block);
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
            // Push the reconstituted block with path information and mark for deletion
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(BlockWithPath {
                    block: block_node,
                    path: node_path.clone(),
                });
            }
            *block = delete();
        } else if end {
            // Pop off the collected blocks and assign them to the content of the reconstituted block
            let blocks_with_path = self.blocks.pop();
            match &mut block_node {
                Block::IncludeBlock(node) => {
                    node.content = blocks_with_path
                        .map(|blocks| blocks.into_iter().map(|bwp| bwp.block).collect())
                }
                Block::ForBlock(ForBlock {
                    content,
                    iterations,
                    ..
                }) => {
                    // For ForBlock, separate content from iterations based on path information
                    if let Some(blocks_with_path) = blocks_with_path {
                        let mut content_blocks = Vec::new();
                        let mut iteration_blocks = Vec::new();

                        for block_with_path in blocks_with_path {
                            let is_iteration = block_with_path
                                .path
                                .as_ref()
                                .map(Self::is_iteration_path)
                                .unwrap_or(false);

                            if is_iteration {
                                iteration_blocks.push(block_with_path.block);
                            } else {
                                content_blocks.push(block_with_path.block);
                            }
                        }

                        *content = content_blocks;
                        *iterations = if iteration_blocks.is_empty() {
                            None
                        } else {
                            Some(iteration_blocks)
                        };
                    } else {
                        *content = Vec::new();
                        *iterations = None;
                    }
                }
                Block::IfBlock(IfBlock { clauses, .. }) => {
                    // Find the active clause and set its content to all collected blocks
                    if let Some(blocks_with_path) = blocks_with_path {
                        let collected_blocks: Vec<Block> =
                            blocks_with_path.into_iter().map(|bwp| bwp.block).collect();

                        // Find the first active clause
                        if let Some(active_clause) = clauses
                            .iter_mut()
                            .find(|clause| clause.is_active == Some(true))
                        {
                            active_clause.content = collected_blocks;
                        }
                    }
                }
                Block::Section(Section { content, .. })
                | Block::StyledBlock(StyledBlock { content, .. }) => {
                    *content = blocks_with_path
                        .map(|blocks| blocks.into_iter().map(|bwp| bwp.block).collect())
                        .unwrap_or_default()
                }
                _ => {}
            }
            // If there is a list of blocks (ie this is nested)
            // then push there, otherwise overwrite
            if let Some(blocks) = self.blocks.last_mut() {
                blocks.push(BlockWithPath {
                    block: block_node,
                    path: node_path.clone(),
                });
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

        // Handle ForBlock separately for both content and iterations
        if let Block::ForBlock(ForBlock {
            content,
            iterations,
            ..
        }) = block
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

            if let Some(iterations) = iterations {
                iterations.retain(|block| {
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

#[cfg(test)]
mod tests {
    use super::*;

    use common::eyre::{Result, bail};
    use schema::{
        Article, Block, ForBlock, Inline, NodePath, NodeType, RawBlock, node_url_path,
        shortcuts::{art, cb, cc, ce, em, fig, frb, ibc, ifb, inb, lnk, p, qb, sec, stb, stg, t},
    };

    #[test]
    fn test_is_iteration_path() -> Result<()> {
        // Should detect iteration paths
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/iterations/0"
        )?));
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/iterations/1"
        )?));
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/5/iterations/3"
        )?));
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "iterations/0"
        )?));

        // Should not detect non-iteration paths
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0"
        )?));
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/content/1"
        )?));
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/iterations"
        )?));
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/1/content/2"
        )?));

        Ok(())
    }

    #[test]
    fn test_is_iteration_path_edge_cases() -> Result<()> {
        // Test edge cases with just "iterations" property (no index)
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/iterations"
        )?));

        // Test nested iterations paths - only true if ends with iterations/index
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/content/5/iterations/2"
        )?));

        // Test iterations at the beginning
        assert!(Reconstituter::is_iteration_path(&NodePath::from_str(
            "iterations/0"
        )?));

        // Test paths that have iterations/index but NOT at the end - should be false
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "iterations/0/content/1"
        )?));
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "content/0/iterations/1/content/2"
        )?));
        assert!(!Reconstituter::is_iteration_path(&NodePath::from_str(
            "iterations/0/content"
        )?));

        Ok(())
    }

    fn node_link_begin(node_type: NodeType, path: &str) -> Result<Inline> {
        Ok(lnk(
            [t(format!("[Begin {node_type}]"))],
            node_url_path(
                node_type,
                NodePath::from_str(path)?,
                Some(NodePosition::Begin),
            )
            .to_string(),
        ))
    }

    fn node_link_end(node_type: NodeType, path: &str) -> Result<Inline> {
        Ok(lnk(
            [t(format!("[End {node_type}]"))],
            node_url_path(
                node_type,
                NodePath::from_str(path)?,
                Some(NodePosition::End),
            )
            .to_string(),
        ))
    }

    #[test]
    fn paragraph_added_before_include_block() -> Result<()> {
        let original = art([inb("included.smd")]);

        let mut edited = art([
            p([t("New paragraph")]),
            p([node_link_begin(NodeType::IncludeBlock, "content/0")?]),
            p([t("Paragraph included from included.smd with edits")]),
            p([node_link_end(NodeType::IncludeBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(
            content.len(),
            2,
            "Should have 2 blocks: new paragraph and edited include block"
        );

        let Block::Paragraph(para) = &content[0] else {
            bail!("First block should be a paragraph");
        };

        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("First paragraph should contain text");
        };

        assert_eq!(text.value.as_str(), "New paragraph");

        let Block::IncludeBlock(include) = &content[1] else {
            bail!("Second block should be an include block");
        };

        assert_eq!(include.source.as_str(), "included.smd");

        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };

        assert_eq!(
            blocks.len(),
            1,
            "Include block should have 1 collected block"
        );

        let Block::Paragraph(para) = &blocks[0] else {
            bail!("Collected block should be a code block");
        };

        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph included from included.smd with edits");
        };

        assert_eq!(
            text.value.as_str(),
            "Paragraph included from included.smd with edits"
        );

        Ok(())
    }

    #[test]
    fn include_block_with_multiple_content_types() -> Result<()> {
        let original = art([inb("mixed-content.md")]);

        let mut edited = art([
            p([node_link_begin(NodeType::IncludeBlock, "content/0")?]),
            p([
                t("Regular paragraph with "),
                stg([t("strong text")]),
                t(" and "),
                em([t("emphasized text")]),
            ]),
            cb("def hello():\n    print('Hello, World!')", Some("python")),
            qb([p([t("A quoted paragraph")])]),
            fig([p([t("Figure caption")])]),
            p([node_link_end(NodeType::IncludeBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 include block");

        let Block::IncludeBlock(include) = &content[0] else {
            bail!("Block should be an include block");
        };

        assert_eq!(include.source.as_str(), "mixed-content.md");

        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };

        assert_eq!(
            blocks.len(),
            4,
            "Include block should have 4 different blocks"
        );

        // Check paragraph with inline formatting
        let Block::Paragraph(para) = &blocks[0] else {
            bail!("First block should be a paragraph");
        };
        assert_eq!(
            para.content.len(),
            4,
            "Paragraph should have 4 inline elements"
        );

        // Check code block
        let Block::CodeBlock(code_block) = &blocks[1] else {
            bail!("Second block should be a code block");
        };
        assert_eq!(code_block.programming_language.as_deref(), Some("python"));
        assert_eq!(
            code_block.code.as_str(),
            "def hello():\n    print('Hello, World!')"
        );

        // Check quote block
        let Block::QuoteBlock(quote_block) = &blocks[2] else {
            bail!("Third block should be a quote block");
        };
        assert_eq!(quote_block.content.len(), 1);

        // Check figure
        let Block::Figure(figure) = &blocks[3] else {
            bail!("Fourth block should be a figure");
        };
        assert_eq!(
            figure.content.len(),
            1,
            "Figure should have 1 content block"
        );

        Ok(())
    }

    #[test]
    fn include_block_empty() -> Result<()> {
        let original = art([inb("empty.md"), p([t("After empty include")])]);

        let mut edited = art([
            p([node_link_begin(NodeType::IncludeBlock, "content/0")?]),
            p([node_link_end(NodeType::IncludeBlock, "content/0")?]),
            p([t("After empty include")]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 2, "Should have 2 blocks");

        // Check empty include block
        let Block::IncludeBlock(include) = &content[0] else {
            bail!("First block should be an include block");
        };
        assert_eq!(include.source.as_str(), "empty.md");
        assert_eq!(
            include.content,
            Some(vec![]),
            "Include block should have empty content"
        );

        // Check paragraph after include
        let Block::Paragraph(para) = &content[1] else {
            bail!("Second block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "After empty include");

        Ok(())
    }

    #[test]
    fn include_block_with_code_chunks() -> Result<()> {
        let original = art([inb("analysis.ipynb")]);

        let mut edited = art([
            p([node_link_begin(NodeType::IncludeBlock, "content/0")?]),
            p([t("Analysis notebook with code")]),
            cc(
                "import pandas as pd\ndf = pd.read_csv('data.csv')",
                Some("python"),
            ),
            p([t("Data loaded, now processing...")]),
            cc(
                "result = df.groupby('category').sum()\nprint(result)",
                Some("python"),
            ),
            p([node_link_end(NodeType::IncludeBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 include block");

        let Block::IncludeBlock(include) = &content[0] else {
            bail!("Block should be an include block");
        };

        assert_eq!(include.source.as_str(), "analysis.ipynb");

        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };

        assert_eq!(blocks.len(), 4, "Include block should have 4 blocks");

        // Check first paragraph
        let Block::Paragraph(para) = &blocks[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Analysis notebook with code");

        // Check first code chunk
        let Block::CodeChunk(chunk) = &blocks[1] else {
            bail!("Second block should be a code chunk");
        };
        assert_eq!(chunk.programming_language.as_deref(), Some("python"));
        assert_eq!(
            chunk.code.as_str(),
            "import pandas as pd\ndf = pd.read_csv('data.csv')"
        );

        // Check middle paragraph
        let Block::Paragraph(para) = &blocks[2] else {
            bail!("Third block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Data loaded, now processing...");

        // Check second code chunk
        let Block::CodeChunk(chunk) = &blocks[3] else {
            bail!("Fourth block should be a code chunk");
        };
        assert_eq!(chunk.programming_language.as_deref(), Some("python"));
        assert_eq!(
            chunk.code.as_str(),
            "result = df.groupby('category').sum()\nprint(result)"
        );

        Ok(())
    }

    #[test]
    fn include_block_with_raw_blocks() -> Result<()> {
        let original = art([inb("template.html")]);

        let mut edited = art([
            p([node_link_begin(NodeType::IncludeBlock, "content/0")?]),
            Block::RawBlock(RawBlock::new(
                "html".into(),
                "<div class=\"container\">\n  <h1>Title</h1>".into(),
            )),
            p([t("Some content in between")]),
            Block::RawBlock(RawBlock::new("html".into(), "</div>".into())),
            Block::RawBlock(RawBlock::new(
                "latex".into(),
                "\\begin{equation}\n  E = mc^2\n\\end{equation}".into(),
            )),
            p([node_link_end(NodeType::IncludeBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 include block");

        let Block::IncludeBlock(include) = &content[0] else {
            bail!("Block should be an include block");
        };

        assert_eq!(include.source.as_str(), "template.html");

        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };

        assert_eq!(blocks.len(), 4, "Include block should have 4 blocks");

        // Check first raw block
        let Block::RawBlock(raw) = &blocks[0] else {
            bail!("First block should be a raw block");
        };
        assert_eq!(raw.format.as_str(), "html");
        assert_eq!(
            raw.content.as_str(),
            "<div class=\"container\">\n  <h1>Title</h1>"
        );

        // Check paragraph
        let Block::Paragraph(para) = &blocks[1] else {
            bail!("Second block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Some content in between");

        // Check second raw block
        let Block::RawBlock(raw) = &blocks[2] else {
            bail!("Third block should be a raw block");
        };
        assert_eq!(raw.format.as_str(), "html");
        assert_eq!(raw.content.as_str(), "</div>");

        // Check LaTeX raw block
        let Block::RawBlock(raw) = &blocks[3] else {
            bail!("Fourth block should be a raw block");
        };
        assert_eq!(raw.format.as_str(), "latex");
        assert_eq!(
            raw.content.as_str(),
            "\\begin{equation}\n  E = mc^2\n\\end{equation}"
        );

        Ok(())
    }

    #[test]
    fn for_block_with_content_only() -> Result<()> {
        let original = art([frb("item", "[1, 2, 3]", [p([t("Item value: ${item}")])])]);

        let mut edited = art([
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            p([t("Item value: ${item} edited")]),
            p([t("Extra content added")]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 for block");

        let Block::ForBlock(for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        assert_eq!(for_block.variable.as_str(), "item");
        assert_eq!(for_block.code.as_str(), "[1, 2, 3]");

        // ForBlock content should now contain the edited blocks (no sections, so no iterations)
        assert_eq!(
            for_block.content.len(),
            2,
            "ForBlock should have 2 edited content blocks"
        );

        let Block::Paragraph(para) = &for_block.content[0] else {
            bail!("First block in ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Item value: ${item} edited");

        let Block::Paragraph(para) = &for_block.content[1] else {
            bail!("Second block in ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Extra content added");

        // Check no iterations
        assert_eq!(
            for_block.iterations, None,
            "ForBlock should have no iterations"
        );

        Ok(())
    }

    #[test]
    fn for_block_with_content_and_iterations() -> Result<()> {
        let mut original_for_block = ForBlock::new(
            "[1, 2]".into(),
            "item".into(),
            vec![p([t("Iteration content")])],
        );
        original_for_block.iterations = Some(vec![
            sec([p([t("Original iteration")])]),
            sec([p([t("Another original iteration")])]),
        ]);
        let original = art([Block::ForBlock(original_for_block)]);

        let mut edited = art([
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            p([t("Iteration content")]),
            cb("// Extra content add in content", Some("javascript")),
            p([node_link_begin(
                NodeType::Section,
                "content/0/iterations/0",
            )?]),
            p([t("Iteration content")]),
            p([node_link_end(NodeType::Section, "content/0/iterations/0")?]),
            p([node_link_begin(
                NodeType::Section,
                "content/0/iterations/1",
            )?]),
            p([t("Iteration content")]),
            p([t("Extra content added in iteration")]),
            p([node_link_end(NodeType::Section, "content/0/iterations/1")?]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 for block");

        let Block::ForBlock(for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        assert_eq!(for_block.variable.as_str(), "item");
        assert_eq!(for_block.code.as_str(), "[1, 2]");

        // Check content (blocks before first iteration)
        assert_eq!(
            for_block.content.len(),
            2,
            "ForBlock content should have 2 blocks"
        );

        let Block::Paragraph(para) = &for_block.content[0] else {
            bail!("First content block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Iteration content");

        let Block::CodeBlock(code_block) = &for_block.content[1] else {
            bail!("Second content block should be a code block");
        };
        assert_eq!(
            code_block.programming_language.as_deref(),
            Some("javascript")
        );
        assert_eq!(code_block.code.as_str(), "// Extra content add in content");

        // Check iterations (Section blocks)
        let Some(iterations) = &for_block.iterations else {
            bail!("ForBlock should have iterations");
        };
        assert_eq!(iterations.len(), 2, "ForBlock should have 2 iterations");

        // Check first iteration
        let Block::Section(section1) = &iterations[0] else {
            bail!("First iteration should be a section");
        };
        assert_eq!(section1.content.len(), 1);
        let Block::Paragraph(para) = &section1.content[0] else {
            bail!("First iteration should contain a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Iteration content");

        // Check second iteration
        let Block::Section(section2) = &iterations[1] else {
            bail!("Second iteration should be a section");
        };
        assert_eq!(section2.content.len(), 2);
        let Block::Paragraph(para) = &section2.content[0] else {
            bail!("Second iteration should contain paragraphs");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Iteration content");

        Ok(())
    }

    #[test]
    fn for_block_with_only_one_iteration() -> Result<()> {
        let mut original_for_block = ForBlock::new(
            "[1, 2, 3]".into(),
            "item".into(),
            vec![p([t("Original template")])],
        );
        original_for_block.iterations = Some(vec![sec([p([t("Original iteration content")])])]);
        let original = art([Block::ForBlock(original_for_block)]);

        let mut edited = art([
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            // Maybe user deleted the content
            p([node_link_begin(
                NodeType::Section,
                "content/0/iterations/0",
            )?]),
            p([t("Only iteration content")]),
            p([node_link_end(NodeType::Section, "content/0/iterations/0")?]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::ForBlock(for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        // Content should be empty since we start with an iteration
        assert_eq!(
            for_block.content.len(),
            0,
            "ForBlock content should be empty"
        );

        // Should have one iteration
        let Some(iterations) = &for_block.iterations else {
            bail!("ForBlock should have iterations");
        };
        assert_eq!(iterations.len(), 1, "ForBlock should have 1 iteration");

        Ok(())
    }

    #[test]
    fn for_block_empty_content_and_iterations() -> Result<()> {
        let original = art([frb("item", "[1, 2, 3]", [p([t("Item value: ${item}")])])]);

        let mut edited = art([
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::ForBlock(for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        // Both content and iterations should be empty
        assert_eq!(
            for_block.content.len(),
            0,
            "ForBlock content should be empty"
        );
        assert_eq!(
            for_block.iterations, None,
            "ForBlock should have no iterations"
        );

        Ok(())
    }

    #[test]
    fn for_blocks_nested_three_levels_deep() -> Result<()> {
        let original = art([frb(
            "x",
            "[1, 2, 3]",
            vec![
                p([t("Outer ForBlock content")]),
                frb(
                    "y",
                    "[4, 5, 6]",
                    vec![
                        p([t("Middle ForBlock content")]),
                        frb("z", "[7, 8, 9]", vec![p([t("Inner ForBlock content")])]),
                        p([t("After inner ForBlock")]),
                    ],
                ),
                p([t("After middle ForBlock")]),
            ],
        )]);

        let mut edited = art([
            // Outer ForBlock
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            p([t("Outer ForBlock content - edited")]),
            p([t("New outer content")]),
            // Middle ForBlock
            p([node_link_begin(NodeType::ForBlock, "content/0/content/1")?]),
            p([t("Middle ForBlock content - edited")]),
            p([t("New middle content")]),
            // Inner ForBlock
            p([node_link_begin(
                NodeType::ForBlock,
                "content/0/content/1/content/1",
            )?]),
            p([t("Inner ForBlock content - edited")]),
            p([t("New inner content")]),
            p([node_link_end(
                NodeType::ForBlock,
                "content/0/content/1/content/1",
            )?]),
            p([t("After inner ForBlock - edited")]),
            p([t("New content after inner")]),
            p([node_link_end(NodeType::ForBlock, "content/0/content/1")?]),
            p([t("After middle ForBlock - edited")]),
            p([t("New content after middle")]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 outer for block");

        // Check outer ForBlock
        let Block::ForBlock(outer_for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        assert_eq!(outer_for_block.variable.as_str(), "x");
        assert_eq!(outer_for_block.code.as_str(), "[1, 2, 3]");
        assert_eq!(
            outer_for_block.content.len(),
            5,
            "Outer ForBlock should have 5 content blocks"
        );

        // Check outer ForBlock first paragraph
        let Block::Paragraph(para) = &outer_for_block.content[0] else {
            bail!("First block in outer ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Outer ForBlock content - edited");

        // Check new outer content
        let Block::Paragraph(para) = &outer_for_block.content[1] else {
            bail!("Second block in outer ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New outer content");

        // Check middle ForBlock
        let Block::ForBlock(middle_for_block) = &outer_for_block.content[2] else {
            bail!("Third block in outer ForBlock should be a middle for block");
        };

        assert_eq!(middle_for_block.variable.as_str(), "y");
        assert_eq!(middle_for_block.code.as_str(), "[4, 5, 6]");
        assert_eq!(
            middle_for_block.content.len(),
            5,
            "Middle ForBlock should have 5 content blocks"
        );

        // Check middle ForBlock first paragraph
        let Block::Paragraph(para) = &middle_for_block.content[0] else {
            bail!("First block in middle ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Middle ForBlock content - edited");

        // Check new middle content
        let Block::Paragraph(para) = &middle_for_block.content[1] else {
            bail!("Second block in middle ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New middle content");

        // Check inner ForBlock
        let Block::ForBlock(inner_for_block) = &middle_for_block.content[2] else {
            bail!("Third block in middle ForBlock should be an inner for block");
        };

        assert_eq!(inner_for_block.variable.as_str(), "z");
        assert_eq!(inner_for_block.code.as_str(), "[7, 8, 9]");
        assert_eq!(
            inner_for_block.content.len(),
            2,
            "Inner ForBlock should have 2 content blocks"
        );

        // Check inner ForBlock content
        let Block::Paragraph(para) = &inner_for_block.content[0] else {
            bail!("First block in inner ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Inner ForBlock content - edited");

        // Check new inner content
        let Block::Paragraph(para) = &inner_for_block.content[1] else {
            bail!("Second block in inner ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New inner content");

        // Check content after inner ForBlock in middle ForBlock
        let Block::Paragraph(para) = &middle_for_block.content[3] else {
            bail!("Fourth block in middle ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "After inner ForBlock - edited");

        // Check new content after inner in middle ForBlock
        let Block::Paragraph(para) = &middle_for_block.content[4] else {
            bail!("Fifth block in middle ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New content after inner");

        // Check content after middle ForBlock in outer ForBlock
        let Block::Paragraph(para) = &outer_for_block.content[3] else {
            bail!("Fourth block in outer ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "After middle ForBlock - edited");

        // Check new content after middle in outer ForBlock
        let Block::Paragraph(para) = &outer_for_block.content[4] else {
            bail!("Fifth block in outer ForBlock should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New content after middle");

        // Verify no iterations for any ForBlock (since no /iterations/ paths were used)
        assert_eq!(
            outer_for_block.iterations, None,
            "Outer ForBlock should have no iterations"
        );
        assert_eq!(
            middle_for_block.iterations, None,
            "Middle ForBlock should have no iterations"
        );
        assert_eq!(
            inner_for_block.iterations, None,
            "Inner ForBlock should have no iterations"
        );

        Ok(())
    }

    #[test]
    fn for_block_with_section_in_content() -> Result<()> {
        let original = art([frb(
            "item",
            "[1, 2, 3]",
            vec![
                p([t("Original content")]),
                sec([p([t("Original section content")])]),
            ],
        )]);

        let mut edited = art([
            p([node_link_begin(NodeType::ForBlock, "content/0")?]),
            p([t("Content before section")]),
            // This section should go to content since path doesn't contain /iterations/
            p([node_link_begin(NodeType::Section, "content/0/content/1")?]),
            p([t("This is a content section header")]),
            p([t("This is content within the section")]),
            p([node_link_end(NodeType::Section, "content/0/content/1")?]),
            p([t("Content after section")]),
            p([node_link_end(NodeType::ForBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::ForBlock(for_block) = &content[0] else {
            bail!("Block should be a for block");
        };

        // Content should have 3 blocks: paragraph, section, paragraph
        assert_eq!(
            for_block.content.len(),
            3,
            "ForBlock content should have 3 blocks: paragraph, section, paragraph"
        );

        // Check first content paragraph
        let Block::Paragraph(para) = &for_block.content[0] else {
            bail!("First content block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Content before section");

        // Check section (should be in content, not iterations, because path doesn't contain /iterations/)
        let Block::Section(section) = &for_block.content[1] else {
            bail!("Second content block should be a section");
        };
        assert_eq!(section.content.len(), 2, "Section should have 2 paragraphs");

        // Check section content
        let Block::Paragraph(para) = &section.content[0] else {
            bail!("Section should contain paragraphs");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Section paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "This is a content section header");

        // Check third content paragraph
        let Block::Paragraph(para) = &for_block.content[2] else {
            bail!("Third content block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Content after section");

        // No iterations since there were no /iterations/ paths
        assert_eq!(
            for_block.iterations, None,
            "ForBlock should have no iterations since no /iterations/ paths were used"
        );

        Ok(())
    }

    #[test]
    fn if_block_with_single_active_clause() -> Result<()> {
        // Create an IfBlock with a single active clause
        let mut if_clause = ibc("x > 0", Some("python"), vec![p([t("Original if content")])]);
        if_clause.is_active = Some(true);
        let original = art([ifb(vec![if_clause])]);

        let mut edited = art([
            p([node_link_begin(NodeType::IfBlock, "content/0")?]),
            p([t("Edited if content")]),
            p([t("Additional content added")]),
            p([node_link_end(NodeType::IfBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 if block");

        let Block::IfBlock(if_block) = &content[0] else {
            bail!("Block should be an if block");
        };

        assert_eq!(if_block.clauses.len(), 1, "Should have 1 clause");

        let clause = &if_block.clauses[0];
        assert_eq!(clause.code.as_str(), "x > 0");
        assert_eq!(clause.programming_language.as_deref(), Some("python"));
        assert_eq!(clause.is_active, Some(true));

        // Check that content was updated
        assert_eq!(clause.content.len(), 2, "Clause should have 2 blocks");

        let Block::Paragraph(para) = &clause.content[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Edited if content");

        let Block::Paragraph(para) = &clause.content[1] else {
            bail!("Second block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Additional content added");

        Ok(())
    }

    #[test]
    fn if_block_with_if_else_clauses() -> Result<()> {
        // Create an IfBlock with if and else, where else is active
        let mut if_clause = ibc("x > 0", Some("python"), vec![p([t("If content")])]);
        if_clause.is_active = Some(false);

        let mut else_clause = ibc("", None::<String>, vec![p([t("Original else content")])]);
        else_clause.is_active = Some(true);

        let original = art([ifb(vec![if_clause, else_clause])]);

        let mut edited = art([
            p([node_link_begin(NodeType::IfBlock, "content/0")?]),
            p([t("Edited else content")]),
            cb("// Some code in else", Some("javascript")),
            p([t("More else content")]),
            p([node_link_end(NodeType::IfBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::IfBlock(if_block) = &content[0] else {
            bail!("Block should be an if block");
        };

        assert_eq!(if_block.clauses.len(), 2, "Should have 2 clauses");

        // Check if clause (should be inactive with original content)
        let if_clause = &if_block.clauses[0];
        assert_eq!(if_clause.code.as_str(), "x > 0");
        assert_eq!(if_clause.is_active, Some(false));
        assert_eq!(
            if_clause.content.len(),
            1,
            "If clause should have original content"
        );

        // Check else clause (should be active with edited content)
        let else_clause = &if_block.clauses[1];
        assert_eq!(else_clause.code.as_str(), "");
        assert_eq!(else_clause.is_active, Some(true));
        assert_eq!(
            else_clause.content.len(),
            3,
            "Else clause should have 3 blocks"
        );

        // Verify edited content in else clause
        let Block::Paragraph(para) = &else_clause.content[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Edited else content");

        let Block::CodeBlock(code_block) = &else_clause.content[1] else {
            bail!("Second block should be a code block");
        };
        assert_eq!(code_block.code.as_str(), "// Some code in else");
        assert_eq!(
            code_block.programming_language.as_deref(),
            Some("javascript")
        );

        Ok(())
    }

    #[test]
    fn if_block_with_elif_clauses() -> Result<()> {
        // Create an IfBlock with if, elif, and else, where elif is active
        let mut if_clause = ibc("x > 0", Some("python"), vec![p([t("If content")])]);
        if_clause.is_active = Some(false);

        let mut elif_clause = ibc(
            "x == 0",
            Some("python"),
            vec![p([t("Original elif content")])],
        );
        elif_clause.is_active = Some(true);

        let mut else_clause = ibc("", None::<String>, vec![p([t("Else content")])]);
        else_clause.is_active = Some(false);

        let original = art([ifb(vec![if_clause, elif_clause, else_clause])]);

        let mut edited = art([
            p([node_link_begin(NodeType::IfBlock, "content/0")?]),
            p([t("Edited elif content")]),
            qb([p([t("Quote in elif")])]),
            p([node_link_end(NodeType::IfBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::IfBlock(if_block) = &content[0] else {
            bail!("Block should be an if block");
        };

        assert_eq!(if_block.clauses.len(), 3, "Should have 3 clauses");

        // Check elif clause (should be active with edited content)
        let elif_clause = &if_block.clauses[1];
        assert_eq!(elif_clause.code.as_str(), "x == 0");
        assert_eq!(elif_clause.is_active, Some(true));
        assert_eq!(
            elif_clause.content.len(),
            2,
            "Elif clause should have 2 blocks"
        );

        // Verify other clauses remain unchanged
        assert_eq!(if_block.clauses[0].is_active, Some(false));
        assert_eq!(if_block.clauses[0].content.len(), 1);
        assert_eq!(if_block.clauses[2].is_active, Some(false));
        assert_eq!(if_block.clauses[2].content.len(), 1);

        Ok(())
    }

    #[test]
    fn if_block_nested_in_section() -> Result<()> {
        // Create an IfBlock nested in a section
        let mut if_clause = ibc(
            "condition",
            Some("javascript"),
            vec![p([t("Nested if content")])],
        );
        if_clause.is_active = Some(true);

        let original = art([sec([
            p([t("Section header")]),
            ifb(vec![if_clause]),
            p([t("After if block")]),
        ])]);

        let mut edited = art([
            p([node_link_begin(NodeType::Section, "content/0")?]),
            p([t("Section header - edited")]),
            p([node_link_begin(NodeType::IfBlock, "content/0/content/1")?]),
            p([t("Nested if content - edited")]),
            p([t("New nested content")]),
            p([node_link_end(NodeType::IfBlock, "content/0/content/1")?]),
            p([t("After if block - edited")]),
            p([node_link_end(NodeType::Section, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::Section(section) = &content[0] else {
            bail!("Block should be a section");
        };

        assert_eq!(section.content.len(), 3, "Section should have 3 blocks");

        let Block::IfBlock(if_block) = &section.content[1] else {
            bail!("Second block in section should be an if block");
        };

        let clause = &if_block.clauses[0];
        assert_eq!(clause.is_active, Some(true));
        assert_eq!(
            clause.content.len(),
            2,
            "Active clause should have 2 blocks"
        );

        // Verify edited content
        let Block::Paragraph(para) = &clause.content[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Nested if content - edited");

        Ok(())
    }

    #[test]
    fn if_block_empty_content() -> Result<()> {
        // Create an IfBlock where active clause has empty content after editing
        let mut if_clause = ibc("true", Some("python"), vec![p([t("Will be removed")])]);
        if_clause.is_active = Some(true);
        let original = art([ifb(vec![if_clause])]);

        let mut edited = art([
            p([node_link_begin(NodeType::IfBlock, "content/0")?]),
            p([node_link_end(NodeType::IfBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        let Block::IfBlock(if_block) = &content[0] else {
            bail!("Block should be an if block");
        };

        let clause = &if_block.clauses[0];
        assert_eq!(clause.is_active, Some(true));
        assert_eq!(
            clause.content.len(),
            0,
            "Active clause should have empty content"
        );

        Ok(())
    }

    #[test]
    fn section_with_changes() -> Result<()> {
        let original = art([sec([
            p([t("Section original content")]),
            p([t("Another paragraph in section")]),
        ])]);

        let mut edited = art([
            p([node_link_begin(NodeType::Section, "content/0")?]),
            p([t("Section edited content")]),
            p([t("Another paragraph in section with changes")]),
            p([t("New paragraph added to section")]),
            p([node_link_end(NodeType::Section, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 section");

        let Block::Section(section) = &content[0] else {
            bail!("Block should be a section");
        };

        assert_eq!(
            section.content.len(),
            3,
            "Section should have 3 edited paragraphs"
        );

        // Check first paragraph (edited content)
        let Block::Paragraph(para) = &section.content[0] else {
            bail!("First block in section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("First paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Section edited content");

        // Check second paragraph (edited content)
        let Block::Paragraph(para) = &section.content[1] else {
            bail!("Second block in section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Second paragraph should contain text");
        };
        assert_eq!(
            text.value.as_str(),
            "Another paragraph in section with changes"
        );

        // Check third paragraph (new content)
        let Block::Paragraph(para) = &section.content[2] else {
            bail!("Third block in section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Third paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New paragraph added to section");

        Ok(())
    }

    #[test]
    fn sections_nested_three_levels_deep() -> Result<()> {
        let original = art([sec([
            p([t("Level 1 content")]),
            sec([
                p([t("Level 2 content")]),
                sec([p([t("Level 3 content")]), p([t("More level 3 content")])]),
                p([t("More level 2 content")]),
            ]),
            p([t("More level 1 content")]),
        ])]);

        let mut edited = art([
            p([node_link_begin(NodeType::Section, "content/0")?]),
            p([t("Level 1 content - edited")]),
            p([node_link_begin(NodeType::Section, "content/0/content/1")?]),
            p([t("Level 2 content - edited")]),
            p([node_link_begin(
                NodeType::Section,
                "content/0/content/1/content/1",
            )?]),
            p([t("Level 3 content - edited")]),
            p([t("More level 3 content - edited")]),
            p([t("New level 3 content")]),
            p([node_link_end(
                NodeType::Section,
                "content/0/content/1/content/1",
            )?]),
            p([t("More level 2 content - edited")]),
            p([t("New level 2 content")]),
            p([node_link_end(NodeType::Section, "content/0/content/1")?]),
            p([t("More level 1 content - edited")]),
            p([t("New level 1 content")]),
            p([node_link_end(NodeType::Section, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 top-level section");

        // Check level 1 section
        let Block::Section(level1_section) = &content[0] else {
            bail!("Block should be a section");
        };
        assert_eq!(
            level1_section.content.len(),
            4,
            "Level 1 section should have 4 blocks"
        );

        // Check level 1 content
        let Block::Paragraph(para) = &level1_section.content[0] else {
            bail!("First block in level 1 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Level 1 content - edited");

        // Check level 2 section
        let Block::Section(level2_section) = &level1_section.content[1] else {
            bail!("Second block in level 1 section should be a nested section");
        };
        assert_eq!(
            level2_section.content.len(),
            4,
            "Level 2 section should have 4 blocks"
        );

        // Check level 2 content
        let Block::Paragraph(para) = &level2_section.content[0] else {
            bail!("First block in level 2 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 2 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Level 2 content - edited");

        // Check level 3 section
        let Block::Section(level3_section) = &level2_section.content[1] else {
            bail!("Second block in level 2 section should be a nested section");
        };
        assert_eq!(
            level3_section.content.len(),
            3,
            "Level 3 section should have 3 blocks"
        );

        // Check level 3 content
        let Block::Paragraph(para) = &level3_section.content[0] else {
            bail!("First block in level 3 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 3 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Level 3 content - edited");

        let Block::Paragraph(para) = &level3_section.content[1] else {
            bail!("Second block in level 3 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 3 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "More level 3 content - edited");

        let Block::Paragraph(para) = &level3_section.content[2] else {
            bail!("Third block in level 3 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 3 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New level 3 content");

        // Check remaining level 2 content
        let Block::Paragraph(para) = &level2_section.content[2] else {
            bail!("Third block in level 2 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 2 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "More level 2 content - edited");

        let Block::Paragraph(para) = &level2_section.content[3] else {
            bail!("Fourth block in level 2 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 2 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New level 2 content");

        // Check remaining level 1 content
        let Block::Paragraph(para) = &level1_section.content[2] else {
            bail!("Third block in level 1 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 1 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "More level 1 content - edited");

        let Block::Paragraph(para) = &level1_section.content[3] else {
            bail!("Fourth block in level 1 section should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Level 1 paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New level 1 content");

        Ok(())
    }

    #[test]
    fn styled_block_with_changes() -> Result<()> {
        let original = art([stb(
            "highlight",
            [
                p([t("This is highlighted content")]),
                p([t("Another highlighted paragraph")]),
            ],
        )]);

        let mut edited = art([
            p([node_link_begin(NodeType::StyledBlock, "content/0")?]),
            p([t("This is highlighted content with edits")]),
            p([t("Another highlighted paragraph modified")]),
            p([t("New paragraph in styled block")]),
            p([node_link_end(NodeType::StyledBlock, "content/0")?]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 styled block");

        let Block::StyledBlock(styled_block) = &content[0] else {
            bail!("Block should be a styled block");
        };

        assert_eq!(styled_block.code.as_str(), "highlight");

        assert_eq!(
            styled_block.content.len(),
            3,
            "StyledBlock should have 3 edited blocks"
        );

        // Check first paragraph (edited content)
        let Block::Paragraph(para) = &styled_block.content[0] else {
            bail!("First block in styled block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("First paragraph should contain text");
        };
        assert_eq!(
            text.value.as_str(),
            "This is highlighted content with edits"
        );

        // Check second paragraph (edited content)
        let Block::Paragraph(para) = &styled_block.content[1] else {
            bail!("Second block in styled block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Second paragraph should contain text");
        };
        assert_eq!(
            text.value.as_str(),
            "Another highlighted paragraph modified"
        );

        // Check third paragraph (new content)
        let Block::Paragraph(para) = &styled_block.content[2] else {
            bail!("Third block in styled block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Third paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "New paragraph in styled block");

        Ok(())
    }

    #[test]
    fn code_expression_reconstituted_unchanged() -> Result<()> {
        let original = art([p([t("The result:"), ce("1 + 1", Some("python")), t(".")])]);

        let mut edited = art([p([
            t("The result is "),
            lnk(
                [t("2")],
                node_url_path(
                    NodeType::CodeExpression,
                    NodePath::from_str("content/0/content/1")?,
                    None,
                )
                .to_string(),
            ),
            t(" as calculated."),
        ])]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 1, "Should have 1 paragraph");

        let Block::Paragraph(para) = &content[0] else {
            bail!("Block should be a paragraph");
        };

        assert_eq!(
            para.content.len(),
            3,
            "Paragraph should have 3 inline elements"
        );

        // Check first text
        let Inline::Text(text) = &para.content[0] else {
            bail!("First inline should be text");
        };
        assert_eq!(text.value.as_str(), "The result is ");

        // Check reconstituted code expression
        let Inline::CodeExpression(code_expr) = &para.content[1] else {
            bail!("Second inline should be a code expression");
        };
        assert_eq!(code_expr.code.as_str(), "1 + 1");
        assert_eq!(code_expr.programming_language.as_deref(), Some("python"));

        // Check last text
        let Inline::Text(text) = &para.content[2] else {
            bail!("Third inline should be text");
        };
        assert_eq!(text.value.as_str(), " as calculated.");

        Ok(())
    }

    #[test]
    fn article_with_multiple_edited_blocks() -> Result<()> {
        let original = art([
            p([t("First paragraph")]),
            inb("include1.md"),
            sec([p([t("Section content")])]),
            inb("include2.md"),
            p([t("Last paragraph")]),
        ]);

        let mut edited = art([
            p([t("First paragraph - edited")]),
            p([node_link_begin(NodeType::IncludeBlock, "content/1")?]),
            p([t("Content from include1.md - edited")]),
            p([t("New content in include1")]),
            p([node_link_end(NodeType::IncludeBlock, "content/1")?]),
            p([node_link_begin(NodeType::Section, "content/2")?]),
            p([t("Section content - edited")]),
            p([node_link_end(NodeType::Section, "content/2")?]),
            p([node_link_begin(NodeType::IncludeBlock, "content/3")?]),
            p([t("Content from include2.md - edited")]),
            p([node_link_end(NodeType::IncludeBlock, "content/3")?]),
            p([t("Last paragraph - edited")]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 5, "Should have 5 blocks");

        // Check first paragraph (edited)
        let Block::Paragraph(para) = &content[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("First paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "First paragraph - edited");

        // Check first include block (edited content)
        let Block::IncludeBlock(include) = &content[1] else {
            bail!("Second block should be an include block");
        };
        assert_eq!(include.source.as_str(), "include1.md");
        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };
        assert_eq!(blocks.len(), 2, "First include should have 2 blocks");

        // Check section (edited content)
        let Block::Section(section) = &content[2] else {
            bail!("Third block should be a section");
        };
        assert_eq!(
            section.content.len(),
            1,
            "Section should have 1 edited block"
        );
        let Block::Paragraph(para) = &section.content[0] else {
            bail!("Section should contain a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Section paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Section content - edited");

        // Check second include block
        let Block::IncludeBlock(include) = &content[3] else {
            bail!("Fourth block should be an include block");
        };
        assert_eq!(include.source.as_str(), "include2.md");
        let Some(blocks) = &include.content else {
            bail!("Include block should have content");
        };
        assert_eq!(blocks.len(), 1, "Second include should have 1 block");

        // Check last paragraph (edited)
        let Block::Paragraph(para) = &content[4] else {
            bail!("Last block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Last paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Last paragraph - edited");

        Ok(())
    }

    #[test]
    fn article_with_mixed_edited_and_non_edited_content() -> Result<()> {
        let original = art([
            p([t("First unedited paragraph")]),
            sec([
                p([t("Section paragraph 1")]),
                inb("file.md"),
                p([t("Section paragraph 2")]),
            ]),
            p([t("Middle unedited paragraph")]),
            frb("x", "range(1, 5)", [p([t("Iteration")])]),
            p([t("Last unedited paragraph")]),
        ]);

        let mut edited = art([
            p([t("First unedited paragraph")]),
            p([node_link_begin(NodeType::Section, "content/1")?]),
            p([t("Section paragraph 1 - edited")]),
            p([node_link_begin(
                NodeType::IncludeBlock,
                "content/1/content/1",
            )?]),
            p([t("Content from file.md - edited")]),
            p([node_link_end(
                NodeType::IncludeBlock,
                "content/1/content/1",
            )?]),
            p([t("Section paragraph 2 - edited")]),
            p([t("New section paragraph 3")]),
            p([node_link_end(NodeType::Section, "content/1")?]),
            p([t("Middle unedited paragraph")]),
            p([node_link_begin(NodeType::ForBlock, "content/3")?]),
            p([t("Iteration - edited")]),
            p([node_link_end(NodeType::ForBlock, "content/3")?]),
            p([t("Last unedited paragraph")]),
        ]);

        reconstitute(&mut edited, Some(original));

        let Node::Article(Article { content, .. }) = edited else {
            bail!("Node should be an article");
        };

        assert_eq!(content.len(), 5, "Should have 5 blocks");

        // Check first paragraph (unedited)
        let Block::Paragraph(para) = &content[0] else {
            bail!("First block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "First unedited paragraph");

        // Check section
        let Block::Section(section) = &content[1] else {
            bail!("Second block should be a section");
        };
        assert_eq!(
            section.content.len(),
            4,
            "Section should have 4 edited blocks"
        );

        // Check middle paragraph (unedited)
        let Block::Paragraph(para) = &content[2] else {
            bail!("Third block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Middle unedited paragraph");

        // Check for block (edited content)
        let Block::ForBlock(for_block) = &content[3] else {
            bail!("Fourth block should be a for block");
        };
        assert_eq!(for_block.variable.as_str(), "x");
        assert_eq!(for_block.code.as_str(), "range(1, 5)");
        assert_eq!(
            for_block.content.len(),
            1,
            "ForBlock should have 1 edited content block"
        );

        let Block::Paragraph(para) = &for_block.content[0] else {
            bail!("ForBlock content should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Iteration - edited");

        // No iterations since there were no Section blocks in the edit
        assert_eq!(
            for_block.iterations, None,
            "ForBlock should have no iterations"
        );

        // Check last paragraph (unedited)
        let Block::Paragraph(para) = &content[4] else {
            bail!("Last block should be a paragraph");
        };
        let Some(Inline::Text(text)) = para.content.first() else {
            bail!("Paragraph should contain text");
        };
        assert_eq!(text.value.as_str(), "Last unedited paragraph");

        Ok(())
    }
}
