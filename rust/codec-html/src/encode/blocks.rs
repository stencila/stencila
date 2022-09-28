//! Encode a `BlockContent` nodes to HTML

use codec::common::itertools::Itertools;
use node_dispatch::dispatch_block;
use stencila_schema::*;

use super::{
    attr, attr_and_meta, attr_and_meta_opt, attr_id, attr_itemprop, attr_itemtype, attr_prop,
    attr_slot, concat, data::label_and_input, elem, elem_empty, elem_meta, elem_meta_opt,
    elem_placeholder, elem_property, json, nothing, EncodeContext, ToHtml,
};

impl ToHtml for BlockContent {
    fn to_html(&self, context: &EncodeContext) -> String {
        dispatch_block!(self, to_html, context)
    }
}

impl ToHtml for ClaimSimple {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem(
            "pre",
            &[
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
    fn to_html(&self, context: &EncodeContext) -> String {
        let (lang_attr, lang_class, lang_meta) = match &self.programming_language {
            Some(programming_language) => (
                attr("programming-language", programming_language),
                attr("class", &["language-", programming_language].concat()),
                elem_meta("programmingLanguage", programming_language),
            ),
            None => (nothing(), nothing(), nothing()),
        };

        let text = elem(
            "pre",
            // The `slot="text"` attribute needs to be on the direct descendant of the
            // <stencila-code-block> element for WebComponent compatibility.
            // See https://github.com/stencila/designa/pull/268#discussion_r764363050
            &[attr_slot("text")],
            &elem(
                "code",
                &[attr_itemprop("text"), lang_class],
                &self.text.to_html(context),
            ),
        );

        elem(
            "stencila-code-block",
            &[attr_itemtype::<Self>(), attr_id(&self.id), lang_attr],
            &[lang_meta, text].concat(),
        )
    }
}

impl ToHtml for CodeChunk {
    fn to_html(&self, context: &EncodeContext) -> String {
        let lang = attr_and_meta("programming_language", &self.programming_language);

        let compile_digest = elem_meta_opt(
            "compile_digest",
            self.compile_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_digest = elem_meta_opt(
            "execute_digest",
            self.execute_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_auto = attr_and_meta_opt(
            "execute_auto",
            self.execute_auto
                .as_ref()
                .map(|auto| (*auto).as_ref().to_string()),
        );

        let execute_pure = attr_and_meta_opt(
            "execute_pure",
            self.execute_pure.as_ref().map(|value| value.to_string()),
        );

        let execute_required = attr_and_meta_opt(
            "execute_required",
            self.execute_required
                .as_ref()
                .map(|required| (*required).as_ref().to_string()),
        );

        let execute_kernel = attr_and_meta_opt(
            "execute_kernel",
            self.execute_kernel
                .as_ref()
                .map(|kernel| (*kernel).as_ref().to_string()),
        );

        let execute_status = attr_and_meta_opt(
            "execute_status",
            self.execute_status
                .as_ref()
                .map(|status| (*status).as_ref().to_string()),
        );

        let execute_count = attr_and_meta_opt(
            "execute_count",
            self.execute_count.map(|count| count.to_string()),
        );

        let text = elem(
            "pre",
            &[attr_prop("text"), attr_slot("text")],
            &self.text.to_html(context),
        );

        // For code_dependencies and code_dependents it is necessary to
        // place the items in a <div> under the custom element to avoid
        // elements added by the Web Component interfering with patch indexes.

        let dependencies = elem(
            "stencila-code-dependencies",
            &[attr_slot("code-dependencies")],
            &elem_placeholder(
                "div",
                &[attr_prop("code-dependencies")],
                &self.code_dependencies,
                context,
            ),
        );

        let dependents = elem(
            "stencila-code-dependencies",
            &[attr_slot("code-dependents")],
            &elem_placeholder(
                "div",
                &[attr_prop("code-dependents")],
                &self.code_dependents,
                context,
            ),
        );

        let execute_ended = elem_property(
            &[attr_prop("execute_ended"), attr_slot("execute-ended")],
            &self.execute_ended,
            context,
        );

        let execute_duration = elem_property(
            &[attr_prop("execute_duration"), attr_slot("execute-duration")],
            &self.execute_duration,
            context,
        );

        let outputs = elem_placeholder(
            "div",
            &[attr_prop("outputs"), attr_slot("outputs")],
            &self.outputs,
            context,
        );

        let errors = elem_placeholder(
            "div",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        let label = elem_placeholder(
            "span",
            &[attr_prop("label"), attr_slot("label")],
            &self.label,
            context,
        );

        let caption = elem_placeholder(
            "figcaption",
            &[attr_prop("caption"), attr_slot("caption")],
            &self.caption,
            context,
        );

        elem(
            "stencila-code-chunk",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                lang.0,
                execute_auto.0,
                execute_pure.0,
                execute_required.0,
                execute_status.0,
                execute_kernel.0,
                execute_count.0,
            ],
            &[
                lang.1,
                compile_digest,
                execute_digest,
                execute_auto.1,
                execute_pure.1,
                execute_required.1,
                execute_status.1,
                execute_kernel.1,
                execute_count.1,
                text,
                dependencies,
                dependents,
                execute_ended,
                execute_duration,
                outputs,
                errors,
                label,
                caption,
            ]
            .concat(),
        )
    }
}

impl ToHtml for CodeChunkCaption {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            CodeChunkCaption::String(string) => string.to_html(context),
            CodeChunkCaption::VecBlockContent(blocks) => blocks.to_html(context),
        }
    }
}

/// Encode a code error to HTML
///
/// In the future the current `CodeError` is likely to be replaced by a `CodeNotification`
/// (or similar) with a `level` property for info, warnings, errors etc.
impl ToHtml for CodeError {
    fn to_html(&self, context: &EncodeContext) -> String {
        let error_type = elem(
            "span",
            &[attr_prop("errorType"), attr_slot("type")],
            self.error_type
                .as_deref()
                .map(|value| value.as_str())
                .unwrap_or("Error"),
        );

        let error_message = elem(
            "span",
            &[attr_prop("errorMessage"), attr_slot("message")],
            &self.error_message.to_html(context),
        );

        // For inline `CodeExpression.errors` we need to use a span. In this context the stack traces are normally
        // hidden but we include an element anyway for consistency (e.g. so any patches on `CodeError` nodes
        // do not break unexpectedly). We can not rely on `context` inline since that is not populated when
        // generating HTML for patches.
        let stack_trace = elem_placeholder(
            "span",
            &[attr_prop("stackTrace"), attr_slot("stacktrace")],
            &self.stack_trace,
            context,
        );

        elem(
            "stencila-code-error",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &[error_type, error_message, stack_trace].concat(),
        )
    }
}

/// Encode a figure as HTML
///
/// Similar to as for tables, except that the label and caption are at the bottom
/// (although themes should be able to move them) and are not grouped together in a `<caption>`
/// element as they are in a table.
impl ToHtml for FigureSimple {
    fn to_html(&self, context: &EncodeContext) -> String {
        let content = elem_placeholder("div", &[attr_prop("content")], &self.content, context);

        let label = elem_placeholder("span", &[attr_prop("label")], &self.label, context);

        let caption = elem_placeholder(
            "figcaption",
            &[attr_prop("caption")],
            &self.caption,
            context,
        );

        elem(
            "figure",
            &[attr_itemtype::<Figure>(), attr_id(&self.id)],
            &[content, label, caption].concat(),
        )
    }
}

impl ToHtml for FigureCaption {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            FigureCaption::String(string) => string.to_html(context),
            FigureCaption::VecBlockContent(blocks) => blocks.to_html(context),
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
    fn to_html(&self, context: &EncodeContext) -> String {
        let depth = match &self.depth {
            Some(depth) => std::cmp::min(*depth + 1, 6),
            None => 2,
        };

        elem(
            &["h", &depth.to_string()].concat(),
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html(context),
        )
    }
}

impl ToHtml for Include {
    fn to_html(&self, context: &EncodeContext) -> String {
        let source = attr_and_meta("source", &self.source);

        let media_type = attr_and_meta_opt(
            "media_type",
            self.media_type.as_ref().map(|boxed| boxed.to_string()),
        );

        let select = attr_and_meta_opt(
            "select",
            self.select.as_ref().map(|boxed| boxed.to_string()),
        );

        let compile_digest = attr_and_meta_opt(
            "compile_digest",
            self.compile_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_digest = attr_and_meta_opt(
            "execute_digest",
            self.execute_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_auto = attr_and_meta_opt(
            "execute_auto",
            self.execute_auto
                .as_ref()
                .map(|auto| (*auto).as_ref().to_string()),
        );

        let execute_required = attr_and_meta_opt(
            "execute_required",
            self.execute_required
                .as_ref()
                .map(|required| (*required).as_ref().to_string()),
        );

        let execute_status = attr_and_meta_opt(
            "execute_status",
            self.execute_status
                .as_ref()
                .map(|status| (*status).as_ref().to_string()),
        );

        let dependencies = elem(
            "stencila-code-dependencies",
            &[attr_slot("code-dependencies")],
            &elem_placeholder(
                "div",
                &[attr_prop("code-dependencies")],
                &self.code_dependencies,
                context,
            ),
        );

        let errors = elem_placeholder(
            "div",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        let content = elem(
            "div",
            &[attr_prop("content"), attr_slot("content")],
            &self
                .content
                .as_ref()
                .map_or_else(|| "".to_string(), |content| content.to_html(context)),
        );

        elem(
            "stencila-include",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                source.0,
                media_type.0,
                select.0,
                compile_digest.0,
                execute_auto.0,
                execute_digest.0,
                execute_required.0,
                execute_status.0,
            ],
            &[
                source.1,
                media_type.1,
                select.1,
                compile_digest.1,
                execute_auto.1,
                execute_digest.1,
                execute_required.1,
                execute_status.1,
                dependencies,
                errors,
                content,
            ]
            .concat(),
        )
    }
}

impl ToHtml for Call {
    fn to_html(&self, context: &EncodeContext) -> String {
        let source = attr_and_meta("source", &self.source);

        let media_type = attr_and_meta_opt(
            "media_type",
            self.media_type.as_ref().map(|boxed| boxed.to_string()),
        );

        let select = attr_and_meta_opt(
            "select",
            self.select.as_ref().map(|boxed| boxed.to_string()),
        );

        let compile_digest = attr_and_meta_opt(
            "compile_digest",
            self.compile_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_digest = attr_and_meta_opt(
            "execute_digest",
            self.execute_digest.as_ref().map(|cord| cord.0.to_string()),
        );

        let execute_auto = attr_and_meta_opt(
            "execute_auto",
            self.execute_auto
                .as_ref()
                .map(|auto| (*auto).as_ref().to_string()),
        );

        let execute_required = attr_and_meta_opt(
            "execute_required",
            self.execute_required
                .as_ref()
                .map(|required| (*required).as_ref().to_string()),
        );

        let execute_status = attr_and_meta_opt(
            "execute_status",
            self.execute_status
                .as_ref()
                .map(|status| (*status).as_ref().to_string()),
        );

        let execute_ended = attr_and_meta_opt(
            "execute_ended",
            self.execute_ended
                .as_ref()
                .map(|date| (**date).value.to_string()),
        );

        let execute_duration = attr_and_meta_opt(
            "execute_duration",
            self.execute_duration
                .as_ref()
                .map(|seconds| seconds.to_string()),
        );

        let arguments = elem(
            "div",
            &[attr_prop("arguments"), attr_slot("arguments")],
            &self
                .arguments
                .iter()
                .flatten()
                .enumerate()
                .map(|(index, arg)| {
                    let symbol_value_attr = match &arg.symbol {
                        Some(symbol) => attr("value", symbol),
                        None => "".to_string(),
                    };
                    let symbol = elem_empty(
                        "input",
                        &[attr_prop("symbol"), attr_slot("symbol"), symbol_value_attr],
                    );

                    let (name, value) = label_and_input(
                        &arg.name,
                        &arg.validator,
                        &arg.value,
                        &arg.default,
                        context,
                    );

                    elem(
                        "stencila-call-argument",
                        &[
                            attr_itemtype::<CallArgument>(),
                            attr("index", &index.to_string()),
                        ],
                        &[name, symbol, value].concat(),
                    )
                })
                .join(""),
        );

        let dependencies = elem(
            "stencila-code-dependencies",
            &[attr_slot("code-dependencies")],
            &elem_placeholder(
                "div",
                &[attr_prop("code-dependencies")],
                &self.code_dependencies,
                context,
            ),
        );

        let errors = elem_placeholder(
            "div",
            &[attr_prop("errors"), attr_slot("errors")],
            &self.errors,
            context,
        );

        let content = elem(
            "div",
            &[attr_prop("content"), attr_slot("content")],
            &self
                .content
                .as_ref()
                .map_or_else(|| "".to_string(), |content| content.to_html(context)),
        );

        elem(
            "stencila-call",
            &[
                attr_itemtype::<Self>(),
                attr_id(&self.id),
                source.0,
                media_type.0,
                select.0,
                compile_digest.0,
                execute_auto.0,
                execute_digest.0,
                execute_required.0,
                execute_status.0,
                execute_ended.0,
                execute_duration.0,
            ],
            &[
                source.1,
                media_type.1,
                select.1,
                compile_digest.1,
                execute_auto.1,
                execute_digest.1,
                execute_required.1,
                execute_status.1,
                execute_ended.1,
                execute_duration.1,
                arguments,
                dependencies,
                errors,
                content,
            ]
            .concat(),
        )
    }
}

impl ToHtml for For {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem(
            "stencila-for",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            "",
        )
    }
}

impl ToHtml for If {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem(
            "stencila-if",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            "",
        )
    }
}

impl ToHtml for List {
    fn to_html(&self, context: &EncodeContext) -> String {
        let tag = match &self.order {
            Some(ListOrder::Ascending) => "ol",
            _ => "ul",
        };

        elem(
            tag,
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self
                .items
                .iter()
                .map(|item| item.to_html(context))
                .collect::<Vec<String>>()
                .concat(),
        )
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, context: &EncodeContext) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("☑ ".to_string()),
            false => InlineContent::String("☐ ".to_string()),
        });

        let content = match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()].concat().to_html(context),
                    None => inlines.to_html(context),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added if the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [
                                paragraph.to_html(context),
                                concat(&blocks[1..], |block| block.to_html(context)),
                            ]
                            .concat()
                        } else {
                            blocks.to_html(context)
                        }
                    }
                    None => blocks.to_html(context),
                },
            },
            None => "".to_string(),
        };

        elem(
            "li",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &content,
        )
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "p",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html(context),
        )
    }
}

