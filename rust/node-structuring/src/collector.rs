use std::{collections::HashMap, sync::LazyLock};

use inflector::Inflector;
use itertools::Itertools;
use regex::Regex;
use strum::Display;

use stencila_codec_biblio::decode::{
    bracketed_numeric_citation, parenthetic_numeric_citation, superscripted_numeric_citation,
    text_to_reference, text_with_author_year_citations, text_with_bracketed_numeric_citations,
    text_with_parenthetic_numeric_citations,
};
use stencila_codec_links::decode_inlines as text_with_links;
use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Admonition, Article, Block, Figure, ForBlock, Heading, IncludeBlock, Inline, List, ListOrder,
    MathInline, Node, NodeId, Paragraph, Reference, Section, SectionType, StyledBlock, Text,
    VisitorMut, WalkControl, shortcuts::t,
};

use crate::{CitationStyle, StructuringOptions};

/// A type of potential block replacement
#[derive(Debug)]
pub(super) enum BlockReplacement {
    /// Remove empty blocks
    Empty,

    /// Remove the title from the content
    Title,

    /// Remove the abstract from the content
    Abstract,

    /// Remove keywords from the content
    Keywords,

    /// Remove other frontmatter
    Frontmatter,

    /// Remove references (including header) from the content
    References,
}

/// A type of potential inline replacement
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
pub(super) enum InlineReplacement {
    /// Replace text with a mix of text and author-year citations
    AuthorYearCitations,

    /// Replace text with a mix of text and bracketed numeric citations
    BracketedNumericCitations,

    /// Replace text with a mix of text and parenthetic numeric citations
    ParentheticNumericCitations,

    /// Replace text with a mix of text and superscripted numeric citations
    SuperscriptedNumericCitations,

    /// Replace text with a mix of text and links (e.g. to figures, tables, appendices, URLs)
    Links,
}

/// Walks over the node collecting replacements, citations and references
#[derive(Debug, Default)]
pub(super) struct Collector {
    /// The structuring options
    options: StructuringOptions,

    /// Replacements for block nodes
    pub block_replacements: HashMap<NodeId, (BlockReplacement, Vec<Block>)>,

    /// Replacements for inline nodes
    pub inline_replacements: HashMap<NodeId, (InlineReplacement, Vec<Inline>)>,

    /// The extracted title of the work
    pub title: Option<Vec<Inline>>,

    /// Whether in frontmatter (after title and before first section)
    in_frontmatter: bool,

    /// Whether a primary section has been hit
    hit_primary_section: bool,

    /// Whether in an abstract section
    ///
    /// Whether a "keywords" heading has been encountered, in which case keywords will
    /// be extracted until the next heading is encountered. Note that keywords are
    /// also extracted from paragraphs starting with "Keywords" and punctuation.
    in_abstract: bool,

    /// The extracted keywords
    pub abstract_: Option<Vec<Block>>,

    /// Whether in a keyword section
    ///
    /// Whether a "keywords" heading has been encountered, in which case keywords will
    /// be extracted until the next heading is encountered. Note that keywords are
    /// also extracted from paragraphs starting with "Keywords" and punctuation.
    in_keywords: bool,

    /// The extracted keywords
    pub keywords: Option<Vec<String>>,

    /// Whether currently in the References (or Bibliography) section
    in_references: bool,

    /// References extracted from walking node
    pub references: Option<Vec<Reference>>,

    /// Whether references were found in an ordered (numbered) list
    pub references_are_ordered: bool,

    /// Determined citation style based on heuristics
    pub citation_style: Option<InlineReplacement>,

    /// Last numbered heading level seen, for fallback level assignment
    last_numbered_level: Option<i64>,
}

impl Collector {
    pub fn new(options: StructuringOptions) -> Self {
        Self {
            options,
            ..Default::default()
        }
    }
}

impl VisitorMut for Collector {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::Article(Article { content, .. }) = node {
            self.in_frontmatter = true;
            self.visit_blocks(content);
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Heading(heading) => self.visit_heading(heading),
            Block::Paragraph(paragraph) => self.visit_paragraph(paragraph),
            Block::List(list) => self.visit_list(list),

