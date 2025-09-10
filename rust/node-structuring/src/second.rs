use stencila_codec_links::decode_inlines as text_with_links;
use stencila_schema::{
    Article, Block, Emphasis, Heading, Inline, InlinesBlock, Node, Paragraph, Strikeout, Strong,
    StyledInline, Subscript, Superscript, Underline, VisitorMut, WalkControl,
};

use crate::{FirstWalk, StructuringOperation::*, StructuringOptions};

/// Second structuring walk
///
/// Walks over a node and uses information collected during the first walk to
/// perform a second round of structuring focussed on inline content.
pub(super) struct SecondWalk {
    /// The structuring options
    options: StructuringOptions,

    /// The first structuring walk
    first_walk: FirstWalk,
}

impl VisitorMut for SecondWalk {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::Article(article) = node {
            self.visit_article(article);
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::Paragraph(Paragraph { content, .. })
        | Block::Heading(Heading { content, .. })
        | Block::InlinesBlock(InlinesBlock { content, .. }) = block
        {
            // Visit nested inline content
            self.visit_inlines(content);
        }

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
            // Visit nested inline content
            self.visit_inlines(content);
        }

        WalkControl::Continue
    }
}

impl SecondWalk {
    pub fn new(options: StructuringOptions, first_walk: FirstWalk) -> Self {
        Self {
            options,
            first_walk,
        }
    }

    /// Visit an article
    fn visit_article(&mut self, article: &mut Article) {
        //  Apply any title collected in the first walk
        if article.title.is_none()
            && let Some(title) = self.first_walk.title.take()
        {
            article.title = Some(title);
        }

        //  Apply any abstract collected in the first walk
        if article.r#abstract.is_none()
            && let Some(abstract_) = self.first_walk.abstract_.take()
        {
            article.r#abstract = Some(abstract_);
        }

        //  Apply any keywords collected in the first walk
        if article.options.keywords.is_none()
            && let Some(keywords) = self.first_walk.keywords.take()
        {
            article.options.keywords = Some(keywords);
        }

        // Apply any references collected in the first walk
        if let Some(references) = self.first_walk.references.take() {
            article.references = Some(references);
        }
    }

    /// Visit a vector of inlines
    fn visit_inlines(&mut self, inlines: &mut Vec<Inline>) {
        if self.options.should_perform(TextCitations) {
            // Apply any citation replacements
            let mut inlines_new = Vec::with_capacity(inlines.len());
            for inline in inlines.drain(..) {
                if let Some(node_id) = inline.node_id()
                    && self.first_walk.citations.contains_key(&node_id)
                {
                    let (replacement_style, replacements) = self
                        .first_walk
                        .citations
                        .remove(&node_id)
                        .expect("checked above");
                    if let Some(citation_style) = &self.first_walk.citation_style {
                        if &replacement_style == citation_style {
                            // Matches determined citation style so replace
                            inlines_new.extend(replacements);
                        } else {
                            // Does not match determined citation style so keep original
                            inlines_new.push(inline);
                        }
                    } else {
                        // If no citation style determined, apply all replacements (fallback)
                        inlines_new.extend(replacements);
                    }
                } else {
                    // No possible citation replacement so keep original
                    inlines_new.push(inline);
                }
            }

            *inlines = inlines_new;
        }

        if self.options.should_perform(TextLinks) {
            // Apply any further structuring, including within replacements
            // from the first pass
            let mut inlines_new = Vec::with_capacity(inlines.len());
            for inline in inlines.drain(..) {
                if let Inline::Text(text) = &inline {
                    if let Some(inlines) = has_links(text_with_links(&text.value)) {
                        inlines_new.extend(inlines);
                    } else {
                        inlines_new.push(inline)
                    }
                } else {
                    inlines_new.push(inline)
                }
            }

            *inlines = inlines_new;
        }
    }
}

/// Determine if inlines contain at least one [`Link`]
fn has_links(inlines: Vec<Inline>) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Link(..)))
        .then_some(inlines)
}
