use codec_info::lost_options;

use crate::{HorizontalAlignment, Table, TableRow, TableRowType, prelude::*};

use super::utils::caption_to_dom;

impl Table {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let mut losses = lost_options!(self, id);

        let mut attrs = Vec::new();
        if let Some(value) = &self.label_automatically {
            attrs.push(("label-automatically", value));
        }

        let mut table_wrap = String::new();

        if let Some(label) = &self.label {
            let (label, label_losses) = label.to_jats();
            losses.merge(label_losses);
            table_wrap.push_str(&elem_no_attrs("label", label));
        }

        if let Some(caption) = &self.caption {
            let (caption, caption_losses) = caption.to_jats();
            losses.merge(caption_losses);
            table_wrap.push_str(&elem_no_attrs("caption", caption));
        }

        let mut thead = String::new();
        let mut tbody = String::new();
        let mut tfoot = String::new();
        for row in &self.rows {
            let (row_jats, row_losses) = row.to_jats();

            match row.row_type {
                Some(TableRowType::HeaderRow) => thead.push_str(&row_jats),
                Some(TableRowType::FooterRow) => tfoot.push_str(&row_jats),
                _ => tbody.push_str(&row_jats),
            }

            losses.merge(row_losses);
        }

        let mut table = String::new();
        if !thead.is_empty() {
            table.push_str(&elem_no_attrs("thead", thead));
        }
        if !tbody.is_empty() {
            table.push_str(&elem_no_attrs("tbody", tbody));
        }
        if !tfoot.is_empty() {
            table.push_str(&elem_no_attrs("tfoot", tfoot));
        }

        let table = elem_no_attrs("table", table);
        table_wrap.push_str(&table);

        if let Some(notes) = &self.notes {
            let (notes, notes_losses) = notes.to_jats();
            table_wrap.push_str(&elem_no_attrs("table-wrap-foot", notes));
            losses.merge(notes_losses);
        }

        (elem("table-wrap", attrs, table_wrap), losses)
    }

    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::{attr, elem};

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

impl MarkdownCodec for Table {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        use codec_markdown_trait::to_markdown;

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

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
                        encode_rows_to_markdown(&self.rows, context);
                        context.newline();
                    },
                );
            } else {
                encode_rows_to_markdown(&self.rows, context);
            }

            context.exit_node().newline();
        } else {
            let wrapped = if (self.label.is_some() && !self.label_automatically.unwrap_or(true))
                || self.caption.is_some()
                || self.notes.is_some()
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

            encode_rows_to_markdown(&self.rows, context);

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

            context.exit_node().newline();
        }
    }
}

// Encode the rows of the table to Markdown
fn encode_rows_to_markdown(self_rows: &[TableRow], context: &mut MarkdownEncodeContext) {
    // Do a first iteration over rows and cells to generate the Markdown
    // for each cell and determine column widths and alignments
    let mut column_widths: Vec<usize> = Vec::new();
    let mut column_alignments: Vec<Option<HorizontalAlignment>> = Vec::new();
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

            // Column alignment determined by the first cell with a non-None alignment
            match column_alignments.get_mut(column) {
                Some(column_alignment) => {
                    if column_alignment.is_none() && cell.horizontal_alignment.is_some() {
                        *column_alignment = cell.horizontal_alignment;
                    }
                }
                None => column_alignments.push(cell.horizontal_alignment),
            }

            cells.push(cell_md);
            context.merge_losses(cell_context.losses);
        }
        rows.push(cells);
    }

    // Rows
    let divider_row = |context: &mut MarkdownEncodeContext| {
        context.push_str("|");
        for (width, alignment) in column_widths.iter().zip(column_alignments.iter()) {
            match alignment {
                Some(HorizontalAlignment::AlignLeft) => {
                    context
                        .push_str(" :")
                        .push_str(&"-".repeat(width.saturating_sub(1)))
                        .push_str(" |");
                }
                Some(HorizontalAlignment::AlignCenter) => {
                    context
                        .push_str(" :")
                        .push_str(&"-".repeat(width.saturating_sub(2)))
                        .push_str(": |");
                }
                Some(HorizontalAlignment::AlignRight) => {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(width.saturating_sub(1)))
                        .push_str(": |");
                }
                _ => {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(*width))
                        .push_str(" |");
                }
            }
        }
        context.newline();
    };
    let empty_row = |context: &mut MarkdownEncodeContext| {
        context.push_str("|");
        for width in &column_widths {
            context.push_str(&" ".repeat(width + 2)).push_str("|");
        }
        context.newline();
    };
    for (row_index, row) in self_rows.iter().enumerate() {
        // If this is the first and only row then add an empty header if not
        // a header row and and empty body otherwise
        let (empty_header, empty_body) = if row_index == 0 && self_rows.len() == 1 {
            let empty_header = !matches!(row.row_type, Some(TableRowType::HeaderRow));
            (empty_header, !empty_header)
        } else {
            (false, false)
        };

        if empty_header {
            empty_row(context)
        }

        if empty_header || row_index == 1 {
            divider_row(context)
        }

        context.enter_node(row.node_type(), row.node_id());

        let cells = &rows[row_index];
        for (cell_index, cell) in row.cells.iter().enumerate() {
            if cell_index == 0 {
                context.push_str("|");
            }

            context.enter_node(cell.node_type(), cell.node_id());

            let content = &cells[cell_index];
            let width = column_widths[cell_index];

            let aligned_cell = match column_alignments.get(cell_index).unwrap_or(&None) {
                Some(HorizontalAlignment::AlignLeft) => format!(" {content:<width$} "),
                Some(HorizontalAlignment::AlignCenter) => format!(" {content:^width$} "),
                Some(HorizontalAlignment::AlignRight) => format!(" {content:>width$} "),
                _ => format!(" {content:<width$} "),
            };

            context.push_str(&aligned_cell).exit_node().push_str("|");
        }
        context.newline().exit_node();

        if empty_body {
            divider_row(context);
            empty_row(context);
        }
    }
}