            // Process nested block content for figure detection
            Block::Admonition(Admonition { content, .. })
            | Block::IncludeBlock(IncludeBlock {
                content: Some(content),
                ..
            })
            | Block::Section(Section { content, .. })
            | Block::StyledBlock(StyledBlock { content, .. }) => self.visit_blocks(content),
            Block::ForBlock(ForBlock {
                content,
                iterations,
                ..
            }) => {
                self.visit_blocks(content);
                if let Some(iterations) = iterations {
                    self.visit_blocks(iterations);
                }
                WalkControl::Continue
            }

            _ => WalkControl::Continue,
        }
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if self.in_references {
            // Do not do the following if in references section since things like
            // number in brackets are normal parts of the formatting of references,
            // not citations!
            return WalkControl::Continue;
        }

        match inline {
            Inline::MathInline(math) => self.visit_math_inline(math),
            Inline::Text(text) => self.visit_text(text),
            _ => {}
        }

        WalkControl::Continue
    }
}

impl Collector {
    /// Visit a vector of blocks such as Article or Section content
    ///
    /// Detects adjacent ImageObject and caption pairs in the article content
    /// and creates Figure block replacements. Does the same for tables and
    /// preceding captions.
    ///
    /// Previously this generated block replacements to be handled by
    /// `Replacer`. However, that caused issues when there were citations in the
    /// caption due to conflicting replacements. This approach avoids that (and
    /// is more performant).
    fn visit_blocks(&mut self, blocks: &mut Vec<Block>) -> WalkControl {
        let mut index = 0;
        while index < blocks.len().saturating_sub(1) {
            // Check for ImageObject followed by caption
            if let (
                Some(Block::ImageObject(..)),
                Some(
                    Block::Paragraph(Paragraph { content, .. })
                    | Block::Heading(Heading { content, .. }),
                ),
            ) = (blocks.get(index), blocks.get(index + 1))
            {
                if let Some((label, prefix, ..)) = detect_figure_caption(content) {
                    // Remove the image and paragraph so that they can be placed in figure
                    let Some((
                        image,
                        Block::Paragraph(Paragraph { content, .. })
                        | Block::Heading(Heading { content, .. }),
                    )) = blocks.drain(index..=index + 1).collect_tuple()
                    else {
                        unreachable!("asserted above")
                    };

                    let mut caption = Paragraph::new(content);

                    // Remove the prefix from the caption
                    remove_caption_prefix(&mut caption, &prefix);

                    // If the cleaned caption is empty (if just "Figure X" and
                    // the next block is a paragraph that is NOT a caption, then
                    // use it as the caption.
                    if to_text(&caption).trim().is_empty()
                        && let Some(Block::Paragraph(next)) = blocks.get(index)
                        && detect_figure_caption(&next.content).is_none()
                        && let Block::Paragraph(next) = blocks.remove(index)
                    {
                        caption = next;
                    };

                    // Create and insert the figure
                    let mut figure = Figure::new(vec![image]);
                    figure.id = Some(["fig-", &label.to_kebab_case()].concat());
                    figure.label = Some(label);
                    figure.label_automatically = Some(false);
                    figure.caption = Some(vec![Block::Paragraph(caption)]);
                    blocks.insert(index, Block::Figure(figure));
                }
            }
            // Check for caption followed by ImageObject
            else if let (
                Some(
                    Block::Paragraph(Paragraph { content, .. })
                    | Block::Heading(Heading { content, .. }),
                ),
                Some(Block::ImageObject(..)),
            ) = (blocks.get(index), blocks.get(index + 1))
            {
                if let Some((label, prefix, ..)) = detect_figure_caption(content) {
                    // Remove the image and paragraph so that they can be placed in figure
                    let Some((
                        Block::Paragraph(Paragraph { content, .. })
                        | Block::Heading(Heading { content, .. }),
                        image,
                    )) = blocks.drain(index..=index + 1).collect_tuple()
                    else {
                        unreachable!("asserted above")
                    };

                    let mut caption = Paragraph::new(content);

                    // Remove the prefix from the caption
                    remove_caption_prefix(&mut caption, &prefix);

                    // Create and insert the figure
                    let mut figure = Figure::new(vec![image]);
                    figure.id = Some(["fig-", &label.to_kebab_case()].concat());
                    figure.label = Some(label);
                    figure.label_automatically = Some(false);
                    figure.caption = Some(vec![Block::Paragraph(caption)]);
                    blocks.insert(index, Block::Figure(figure));
                }
            }
            // Check for caption followed by Table (only caption before table is
            // considered, not the reverse)
            else if let (
                Some(
                    Block::Paragraph(Paragraph { content, .. })
                    | Block::Heading(Heading { content, .. }),
                ),
                Some(Block::Table(..)),
            ) = (blocks.get(index), blocks.get(index + 1))
            {
                if let Some((label, prefix, ..)) = detect_table_caption(content) {
                    // Remove the paragraph it can be placed in the table
                    let Block::Paragraph(mut caption) = blocks.remove(index) else {
                        unreachable!("asserted above")
                    };

                    // Remove the prefix from the caption
                    remove_caption_prefix(&mut caption, &prefix);

                    // Update the table (note using index, not index + 1, here because paragraph removed)
                    let Block::Table(table) = &mut blocks[index] else {
                        unreachable!("asserted above")
                    };
                    table.id = Some(["tab-", &label.to_kebab_case()].concat());
                    table.label = Some(label);
                    table.label_automatically = Some(false);
                    table.caption = Some(vec![Block::Paragraph(caption)]);
                }
            }
            // Check for table label (no caption), followed paragraph, followed
            // by Table
            else if let (
                Some(
                    Block::Paragraph(Paragraph { content: label, .. })
                    | Block::Heading(Heading { content: label, .. }),
                ),
                Some(Block::Paragraph(Paragraph { content, .. })),
                Some(Block::Table(..)),
            ) = (
                blocks.get(index),
                blocks.get(index + 1),
                blocks.get(index + 2),
            ) && let Some((label, .., false)) = detect_table_caption(label)
                && detect_table_caption(content).is_none()
            {
                // Remove the label and caption blocks so the latter can be placed in the table
                let Some((_label, caption)) = blocks.drain(index..=index + 1).collect_tuple()
                else {
                    unreachable!("asserted above")
                };

                // Update the table (note using index, not index + 1, here because paragraph removed)
                let Block::Table(table) = &mut blocks[index] else {
                    unreachable!("asserted above")
                };
                table.id = Some(["tab-", &label.to_kebab_case()].concat());
                table.label = Some(label);
                table.label_automatically = Some(false);
                table.caption = Some(vec![caption]);
            }

            index += 1;
        }

        WalkControl::Continue
    }

