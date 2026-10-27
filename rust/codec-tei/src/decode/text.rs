use std::iter::once;

use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Block, Citation, CitationGroup, CitationOptions, Heading, Inline, Link, List, ListItem,
    ListOrder, Note, NoteType, QuoteInline, Section, SectionType,
    shortcuts::{em, p, qb, stg, stk, sub, sup, t, u},
};
use stencila_codec_text_trait::to_text;

use super::{
    Context,
    bibl::decode_list_bibl,
    child, collapse_whitespace,
    floats::{decode_figure, decode_formula, decode_inline_formula},
    path, targets, text_of, typed_path, xml_id,
};

/// How the divisions (`<div>`s) of a part of a text are decoded
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Divisions {
    /// As nested sections, each starting with a heading
    Sections,
    /// As headings and content, for divisions within a part that is itself a
    /// section, such as the acknowledgements in the back matter. Divisions
    /// with ids remain sections so that references to them still resolve.
    Flattened,
}

/// Decode the `<body>` of a `<text>`
pub(super) fn decode_body(body: &Node, context: &mut Context) -> Vec<Block> {
    context.lost_attrs(body, &[]);

    decode_divisions(body.children(), 0, Divisions::Sections, context)
}

/// Decode the children of a part of a text, which may include divisions
///
/// GROBID does not nest divisions: each section and subsection of an article
/// is a `<div>` within the body, with its number, if any, in the `n` of its
/// `<head>`. So that sections are nested as they are when decoding JATS, the
/// level of a section is taken from the depth of that numbering (e.g. 2 for
/// "2.1") and a section is nested within the closest preceding section of a
/// lower level.
///
/// Where some sections are numbered, an unnumbered one (often a subsection
/// that the article does not number, or text in a figure that GROBID took to
/// be a heading) is nested one level below the closest preceding numbered
/// section, so that it does not end that section. Where none are numbered,
/// all sections are at the first level. A `<div>` without a `<head>`
/// continues the current section.
///
/// The `depth` is the level of headings in the parent, so that for example
/// the headings of sections within an abstract start at level 2.
pub(super) fn decode_divisions<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    nodes: I,
    depth: i64,
    divisions: Divisions,
    context: &mut Context,
) -> Vec<Block> {
    // Collected so that the recursion below is not over ever more deeply
    // nested iterator types
    decode_division_nodes(nodes.collect(), depth, divisions, context)
}

/// A section that has not yet ended
struct OpenSection {
    /// The level of the section in the numbering of sections
    level: usize,
    /// Whether the section is numbered
    numbered: bool,
    /// The section, unless divisions are flattened, in which case only its
    /// level is tracked, for the levels of the headings that follow it
    section: Option<Section>,
}

/// The sections that have not yet ended, outermost first
#[derive(Default)]
struct OpenSections(Vec<OpenSection>);

impl OpenSections {
    /// End the sections at or below a level, adding each to its parent or,
    /// if it has none, to the blocks
    fn close(&mut self, level: usize, blocks: &mut Vec<Block>) {
        while self.0.last().is_some_and(|open| open.level >= level) {
            if let Some(OpenSection {
                section: Some(section),
                ..
            }) = self.0.pop()
            {
                self.append(blocks, vec![Block::Section(section)]);
            }
        }
    }

    /// Add blocks to the innermost section or, if there is none, to the blocks
    fn append(&mut self, blocks: &mut Vec<Block>, mut new: Vec<Block>) {
        match self.0.last_mut().and_then(|open| open.section.as_mut()) {
            Some(section) => section.content.append(&mut new),
            None => blocks.append(&mut new),
        }
    }

    /// Whether a section at a level, given by its numbering, would be within
    /// one of the open sections
    fn continues(&self, numbering: Option<&str>) -> bool {
        numbering.is_some_and(|numbering| {
            let level = numbering_level(numbering);
            self.0.iter().any(|open| open.level < level)
        })
    }

