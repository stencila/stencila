use std::iter::once;

use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Block, Citation, CitationGroup, CitationOptions, Heading, Inline, Link, List, ListItem,
    ListOrder, Note, NoteType, Section, SectionType,
    shortcuts::{em, p, qb, stg, stk, sub, sup, t, u},
};
use stencila_codec_text_trait::to_text;

use super::{
    Context,
    bibl::decode_list_bibl,
    child, collapse_whitespace,
    floats::{decode_figure, decode_formula, decode_inline_formula},
    path, targets, text_of, xml_id,
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
    section: Section,
}

/// The sections that have not yet ended, outermost first
#[derive(Default)]
struct OpenSections(Vec<OpenSection>);

impl OpenSections {
    /// End the sections at or below a level, adding each to its parent or,
    /// if it has none, to the blocks
    fn close(&mut self, level: usize, blocks: &mut Vec<Block>) {
        while self.0.last().is_some_and(|open| open.level >= level) {
            if let Some(OpenSection { section, .. }) = self.0.pop() {
                self.append(blocks, vec![Block::Section(section)]);
            }
        }
    }

    /// Add blocks to the innermost section or, if there is none, to the blocks
    fn append(&mut self, blocks: &mut Vec<Block>, mut new: Vec<Block>) {
        match self.0.last_mut() {
            Some(open) => open.section.content.append(&mut new),
            None => blocks.append(&mut new),
        }
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

        // Content that is not within a division comes after any sections
        // that precede it (as GROBID's figures and notes do at the end of the
        // body) rather than within the last of them
        if is_content(&others) {
            open.close(0, &mut blocks);
            blocks.append(&mut decode_blocks(others.drain(..), depth, context));
        } else {
            others.clear();
        }

        context.lost_attrs(&node, &["id", "type"]);

        // The type of a section is from its `type`, or else from its heading,
        // as when decoding JATS. A `type` that is not a type of section is
        // recorded as lost.
        let head = child(&node, "head");
        let type_section_type = node
            .attribute("type")
            .and_then(|text| SectionType::from_text(text).ok());
        if type_section_type.is_none() && node.attribute("type").is_some() {
            context.lost_label([&path(&node), "/@type"].concat());
        }
        let section_type = type_section_type.or_else(|| {
            head.as_ref()
                .and_then(|head| SectionType::from_text(&text_of(head)).ok())
        });

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
                    vec![Block::Section(Section {
                        id: Some(id.into()),
                        section_type,
                        content,
                        ..Default::default()
                    })],
                ),
                _ => open.append(&mut blocks, content),
            }
            continue;
        };

        let numbering = head.attribute("n");
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
                    section: Section {
                        id: xml_id(&node).map(String::from),
                        section_type,
                        content,
                        ..Default::default()
                    },
                });
            }
            Divisions::Flattened => {
                let heading_level = depth + level as i64;
                let mut content = vec![decode_head(&head, heading_level, context)];
                content.append(&mut decode_divisions(
                    node.children().filter(|child| *child != head),
                    heading_level,
                    divisions,
                    context,
                ));
                if let Some(id) = xml_id(&node) {
                    blocks.push(Block::Section(Section {
                        id: Some(id.into()),
                        section_type,
                        content,
                        ..Default::default()
                    }));
                } else {
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
    context.lost_attrs(head, &[]);

    Block::Heading(Heading::new(
        level,
        normalize(decode_inlines(head.children(), context)),
    ))
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
                context.lost_attrs(&node, &[]);
                blocks.push(qb(decode_blocks(node.children(), depth, context)));
            }
            "note" if node.attribute("place") == Some("foot") => {
                // Decoded where they are referenced, or at the end of the body
            }
            "note" => {
                // A note with no place is text that GROBID could not assign to
                // the flow of the document, so is kept, without the note, where it is
                context.lost_label(note_label(&node));
                blocks.append(&mut decode_blocks(node.children(), depth, context));
            }
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

/// The loss label for a note, including its type, if any
fn note_label(node: &Node) -> String {
    match node.attribute("type") {
        Some(note_type) => format!("{}[@type='{note_type}']", path(node)),
        None => path(node),
    }
}

/// Create a paragraph from inlines, unless they are all whitespace
fn paragraph(inlines: Vec<Inline>) -> Option<Block> {
    let inlines = normalize(inlines);
    (!inlines.is_empty()).then(|| p(inlines))
}

/// Decode a `<p>`
///
/// A block within a paragraph, such as a figure, ends the paragraph and is
/// decoded as a block in its own right, as it is when decoding JATS.
fn decode_p(node: &Node, depth: i64, context: &mut Context) -> Vec<Block> {
    context.lost_attrs(node, &[]);

    let mut blocks = Vec::new();
    let mut inlines = Vec::new();
    for child in node.children() {
        if child.is_element() && matches!(child.tag_name().name(), "figure" | "list" | "quote") {
            blocks.extend(paragraph(decode_inlines(inlines.drain(..), context)));
            blocks.append(&mut decode_blocks(once(child), depth, context));
        } else {
            inlines.push(child);
        }
    }
    blocks.extend(paragraph(decode_inlines(inlines.into_iter(), context)));

    blocks
}

/// Decode a `<list>` to a [`List`]
fn decode_list(node: &Node, depth: i64, context: &mut Context) -> Block {
    let order = match node.attribute("type").or(node.attribute("rend")) {
        Some("ordered" | "numbered" | "decimal") => ListOrder::Ascending,
        _ => ListOrder::Unordered,
    };

    context.lost_attrs(node, &["type", "rend"]);

    let items = node
        .children()
        .filter_map(|child| {
            if !child.has_tag_name("item") {
                context.lost(&child);
                return None;
            }
            context.lost_attrs(&child, &[]);
            Some(ListItem::new(decode_blocks(
                child.children(),
                depth,
                context,
            )))
        })
        .collect();

    Block::List(List::new(items, order))
}

/// Decode inline content
pub(super) fn decode_inlines<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    nodes: I,
    context: &mut Context,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for node in nodes {
        if node.is_text() {
            inlines.push(t(collapse_whitespace(node.text().unwrap_or_default())));
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
                context.lost_attrs(&node, &[]);
                let address = text_of(&node);
                inlines.push(Inline::Link(Link {
                    target: ["mailto:", &address].concat(),
                    content: vec![t(address)],
                    ..Default::default()
                }));
            }
            "ptr" => {
                context.lost_attrs(&node, &["target"]);
                if let Some(target) = node.attribute("target") {
                    inlines.push(Inline::Link(Link {
                        target: target.to_string(),
                        content: vec![t(target)],
                        ..Default::default()
                    }));
                }
            }
            "note" if node.attribute("place") == Some("foot") => {
                // Unless already decoded where it was referenced earlier
                if context.placed_footnote(&node) {
                    inlines.push(Inline::Note(decode_note(&node, context)))
                }
            }
            _ => {
                // A wrapper the schema has no inline for, such as GROBID's <rs>
                // annotations of funders and grant numbers, still says something
                // with the words inside it, so the wrapper alone is reported and
                // its content is kept
                let label = match node.attribute("type") {
                    Some(typ) => format!("{}[@type='{typ}']", path(&node)),
                    None => path(&node),
                };
                context.lost_label(label);
                inlines.append(&mut decode_inlines(node.children(), context));
            }
        }
    }

    inlines
}