    /// Visit a [`Heading`] node
    ///
    /// Strips numbering from headings, overrides level based on numbering, and
    /// detects when entering or leaving the References/Bibliography section.
    fn visit_heading(&mut self, heading: &mut Heading) -> WalkControl {
        let text = to_text(&heading.content);

        // Extract numbering and determine depth
        let (numbering, numbering_depth, cleaned_text) = extract_heading_numbering(&text);

        // Remove empty heading
        if cleaned_text.is_empty() {
            self.block_replacements
                .insert(heading.node_id(), (BlockReplacement::Empty, Vec::new()));

            return WalkControl::Break;
        }

        // Detect section type from cleaned text
        let section_type = SectionType::from_text(&cleaned_text).ok();

        // Extract title and turn on frontmatter handling
        if self.options.extract_title && self.title.is_none() && !self.hit_primary_section
            && numbering.is_none()
            // Heading level 1 or 2
            && heading.level <= 2
            // Not a recognized section heading
            && section_type.is_none()
        {
            self.title = Some(heading.content.drain(..).collect());

            self.block_replacements
                .insert(heading.node_id(), (BlockReplacement::Title, Vec::new()));

            return WalkControl::Break;
        }

        let cleaned_text_lowercase = cleaned_text.to_lowercase();

        // Determine if in abstract section
        if matches!(section_type, Some(SectionType::Abstract)) {
            self.in_abstract = true;
            self.block_replacements
                .insert(heading.node_id(), (BlockReplacement::Abstract, Vec::new()));
        } else {
            self.in_abstract = false;
        }

        // Determine if in keywords section
        if cleaned_text_lowercase == "keywords" || cleaned_text_lowercase == "key words" {
            self.in_keywords = true;
            self.block_replacements
                .insert(heading.node_id(), (BlockReplacement::Keywords, Vec::new()));
        } else {
            self.in_keywords = false;
        }

        let is_primary_section = section_type
            .as_ref()
            .map(is_primary_section_type)
            .unwrap_or_default();

        // Determine effective level based on priority: known section types > numbering > fallback
        let level = if is_primary_section {
            // Known primary section types always get level 1
            1
        } else if numbering_depth > 0 {
            let numbered_level = numbering_depth as i64;
            // Track the last numbered heading level
            self.last_numbered_level = Some(numbered_level);
            numbered_level
        } else if let Some(last_level) = self.last_numbered_level {
            // If no numbering detected and no known section type, and we've seen numbered headings before,
            // assign level as last numbered + 1 (for OCR inaccuracy in deep headings)
            last_level + 1
        } else {
            heading.level
        };

        // Update heading level and content, if necessary
        heading.level = level;
        if cleaned_text != text {
            heading.content = vec![t(&cleaned_text)];
        }

        // Update flags based on heading level determined above
        if is_primary_section {
            self.hit_primary_section = true;
            self.in_frontmatter = false;
        }

        // If still in frontmatter remove this heading
        if self.options.discard_frontmatter && self.in_frontmatter {
            self.block_replacements.insert(
                heading.node_id(),
                (BlockReplacement::Frontmatter, Vec::new()),
            );
        }

        // Update whether or not in references
        if matches!(section_type, Some(SectionType::References)) {
            self.in_references = true;

            self.block_replacements.insert(
                heading.node_id(),
                (BlockReplacement::References, Vec::new()),
            );
        } else {
            self.in_references = false;
        }

        WalkControl::Continue
    }