    /// The level of a section within the numbering of sections
    fn level(&self, numbering: Option<&str>, any_numbered: bool) -> usize {
        match numbering {
            Some(numbering) => numbering_level(numbering),
            None if any_numbered => {
                self.0
                    .iter()
                    .rev()
                    .find(|open| open.numbered)
                    .map(|open| open.level)
                    .unwrap_or_default()
                    + 1
            }
            None => 1,
        }
    }

    /// The number of sections that have not ended
    fn len(&self) -> usize {
        self.0.len()
    }

    /// Add a section
    fn push(&mut self, open: OpenSection) {
        self.0.push(open);
    }
}

/// Decode the children of a part of a text: see [`decode_divisions`]
fn decode_division_nodes(
    nodes: Vec<Node>,
    depth: i64,
    divisions: Divisions,
    context: &mut Context,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut open = OpenSections::default();
    // Children that are not divisions, decoded together so that a run of
    // inline content directly within a <div> becomes a single paragraph
    let mut others: Vec<Node> = Vec::new();

    let any_numbered = nodes
        .iter()
        .any(|node| child(node, "head").is_some_and(|head| head.attribute("n").is_some()));

    // Whether there is content, other than the whitespace of indentation, in
    // the children that are not divisions
    let is_content = |others: &[Node]| {
        others.iter().any(|node| {
            node.is_element() || node.text().is_some_and(|text| !text.trim().is_empty())
        })
    };

    for node in nodes {
        if !node.has_tag_name("div") {
            others.push(node);
            continue;
        }

        let head = child(&node, "head");
        let numbering = head.and_then(|head| head.attribute("n"));

        // Content that is not within a division comes after any sections
        // that precede it (as GROBID's figures and notes do at the end of the
        // body) rather than within the last of them, unless the division
        // after it is a subsection of one of them
        if is_content(&others) {
            if !open.continues(numbering) {
                open.close(0, &mut blocks);
            }
            let content = decode_blocks(others.drain(..), depth + open.len() as i64, context);
            open.append(&mut blocks, content);
        } else {
            others.clear();
        }

        // The `id`, `coords` and `type` are recorded as lost below by each of
        // the branches that does not decode the division to a section
        context.lost_attrs(&node, &["id", "coords", "type"]);

        // The type of a section is from its `type`, or else from its heading,
        // as when decoding JATS. A `type` that is not a type of section is
        // recorded as lost.
        let type_section_type = node.attribute("type").and_then(section_type_of);
        if type_section_type.is_none() && node.attribute("type").is_some() {
            context.lost_label([&path(&node), "/@type"].concat());
        }
        let section_type = type_section_type.or_else(|| {
            head.as_ref()
                .and_then(|head| SectionType::from_text(&text_of(head)).ok())
        });
        // The type of a division that is not decoded to a section is lost,
        // unless the division has no content to be of that type (e.g. one of
        // references, which are decoded onto the article)
        let lost_type = |content: &[Block]| type_section_type.is_some() && !content.is_empty();

        let Some(head) = head else {
            // A division without a heading continues the current section,
            // unless it has an id, in which case it is kept as a section, as
            // a JATS <sec> with an id is, because it may be the target of a
            // reference
            let inner_depth = depth + open.len() as i64;
            let content = decode_divisions(node.children(), inner_depth, divisions, context);
            match xml_id(&node) {
                Some(id) if !content.is_empty() => open.append(
                    &mut blocks,
                    vec![Block::Section(context.located(
                        &node,
                        Section {
                            id: Some(id.into()),
                            section_type,
                            content,
                            ..Default::default()
                        },
                    ))],
                ),
                _ => {
                    if lost_type(&content) {
                        context.lost_attr(&node, "type");
                    }
                    context.lost_attrs_named(&node, &["id", "coords"]);
                    open.append(&mut blocks, content)
                }
            }
            continue;
        };

        let level = open.level(numbering, any_numbered);

        match divisions {
            Divisions::Sections => {
                open.close(level, &mut blocks);

                let heading_level = depth + open.len() as i64 + 1;
                let heading = decode_head(&head, heading_level, context);
                let mut content = vec![heading];
                content.append(&mut decode_divisions(
                    node.children().filter(|child| *child != head),
                    heading_level,
                    divisions,
                    context,
                ));

                open.push(OpenSection {
                    level,
                    numbered: numbering.is_some(),
                    section: Some(context.located(
                        &node,
                        Section {
                            id: xml_id(&node).map(String::from),
                            section_type,
                            content,
                            ..Default::default()
                        },
                    )),
                });
            }
            Divisions::Flattened => {
                // The level of the heading is from the numbering of the
                // divisions before it, as when they are sections
                open.close(level, &mut blocks);
                let heading_level = depth + open.len() as i64 + 1;
                open.push(OpenSection {
                    level,
                    numbered: numbering.is_some(),
                    section: None,
                });

                let mut content = vec![decode_head(&head, heading_level, context)];
                content.append(&mut decode_divisions(
                    node.children().filter(|child| *child != head),
                    heading_level,
                    divisions,
                    context,
                ));
                if let Some(id) = xml_id(&node) {
                    blocks.push(Block::Section(context.located(
                        &node,
                        Section {
                            id: Some(id.into()),
                            section_type,
                            content,
                            ..Default::default()
                        },
                    )));
                } else {
                    if lost_type(&content) {
                        context.lost_attr(&node, "type");
                    }
                    context.lost_attrs_named(&node, &["coords"]);
                    blocks.append(&mut content);
                }
            }
        }
    }

    if is_content(&others) {
        open.close(0, &mut blocks);
        blocks.append(&mut decode_blocks(others.into_iter(), depth, context));
    }
    open.close(0, &mut blocks);

    blocks
}

