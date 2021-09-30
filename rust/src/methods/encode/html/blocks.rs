use super::{
    attr, attr_data_itemprop, attr_id, attr_itemprop, attr_itemtype, attr_slot, concat, elem,
    elem_empty, json, Context, ToHtml,
};
use html_escape::encode_safe;
use stencila_schema::*;

/// Encode a vector of `BlockContent` as HTML
///
/// Note that if the `slot` is an empty string, then no wrapping `<div>`
/// will be generated. This allows the `content` property of nodes to be "implicit".
/// Use this when there are no other properties of a block that need to be encoded.
impl ToHtml for Vec<BlockContent> {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let blocks = self
            .iter()
            .map(|item| item.to_html("", context))
            .collect::<Vec<String>>()
            .concat();
        if slot.is_empty() {
            blocks
        } else {
            elem("div", &[attr_slot(slot)], &blocks)
        }
    }
}

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
                attr_slot(slot),
                attr_itemtype(self),
                attr_id(&self.id),
                attr("class", "todo"),
            ],
            &json(self),
        )
    }
}

impl ToHtml for CodeBlock {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        let programming_language = match &self.programming_language {
            Some(programming_language) => elem_empty(
                "meta",
                &[
                    attr_itemprop("programmingLanguage"),
                    attr("content", programming_language),
                ],
            ),
            None => "".to_string(),
        };

        let text = elem("code", &[attr_itemprop("text")], &encode_safe(&self.text));

        elem(
            "pre",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &[programming_language, text].concat(),
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[attr_data_itemprop("label")], label),
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
            &[attr_itemtype(self)],
            &[
                label,
                elem(
                    "stencila-code-chunk",
                    &[
                        attr_slot(slot),
                        attr_itemtype(self),
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
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
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
            Some(label) => elem("label", &[attr_data_itemprop("label")], label),
        };

        let content = match &self.content {
            None => String::new(),
            Some(nodes) => nodes.to_html("", context),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "figcaption",
                &[attr_data_itemprop("caption")],
                &match caption {
                    FigureCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    FigureCaption::VecBlockContent(content) => content.to_html("", context),
                },
            ),
        };

        elem(
            "figure",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
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
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
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
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
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

        let items = elem(
            tag,
            &[attr("slot", "items")],
            &self
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| item.to_html(&index.to_string(), context))
                .collect::<Vec<String>>()
                .concat(),
        );

        elem(
            "div",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &items,
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
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for MathBlock {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem(
            "pre",
            &[
                attr_slot(slot),
                attr_itemtype(self),
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
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        elem(
            "blockquote",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &self.content.to_html("", context),
        )
    }
}

impl ToHtml for TableSimple {
    fn to_html(&self, slot: &str, context: &Context) -> String {
        let label = match &self.label {
            None => String::new(),
            Some(label) => elem("label", &[attr_data_itemprop("label")], label),
        };

        let caption = match self.caption.as_deref() {
            None => String::new(),
            Some(caption) => elem(
                "div",
                &[attr_data_itemprop("caption")],
                &match caption {
                    TableCaption::String(string) => encode_safe(&string.clone()).to_string(),
                    TableCaption::VecBlockContent(content) => content.to_html("", context),
                },
            ),
        };

        let caption = elem("caption", &[], &[label, caption].concat());

        // Partition rows into head, body and foot rows
        let mut head = Vec::new();
        let mut body = Vec::new();
        let mut foot = Vec::new();
        for row in &self.rows {
            match &row.row_type {
                Some(row_type) => match row_type {
                    TableRowRowType::Header => head.push(row),
                    TableRowRowType::Footer => foot.push(row),
                },
                _ => body.push(row),
            }
        }

        // Generate table sections with cell types defaulting to appropriate variants
        let head = elem(
            "thead",
            &[],
            &concat(&head, |row| {
                table_row_to_html(row, TableCellCellType::Header, "", context)
            }),
        );
        let body = elem(
            "tbody",
            &[],
            &concat(&body, |row| {
                table_row_to_html(row, TableCellCellType::Data, "", context)
            }),
        );
        let foot = elem(
            "tfoot",
            &[],
            &concat(&foot, |row| {
                table_row_to_html(row, TableCellCellType::Header, "", context)
            }),
        );

        elem(
            "table",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
            &[caption, head, body, foot].concat(),
        )
    }
}

fn table_row_to_html(
    row: &TableRow,
    cell_type: TableCellCellType,
    slot: &str,
    context: &Context,
) -> String {
    let cells = concat(&row.cells, |cell| {
        let cell_type = match &cell.cell_type {
            Some(cell_type) => cell_type.clone(),
            None => cell_type.clone(),
        };
        let tag = match cell_type {
            TableCellCellType::Header => "th",
            TableCellCellType::Data => "td",
        };
        let content = match &cell.content {
            None => String::new(),
            Some(content) => match content {
                TableCellContent::VecInlineContent(nodes) => nodes.to_html("", context),
                TableCellContent::VecBlockContent(nodes) => nodes.to_html("", context),
            },
        };
        elem(tag, &[attr_itemtype(cell)], &content)
    });

    elem(
        "tr",
        &[attr_slot(slot), attr_itemtype(row), attr_id(&row.id)],
        &cells,
    )
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem_empty(
            "hr",
            &[attr_slot(slot), attr_itemtype(self), attr_id(&self.id)],
        )
    }
}