    /// Visit a [`Paragraph`] node
    ///
    /// If in the references section, parses the paragraph as a [`Reference`].
    fn visit_paragraph(&mut self, paragraph: &Paragraph) -> WalkControl {
        let mut remove = None;

        if self.in_abstract {
            let block = Block::Paragraph(paragraph.clone());
            if let Some(abstract_) = self.abstract_.as_mut() {
                abstract_.push(block);
            } else {
                self.abstract_ = Some(vec![block]);
            }
            remove = Some(BlockReplacement::Abstract);
        }

        if self.keywords.is_none() {
            let text = to_text(paragraph);

            if self.in_keywords {
                let words = text
                    .trim_end_matches(['.'])
                    .split(",")
                    .map(|word| word.trim().to_string())
                    .collect_vec();
                if let Some(keywords) = self.keywords.as_mut() {
                    keywords.extend(words);
                } else {
                    self.keywords = Some(words);
                }
                remove = Some(BlockReplacement::Keywords);
            } else if let Some(text) = text
                .strip_prefix("Keywords")
                .or_else(|| text.strip_prefix("KEYWORDS"))
                .or_else(|| text.strip_prefix("Key words"))
                .or_else(|| text.strip_prefix("KEY WORDS"))
            {
                let words = text
                    .trim_start_matches([':', '-', ' '])
                    .trim_end_matches(['.'])
                    .split(",")
                    .map(|word| word.trim().to_string())
                    .collect_vec();
                self.keywords = Some(words);
                remove = Some(BlockReplacement::Keywords);
            }
        }

        // Remove paragraphs in frontmatter (usually authors and
        // their affiliations)
        if self.options.discard_frontmatter && self.in_frontmatter {
            remove = Some(BlockReplacement::Frontmatter);
        }

        if self.in_references {
            let text = to_text(paragraph);
            let reference = text_to_reference(&text);
            if let Some(references) = self.references.as_mut() {
                references.push(reference);
            } else {
                self.references = Some(vec![reference]);
            }
            remove = Some(BlockReplacement::References);
        }

        if let Some(replacement) = remove {
            self.block_replacements
                .insert(paragraph.node_id(), (replacement, Vec::new()));
        }

        WalkControl::Continue
    }

    /// Visit a [`List`] node
    ///
    /// If in the references section, transforms the list to a set of
    /// [`Reference`]s to assign to the root node.
    fn visit_list(&mut self, list: &List) -> WalkControl {
        if self.in_references {
            let is_numeric = matches!(list.order, ListOrder::Ascending);

            // Record whether this references list is ordered/numbered
            self.references_are_ordered = is_numeric;

            let mut references = Vec::new();
            for (index, item) in list.items.iter().enumerate() {
                let text = to_text(item);
                let mut reference = text_to_reference(&text);
                // If the list is numeric then set the id to the reference
                if is_numeric {
                    reference.id = Some(format!("ref-{}", index + 1));
                }
                references.push(reference);
            }

            if !references.is_empty() {
                self.references = Some(references);
            }

            self.block_replacements
                .insert(list.node_id(), (BlockReplacement::References, Vec::new()));

            // Break walk because content in each item already processed
            WalkControl::Break
        } else {
            WalkControl::Continue
        }
    }

