use codec_html_trait::encode::{attr, elem};
use codec_losses::{lost_options, lost_work_options};

use crate::{prelude::*, Table};

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

impl MarkdownCodec for Table {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_work_options!(self));

        let wrapped = if self.label.is_some() || self.caption.is_some() {
            context.push_semis().push_str(" table");

            if let Some(label) = &self.label {
                context.push_str(" ");
                context.push_prop_str("label", label);
            }

            context.push_str("\n\n");

            true
        } else {
            false
        };

        if let Some(caption) = &self.caption {
            context
                .increase_depth()
                .push_prop_fn("caption", |context| caption.to_markdown(context))
                .decrease_depth();
        }

        // Do a first iteration over rows and cells to generate the Markdown
        // for each cell and determine column widths
        let mut column_widths: Vec<usize> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        for row in &self.rows {
            let mut cells: Vec<String> = Vec::new();
            for (column, cell) in row.cells.iter().enumerate() {
                let mut cell_context = MarkdownEncodeContext::default();
                cell.content.to_markdown(&mut cell_context);

                // Trim, replace inner newlines with <br> (because content is blocks, but in
                // Markdown tables must be a single line), & ensure cell has no newlines or pipes
                // which will break table
                let cell_md = cell_context
                    .content
                    .trim()
                    .replace('\n', "<br><br>")
                    .replace("\r\n", " ")
                    .replace('\n', " ")
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
        for (row_index, row) in self.rows.iter().enumerate() {
            // If there is only one row, header row should be empty
            if row_index == 0 && self.rows.len() == 1 {
                context.push_str("| ");
                for width in &column_widths {
                    context.push_str(&" ".repeat(*width)).push_str(" |");
                }
            }

            if (row_index == 0 && self.rows.len() == 1) || row_index == 1 {
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

        if let Some(notes) = &self.notes {
            context
                .newline()
                .increase_depth()
                .push_prop_fn("notes", |context| notes.to_markdown(context))
                .decrease_depth();
        }

        if wrapped {
            if self.notes.is_none() {
                context.newline();
            }
            context.push_semis().newline();
        }

        context.exit_node().newline();
    }
}