/// Get the type of a section from the `type` of a `<div>`
///
/// As well as the names of section types, GROBID's types for the divisions
/// of the back matter (e.g. "annex" and "availability") are recognized.
pub(super) fn section_type_of(div_type: &str) -> Option<SectionType> {
    match div_type {
        "acknowledgement" | "acknowledgment" | "acknowledgements" => {
            Some(SectionType::Acknowledgements)
        }
        "annex" => Some(SectionType::Appendix),
        "availability" => Some(SectionType::DataAvailability),
        "conflict" => Some(SectionType::CompetingInterests),
        "contribution" => Some(SectionType::AuthorContributions),
        "funding" => Some(SectionType::Funding),
        _ => SectionType::from_text(div_type).ok(),
    }
}

/// Get the level of a section from its numbering (e.g. 2 for "2.1." or "A.1")
fn numbering_level(n: &str) -> usize {
    n.split('.')
        .filter(|part| !part.trim().is_empty())
        .count()
        .max(1)
}

/// Decode a `<head>` to a [`Heading`]
///
/// The numbering of a heading (its `n`) is used for the nesting of sections,
/// but is not otherwise kept because a heading's label is only rendered as the
/// label of an appendix. It is recorded as lost.
fn decode_head(head: &Node, level: i64, context: &mut Context) -> Block {
    context.lost_attrs(head, &["id", "coords"]);

    let heading = Heading {
        id: xml_id(head).map(String::from),
        ..Heading::new(level, normalize(decode_inlines(head.children(), context)))
    };
    Block::Heading(context.located(head, heading))
}

/// Whether an element is decoded as a block rather than as inline content
fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "div" | "figure" | "formula" | "head" | "list" | "listBibl" | "note" | "p" | "quote"
    )
}