    /// Visit a [`MathInline`] node
    fn visit_math_inline(&mut self, math: &MathInline) {
        if let Some(inline) = bracketed_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (InlineReplacement::BracketedNumericCitations, vec![inline]),
            );
        }

        if let Some(inline) = parenthetic_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (InlineReplacement::ParentheticNumericCitations, vec![inline]),
            );
        }

        if let Some(inline) = superscripted_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (
                    InlineReplacement::SuperscriptedNumericCitations,
                    vec![inline],
                ),
            );
        }
    }

    /// Visit a [`Text`] node
    fn visit_text(&mut self, text: &mut Text) {
        // Extract any links from newly generated text in a citation replacement
        // We process citations, and then links (instead of the other way
        // around), so that parts of citations that may look like links are not
        // matched (e.g. "(Smith 1990, table 3)")
        macro_rules! extract_links {
            ($inlines: expr) => {
                for inline in &$inlines {
                    if let Inline::Text(text) = inline {
                        if let Some(inlines) = has_links(text_with_links(&text.value)) {
                            self.inline_replacements
                                .insert(text.node_id(), (InlineReplacement::Links, inlines));
                        }
                    }
                }
            };
        }

        let mut citations_detected = false;

        if let Some(inlines) = has_citations(text_with_author_year_citations(&text.value)) {
            citations_detected = true;
            extract_links!(inlines);
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::AuthorYearCitations, inlines),
            );
        }

        if let Some(inlines) = has_citations(text_with_bracketed_numeric_citations(&text.value)) {
            citations_detected = true;
            extract_links!(inlines);
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::BracketedNumericCitations, inlines),
            );
        }

        if let Some(inlines) = has_citations(text_with_parenthetic_numeric_citations(&text.value)) {
            citations_detected = true;
            extract_links!(inlines);
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::ParentheticNumericCitations, inlines),
            );
        }

        // Extract any links in this text. Note that, if this text is replaced because it has
        // citations then this replacement will not apply 9because this text's node id will not
        // be in the first pass.
        if !citations_detected && let Some(inlines) = has_links(text_with_links(&text.value)) {
            self.inline_replacements
                .insert(text.node_id(), (InlineReplacement::Links, inlines));
        }
    }

    /// Determine the citation style of the document
    ///
    /// This method analyzes the collected references and citation replacements
    /// to decide which citation style should be used for the document.
    /// If a citation style is specified, it will be used instead of automatic determination.
    #[tracing::instrument(skip(self))]
    pub fn determine_citation_style(&mut self, specified_style: Option<CitationStyle>) {
        // Count occurrences of each citation style
        let mut style_counts = std::collections::HashMap::new();

        for (replacement_type, _) in self.inline_replacements.values() {
            style_counts
                .entry(replacement_type)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        use InlineReplacement::*;

        // Determine citation style based on heuristics or use specified style
        self.citation_style = if let Some(style) = specified_style {
            tracing::debug!("Using specified citation style");
            Some(match style {
                CitationStyle::AuthorYear => AuthorYearCitations,
                CitationStyle::BracketedNumeric => BracketedNumericCitations,
                CitationStyle::ParentheticNumeric => ParentheticNumericCitations,
                CitationStyle::SuperscriptedNumeric => SuperscriptedNumericCitations,
            })
        } else if style_counts.is_empty() {
            tracing::debug!("No citations found, no style to determine");
            None
        } else if self.references_are_ordered {
            // If references are numbered, choose the style with the greatest count
            let style = style_counts
                .iter()
                .filter(|(key, _)| {
                    matches!(
                        key,
                        BracketedNumericCitations
                            | ParentheticNumericCitations
                            | SuperscriptedNumericCitations
                    )
                })
                .max_by_key(|(_, count)| *count)
                .map(|(style, _)| (*style).clone())
                .unwrap_or(AuthorYearCitations);
            tracing::debug!("Using numeric citation style with highest count: {style}");
            Some(style)
        } else {
            // If references are not numbered then none of the numeric styles will be valid
            // so assume author-year
            tracing::debug!("References are not ordered, so assuming author-year citations");
            Some(AuthorYearCitations)
        };
    }
}

