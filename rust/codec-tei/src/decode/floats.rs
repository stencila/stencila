use quick_xml::escape::escape;
use roxmltree::Node;

use stencila_codec::stencila_schema::{
    Block, Figure, ImageObject, Inline, MathBlock, MathBlockOptions, MathInline, MathInlineOptions,
    Table, TableCell, TableCellOptions, TableCellType, TableRow, TableRowType, shortcuts::p,
};

use super::{
    Context, child, collapse_whitespace, non_empty_text, path,
    text::{decode_blocks, decode_inlines, decode_note, normalize},
    xml_id,
};

/// The math language of formulas decoded from TEI
///
/// GROBID gives the text of a formula as extracted from the PDF, not TeX or
/// MathML, so the formula is decoded as that text rather than as a guess at
/// the TeX it was typeset from.
pub(super) const FORMULA_LANGUAGE: &str = "text";

/// Decode a `<figure>` to a [`Figure`], or, if it is a table, to a [`Table`]
///
/// GROBID gives no image files for figures, only the coordinates of the
/// figure, and of its graphic, on the pages of the PDF. So each figure is
/// decoded without an image, its coordinates are kept in the page map, and its
/// missing image is recorded as a loss (with the figure's id so that each
/// is reported). A `<graphic>` that does have a `url` becomes the image.
/// Notes are kept after the images in the figure's content.
pub(super) fn decode_figure(node: &Node, context: &mut Context) -> Block {
    if node.attribute("type") == Some("table") {
        return decode_table(node, context);
    }

    context.lost_attrs(node, &["id", "coords"]);

    let id = xml_id(node).map(String::from);
    let (label, caption) = decode_label_and_caption(node, LabelPrefix::Figure, context);

    let mut graphics = Vec::new();
    let mut content = Vec::new();
    let mut notes = Vec::new();
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "head" | "label" | "figDesc" => {}
            "graphic" => {
                context.lost_attrs(&child, &["coords", "type", "url"]);
                graphics.push(child);
                if let Some(url) = child.attribute("url") {
                    content.push(Block::ImageObject(ImageObject {
                        content_url: url.to_string(),
                        ..Default::default()
                    }));
                }
            }
            "note" => notes.extend(decode_float_note(&child, context)),
            _ => context.lost(&child),
        }
    }

    if content.is_empty() {
        context.lost_label(missing(node, "no image"));
    }
    content.extend(notes);

    let figure = context.located(
        node,
        Figure {
            id,
            label_automatically: label.is_some().then_some(false),
            label,
            caption,
            content,
            ..Default::default()
        },
    );
    for graphic in graphics {
        context.locate(&graphic, &figure, Some("graphic"));
    }

    Block::Figure(figure)
}

/// Decode a `<figure type="table">` to a [`Table`]
///
/// The rows and cells of the table are decoded where GROBID gives them as
/// `<row>`s of `<cell>`s, with a row or cell that has `role="label"` marked as
/// a header. A table without rows (older GROBID output gives the text of the
/// table instead) is recorded as a loss.
fn decode_table(node: &Node, context: &mut Context) -> Block {
    context.lost_attrs(node, &["id", "coords", "type"]);

    let id = xml_id(node).map(String::from);
    let (label, caption) = decode_label_and_caption(node, LabelPrefix::Table, context);

    let mut tables = Vec::new();
    let mut rows = Vec::new();
    let mut notes = Vec::new();
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "head" | "label" | "figDesc" => {}
            "table" => {
                context.lost_attrs(&child, &["coords"]);
                tables.push(child);
                for grandchild in child.children() {
                    if grandchild.has_tag_name("row") {
                        rows.push(decode_row(&grandchild, context));
                    } else {
                        context.lost(&grandchild);
                    }
                }
            }
            "note" => notes.extend(decode_float_note(&child, context)),
            _ => context.lost(&child),
        }
    }

    if rows.is_empty() {
        context.lost_label(missing(node, "no rows"));
    }

    let table = context.located(
        node,
        Table {
            id,
            label_automatically: label.is_some().then_some(false),
            label,
            caption,
            rows,
            notes: (!notes.is_empty()).then_some(notes),
            ..Default::default()
        },
    );
    for child in tables {
        context.locate(&child, &table, Some("table"));
    }

    Block::Table(table)
}

/// Decode a figure or table note
///
/// A note with an id is kept as a note, in place, so that references can
/// still address it. A note without an id is unwrapped into its blocks.
fn decode_float_note(node: &Node, context: &mut Context) -> Vec<Block> {
    if xml_id(node).is_some() {
        vec![p([Inline::Note(decode_note(node, context))])]
    } else {
        context.lost_attrs(node, &["place"]);
        decode_blocks(node.children(), 0, context)
    }
}