/// Decode block content
///
/// Text and inline elements that are directly within a division, rather than
/// within a `<p>`, become a paragraph.
pub(super) fn decode_blocks<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    nodes: I,
    depth: i64,
    context: &mut Context,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut inlines: Vec<Node> = Vec::new();

    let flush = |inlines: &mut Vec<Node>, blocks: &mut Vec<Block>, context: &mut Context| {
        if !inlines.is_empty() {
            blocks.extend(paragraph(decode_inlines(inlines.drain(..), context)));
        }
    };

    for node in nodes {
        let tag = node.tag_name().name();
        if node.is_text() || (node.is_element() && !is_block_element(tag)) {
            inlines.push(node);
            continue;
        }
        if !node.is_element() {
            continue;
        }

        flush(&mut inlines, &mut blocks, context);

        match tag {
            "p" => blocks.append(&mut decode_p(&node, depth, context)),
            "head" => blocks.push(decode_head(&node, depth + 1, context)),
            "figure" => blocks.push(decode_figure(&node, context)),
            "formula" => blocks.push(decode_formula(&node, context)),
            "list" => blocks.push(decode_list(&node, depth, context)),
            "quote" => {
                context.lost_attrs(&node, &["id", "coords"]);
                let mut quote = qb(decode_blocks(node.children(), depth, context));
                if let Block::QuoteBlock(quote) = &mut quote {
                    quote.id = xml_id(&node).map(String::from);
                    context.locate(&node, quote, None);
                }
                blocks.push(quote);
            }
            "note" => match note_type(&node) {
                Some(NoteType::Footnote) => {
                    // Decoded where they are referenced, or at the end of the body
                }
                Some(..) => {
                    // Wrapped in a paragraph, as a JATS <fn> in a block
                    // context is, so that it remains a note
                    blocks.push(p([Inline::Note(decode_note(&node, context))]))
                }
                None => {
                    // A note with no place is text that GROBID could not assign to
                    // the flow of the document, so is kept, without the note, where it is
                    context.lost_label(typed_path(&node, "type"));
                    blocks.append(&mut decode_blocks(node.children(), depth, context));
                }
            },
            "div" => blocks.append(&mut decode_divisions(
                once(node),
                depth,
                Divisions::Sections,
                context,
            )),
            "listBibl" => {
                let mut references = decode_list_bibl(&node, context);
                context.references.append(&mut references);
            }
            _ => context.lost(&node),
        }
    }

    flush(&mut inlines, &mut blocks, context);

    blocks
}

/// Create a paragraph from inlines, unless they are all whitespace
fn paragraph(inlines: Vec<Inline>) -> Option<Block> {
    let inlines = normalize(inlines);
    (!inlines.is_empty()).then(|| p(inlines))
}

/// Whether a `<quote>` has block content, such as paragraphs or lines of
/// verse, and so is a quotation block rather than an inline quotation
fn is_block_quote(node: &Node) -> bool {
    node.children().any(|child| {
        child.is_element()
            && matches!(
                child.tag_name().name(),
                "p" | "lg" | "l" | "list" | "figure" | "table" | "div"
            )
    })
}

/// Decode a `<p>`
///
/// A block within a paragraph, such as a figure, ends the paragraph and is
/// decoded as a block in its own right, as it is when decoding JATS. A
/// `<quote>` is inline unless it has block content.
fn decode_p(node: &Node, depth: i64, context: &mut Context) -> Vec<Block> {
    context.lost_attrs(node, &["id", "coords"]);

    let mut blocks = Vec::new();
    let mut inlines = Vec::new();
    for child in node.children() {
        let is_block = child.is_element()
            && match child.tag_name().name() {
                "figure" | "list" => true,
                "quote" => is_block_quote(&child),
                _ => false,
            };
        if is_block {
            blocks.extend(paragraph(decode_inlines(inlines.drain(..), context)));
            blocks.append(&mut decode_blocks(once(child), depth, context));
        } else {
            inlines.push(child);
        }
    }
    blocks.extend(paragraph(decode_inlines(inlines.into_iter(), context)));

    // The id is kept on the first paragraph only, but each is where the
    // element is on the page
    let mut id = xml_id(node);
    let mut has_paragraph = false;
    for block in &mut blocks {
        if let Block::Paragraph(paragraph) = block {
            paragraph.id = id.take().map(String::from);
            context.locate(node, paragraph, None);
            has_paragraph = true;
        }
    }
    if !has_paragraph {
        // Only blocks, or nothing, were within the element, so nothing has its
        // id or coordinates
        context.lost_attrs_named(node, &["id", "coords"]);
    }

    blocks
}