/// Check if a section type should always be forced to level 1 (top-level sections)
fn is_primary_section_type(section_type: &SectionType) -> bool {
    matches!(
        section_type,
        SectionType::Abstract
            | SectionType::Summary
            | SectionType::NonTechnicalSummary
            | SectionType::Introduction
            | SectionType::Materials
            | SectionType::Methods
            | SectionType::Results
            | SectionType::Discussion
            | SectionType::Conclusions
            | SectionType::References
            | SectionType::Acknowledgements
            | SectionType::Declarations
            | SectionType::Funding
            | SectionType::CompetingInterests
            | SectionType::AuthorContributions
            | SectionType::DataAvailability
            | SectionType::CodeAvailability
            | SectionType::Ethics
            | SectionType::ConsentStatements
            | SectionType::Reproducibility
            | SectionType::Preregistration
            | SectionType::SupplementaryMaterials
            | SectionType::Appendix
    )
}

/// Detect if inlines match a figure caption pattern
///
/// Returns a tuple of (figure_label, prefix_to_remove) if the text
/// starts with "Figure X" or "Fig X" where X is a number.
fn detect_figure_caption(inlines: &Vec<Inline>) -> Option<(String, String, bool)> {
    // Detect figure captions like "Figure 1.", "Fig 2:", "Figure 12 -", "Figure A2" etc.
    static FIGURE_CAPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(?:Figure|Fig\.?)\s*([A-Z]?\d+)[.:\-\s]*").expect("invalid regex")
    });

    let text = to_text(inlines);

    if let Some(captures) = FIGURE_CAPTION_REGEX.captures(&text) {
        let figure_label = captures[1].to_string();
        let matched_text = captures.get(0)?.as_str();

        let has_caption = !text.replace(matched_text, "").trim().is_empty();

        Some((figure_label, matched_text.to_string(), has_caption))
    } else {
        None
    }
}

/// Detect if inlines match a table caption pattern
///
/// Returns a tuple of (table_label, prefix_to_remove) if the text
/// starts with "Table X" where X is a number.
fn detect_table_caption(inlines: &Vec<Inline>) -> Option<(String, String, bool)> {
    // Detect table captions like "Table 1.", "Table 2:", "Table 12 -", , "Table B3" etc.
    static TABLE_CAPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(?:Table)\s*([A-Z]?\d+)[.:\-\s]*").expect("invalid regex")
    });

    let text = to_text(inlines);

    if let Some(captures) = TABLE_CAPTION_REGEX.captures(&text) {
        let table_label = captures[1].to_string();
        let matched_text = captures.get(0)?.as_str();

        let has_caption = !text.replace(matched_text, "").trim().is_empty();

        Some((table_label, matched_text.to_string(), has_caption))
    } else {
        None
    }
}

/// Strip the figure caption prefix from a paragraph in-place
///
/// This modifies the paragraph to remove the figure prefix while preserving
/// the original node IDs for later text replacements.
fn remove_caption_prefix(paragraph: &mut Paragraph, prefix: &str) {
    remove_prefix_from_inlines(&mut paragraph.content, prefix);
}

/// Recursively remove a prefix from the beginning of a vector of inline elements
///
/// This function handles nested inline elements like Emphasis, Strong, Underline, etc. that might
/// contain the text to be removed. It modifies the inlines vector in place.
fn remove_prefix_from_inlines(inlines: &mut Vec<Inline>, prefix: &str) {
    if prefix.is_empty() || inlines.is_empty() {
        return;
    }

    match inlines[0].clone() {
        Inline::Text(text_node) => {
            remove_prefix_from_text(&text_node, inlines, prefix);
        }
        Inline::Emphasis(emphasis) => {
            remove_prefix_from_nested(&emphasis.content, inlines, prefix);
        }
        Inline::Strong(strong) => {
            remove_prefix_from_nested(&strong.content, inlines, prefix);
        }
        Inline::Underline(underline) => {
            remove_prefix_from_nested(&underline.content, inlines, prefix);
        }
        Inline::Strikeout(strikeout) => {
            remove_prefix_from_nested(&strikeout.content, inlines, prefix);
        }
        Inline::Subscript(subscript) => {
            remove_prefix_from_nested(&subscript.content, inlines, prefix);
        }
        Inline::Superscript(superscript) => {
            remove_prefix_from_nested(&superscript.content, inlines, prefix);
        }
        Inline::StyledInline(styled) => {
            remove_prefix_from_nested(&styled.content, inlines, prefix);
        }
        _ => {
            // For other inline types that don't contain nested content,
            // check if they match the prefix entirely
            let inline_text = to_text(&inlines[0]);
            if prefix.starts_with(&inline_text) {
                let remaining_prefix = &prefix[inline_text.len()..];
                inlines.remove(0);
                remove_prefix_from_inlines(inlines, remaining_prefix);
            }
        }
    }
}