/// Normalize inlines by merging adjacent text, collapsing whitespace, and
/// trimming whitespace from the start and end
pub(super) fn normalize(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut normalized: Vec<Inline> = Vec::with_capacity(inlines.len());
    for inline in inlines {
        if let Inline::Text(text) = &inline
            && let Some(Inline::Text(previous)) = normalized.last_mut()
        {
            let merged = collapse_whitespace(&[previous.value.as_str(), &text.value].concat());
            previous.value = merged.into();
        } else {
            normalized.push(inline);
        }
    }

    if let Some(Inline::Text(text)) = normalized.first_mut() {
        text.value = text.value.trim_start().to_string().into();
    }
    if let Some(Inline::Text(text)) = normalized.last_mut() {
        text.value = text.value.trim_end().to_string().into();
    }
    normalized.retain(|inline| !matches!(inline, Inline::Text(text) if text.value.is_empty()));

    normalized
}

/// Decode a `<ref>`
///
/// A reference to a bibliographic entry (`type="bibr"`) becomes a [`Citation`]
/// of it, a reference to a footnote becomes the [`Note`] itself, and any
/// other reference to an element, such as a figure or table, becomes a
/// [`Link`] to it, as an `<xref>` does when decoding JATS. A reference with a
/// target outside the document is a link to that target. GROBID does not
/// always resolve what a reference refers to, in which case only its text is
/// kept and the reference is recorded as lost.
fn decode_ref(node: &Node, context: &mut Context) -> Vec<Inline> {
    let ref_type = node.attribute("type");

    context.lost_attrs(node, &["type", "target"]);

    let content = normalize(decode_inlines(node.children(), context));

    let ids = targets(node)
        .into_iter()
        .filter(|id| context.has_id(id))
        .collect::<Vec<_>>();

    if ref_type == Some("foot")
        && let Some(id) = ids.first()
        && let Some(note) = context.place_footnote(id)
    {
        return vec![Inline::Note(decode_note(&note, context))];
    }

    if !ids.is_empty() {
        if ref_type == Some("bibr") {
            let content = (!content.is_empty()).then_some(content);

            // A reference to several entries is a group of citations, one of
            // each, because the target of a citation is a single entry
            if ids.len() > 1 {
                return vec![Inline::CitationGroup(CitationGroup {
                    items: ids.into_iter().map(Citation::new).collect(),
                    content,
                    ..Default::default()
                })];
            }

            return vec![Inline::Citation(Citation {
                target: ids.join(" "),
                options: Box::new(CitationOptions {
                    content,
                    ..Default::default()
                }),
                ..Default::default()
            })];
        }

        return vec![Inline::Link(Link {
            target: ids
                .iter()
                .map(|id| ["#", id].concat())
                .collect::<Vec<_>>()
                .join(" "),
            content,
            ..Default::default()
        })];
    }

    if let Some(target) = node.attribute("target")
        && !target.starts_with('#')
    {
        return vec![Inline::Link(Link {
            target: target.to_string(),
            content,
            ..Default::default()
        })];
    }

    context.lost_label(format!(
        "{}[@type='{}'] (unresolved)",
        path(node),
        ref_type.unwrap_or_default()
    ));

    content
}