/// Decode a `<list>` to a [`List`]
///
/// A `type` or `rend` that does not say whether the list is ordered, such as
/// `type="gloss"`, is recorded as lost.
fn decode_list(node: &Node, depth: i64, context: &mut Context) -> Block {
    let mut order = ListOrder::Unordered;
    for name in ["type", "rend"] {
        match node.attribute(name) {
            Some("ordered" | "numbered" | "decimal") => order = ListOrder::Ascending,
            Some("unordered" | "bulleted" | "simple") | None => {}
            Some(..) => context.lost_label(typed_path(node, name)),
        }
    }

    context.lost_attrs(node, &["id", "coords", "type", "rend"]);

    let items = node
        .children()
        .filter_map(|child| {
            if !child.has_tag_name("item") {
                context.lost(&child);
                return None;
            }
            context.lost_attrs(&child, &["id", "coords"]);
            let item = ListItem {
                id: xml_id(&child).map(String::from),
                ..ListItem::new(decode_blocks(child.children(), depth, context))
            };
            Some(context.located(&child, item))
        })
        .collect();

    Block::List(context.located(
        node,
        List {
            id: xml_id(node).map(String::from),
            ..List::new(items, order)
        },
    ))
}

/// Decode inline content
///
/// The inlines are not normalized: whitespace is collapsed, and moved out of
/// marks, links and citations, by [`normalize`] once the inlines of a whole
/// block are decoded.
pub(super) fn decode_inlines<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    nodes: I,
    context: &mut Context,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for node in nodes {
        if node.is_text() {
            inlines.push(t(node.text().unwrap_or_default()));
            continue;
        }
        if !node.is_element() {
            continue;
        }

        let tag = node.tag_name().name();
        match tag {
            "ref" => inlines.append(&mut decode_ref(&node, context)),
            "hi" => inlines.append(&mut decode_hi(&node, context)),
            "formula" => inlines.push(decode_inline_formula(&node, context)),
            "s" => {
                // GROBID's optional sentence segmentation can omit whitespace
                // between adjacent sentences. Keep their boundary when removing
                // the wrappers; normalization collapses any existing whitespace.
                context.lost_attrs(&node, &[]);
                if !inlines.is_empty() {
                    inlines.push(t(" "));
                }
                inlines.append(&mut decode_inlines(node.children(), context));
            }
            "lb" => {
                // A line break within a run of text separates the words on
                // either side of it, which is all that the schema records here
                context.lost_attrs(&node, &[]);
                inlines.push(t(" "));
            }
            "email" => {
                context.lost_attrs(&node, &["id", "coords"]);
                let address = text_of(&node);
                inlines.push(Inline::Link(context.located(
                    &node,
                    Link {
                        id: xml_id(&node).map(String::from),
                        target: ["mailto:", &address].concat(),
                        content: vec![t(address)],
                        ..Default::default()
                    },
                )));
            }
            "ptr" => {
                context.lost_attrs(&node, &["id", "coords", "target"]);
                if let Some(target) = node.attribute("target") {
                    inlines.push(Inline::Link(context.located(
                        &node,
                        Link {
                            id: xml_id(&node).map(String::from),
                            target: target.to_string(),
                            content: vec![t(target)],
                            ..Default::default()
                        },
                    )));
                } else {
                    context.lost_attrs_named(&node, &["id", "coords"]);
                }
            }
            "quote" => {
                // Within a paragraph, unless it has block content, in which
                // case the paragraph is split around it
                context.lost_attrs(&node, &["id", "coords"]);
                let quote = QuoteInline {
                    id: xml_id(&node).map(String::from),
                    content: decode_inlines(node.children(), context),
                    ..Default::default()
                };
                inlines.push(Inline::QuoteInline(context.located(&node, quote)));
            }
            "note" if note_type(&node).is_some() => {
                // A footnote is decoded here unless already decoded where it
                // was referenced earlier
                if note_type(&node) != Some(NoteType::Footnote) {
                    inlines.push(Inline::Note(decode_note(&node, context)))
                } else if context.placed_footnote(&node) {
                    inlines.push(Inline::Note(decode_footnote(&node, context)))
                }
            }
            _ => {
                // A wrapper the schema has no inline for, such as GROBID's <rs>
                // annotations of funders and grant numbers, still says something
                // with the words inside it, so the wrapper alone is reported and
                // its content is kept
                context.lost_label(typed_path(&node, "type"));
                inlines.append(&mut decode_inlines(node.children(), context));
            }
        }
    }

    inlines
}