/// Handle prefix removal for text nodes
fn remove_prefix_from_text(text_node: &Text, inlines: &mut Vec<Inline>, prefix: &str) {
    if let Some(stripped) = text_node.value.strip_prefix(prefix) {
        let remaining_text = stripped.trim_start();
        if remaining_text.is_empty() {
            // Remove the entire text node if it becomes empty
            inlines.remove(0);
        } else {
            // Update the text node with the remaining text
            if let Inline::Text(ref mut text) = inlines[0] {
                text.value = remaining_text.into();
            }
        }
    } else if prefix.starts_with(&*text_node.value) {
        // The prefix spans multiple inline elements
        let remaining_prefix = &prefix[text_node.value.len()..];
        inlines.remove(0); // Remove this text node entirely
        remove_prefix_from_inlines(inlines, remaining_prefix);
    }
}

/// Handle prefix removal for inline elements with nested content
fn remove_prefix_from_nested(nested_content: &[Inline], inlines: &mut Vec<Inline>, prefix: &str) {
    let nested_text = to_text(&nested_content.to_vec());
    if prefix.starts_with(&nested_text) {
        // The prefix spans beyond this nested element
        let remaining_prefix = &prefix[nested_text.len()..];
        inlines.remove(0); // Remove this nested element entirely
        remove_prefix_from_inlines(inlines, remaining_prefix);
    } else if nested_text.starts_with(prefix) {
        // The prefix is entirely within this nested element
        // We need to get a mutable reference to the nested content
        match &mut inlines[0] {
            Inline::Emphasis(emphasis) => {
                remove_prefix_from_inlines(&mut emphasis.content, prefix);
                if emphasis.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Strong(strong) => {
                remove_prefix_from_inlines(&mut strong.content, prefix);
                if strong.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Underline(underline) => {
                remove_prefix_from_inlines(&mut underline.content, prefix);
                if underline.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Strikeout(strikeout) => {
                remove_prefix_from_inlines(&mut strikeout.content, prefix);
                if strikeout.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Subscript(subscript) => {
                remove_prefix_from_inlines(&mut subscript.content, prefix);
                if subscript.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Superscript(superscript) => {
                remove_prefix_from_inlines(&mut superscript.content, prefix);
                if superscript.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::StyledInline(styled) => {
                remove_prefix_from_inlines(&mut styled.content, prefix);
                if styled.content.is_empty() {
                    inlines.remove(0);
                }
            }
            _ => {}
        }
    }
}

/// Determine if inlines contain at least one [`Citation`] or [`CitationGroup`]
fn has_citations(inlines: Vec<Inline>) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Citation(..) | Inline::CitationGroup(..)))
        .then_some(inlines)
}

/// Determine if inlines contain at least one [`Link`]
fn has_links(inlines: Vec<Inline>) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Link(..)))
        .then_some(inlines)
}