/// Decode a `<row>` to a [`TableRow`]
fn decode_row(node: &Node, context: &mut Context) -> TableRow {
    let row_type = (node.attribute("role") == Some("label")).then_some(TableRowType::HeaderRow);

    context.lost_attrs(
        node,
        if row_type.is_some() {
            &["id", "coords", "role"]
        } else {
            &["id", "coords"]
        },
    );

    let cells = node
        .children()
        .filter_map(|child| {
            if child.has_tag_name("cell") {
                Some(decode_cell(&child, context))
            } else {
                context.lost(&child);
                None
            }
        })
        .collect();

    context.located(
        node,
        TableRow {
            id: xml_id(node).map(String::from),
            cells,
            row_type,
            ..Default::default()
        },
    )
}

/// Decode a `<cell>` to a [`TableCell`]
fn decode_cell(node: &Node, context: &mut Context) -> TableCell {
    let cell_type = (node.attribute("role") == Some("label")).then_some(TableCellType::HeaderCell);

    let span = |name| {
        node.attribute(name)
            .and_then(|value| value.parse().ok())
            .filter(|span| *span != 1)
    };
    let row_span = span("rows");
    let column_span = span("cols");

    let mut decoded = vec!["id", "coords", "rows", "cols"];
    if cell_type.is_some() {
        decoded.push("role");
    }
    context.lost_attrs(node, &decoded);

    let inlines = normalize(decode_inlines(node.children(), context));
    let content = if inlines.is_empty() {
        Vec::new()
    } else {
        vec![p(inlines)]
    };

    context.located(
        node,
        TableCell {
            id: xml_id(node).map(String::from),
            cell_type,
            content,
            options: Box::new(TableCellOptions {
                row_span,
                column_span,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Decode a display `<formula>` to a [`MathBlock`]
///
/// The text of the formula is its `code`, and its `<label>` its label. That
/// the formula is text, not TeX or MathML, is recorded as a loss (counted
/// rather than reported for each formula).
pub(super) fn decode_formula(node: &Node, context: &mut Context) -> Block {
    context.lost_attrs(node, &["id", "coords"]);
    context.lost_label(format!("{} (text, not TeX or MathML)", path(node)));

    let label = child(node, "label")
        .and_then(|label| non_empty_text(&label))
        .map(|label| {
            label
                .trim_start_matches(['(', ' '])
                .trim_end_matches([')', ' '])
                .to_string()
        });

    let code = formula_text(node, context);
    let mathml = formula_mathml(&code);

    Block::MathBlock(context.located(
        node,
        MathBlock {
            id: xml_id(node).map(String::from),
            code: code.into(),
            math_language: Some(FORMULA_LANGUAGE.into()),
            label_automatically: label.is_some().then_some(false),
            label,
            options: Box::new(MathBlockOptions {
                mathml: Some(mathml),
                ..Default::default()
            }),
            ..Default::default()
        },
    ))
}

/// Decode a `<formula>` within a paragraph to a [`MathInline`]
pub(super) fn decode_inline_formula(node: &Node, context: &mut Context) -> Inline {
    context.lost_attrs(node, &["id", "coords"]);
    context.lost_label(format!("{} (text, not TeX or MathML)", path(node)));

    let code = formula_text(node, context);
    let mathml = formula_mathml(&code);

    Inline::MathInline(context.located(
        node,
        MathInline {
            id: xml_id(node).map(String::from),
            code: code.into(),
            math_language: Some(FORMULA_LANGUAGE.into()),
            options: Box::new(MathInlineOptions {
                mathml: Some(mathml),
                ..Default::default()
            }),
            ..Default::default()
        },
    ))
}

/// Render extracted formula text without guessing its mathematical structure.
///
/// Math nodes display MathML or images, and there is no kernel for the `text`
/// language. Supplying MathML here also lets compilation preserve this external
/// representation instead of attempting to execute the extracted text.
fn formula_mathml(text: &str) -> String {
    format!(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML"><mtext>{}</mtext></math>"#,
        escape(text)
    )
}

/// Get the text of a formula, excluding its label
fn formula_text(node: &Node, context: &mut Context) -> String {
    let mut text = String::new();
    for child in node.children() {
        if child.is_text() {
            text.push_str(child.text().unwrap_or_default());
        } else if child.has_tag_name("label") {
            context.lost_attrs(&child, &[]);
        } else {
            // e.g. a <hi>, of which only the text is kept
            context.lost(&child);
            text.push_str(
                &child
                    .descendants()
                    .filter(Node::is_text)
                    .filter_map(|node| node.text())
                    .collect::<String>(),
            );
        }
    }
    collapse_whitespace(&text).trim().to_string()
}

/// The word that GROBID's figure and table captions start with
#[derive(Clone, Copy)]
enum LabelPrefix {
    Figure,
    Table,
}

impl LabelPrefix {
    /// The words, longest first, that a caption of this kind may start with
    fn words(&self) -> &'static [&'static str] {
        match self {
            LabelPrefix::Figure => &["figure", "fig.", "fig"],
            LabelPrefix::Table => &["table", "tab."],
        }
    }
}

/// Decode the label and caption of a figure or table
///
/// The label is the `<label>`, which GROBID derives from the `<head>` (e.g.
/// "Figure 1 ."). When there is no `<label>`, the label is taken from a
/// `<head>` that starts with the word for the kind of float (e.g. "Figure 7"
/// or "Table 2:"). A `<head>` that has text but gives no label is recorded as
/// lost. The caption is the `<figDesc>`, from which GROBID does not remove the
/// label, so a leading "Figure 1." or similar is removed so that the label is
/// not rendered twice.
fn decode_label_and_caption(
    node: &Node,
    prefix: LabelPrefix,
    context: &mut Context,
) -> (Option<String>, Option<Vec<Block>>) {
    let mut label = child(node, "label").and_then(|label| {
        context.lost_attrs(&label, &[]);
        non_empty_text(&label)
    });

    if let Some(head) = child(node, "head") {
        context.lost_attrs(&head, &[]);
        if label.is_none()
            && let Some(text) = non_empty_text(&head)
        {
            label = label_from_head(&text, prefix);
            if label.is_none() {
                context.lost(&head);
            }
        }
    }

    let caption = child(node, "figDesc").and_then(|desc| {
        context.lost_attrs(&desc, &[]);
        let mut inlines = normalize(decode_inlines(desc.children(), context));
        if let Some(label) = &label
            && let Some(Inline::Text(text)) = inlines.first_mut()
            && let Some(rest) = strip_label(&text.value, prefix, label)
        {
            text.value = rest.to_string().into();
            inlines = normalize(inlines);
        }
        (!inlines.is_empty()).then(|| vec![p(inlines)])
    });

    (label, caption)
}

/// Get the label from a heading, such as "Figure 7" or "Table 2:"
///
/// Returns `None` if the heading does not start with the word for the kind of
/// float followed by something that looks like a label: one with a number
/// (e.g. "7" or "S1") or in capitals (e.g. "A" or "IV"), rather than a word
/// as in "Figure legends".
fn label_from_head(head: &str, prefix: LabelPrefix) -> Option<String> {
    let rest = strip_prefix_word(head, prefix)?;
    let label = rest
        .split_whitespace()
        .next()?
        .trim_end_matches(['.', ':', '|']);

    let is_label = label.chars().any(|c| c.is_ascii_digit())
        || (!label.is_empty() && label.chars().all(|c| c.is_ascii_uppercase()));

    is_label.then(|| label.to_string())
}

/// Strip the word for a kind of float, such as "Figure" or "Tab.", from the
/// start of some text, ignoring case
///
/// The word must be followed by something other than a letter, so that, for
/// example, the "Fig" of "Figures" is not stripped.
fn strip_prefix_word(text: &str, prefix: LabelPrefix) -> Option<&str> {
    let lowercase = text.to_lowercase();
    let word = prefix
        .words()
        .iter()
        .find(|word| lowercase.starts_with(*word))?;

    let rest = text.get(word.len()..)?;
    if rest.chars().next().is_some_and(char::is_alphabetic) {
        return None;
    }

    Some(rest.trim_start())
}

/// Strip a leading label, such as "Figure 1." or "Table 2:", from a caption
///
/// Returns `None` if the caption does not start with the label.
fn strip_label<'s>(caption: &'s str, prefix: LabelPrefix, label: &str) -> Option<&'s str> {
    let rest = strip_prefix_word(caption, prefix)?;
    let rest = rest.strip_prefix(label)?.trim_start();
    let rest = rest.strip_prefix(['.', ':', '|'])?;

    Some(rest)
}

/// Create a loss label for something missing from an element, with the
/// element's id (if any) so that each instance is reported separately
fn missing(node: &Node, what: &str) -> String {
    match xml_id(node) {
        Some(id) => format!("{}[@xml:id='{id}'] ({what})", path(node)),
        None => format!("{} ({what})", path(node)),
    }
}