/// Normalize inlines by merging adjacent text, collapsing whitespace, and
/// trimming whitespace from the start and end
///
/// Whitespace at the start or end of the content of a mark, link or citation
/// is moved out of it, so that it separates the inline from the text around
/// it (as in `See<ref> this </ref>now`) without being within it. A mark left
/// empty is removed. Each run of text is collapsed once, so that this takes
/// time linear in the length of the text, however many pieces it is in.
pub(super) fn normalize(inlines: Vec<Inline>) -> Vec<Inline> {
    let (_, inlines, _) = normalize_within(inlines);
    inlines
}

/// Normalize inlines, returning whether whitespace was trimmed from their
/// start and from their end
fn normalize_within(inlines: Vec<Inline>) -> (bool, Vec<Inline>, bool) {
    let mut normalized = Vec::with_capacity(inlines.len());

    // The text since the last inline that is not text
    let mut text = String::new();
    let flush = |text: &mut String, normalized: &mut Vec<Inline>| {
        if !text.is_empty() {
            normalized.push(t(collapse_whitespace(&std::mem::take(text))));
        }
    };

    for mut inline in inlines {
        if let Inline::Text(inline) = &inline {
            text.push_str(&inline.value);
            continue;
        }

        let (leading, trailing, is_empty) = match inline_content(&mut inline) {
            Some(content) => {
                let (leading, inner, trailing) = normalize_within(std::mem::take(content));
                let is_empty = inner.is_empty();
                *content = inner;
                (leading, trailing, is_empty)
            }
            None => (false, false, false),
        };

        if leading {
            text.push(' ');
        }
        if is_empty {
            match &mut inline {
                // A citation's content is optional
                Inline::Citation(citation) => citation.options.content = None,
                Inline::CitationGroup(group) => group.content = None,
                _ => {}
            }
        }
        if !(is_empty && is_mark(&inline)) {
            flush(&mut text, &mut normalized);
            normalized.push(inline);
        }
        if trailing {
            text.push(' ');
        }
    }
    flush(&mut text, &mut normalized);

    let mut leading = false;
    if let Some(Inline::Text(text)) = normalized.first_mut()
        && let Some(trimmed) = text.value.strip_prefix(' ')
    {
        text.value = trimmed.to_string().into();
        leading = true;
    }
    let mut trailing = false;
    if let Some(Inline::Text(text)) = normalized.last_mut()
        && let Some(trimmed) = text.value.strip_suffix(' ')
    {
        text.value = trimmed.to_string().into();
        trailing = true;
    }
    normalized.retain(|inline| !matches!(inline, Inline::Text(text) if text.value.is_empty()));

    (leading, normalized, trailing)
}

/// Get the inline content of a mark, quote, link or citation
fn inline_content(inline: &mut Inline) -> Option<&mut Vec<Inline>> {
    match inline {
        Inline::Emphasis(mark) => Some(&mut mark.content),
        Inline::Strong(mark) => Some(&mut mark.content),
        Inline::Superscript(mark) => Some(&mut mark.content),
        Inline::Subscript(mark) => Some(&mut mark.content),
        Inline::Underline(mark) => Some(&mut mark.content),
        Inline::Strikeout(mark) => Some(&mut mark.content),
        Inline::QuoteInline(quote) => Some(&mut quote.content),
        Inline::Link(link) => Some(&mut link.content),
        Inline::Citation(citation) => citation.options.content.as_mut(),
        Inline::CitationGroup(group) => group.content.as_mut(),
        _ => None,
    }
}