/// Extract numbering from heading text and calculate its depth
///
/// Detects hierarchical numbering patterns like:
/// - "1.2.3" (depth 3)
/// - "A.1" (depth 2)
/// - "IV.2.1" (depth 3)
/// - "1" (depth 1)
///
/// Returns (numbering_string, depth, cleaned_text) if numbering is found
fn extract_heading_numbering(text: &str) -> (Option<String>, usize, String) {
    static HEADING_NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:(?:Chapter|Section|Part)\s+)?([A-Z]\.(?:[0-9]+(?:\.[A-Z]?[0-9]+)*)?|[A-Z][0-9]+(?:\.[A-Z]?[0-9]+)*|[0-9]+(?:\.[A-Z]?[0-9]+)*|[IVX]+\.(?:[0-9]+(?:\.[A-Z]?[0-9]+)*)?)[.\s]*(.*)$")
            .expect("invalid regex")
    });

    if let Some(captures) = HEADING_NUMBER_REGEX.captures(text.trim()) {
        let numbering = captures[1].to_string();
        let cleaned_text = captures[2].trim().to_string();

        // Calculate depth by counting non-empty parts when split by dot
        let depth = numbering
            .split('.')
            .filter(|part| !part.trim().is_empty())
            .count();

        (Some(numbering), depth, cleaned_text)
    } else {
        (None, 0, text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_schema::shortcuts::t;

    use super::*;

    #[test]
    fn test_extract_heading_numbering() {
        // Test numbered headings
        let (numbering, depth, cleaned) = extract_heading_numbering("1.2.3 Results and Discussion");
        assert_eq!(numbering, Some("1.2.3".to_string()));
        assert_eq!(depth, 3);
        assert_eq!(cleaned, "Results and Discussion");

        let (numbering, depth, cleaned) = extract_heading_numbering("1. Introduction");
        assert_eq!(numbering, Some("1".to_string()));
        assert_eq!(depth, 1);
        assert_eq!(cleaned, "Introduction");

        let (numbering, depth, cleaned) = extract_heading_numbering("A.2 Methodology");
        assert_eq!(numbering, Some("A.2".to_string()));
        assert_eq!(depth, 2);
        assert_eq!(cleaned, "Methodology");

        let (numbering, depth, cleaned) = extract_heading_numbering("A. Bibliography");
        assert_eq!(numbering, Some("A.".to_string()));
        assert_eq!(depth, 1);
        assert_eq!(cleaned, "Bibliography");

        let (numbering, depth, cleaned) = extract_heading_numbering("IV.1.2 Analysis");
        assert_eq!(numbering, Some("IV.1.2".to_string()));
        assert_eq!(depth, 3);
        assert_eq!(cleaned, "Analysis");

        // Test with prefixes
        let (numbering, depth, cleaned) = extract_heading_numbering("Chapter 1 Introduction");
        assert_eq!(numbering, Some("1".to_string()));
        assert_eq!(depth, 1);
        assert_eq!(cleaned, "Introduction");

        let (numbering, depth, cleaned) = extract_heading_numbering("Section 2.1 Background");
        assert_eq!(numbering, Some("2.1".to_string()));
        assert_eq!(depth, 2);
        assert_eq!(cleaned, "Background");

        // Test non-numbered headings
        let (numbering, depth, cleaned) = extract_heading_numbering("Introduction");
        assert_eq!(numbering, None);
        assert_eq!(depth, 0);
        assert_eq!(cleaned, "Introduction");

        let (numbering, depth, cleaned) = extract_heading_numbering("Results");
        assert_eq!(numbering, None);
        assert_eq!(depth, 0);
        assert_eq!(cleaned, "Results");
    }

    #[test]
    fn test_detect_figure_caption() {
        // Valid figure captions
        let test_cases = [
            ("Figure 1. This is a caption", "1"),
            ("Fig 2: Another caption", "2"),
            ("Figure 12 - A longer caption", "12"),
            ("Fig. 5 Some caption", "5"),
            ("FIGURE 3. Case insensitive", "3"),
            ("figure 7: lowercase", "7"),
            ("Figure A1: Appendix figure", "A1"),
        ];

        for (input, expected_label, ..) in test_cases {
            let result = detect_figure_caption(&vec![t(input)]);

            assert!(result.is_some(), "Should detect figure caption: {input}");
            let (figure_number, ..) = result.expect("Should detect figure caption");
            assert_eq!(
                figure_number, expected_label,
                "Wrong figure number for: {input}"
            );
        }

        // Invalid cases - should return None
        let invalid_cases = [
            "Just regular text",
            "Figure without number",
            "Fig A: with letter instead of number",
            "Not a figure caption",
            "Figure: missing number",
            "fig missing number",
        ];

        for input in invalid_cases {
            let result = detect_figure_caption(&vec![t(input)]);
            assert!(
                result.is_none(),
                "Should not detect figure caption: {input}"
            );
        }

        // Test with complex paragraph structure
        let result = detect_figure_caption(&vec![
            t("Figure 5. This caption has "),
            stencila_schema::shortcuts::em([t("emphasis")]),
            t(" and more text."),
        ]);
        assert!(
            result.is_some(),
            "Should handle complex paragraph structure"
        );
        let (figure_number, ..) = result.expect("Should detect complex figure caption");
        assert_eq!(figure_number, "5");

        // Test edge case: figure prefix is the entire first text node

        let result =
            detect_figure_caption(&vec![t("Figure 1. "), t("Second text node with caption.")]);
        assert!(
            result.is_some(),
            "Should handle prefix as entire first text node"
        );
        let (figure_number, ..) = result.expect("Should detect edge case figure caption");
        assert_eq!(figure_number, "1");
    }
}