/// Decode a `<hi>` (highlighted text) according to its rendition
fn decode_hi(node: &Node, context: &mut Context) -> Vec<Inline> {
    context.lost_attrs(node, &["rend"]);

    let content = decode_inlines(node.children(), context);
    let inline = match node.attribute("rend") {
        Some("bold") => stg(content),
        Some("italic") => em(content),
        Some("superscript" | "sup") => sup(content),
        Some("subscript" | "sub") => sub(content),
        Some("underline") => u(content),
        Some("strikethrough") => stk(content),
        rend => {
            context.lost_label(format!(
                "{}[@rend='{}']",
                path(node),
                rend.unwrap_or_default()
            ));
            return content;
        }
    };

    vec![inline]
}

/// Decode a `<note>` to a [`Note`]
///
/// The number of a footnote (its `n`) is a marker regenerated when the note is
/// rendered, so is recorded as lost, as the `<label>` of a JATS `<fn>` is.
pub(super) fn decode_note(node: &Node, context: &mut Context) -> Note {
    let note_type = match node.attribute("place") {
        Some("end") => NoteType::Endnote,
        Some("margin" | "left" | "right") => NoteType::Sidenote,
        _ => NoteType::Footnote,
    };

    context.lost_attrs(node, &["id", "place"]);

    Note {
        id: xml_id(node).map(String::from),
        note_type,
        content: decode_blocks(node.children(), 0, context),
        ..Default::default()
    }
}

/// Decode the footnotes that were not decoded where they are referenced
///
/// Each is wrapped in a paragraph, as a JATS `<fn>` in a block context is, so
/// that it remains a [`Note`] addressable by its id.
pub(super) fn decode_unplaced_footnotes(context: &mut Context) -> Vec<Block> {
    context
        .take_unplaced_footnotes()
        .into_iter()
        .filter_map(|node| {
            let note = decode_note(&node, context);
            (!to_text(&note.content).trim().is_empty()).then(|| p([Inline::Note(note)]))
        })
        .collect()
}
