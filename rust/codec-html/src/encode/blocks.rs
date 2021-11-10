//! Encode a `BlockContent` nodes to HTML

use super::{
    attr, attr_id, attr_itemprop, attr_itemtype, attr_prop, attr_slot, concat, elem, elem_empty,
    elem_meta, elem_placeholder, json, nothing, EncodeContext, ToHtml,
};
use html_escape::encode_safe;
use stencila_schema::*;

impl ToHtml for BlockContent {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            BlockContent::Claim(node) => node.to_html(slot, context),
            BlockContent::CodeBlock(node) => node.to_html(slot, context),
            BlockContent::CodeChunk(node) => node.to_html(slot, context),
            BlockContent::Collection(node) => node.to_html(slot, context),
            BlockContent::Figure(node) => node.to_html(slot, context),
            BlockContent::Heading(node) => node.to_html(slot, context),
            BlockContent::Include(node) => node.to_html(slot, context),
            BlockContent::List(node) => node.to_html(slot, context),
            BlockContent::MathBlock(node) => node.to_html(slot, context),
            BlockContent::Paragraph(node) => node.to_html(slot, context),
            BlockContent::QuoteBlock(node) => node.to_html(slot, context),
            BlockContent::Table(node) => node.to_html(slot, context),
            BlockContent::ThematicBreak(node) => node.to_html(slot, context),
        }
    }
}

impl ToHtml for ClaimSimple {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem(
            "pre",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for CodeBlock {
    /// Encode a [`CodeBlock`] as HTML
    ///
    /// The `programming_language` is encoded as both a `class` attribute and a `<meta>` element.
    /// The `<meta>` element is for Microdata and Stencila WebComponent compatibility.
    /// The `class` follows the recommendation of [HTML5 spec](https://html.spec.whatwg.org/#the-code-element)
    /// to "use the class attribute, e.g. by adding a class prefixed with "language-" to the element."
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        let (lang_class, lang_meta) = match &self.programming_language {
            Some(programming_language) => (
                attr("class", &["language-", programming_language].concat()),
                elem_meta("programmingLanguage", programming_language),
            ),
            None => (nothing(), nothing()),
        };

        let text = elem(
            "code",
            &[attr_itemprop("text"), lang_class],
            &encode_safe(&self.text),
        );

        elem(
            "pre",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[lang_meta, text].concat(),
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let text = elem(
            "pre",
            &[attr_prop("text"), attr_slot("text")],
            &encode_safe(&self.text),
        );

        let lang_attr = attr("programming-language", &self.programming_language);
        let lang_meta = elem_meta("programmingLanguage", &self.programming_language);

        let outputs = elem(
            "div",
            &[attr_prop("outputs"), attr_slot("outputs")],
            &match &self.outputs {
                Some(outputs) => outputs.to_html("", context),
                None => nothing(),
            },
        );

        let errors = elem(
            "div",
            &[attr_prop("errors"), attr_slot("errors")],
            &match &self.errors {
                Some(errors) => errors.to_html("", context),
                None => nothing(),
            },
        );

        let label = match &self.label {
            Some(label) => label.to_html("label", context),
            None => elem_placeholder("span", "label"),
        };

        let caption = match &self.caption {
            Some(caption) => caption.to_html("caption", context),
            None => elem_placeholder("figcaption", "caption"),
        };

        elem(
            "stencila-code-chunk",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                lang_attr,
            ],
            &[lang_meta, text, outputs, errors, label, caption].concat(),
        )
    }
}

impl ToHtml for CodeChunkCaption {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            CodeChunkCaption::String(string) => {
                elem("figcaption", &[], &string.to_html(slot, context))
            }
            CodeChunkCaption::VecBlockContent(content) => elem(
                "figcaption",
                &[attr_prop(slot)],
                &content.to_html("", context),
            ),
        }
    }
}

impl ToHtml for CodeError {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        let error_message = elem("span", &[attr_prop("errorMessage")], &self.error_message);

        let error_type = match &self.error_type {
            None => nothing(),
            Some(error_type) => elem("span", &[attr_prop("errorType")], error_type),
        };

        let stack_trace = match &self.stack_trace {
            None => nothing(),
            Some(stack_trace) => elem("pre", &[attr_prop("stackTrace")], stack_trace),
        };

        elem(
            "div",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[error_message, error_type, stack_trace].concat(),
        )
    }
}

impl ToHtml for CollectionSimple {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        elem(
            "ol",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &concat(&self.parts, |part| {
                elem("li", &[], &part.to_html("", context))
            }),
        )
    }
}

/// Encode a figure as HTML
///
/// Similar to as for tables, except that the label and caption are at the bottom
/// (although themes should be able to move them) and are not grouped together in a `<caption>`
/// element as they are in a table.
impl ToHtml for FigureSimple {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let content = match &self.content {
            Some(content) => elem(
                "div",
                &[attr_prop("content")],
                &content.to_html("", context),
            ),
            None => elem_placeholder("div", "content"),
        };

        let label = match &self.label {
            Some(label) => label.to_html("label", context),
            None => elem_placeholder("span", "label"),
        };

        let caption = match &self.caption {
            Some(caption) => caption.to_html("caption", context),
            None => elem_placeholder("figcaption", "caption"),
        };

