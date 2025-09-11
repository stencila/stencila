use stencila_codec::{StructuringOperation::*, StructuringOptions};
use stencila_codec_links::decode_inlines as text_with_links;
use stencila_schema::{
    Article, Block, Emphasis, Heading, Inline, InlinesBlock, Node, Paragraph, Sentence, Strikeout,
    Strong, StyledInline, Subscript, Superscript, Text, Underline, VisitorMut, WalkControl,
};

use crate::{FirstWalk, should_remove_inline};

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

        if let Block::Paragraph(paragraph) = block {
            self.visit_paragraph(paragraph);
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::Emphasis(Emphasis { content, .. })
        | Inline::Sentence(Sentence { content, .. })
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
        if self.options.should_perform(TextToCitations) || self.options.should_perform(MathToCitations){
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

        if self.options.should_perform(RemoveEmptyText) || self.options.should_perform(TextToLinks)
        {
            // Apply any further structuring, including within replacements
            // from the first pass
            let mut inlines_new = Vec::with_capacity(inlines.len());
            for inline in inlines.drain(..) {
                if should_remove_inline(&inline) {
                    continue;
                } else if self.options.should_perform(TextToLinks)
                    && let Inline::Text(text) = &inline
                {
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

    fn visit_paragraph(&self, paragraph: &mut Paragraph) {
        if self.options.should_perform(ParagraphsToSentences) {
            // Split the paragraph into sentences based on punctuation
            let mut sentences = Vec::with_capacity(paragraph.content.len());
            let mut sentence = Vec::new();
            for mut inline in paragraph.content.drain(..) {
                if let Inline::Text(Text { value, .. }) = &mut inline {
                    let sentence_parts = split_text_into_sentences(value);
                    for (text, is_sentence_end) in sentence_parts {
                        if !text.is_empty() {
                            sentence.push(Inline::Text(Text::new(text.into())));
                        }

                        if is_sentence_end {
                            sentences.push(Inline::Sentence(Sentence::new(std::mem::take(
                                &mut sentence,
                            ))));
                        }
                    }
                } else {
                    sentence.push(inline);
                }
            }

            if !sentence.is_empty() {
                sentences.push(Inline::Sentence(Sentence::new(sentence)));
            }

            paragraph.content.append(&mut sentences)
        }
    }
}

/// Split text into sentence parts, returning (text, is_sentence_end) tuples
///
/// This function splits text on sentence-ending punctuation ('.', '!', '?')
/// followed by whitespace, but avoids splitting on abbreviations like "Mr." and "etc."
fn split_text_into_sentences(text: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        current.push(c);

        // Check if this character is sentence-ending punctuation
        if c == '.' || c == '!' || c == '?' {
            // Look ahead to see if followed by whitespace
            if let Some(&next) = chars.peek() {
                if next.is_whitespace() {
                    // Check if this is likely an abbreviation (only for periods)
                    let is_abbreviation = if c == '.' {
                        is_likely_abbreviation(&current)
                    } else {
                        false
                    };

                    if !is_abbreviation {
                        // Consume and include the whitespace in the current sentence
                        while let Some(&w) = chars.peek() {
                            if w.is_whitespace() {
                                if let Some(whitespace_char) = chars.next() {
                                    current.push(whitespace_char);
                                }
                            } else {
                                break;
                            }
                        }
                        // This is a sentence boundary
                        result.push((current.clone(), true));
                        current.clear();
                    }
                }
            } else {
                // End of string - this is also a sentence boundary
                result.push((current.clone(), true));
                current.clear();
            }
        }
    }

    // Add any remaining text as a non-sentence-ending part
    if !current.is_empty() {
        result.push((current, false));
    }

    result
}

/// Check if the current text likely ends with an abbreviation
fn is_likely_abbreviation(text: &str) -> bool {
    let trimmed = text.trim_end();
    if !trimmed.ends_with('.') {
        return false;
    }

    // Get the word before the period
    let without_period = &trimmed[..trimmed.len() - 1];
    let last_word = without_period.split_whitespace().last().unwrap_or("");

    // Common abbreviations that shouldn't trigger sentence splits
    #[rustfmt::skip]
    const ABBREVIATIONS: &[&str] = &[
        // Honorifics
        "Mr", "Mrs", "Ms", "Dr", "Prof", "Sr", "Jr",
        // Organizations
        "Inc", "Ltd", "Corp", "Co",
        // Addresses
        "St", "Ave", "Blvd", "Rd", "Ln",
        // Months
        "Jan", "Feb", "Mar", "Apr", "Jun", "Jul", "Aug", "Sep", "Sept",
        "Oct", "Nov", "Dec",
        // Days
        "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
        // Time
        "a.m", "p.m",
        // Other
        "e.g", "i.e",  "etc", "vs", "cf", "al",
    ];

    ABBREVIATIONS.contains(&last_word)
}

/// Determine if inlines contain at least one [`Link`]
fn has_links(inlines: Vec<Inline>) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Link(..)))
        .then_some(inlines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_text_into_sentences() {
        // Basic period
        assert_eq!(
            split_text_into_sentences("Hello. World"),
            vec![("Hello. ".to_string(), true), ("World".to_string(), false)]
        );

        // Question mark
        assert_eq!(
            split_text_into_sentences("Are you sure? Yes I am."),
            vec![
                ("Are you sure? ".to_string(), true),
                ("Yes I am.".to_string(), true)
            ]
        );

        // Exclamation mark
        assert_eq!(
            split_text_into_sentences("Great! That works."),
            vec![
                ("Great! ".to_string(), true),
                ("That works.".to_string(), true)
            ]
        );

        // Multiple spaces
        assert_eq!(
            split_text_into_sentences("First.  Second"),
            vec![
                ("First.  ".to_string(), true),
                ("Second".to_string(), false)
            ]
        );

        // No sentence boundary (no space after punctuation)
        assert_eq!(
            split_text_into_sentences("Mr. Smith went home"),
            vec![("Mr. Smith went home".to_string(), false)]
        );

        // End of string boundary
        assert_eq!(
            split_text_into_sentences("This is the end."),
            vec![("This is the end.".to_string(), true)]
        );

        // Mixed punctuation
        assert_eq!(
            split_text_into_sentences("First. Second! Third? Fourth"),
            vec![
                ("First. ".to_string(), true),
                ("Second! ".to_string(), true),
                ("Third? ".to_string(), true),
                ("Fourth".to_string(), false)
            ]
        );

        // Empty string
        assert_eq!(split_text_into_sentences(""), vec![]);

        // No punctuation
        assert_eq!(
            split_text_into_sentences("Just text"),
            vec![("Just text".to_string(), false)]
        );
    }
}
