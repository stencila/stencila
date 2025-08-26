use schema::{
    Admonition, Block, Emphasis, ForBlock, Heading, IfBlockClause, IncludeBlock, Inline, Node,
    Paragraph, Section, Strikeout, Strong, StyledBlock, StyledInline, Subscript, Superscript,
    Underline, VisitorMut, WalkControl,
};

use crate::Collector;

/// Replaces nodes and node properties with nodes collected by a [`Collector`]
pub(super) struct Replacer {
    collector: Collector,
}

impl VisitorMut for Replacer {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::Article(article) = node {
            //  Apply collected title
            if article.title.is_none()
                && let Some(title) = self.collector.title.take()
            {
                article.title = Some(title);
            }

            //  Apply collected abstract_
            if article.r#abstract.is_none()
                && let Some(abstract_) = self.collector.abstract_.take()
            {
                article.r#abstract = Some(abstract_);
            }

            //  Apply collected keywords
            if article.keywords.is_none()
                && let Some(keywords) = self.collector.keywords.take()
            {
                article.keywords = Some(keywords);
            }

            // Replace blocks in the content of the article
            self.replace_blocks(&mut article.content);

            // If any references were collected then assign them to article
            if let Some(references) = self.collector.references.take() {
                article.references = Some(references);
            }
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::Admonition(Admonition { content, .. })
        | Block::IncludeBlock(IncludeBlock {
            content: Some(content),
            ..
        })
        | Block::Section(Section { content, .. })
        | Block::StyledBlock(StyledBlock { content, .. }) = block
        {
            // Apply replacements to nested block content
            self.replace_blocks(content);
        } else if let Block::ForBlock(ForBlock {
            content,
            iterations,
            ..
        }) = block
        {
            // Apply replacements to nested block content
            self.replace_blocks(content);
            if let Some(iterations) = iterations {
                self.replace_blocks(iterations);
            }
        } else if let Block::Paragraph(Paragraph { content, .. })
        | Block::Heading(Heading { content, .. }) = block
        {
            // Apply replacements to nested inline content
            self.replace_inlines(content);
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &mut IfBlockClause) -> WalkControl {
        // Apply replacements to nested block content
        self.replace_blocks(&mut clause.content);

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::Emphasis(Emphasis { content, .. })
        | Inline::Strikeout(Strikeout { content, .. })
        | Inline::Strong(Strong { content, .. })
        | Inline::StyledInline(StyledInline { content, .. })
        | Inline::Subscript(Subscript { content, .. })
        | Inline::Superscript(Superscript { content, .. })
        | Inline::Underline(Underline { content, .. }) = inline
        {
            // Apply replacements to nested inline content
            self.replace_inlines(content);
        }

        WalkControl::Continue
    }
}

impl Replacer {
    /// Create a [`Replacer`] from a [`Collector`]
    pub fn new(collector: Collector) -> Self {
        Self { collector }
    }

    /// Replace blocks within a vector of blocks
    fn replace_blocks(&mut self, blocks: &mut Vec<Block>) {
        let mut new_blocks = Vec::with_capacity(blocks.len());

        for block in blocks.drain(..) {
            if let Some(node_id) = block.node_id()
                && let Some((.., replacements)) = self.collector.block_replacements.remove(&node_id)
            {
                new_blocks.extend(replacements);
                continue;
            }
            new_blocks.push(block);
        }

        *blocks = new_blocks;
    }

    /// Replace inlines within a vector of inlines
    fn replace_inlines(&mut self, inlines: &mut Vec<Inline>) {
        let mut new_inlines = Vec::with_capacity(inlines.len());

        for inline in inlines.drain(..) {
            if let Some(node_id) = inline.node_id()
                && let Some((replacement_type, replacements)) =
                    self.collector.inline_replacements.remove(&node_id)
            {
                use crate::collector::InlineReplacement;

                // Always apply link replacements
                if replacement_type == InlineReplacement::Links {
                    new_inlines.extend(replacements);
                    continue;
                }

                // Only apply citation replacement if it matches the collector's
                // determined citation style
                if let Some(ref citation_style) = self.collector.citation_style {
                    if &replacement_type == citation_style {
                        new_inlines.extend(replacements);
                        continue;
                    }
                } else {
                    // If no citation style determined, apply all replacements (fallback)
                    new_inlines.extend(replacements);
                    continue;
                }
            }
            new_inlines.push(inline);
        }

        *inlines = new_inlines;
    }
}