        elem(
            "figure",
            &[
                attr_prop(slot),
                attr_itemtype::<Figure>(),
                attr_id(&self.id),
            ],
            &[content, label, caption].concat(),
        )
    }
}

impl ToHtml for FigureCaption {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            FigureCaption::String(string) => {
                elem("figcaption", &[], &string.to_html(slot, context))
            }
            FigureCaption::VecBlockContent(content) => elem(
                "figcaption",
                &[attr_prop(slot)],
                &content.to_html("", context),
            ),
        }
    }
}

impl ToHtml for Heading {
    /// Encode a `Heading` node to a `<h2>`, `<h3>` etc element.
    ///
    /// > Generally, it is a best practice to ensure that the beginning of a
    /// > page's main content starts with a h1 element, and also to ensure
    /// > that the page contains only one h1 element.
    /// > From https://dequeuniversity.com/rules/axe/3.5/page-has-heading-one
    ///
    /// This codec follows that recommendation and reserves `<h1>` for the
    /// `title` property of a creative work.
    ///
    /// In rare cases that there is no content in the heading, return an empty
    /// text node to avoid the 'Heading tag found with no content' accessibility error.
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let depth = match &self.depth {
            Some(depth) => std::cmp::min(*depth + 1, 6),
            None => 2,
        };

        elem(
            &["h", &depth.to_string()].concat(),
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for Include {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let content = self
            .content
            .as_ref()
            .map_or_else(|| "".to_string(), |content| content.to_html("", context));

        elem(
            "div",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for List {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let tag = match &self.order {
            Some(ListOrder::Ascending) => "ol",
            _ => "ul",
        };

        elem(
            tag,
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self
                .items
                .iter()
                .map(|item| item.to_html("", context))
                .collect::<Vec<String>>()
                .concat(),
        )
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("☑ ".to_string()),
            false => InlineContent::String("☐ ".to_string()),
        });

        let content = match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()]
                        .concat()
                        .to_html("", context),
                    None => inlines.to_html("", context),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added if the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [
                                paragraph.to_html("", context),
                                concat(&blocks[1..], |block| block.to_html("", context)),
                            ]
                            .concat()
                        } else {
                            blocks.to_html("", context)
                        }
                    }
                    None => blocks.to_html("", context),
                },
            },
            None => "".to_string(),
        };

        elem(
            "li",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for MathBlock {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem(
            "pre",
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                attr("class", "todo"),
            ],
            &encode_safe(&self.text),
        )
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        elem(
            "p",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        elem(
            "blockquote",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

/// Encode a table as HTML
///
/// Previously this function split the table cell's into `thead`, `tbody` and `tfoot` sections.
/// However, that interferes with resolving cell addresses in the DOM, so we reverted to a
/// simpler approach of placing all cell into `tbody`
///
/// Note that both the `label` and `caption` properties are nested within a `<caption>` element.
impl ToHtml for TableSimple {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let label = match &self.label {
            Some(label) => label.to_html("label", context),
            None => elem_placeholder("span", "label"),
        };

        let caption = match &self.caption {
            Some(caption) => caption.to_html("caption", context),
            None => elem_placeholder("div", "caption"),
        };

        let body = elem(
            "tbody",
            &[attr_prop("rows")],
            &concat(&self.rows, |row| row.to_html(slot, context)),
        );

        elem(
            "table",
            &[attr_prop(slot), attr_itemtype::<Table>(), attr_id(&self.id)],
            &[elem("caption", &[], &[label, caption].concat()), body].concat(),
        )
    }
}

impl ToHtml for TableCaption {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            TableCaption::String(string) => string.to_html(slot, context),
            TableCaption::VecBlockContent(content) => {
                elem("div", &[attr_prop(slot)], &content.to_html("", context))
            }
        }
    }
}

/// Encode a table row to HTML
///
/// Previously we passed the row type down to the cells so that they could use `<th>`
/// or `<td>` if a cell type was not specified. However, that does not allow adding
/// a `th` cell using a patch (because when it is part of the patch, the new cell does
/// not know it's row context). Therefore we deprecate the use of row type alone, and
/// encourage use of both for header rows.
impl ToHtml for TableRow {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        elem(
            "tr",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.cells.to_html("", context),
        )
    }
}

impl ToHtml for TableCell {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        let tag = match &self.cell_type {
            Some(cell_type) => match cell_type {
                TableCellCellType::Header => "th",
                TableCellCellType::Data => "td",
            },
            None => "td",
        };

        let colspan = match self.colspan {
            Some(colspan) => attr("colspan", &colspan.to_string()),
            None => nothing(),
        };

        let rowspan = match self.rowspan {
            Some(rowspan) => attr("rowspan", &rowspan.to_string()),
            None => nothing(),
        };

        elem(
            tag,
            &[
                attr_prop(slot),
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                colspan,
                rowspan,
            ],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for TableCellCellType {
    fn to_html(&self, _slot: &str, _context: &EncodeContext) -> String {
        match self {
            TableCellCellType::Header => "Header".to_string(),
            TableCellCellType::Data => "Data".to_string(),
        }
    }
}

impl ToHtml for TableCellContent {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            TableCellContent::VecInlineContent(nodes) => nodes.to_html(slot, context),
            TableCellContent::VecBlockContent(nodes) => nodes.to_html(slot, context),
        }
    }
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem_empty(
            "hr",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
        )
    }
}