impl ToHtml for QuoteBlock {
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "blockquote",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.content.to_html(context),
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
    fn to_html(&self, context: &EncodeContext) -> String {
        let label = elem_placeholder("span", &[attr_prop("label")], &self.label, context);

        let caption = elem_placeholder("div", &[attr_prop("caption")], &self.caption, context);

        let body = elem(
            "tbody",
            &[attr_prop("rows")],
            &concat(&self.rows, |row| row.to_html(context)),
        );

        elem(
            "table",
            &[attr_itemtype::<Table>(), attr_id(&self.id)],
            &[elem("caption", &[], &[label, caption].concat()), body].concat(),
        )
    }
}

impl ToHtml for TableCaption {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            TableCaption::String(string) => string.to_html(context),
            TableCaption::VecBlockContent(blocks) => blocks.to_html(context),
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
    fn to_html(&self, context: &EncodeContext) -> String {
        elem(
            "tr",
            &[attr_itemtype::<Self>(), attr_id(&self.id)],
            &self.cells.to_html(context),
        )
    }
}

impl ToHtml for TableCell {
    fn to_html(&self, context: &EncodeContext) -> String {
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
            &[attr_itemtype::<Self>(), attr_id(&self.id), colspan, rowspan],
            &self.content.to_html(context),
        )
    }
}

impl ToHtml for TableCellCellType {
    fn to_html(&self, _context: &EncodeContext) -> String {
        match self {
            TableCellCellType::Header => "Header".to_string(),
            TableCellCellType::Data => "Data".to_string(),
        }
    }
}

impl ToHtml for TableCellContent {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            TableCellContent::VecInlineContent(nodes) => nodes.to_html(context),
            TableCellContent::VecBlockContent(nodes) => nodes.to_html(context),
        }
    }
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem_empty("hr", &[attr_itemtype::<Self>(), attr_id(&self.id)])
    }
}
