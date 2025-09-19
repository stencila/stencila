use itertools::Itertools;
use stencila_codec::{StructuringOperation::*, StructuringOptions};
use stencila_codec_links::decode_inlines as text_with_links;
use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Article, Block, Citation, CitationGroup, CitationMode, CitationOptions, Emphasis, Heading,
    Inline, InlinesBlock, Node, Paragraph, Sentence, Strikeout, Strong, StyledInline, Subscript,
    Superscript, Text, Underline, VisitorMut, WalkControl,
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
        if self.options.should_perform(TextToCitations)
            || self.options.should_perform(MathToCitations)
        {
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

        if self.options.should_perform(NormalizeCitations) {
            // Normalize citation formatting and grouping. Two passes are made over the inlines so that in
            // the first pass any citations in superscripts can be "extracted" and then in the second pass
            // adjacent citations (that were originally superscripted) can be merged is necessary.
            *inlines = normalize_citations(normalize_citations(std::mem::take(inlines)));
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

/// Create a [`CitationGroup`] from a pair of [`Citation`]s
fn citation_group_from_pair(mut start: Citation, mut end: Citation) -> Inline {
    start.citation_mode = None;
    end.citation_mode = None;

    Inline::CitationGroup(CitationGroup {
        items: vec![start, end],
        ..Default::default()
    })
}

/// Create a [`CitationGroup`] by expanding the range defined by start and end [`Citation`]s
fn citation_group_from_range(mut start: Citation, mut end: Citation) -> Inline {
    start.citation_mode = None;
    end.citation_mode = None;

    // Extract the target prefix from the start citation first, then parse numeric suffixes
    // This is more consistent than parsing from content and then determining prefix
    let mut target_prefix = start.target.chars().collect_vec();
    while target_prefix
        .last()
        .map(|c| c.is_ascii_digit())
        .unwrap_or_default()
    {
        target_prefix.pop();
    }
    let target_prefix: String = target_prefix.into_iter().collect();

    // Parse the numeric suffixes from the targets
    let start_num = start.target[target_prefix.len()..]
        .parse::<u32>()
        .unwrap_or(0);
    let end_num = end.target[target_prefix.len()..]
        .parse::<u32>()
        .unwrap_or(0);

    // Generate the range if valid
    let items = if end_num > start_num {
        // Create a vector with the start citation, then the range between, then the end citation
        let mut items = vec![start];

        // Generate citations for the numbers between start and end (exclusive)
        for target in (start_num + 1)..end_num {
            items.push(Citation {
                target: [target_prefix.clone(), target.to_string()].concat(),
                options: Box::new(CitationOptions {
                    content: Some(vec![Inline::Text(Text::new(target.to_string().into()))]),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }

        // Add the end citation
        items.push(end);
        items
    } else {
        // If not a valid range, just return both citations as-is
        vec![start, end]
    };

    Inline::CitationGroup(CitationGroup {
        items,
        ..Default::default()
    })
}

/// Prepend a single [`Citation`] to a [`CitationGroup`]
fn prepend_citation_group(citation_group: &mut CitationGroup, mut citation: Citation) {
    citation.citation_mode = None;

    citation_group.items.insert(0, citation);
}

/// Append a single [`Citation`] to a [`CitationGroup`]
fn append_citation_group(citation_group: &mut CitationGroup, mut citation: Citation) {
    citation.citation_mode = None;

    citation_group.items.push(citation);
}

/// Extend the items in a [`CitationGroup`] from the last in the current range to the new end
fn extend_citation_group(citation_group: &mut CitationGroup, mut end: Citation) {
    end.citation_mode = None;

    // Get the last citation in the group to use as the start of the range
    if let Some(last) = citation_group.items.last() {
        // Extract the target prefix from the last citation first, then parse numeric suffixes
        // This is more consistent than parsing from content and then determining prefix
        let mut target_prefix = last.target.chars().collect_vec();
        while target_prefix
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or_default()
        {
            target_prefix.pop();
        }
        let target_prefix: String = target_prefix.into_iter().collect();

        // Parse the numeric suffixes from the targets
        let start_num = last.target[target_prefix.len()..]
            .parse::<u32>()
            .unwrap_or(0);
        let end_num = end.target[target_prefix.len()..]
            .parse::<u32>()
            .unwrap_or(0);

        // Only extend if this is a valid range
        if end_num > start_num {
            // Generate the range starting from the next number after the last citation
            // (we don't want to duplicate the last citation)
            let mut new_items = Vec::new();

            // Generate citations for the numbers between start and end (exclusive of end)
            for target in (start_num + 1)..end_num {
                new_items.push(Citation {
                    target: [target_prefix.clone(), target.to_string()].concat(),
                    options: Box::new(CitationOptions {
                        content: Some(vec![Inline::Text(Text::new(target.to_string().into()))]),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }

            // Add the end citation (which we already have and has citation_mode = None)
            new_items.push(end.clone());

            // Append the new items to the group
            citation_group.items.append(&mut new_items);
        } else {
            // If not a valid range, just add the end citation as-is
            citation_group.items.push(end);
        }
    } else {
        // If the group is somehow empty, just add the end citation
        citation_group.items.push(end);
    }
}

/// Normalize citation formatting and grouping
///
/// - removes parentheses and square brackets around citations,
/// - groups adjacent citations into citation groups,
/// - handles commas and semicolons between citations by grouping them,
/// - handles simple citation ranges (e.g., dash between two numeric citations),
/// - extracts citations from superscripts,
/// - sets appropriate citation modes
fn normalize_citations(input: Vec<Inline>) -> Vec<Inline> {
    let mut output = Vec::with_capacity(input.len());
    for mut inline in input.into_iter() {
        if let Inline::Citation(current) = &mut inline {
            let mut had_parentheses = false;

            // Text followed by Citation
            if let Some(Inline::Text(Text { value, .. })) = output.last_mut() {
                let trimmed = value.trim().to_string();
                if value.ends_with("(") || value.ends_with("[") {
                    // Remove opening parenthesis/bracket before citation/s
                    value.pop();
                    had_parentheses = true;
                } else if trimmed == ";"
                    || trimmed == ","
                        && matches!(
                            output.iter().rev().nth(1),
                            Some(Inline::Citation(..) | Inline::CitationGroup(..))
                        )
                {
                    // Pop off semicolon or comma between citations/groups
                    output.pop();
                } else if trimmed == "-"
                    || trimmed == "–"
                        && matches!(
                            output.iter().rev().nth(1),
                            Some(Inline::Citation(..) | Inline::CitationGroup(..))
                        )
                {
                    let previous = match output.iter().rev().nth(1) {
                        Some(Inline::Citation(previous)) => Some((previous, false)),
                        Some(Inline::CitationGroup(previous)) => {
                            previous.items.last().map(|c| (c, true))
                        }
                        _ => None,
                    };

                    if let Some((previous, previous_is_group)) = previous {
                        let mut target_prefix = previous.target.chars().collect_vec();
                        while target_prefix
                            .last()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or_default()
                        {
                            target_prefix.pop();
                        }
                        let target_prefix: String = target_prefix.into_iter().collect();

                        if let (Ok(start), Ok(end)) = (
                            to_text(&previous.options.content).parse::<u32>(),
                            to_text(&current.options.content).parse::<u32>(),
                        ) && end > start
                        {
                            // Dash between two numeric citations

                            // Pop off dash
                            output.pop();

                            // Generate citations over numeric range
                            let mut items = (start..=end)
                                .map(|target| Citation {
                                    target: [target_prefix.clone(), target.to_string()].concat(),
                                    options: Box::new(CitationOptions {
                                        content: Some(vec![Inline::Text(Text::new(
                                            target.to_string().into(),
                                        ))]),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                })
                                .collect_vec();

                            if !previous_is_group {
                                // Dash separated pair of citations so pop off the
                                // first citation and replace with a citation group with range
                                output.pop();
                                let mut group = CitationGroup {
                                    items,
                                    ..Default::default()
                                };
                                // Set citation mode to None for all items in group
                                for citation in &mut group.items {
                                    citation.citation_mode = None;
                                }
                                output.push(Inline::CitationGroup(group));
                            } else if let Some(Inline::CitationGroup(group)) = output.last_mut() {
                                // Citation after an existing citation group so extend
                                // the group with the new items (removing the last existing first
                                // since it is the start of the new range of items)
                                group.items.pop();
                                // Set citation mode to None for all items in group
                                for citation in &mut items {
                                    citation.citation_mode = None;
                                }
                                group.items.append(&mut items);
                            }

                            continue;
                        }
                    };
                }
            }

            // Set citation mode based on whether parentheses were removed
            if current.citation_mode.is_none() {
                current.citation_mode = Some(if had_parentheses {
                    CitationMode::Parenthetical
                } else {
                    CitationMode::Narrative
                });
            }

            // Superscript followed by Citation
            if let Some(Inline::Superscript(Superscript { content, .. })) = output.last_mut() {
                let text = to_text(content);
                let trimmed = text.trim();
                if matches!(trimmed, "," | "-" | "–") {
                    // Citation followed by a Superscript (with comma or dash) followed by a Citation
                    if let Some(Inline::Citation(citation)) = output.iter().rev().nth(1) {
                        let previous = citation.clone();

                        // Pop off the Superscript and previous citation combine
                        // the previous and current citations into a citation group
                        output.pop(); // Remove Superscript
                        output.pop(); // Remove previous Citation

                        if trimmed == "," {
                            output.push(citation_group_from_pair(previous, current.clone()));
                        } else {
                            output.push(citation_group_from_range(previous, current.clone()));
                        }
                        continue;
                    }

                    // CitationGroup followed by a Superscript (with comma or dash) followed by a Citation
                    if matches!(output.iter().rev().nth(1), Some(Inline::CitationGroup(..))) {
                        // Pop off the Superscript first
                        output.pop(); // Remove Superscript

                        // Now we can safely get a mutable reference to the CitationGroup
                        if let Some(Inline::CitationGroup(citation_group)) = output.last_mut() {
                            if trimmed == "," {
                                append_citation_group(citation_group, current.clone());
                            } else {
                                extend_citation_group(citation_group, current.clone());
                            }
                            continue;
                        }
                    }
                }
            }

            // Citation followed by Citation
            if matches!(output.last(), Some(Inline::Citation(..)))
                && let Some(Inline::Citation(previous)) = output.pop()
            {
                // Combine adjacent citations into a CitationGroup
                output.push(citation_group_from_pair(previous, current.clone()));
                continue;
            };

            // CitationGroup followed by Citation
            if let Some(Inline::CitationGroup(citation_group)) = output.last_mut() {
                // Add the Citation to the CitationGroup
                append_citation_group(citation_group, current.clone());
                continue;
            }
        } else if let Inline::CitationGroup(citation_group) = &mut inline {
            // Citation followed by Text containing a comma (optionally surrounded by whitespace) followed by a CitationGroup
            if let Some(Inline::Citation(citation)) = output.iter().rev().nth(1)
                && let Some(Inline::Text(Text { value, .. })) = output.last()
                && value.trim() == ","
            {
                let citation = citation.clone();

                // Pop off both the Citation and the Text and add the Citation to the current CitationGroup
                output.pop(); // Remove comma Text
                output.pop(); // Remove Citation
                prepend_citation_group(citation_group, citation);
                // Do NOT `continue` here because the current Citation Group
                // needs to be pushed onto the output still
            }
        } else if let Inline::Superscript(Superscript { content, .. }) = &inline {
            // Citation or CitationGroup within Superscript
            if let (1, Some(Inline::Citation(..) | Inline::CitationGroup(..))) =
                (content.len(), content.first())
            {
                // Replace Superscript with the Citation or CitationGroup
                let mut inline = content[0].clone();
                if let Inline::Citation(Citation { citation_mode, .. }) = &mut inline {
                    // Ensure that is citation it is made parenthetical
                    *citation_mode = Some(CitationMode::Parenthetical);
                };
                output.push(inline);
                continue;
            }
            // Citation or CitationGroup within Superscript (with only surrounding whitespace)
            else if let (
                3,
                Some(Inline::Text(Text { value: before, .. })),
                Some(Inline::Citation(..) | Inline::CitationGroup(..)),
                Some(Inline::Text(Text { value: after, .. })),
            ) = (
                content.len(),
                content.first(),
                content.get(1),
                content.last(),
            ) && before.trim().is_empty()
                && after.trim().is_empty()
            {
                // Replace Superscript with the Citation or CitationGroup
                let mut inline = content[1].clone();
                if let Inline::Citation(Citation { citation_mode, .. }) = &mut inline {
                    // Ensure that is citation it is made parenthetical
                    *citation_mode = Some(CitationMode::Parenthetical);
                };
                output.push(inline);
                continue;
            }
        }
        // Citation or CitationGroup followed by Text starting with closing parenthesis or bracket
        else if let Inline::Text(Text { value, .. }) = &inline
            && matches!(
                output.last(),
                Some(Inline::Citation(..) | Inline::CitationGroup(..))
            )
            && (value.starts_with(")") || value.starts_with("]"))
        {
            // Remove closing parenthesis/bracket after citation/s and mark previous citation as parenthetical
            if let Some(Inline::Citation(citation)) = output.last_mut()
                && citation.citation_mode.is_none()
            {
                citation.citation_mode = Some(CitationMode::Parenthetical);
            }
            output.push(Inline::Text(Text::new(value[1..].into())));
            continue;
        }

        output.push(inline)
    }

    output
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