/// Whether an inline is a mark, which has nothing to say without content
fn is_mark(inline: &Inline) -> bool {
    matches!(
        inline,
        Inline::Emphasis(..)
            | Inline::Strong(..)
            | Inline::Superscript(..)
            | Inline::Subscript(..)
            | Inline::Underline(..)
            | Inline::Strikeout(..)
    )
}

/// Whether some text looks like an absolute URL
fn is_url(text: &str) -> bool {
    ["http://", "https://", "ftp://"]
        .iter()
        .any(|scheme| text.starts_with(scheme))
        && !text.contains(char::is_whitespace)
}

/// Decode a `<ref>`
///
/// A reference to a bibliographic entry (`type="bibr"`) becomes a [`Citation`]
/// of it, a reference to a footnote becomes the [`Note`] itself, and any
/// other reference to an element, such as a figure or table, becomes a
/// [`Link`] to it, as an `<xref>` does when decoding JATS. A reference with a
/// target outside the document, or a `type="url"` reference whose text is a
/// URL, is a link to that URL. GROBID does not always resolve what a
/// reference refers to, in which case only its text is kept and the
/// reference is recorded as lost. A reference to several elements, of which
/// only some exist, is recorded as partially unresolved.
fn decode_ref(node: &Node, context: &mut Context) -> Vec<Inline> {
    let ref_type = node.attribute("type");

    context.lost_attrs(node, &["id", "coords", "type", "target"]);

    let content = decode_inlines(node.children(), context);
    let label = typed_path(node, "type");

    let target_count = node
        .attribute("target")
        .map_or(0, |target| target.split_whitespace().count());
    let ids = targets(node)
        .into_iter()
        .filter(|id| context.has_id(id))
        .collect::<Vec<_>>();

    if ref_type == Some("foot")
        && let Some(id) = ids.first()
        && let Some(note) = context.place_footnote(id)
    {
        let note = decode_footnote(&note, context);
        // The note takes the place of the reference, and has its own id. The
        // reference is where the note's marker is.
        context.locate(node, &note, Some("marker"));
        context.lost_attrs_named(node, &["id"]);
        return vec![Inline::Note(note)];
    }

    // A citation is of a bibliographic entry, so a reference of that type to
    // other elements (a misclassification) is a link to them
    let citation_ids = if ref_type == Some("bibr") {
        ids.iter()
            .filter(|id| context.has_bibl_id(id))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    let resolved_count = if citation_ids.is_empty() {
        ids.len()
    } else {
        citation_ids.len()
    };
    if resolved_count > 0 && resolved_count < target_count {
        context.lost_label(format!("{label} (partially unresolved)"));
    }

    if !citation_ids.is_empty() {
        let content = (!content.is_empty()).then_some(content);

        // A reference to several entries is a group of citations, one of
        // each, because the target of a citation is a single entry
        let citation = if citation_ids.len() > 1 {
            Inline::CitationGroup(context.located(
                node,
                CitationGroup {
                    id: xml_id(node).map(String::from),
                    items: citation_ids.into_iter().map(Citation::new).collect(),
                    content,
                    ..Default::default()
                },
            ))
        } else {
            Inline::Citation(context.located(
                node,
                Citation {
                    id: xml_id(node).map(String::from),
                    target: citation_ids.join(" "),
                    options: Box::new(CitationOptions {
                        content,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ))
        };
        context.internal_ref(&citation, label);
        return vec![citation];
    }

    if !ids.is_empty() {
        let link = Inline::Link(
            context.located(
                node,
                Link {
                    id: xml_id(node).map(String::from),
                    target: ids
                        .iter()
                        .map(|id| ["#", id].concat())
                        .collect::<Vec<_>>()
                        .join(" "),
                    content,
                    ..Default::default()
                },
            ),
        );
        context.internal_ref(&link, label);
        return vec![link];
    }

    let target = match node.attribute("target") {
        Some(target) if !target.starts_with('#') => Some(target.to_string()),
        None if ref_type == Some("url") => Some(text_of(node)).filter(|text| is_url(text)),
        _ => None,
    };
    if let Some(target) = target {
        return vec![Inline::Link(context.located(
            node,
            Link {
                id: xml_id(node).map(String::from),
                target,
                content,
                ..Default::default()
            },
        ))];
    }

    context.lost_label(format!("{label} (unresolved)"));

    // An unresolved reference has no decoded wrapper to carry this metadata.
    context.lost_attrs_named(node, &["id", "coords"]);

    content
}

/// Decode a `<hi>` (highlighted text) according to its rendition
///
/// A `rend` may have several space-separated renditions (e.g. "bold
/// italic"), which are nested in the order given. Those that are not
/// recognized are recorded as lost.
fn decode_hi(node: &Node, context: &mut Context) -> Vec<Inline> {
    context.lost_attrs(node, &["rend"]);

    let mut content = decode_inlines(node.children(), context);

    let rend = node.attribute("rend").unwrap_or_default();
    let mut unrecognized = Vec::new();
    for token in rend.split_whitespace().rev() {
        let mark: fn(Vec<Inline>) -> Inline = match token {
            "bold" => stg,
            "italic" => em,
            "superscript" | "sup" => sup,
            "subscript" | "sub" => sub,
            "underline" => u,
            "strikethrough" => stk,
            _ => {
                unrecognized.push(token);
                continue;
            }
        };
        content = vec![mark(content)];
    }

    if !unrecognized.is_empty() || rend.trim().is_empty() {
        unrecognized.reverse();
        context.lost_label(format!(
            "{}[@rend='{}']",
            path(node),
            unrecognized.join(" ")
        ));
    }

    content
}

/// Get the type of a `<note>` from its `place`
///
/// Returns `None` for a note without a place, or with a place that is not
/// one of the kinds of note (e.g. `place="inline"`).
pub(super) fn note_type(node: &Node) -> Option<NoteType> {
    match node.attribute("place")? {
        "foot" => Some(NoteType::Footnote),
        "end" => Some(NoteType::Endnote),
        "margin" | "left" | "right" => Some(NoteType::Sidenote),
        _ => None,
    }
}

/// Decode a `<note>` to a [`Note`]
///
/// The number of a footnote (its `n`) is a marker regenerated when the note is
/// rendered, so is recorded as lost, as the `<label>` of a JATS `<fn>` is. A
/// note without a recognized `place` is a footnote, with its place recorded
/// as lost.
pub(super) fn decode_note(node: &Node, context: &mut Context) -> Note {
    let note_type = note_type(node);

    context.lost_attrs(
        node,
        if note_type.is_some() {
            &["id", "coords", "place"]
        } else {
            &["id", "coords"]
        },
    );

    let note = Note {
        id: xml_id(node).map(String::from),
        note_type: note_type.unwrap_or(NoteType::Footnote),
        content: decode_blocks(node.children(), 0, context),
        ..Default::default()
    };
    context.located(node, note)
}

/// Decode a footnote of the text, recording the note it was decoded to
fn decode_footnote(node: &Node, context: &mut Context) -> Note {
    let note = decode_note(node, context);
    context.decoded_footnote(node, &note);
    note
}

/// Decode the footnotes that were not decoded where they are referenced
///
/// Each is wrapped in a paragraph, as a JATS `<fn>` in a block context is, so
/// that it remains a [`Note`] addressable by its id. Whether a footnote has
/// been decoded is checked just before decoding it, because it may be within
/// another of them.
pub(super) fn decode_unplaced_footnotes(context: &mut Context) -> Vec<Block> {
    context
        .unplaced_footnotes()
        .into_iter()
        .filter_map(|node| {
            if !context.placed_footnote(&node) {
                return None;
            }
            let note = decode_footnote(&node, context);
            (!to_text(&note.content).trim().is_empty()).then(|| p([Inline::Note(note)]))
        })
        .collect()
}
