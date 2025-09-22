use itertools::Itertools;
use stencila_codec::{StructuringOperation::*, StructuringOptions};
use stencila_codec_links::decode_inlines as text_with_links;
use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Article, Block, Citation, CitationGroup, CitationMode, CitationOptions, Emphasis, Heading,
    Inline, InlinesBlock, Link, Node, Paragraph, Sentence, Strikeout, Strong, StyledInline,
    Subscript, Superscript, Text, Underline, VisitorMut, WalkControl,
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

        if let Inline::Superscript(..) = inline {
            self.visit_superscript(inline);
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
        if let Some(references) = self.first_walk.references.clone() {
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

        if self.options.should_perform(LinksToCitations) {
            for inline in inlines.iter_mut() {
                if matches!(inline, Inline::Link(..)) {
                    self.visit_link(inline);
                }
            }
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

    /// Convert links to citations
    ///
    /// Converts anchor links that point to reference IDs (e.g., href="#ref-1")
    /// to proper Citation nodes. Only converts links whose targets match
    /// existing reference IDs, preserving the link content as citation content.
    fn visit_link(&self, link: &mut Inline) {
        if let Inline::Link(Link {
            target, content, ..
        }) = &link
        {
            // Check if target starts with # and matches a reference ID
            if let Some(reference_id) = target.strip_prefix('#') {
                // Check if this reference ID exists in the collected references
                if let Some(references) = &self.first_walk.references {
                    let has_matching_reference = references
                        .iter()
                        .any(|reference| reference.id.as_deref() == Some(reference_id));

                    if has_matching_reference {
                        // Convert the link to a citation
                        *link = Inline::Citation(Citation {
                            target: reference_id.to_string(),
                            options: Box::new(CitationOptions {
                                content: Some(content.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    /// Extract single citations and citation groups from superscripts.
    ///
    /// This is also done in `normalize_citations`. See note their about why it is done in both places.
    fn visit_superscript(&self, superscript: &mut Inline) {
        if self.options.should_perform(NormalizeCitations)
            && let Inline::Superscript(Superscript { content, .. }) = superscript
            && content.len() == 1
        {
            if let Some(Inline::Citation(citation)) = content.first() {
                let mut citation = citation.clone();
                citation.citation_mode = Some(CitationMode::Parenthetical);
                *superscript = Inline::Citation(citation);
            } else if let Some(Inline::CitationGroup(..)) = content.first() {
                *superscript = content[0].clone();
            }
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
fn prepend_citation_group_item(citation_group: &mut CitationGroup, mut citation: Citation) {
    citation.citation_mode = None;

    citation_group.items.insert(0, citation);
}

/// Append a single [`Citation`] to a [`CitationGroup`]
fn append_citation_group_item(citation_group: &mut CitationGroup, mut citation: Citation) {
    citation.citation_mode = None;

    citation_group.items.push(citation);
}

/// Prepend items to a [`CitationGroup`] from a new start to the first in the current range
fn prepend_citation_group_range(citation_group: &mut CitationGroup, mut start: Citation) {
    start.citation_mode = None;

    // Get the first citation in the group to use as the end of the range
    if let Some(first) = citation_group.items.first() {
        // Extract the target prefix from the first citation first, then parse numeric suffixes
        // This is more consistent than parsing from content and then determining prefix
        let mut target_prefix = first.target.chars().collect_vec();
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
        let end_num = first.target[target_prefix.len()..]
            .parse::<u32>()
            .unwrap_or(0);

        // Only prepend if this is a valid range
        if start_num < end_num {
            // Generate the range ending before the first citation
            // (we don't want to duplicate the first citation)
            let mut new_items = Vec::new();

            // Add the start citation (which we already have and has citation_mode = None)
            new_items.push(start.clone());

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

            // Prepend the new items to the group (insert at the beginning)
            new_items.append(&mut citation_group.items);
            citation_group.items = new_items;
        } else {
            // If not a valid range, just prepend the start citation as-is
            citation_group.items.insert(0, start);
        }
    } else {
        // If the group is somehow empty, just add the start citation
        citation_group.items.push(start);
    }
}

/// Extend the items in a [`CitationGroup`] from the last in the current range to the new end
fn append_citation_group_range(citation_group: &mut CitationGroup, mut end: Citation) {
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

/// Merge adjacent citation groups
///
/// Does not attempt to extend numeric ranges of citations, simply concatenated
/// the two sets of citations. Because of how this function is used it mutates
/// the second group.
fn merge_citation_groups(first: CitationGroup, second: &mut CitationGroup) {
    second.items.splice(0..0, first.items);
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
                let text = value.to_string();
                let trimmed = text.trim();

                if value.ends_with("(") || value.ends_with("[") {
                    // Remove opening parenthesis/bracket before citation/s
                    value.pop();
                    had_parentheses = true;
                } else if matches!(trimmed, "," | ";" | "-" | "–")
                    && matches!(
                        output.iter().rev().nth(1),
                        Some(Inline::Citation(..) | Inline::CitationGroup(..))
                    )
                {
                    // Citation followed by comma or dash followed by Citation
                    if let Some(Inline::Citation(citation)) = output.iter().rev().nth(1) {
                        let previous = citation.clone();

                        // Pop off the separator and previous citation, combine into a citation group
                        output.pop(); // Remove separator
                        output.pop(); // Remove previous Citation

                        if matches!(trimmed, "," | ";") {
                            output.push(citation_group_from_pair(previous, current.clone()));
                        } else {
                            output.push(citation_group_from_range(previous, current.clone()));
                        }
                        continue;
                    }

                    // CitationGroup followed by comma or dash followed by Citation
                    if matches!(output.iter().rev().nth(1), Some(Inline::CitationGroup(..))) {
                        // Pop off the separator first
                        output.pop(); // Remove separator

                        // Now we can safely get a mutable reference to the CitationGroup
                        if let Some(Inline::CitationGroup(citation_group)) = output.last_mut() {
                            if matches!(trimmed, "," | ";") {
                                append_citation_group_item(citation_group, current.clone());
                            } else {
                                append_citation_group_range(citation_group, current.clone());
                            }
                            continue;
                        }
                    }
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

                if matches!(trimmed, "," | ";" | "-" | "–") {
                    // Citation followed by a Superscript (with comma or dash) followed by a Citation
                    if let Some(Inline::Citation(citation)) = output.iter().rev().nth(1) {
                        let previous = citation.clone();

                        // Pop off the Superscript and previous citation combine
                        // the previous and current citations into a citation group
                        output.pop(); // Remove Superscript
                        output.pop(); // Remove previous Citation

                        if matches!(trimmed, "," | ";") {
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
                            if matches!(trimmed, "," | ";") {
                                append_citation_group_item(citation_group, current.clone());
                            } else {
                                append_citation_group_range(citation_group, current.clone());
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
                append_citation_group_item(citation_group, current.clone());
                continue;
            }
        } else if let Inline::CitationGroup(citation_group) = &mut inline {
            // Citation followed by Text containing a separator (comma, semicolon, or dash) followed by a CitationGroup
            if let Some(Inline::Citation(citation)) = output.iter().rev().nth(1)
                && let Some(Inline::Text(Text { value, .. })) = output.last()
                && matches!(value.trim(), "," | ";" | "-" | "–")
            {
                let citation = citation.clone();
                let trimmed = value.trim().to_string(); // Convert to owned String to avoid borrowing issues

                // Pop off both the Citation and the Text and add the Citation to the current CitationGroup
                output.pop(); // Remove separator Text
                output.pop(); // Remove Citation

                if matches!(trimmed.as_str(), "," | ";") {
                    prepend_citation_group_item(citation_group, citation);
                } else {
                    prepend_citation_group_range(citation_group, citation);
                }
            }
            // CitationGroup followed by Text containing a separator followed by another CitationGroup
            else if let Some(Inline::CitationGroup(first_group)) = output.iter().rev().nth(1)
                && let Some(Inline::Text(Text { value, .. })) = output.last()
                && matches!(value.trim(), "," | ";" | "-" | "–")
            {
                let first_group = first_group.clone();

                // Pop off both the CitationGroup and the Text, then merge the groups
                output.pop(); // Remove separator Text
                output.pop(); // Remove first CitationGroup

                merge_citation_groups(first_group, citation_group);
            }
            // Citation followed by CitationGroup (no separator)
            else if let Some(Inline::Citation(citation)) = output.last() {
                let citation = citation.clone();

                // Pop off the Citation and prepend it to the current CitationGroup
                output.pop(); // Remove Citation

                prepend_citation_group_item(citation_group, citation);
            }
            // CitationGroup followed by another CitationGroup (no separator)
            else if let Some(Inline::CitationGroup(first_group)) = output.last() {
                let first_group = first_group.clone();

                // Pop off the first CitationGroup and merge with the current one
                output.pop(); // Remove first CitationGroup

                merge_citation_groups(first_group, citation_group);
            }
            // Do NOT `continue` here because the current Citation Group
            // needs to be pushed onto the output still
        } else if let Inline::Superscript(Superscript { content, .. }) = &inline {
            // Single Citation or CitationGroup within Superscript. This needs
            // to be done in `visit_superscript` as well (I don't understand why
            // that is the case after lots of debugging and testing I cam eot
            // the conclusion that it needs to be done in both places.)
            if let (1, Some(Inline::Citation(..) | Inline::CitationGroup(..))) =
                (content.len(), content.first())
            {
                // Replace Superscript with the Citation or CitationGroup
                let mut inline = content[0].clone();
                if let Inline::Citation(Citation { citation_mode, .. }) = &mut inline {
                    // Ensure that citation it is made parenthetical
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
                    // Ensure that citation it is made parenthetical
                    *citation_mode = Some(CitationMode::Parenthetical);
                };
                output.push(inline);
                continue;
            }
        } else if let Inline::Text(Text { value, .. }) = &inline
            && matches!(
                output.last(),
                Some(Inline::Citation(..) | Inline::CitationGroup(..))
            )
            && (value.starts_with(")") || value.starts_with("]"))
        {
            // Citation or citation group followed by Text starting with closing parenthesis or bracket
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
