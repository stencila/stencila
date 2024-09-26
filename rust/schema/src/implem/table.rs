use codec_html_trait::encode::{attr, elem};
use codec_info::lost_options;
use codec_markdown_trait::to_markdown;

use crate::{prelude::*, Table, TableCell, TableCellType, TableRow};

use super::utils::caption_to_dom;

impl Table {
    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        let label = self
            .label
            .as_ref()
            .map(|label| elem("span", &[attr("slot", "label")], &[label.to_html(context)]));

        let caption = self.caption.as_ref().map(|caption| {
            elem(
                "span",
                &[attr("slot", "caption")],
                &[caption.to_html(context)],
            )
        });

        let caption = if label.is_some() && caption.is_some() {
            elem(
                "caption",
                &[],
                &[label.unwrap_or_default(), caption.unwrap_or_default()],
            )
        } else {
            String::new()
        };

        let body = elem("tbody", &[], &[self.rows.to_html(context)]);

        elem("table", &[], &[caption, body])
    }
}

impl DomCodec for Table {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        // Strictly, <caption> should be within <table>, but that causes issues for styling of web component,
        // so we make it a sibling <div> (because the browser will unwrap a <caption> if not within a <table>)
        // See https://github.com/stencila/stencila/pull/2240#issuecomment-2136358172
        if self.caption.is_some() {
            context.push_slot_fn("div", "caption", |context| {
                caption_to_dom(context, "table-label", "Table", &self.label, &self.caption)
            });
        }

        context.push_slot_fn("table", "rows", |context| self.rows.to_dom(context));

        if let Some(notes) = &self.notes {
            context.push_slot_fn("aside", "notes", |context| notes.to_dom(context));
        }

        context.exit_node();
    }
}

impl DomCodec for TableRow {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Can not use a custom element (i.e. <stencila-table-row>) because only <tr> elements
        // are allowed in a <tbody>
        context.enter_node_elem("tr", self.node_type(), self.node_id());
        self.cells.to_dom(context);
        context.exit_node();
    }
}

impl DomCodec for TableCell {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Can not use a custom element (i.e. <stencila-table-cell>) because only <th> or <td> elements
        // are allowed in a <tr>.
        let name = match self.cell_type {
            Some(TableCellType::HeaderCell) => "th",
            _ => "td",
        };
        context.enter_node_elem(name, self.node_type(), self.node_id());

        if let Some(row_span) = self.options.row_span {
            context.push_attr("rowspan", &row_span.to_string());
        }
        if let Some(column_span) = self.options.column_span {
            context.push_attr("colspan", &column_span.to_string());
        }

        self.content.to_dom(context);
        context.exit_node();
    }
}

impl MarkdownCodec for Table {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        // Encode the rows of the table
        fn encode_rows(self_rows: &[TableRow], context: &mut MarkdownEncodeContext) {
            // Do a first iteration over rows and cells to generate the Markdown
            // for each cell and determine column widths
            let mut column_widths: Vec<usize> = Vec::new();
            let mut rows: Vec<Vec<String>> = Vec::new();
            for row in self_rows {
                let mut cells: Vec<String> = Vec::new();
                for (column, cell) in row.cells.iter().enumerate() {
                    let mut cell_context = MarkdownEncodeContext::default();
                    cell.content.to_markdown(&mut cell_context);

                    // Trim, replace inner newlines with <br> (because content is blocks, but in
                    // Markdown tables must be a single line), & ensure cell has no carriage returns or pipes
                    // which will break table
                    let cell_md = cell_context
                        .content
                        .trim()
                        .replace('\n', "<br><br>")
                        .replace('\r', " ")
                        .replace('|', "\\|");

                    let width = cell_md.chars().count();
                    match column_widths.get_mut(column) {
                        Some(column_width) => {
                            if width > *column_width {
                                *column_width = width
                            }
                        }
                        None => column_widths.push(3.max(width)),
                    }

                    cells.push(cell_md);
                    context.merge_losses(cell_context.losses);
                }
                rows.push(cells);
            }

            // Rows
            for (row_index, row) in self_rows.iter().enumerate() {
                // If there is only one row, header row should be empty
                if row_index == 0 && self_rows.len() == 1 {
                    context.push_str("| ");
                    for width in &column_widths {
                        context.push_str(&" ".repeat(*width)).push_str(" |");
                    }
                }

                if (row_index == 0 && self_rows.len() == 1) || row_index == 1 {
                    context.push_str("|");
                    for width in &column_widths {
                        context
                            .push_str(" ")
                            .push_str(&"-".repeat(*width))
                            .push_str(" |");
                    }
                    context.newline();
                }

                context.enter_node(row.node_type(), row.node_id());

                let cells = &rows[row_index];
                for (cell_index, cell) in row.cells.iter().enumerate() {
                    if cell_index == 0 {
                        context.push_str("|");
                    }

                    context
                        .enter_node(cell.node_type(), cell.node_id())
                        .push_str(&format!(
                            " {md:width$} ",
                            md = cells[cell_index],
                            width = column_widths[cell_index]
                        ))
                        .exit_node()
                        .push_str("|");
                }
                context.newline().exit_node();
            }
        }

        if matches!(context.format, Format::Myst) {
            if self.label.is_some() || self.caption.is_some() {
                context.myst_directive(
                    ':',
                    "table",
                    |context| {
                        if let Some(caption) = &self.caption {
                            // Note: caption must be a single line
                            let caption = to_markdown(caption).replace('\n', " ");
                            context
                                .push_str(" ")
                                .push_prop_str(NodeProperty::Caption, &caption);
                        }
                    },
                    |context| {
                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }
                    },
                    |context| {
                        encode_rows(&self.rows, context);
                        context.newline();
                    },
                );
            } else {
                encode_rows(&self.rows, context);
            }
        } else {
            let wrapped = if self.label.is_some() || self.caption.is_some() || self.notes.is_some()
            {
                context.push_colons().push_str(" table");

                if !self.label_automatically.unwrap_or(true) {
                    if let Some(label) = &self.label {
                        context.push_str(" ");
                        context.push_prop_str(NodeProperty::Label, label);
                    }
                }

                context.push_str("\n\n");

                true
            } else {
                false
            };

            if let Some(caption) = &self.caption {
                context
                    .increase_depth()
                    .push_prop_fn(NodeProperty::Caption, |context| {
                        caption.to_markdown(context)
                    })
                    .decrease_depth();
            }

            encode_rows(&self.rows, context);

            if let Some(notes) = &self.notes {
                context
                    .newline()
                    .increase_depth()
                    .push_prop_fn(NodeProperty::Notes, |context| notes.to_markdown(context))
                    .decrease_depth();
            }

            if wrapped {
                if self.notes.is_none() {
                    context.newline();
                }
                context.push_colons().newline();
            }
        }

        context.exit_node().newline();
    }
}
