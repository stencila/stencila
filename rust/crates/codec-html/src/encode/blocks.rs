//! Encode a `BlockContent` nodes to HTML

use super::{
    attr, attr_id, attr_itemprop, attr_itemtype, attr_prop, concat, elem, elem_empty, json,
    Context, ToHtml,
};
use html_escape::encode_safe;
use stencila_schema::*;

impl ToHtml for BlockContent {
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
    fn to_html(&self, slot: &str, _context: &Context) -> String {
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
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        let (class, meta) = match &self.programming_language {
            Some(programming_language) => (
                attr("class", &["language-", programming_language].concat()),
                elem_empty(
                    "meta",
                    &[
                        attr_itemprop("programming_language"),
                        attr("content", programming_language),
                    ],
                ),
            ),
            None => ("".to_string(), "".to_string()),
        };

        let text = elem(
            "code",
            &[attr_itemprop("text"), class],
            &encode_safe(&self.text),
        );

        elem(
            "pre",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[meta, text].concat(),
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[attr_prop("label")], label),
        };

        let caption = match &self.caption {
            None => String::new(),
            Some(boxed) => match &**boxed {
                CodeChunkCaption::String(string) => string.clone(),
                CodeChunkCaption::VecBlockContent(content) => content.to_html("", context),
            },
        };

        let text = elem("pre", &[attr("slot", "text")], &encode_safe(&self.text));

        let outputs = match &self.outputs {
            None => String::new(),
            Some(outputs) => elem(
                "pre",
                &[attr("slot", "outputs")],
                &outputs.to_html("", context),
            ),
        };

        elem(
            "figure",
            &[attr_itemtype::<Self>()],
            &[
                label,
                elem(
                    "stencila-code-chunk",
                    &[
                        attr_prop(slot),
                        attr_itemtype::<Self>(),
                        attr_id(&self.id),
                        attr("programming-language", &self.programming_language),
                    ],
                    &[text, outputs].concat(),
                ),
                caption,
            ]
            .concat(),
        )
    }
}

impl ToHtml for CollectionSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "ol",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &concat(&self.parts, |part| {
                elem("li", &[], &part.to_html("", context))
            }),
        )
    }
}

impl ToHtml for FigureSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[attr_prop("label")], label),
        };

        let content = match &self.content {
            None => String::new(),
            Some(nodes) => nodes.to_html("", context),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "figcaption",
                &[attr_prop("caption")],
                &match caption {
                    FigureCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    FigureCaption::VecBlockContent(content) => content.to_html("", context),
                },
            ),
        };

        elem(
            "figure",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[label, content, caption].concat(),
        )
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
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
    fn to_html(&self, slot: &str, _context: &Context) -> String {
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
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "p",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, slot: &str, context: &Context) -> String {
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
impl ToHtml for TableSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[attr_prop("label")], label),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "div",
                &[attr_prop("caption")],
                &match caption {
                    TableCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    TableCaption::VecBlockContent(content) => content.to_html("", context),
                },
            ),
        };

        let caption = elem("caption", &[], &[label, caption].concat());

        let body = elem(
            "tbody",
            &[attr_prop("rows")],
            &concat(&self.rows, |row| row.to_html(slot, context)),
        );

        elem(
            "table",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
            &[caption, body].concat(),
        )
    }
}

impl ToHtml for TableRow {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let cells = concat(&self.cells, |cell| {
            let tag = match &cell.cell_type {
                Some(cell_type) => match cell_type {
                    TableCellCellType::Header => "th",
                    TableCellCellType::Data => "td",
                },

                None => match &self.row_type {
                    Some(TableRowRowType::Header) | Some(TableRowRowType::Footer) => "th",
                    _ => "td",
                },
            };

            let content = match &cell.content {
                None => String::new(),
                Some(content) => match content {
                    TableCellContent::VecInlineContent(nodes) => nodes.to_html("", context),
                    TableCellContent::VecBlockContent(nodes) => nodes.to_html("", context),
                },
            };

            elem(tag, &[attr_itemtype::<TableCell>()], &content)
        });

        elem(
            "tr",
            &[
                attr_prop(slot),
                attr_itemtype::<TableRow>(),
                attr_id(&self.id),
            ],
            &cells,
        )
    }
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem_empty(
            "hr",
            &[attr_prop(slot), attr_itemtype::<Self>(), attr_id(&self.id)],
        )
    }
}
