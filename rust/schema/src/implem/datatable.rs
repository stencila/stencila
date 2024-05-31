use codec_info::lost_options;

use crate::{prelude::*, ArrayValidator, Datatable, Primitive};

impl Datatable {
    /// Get the number of rows in the `Datatable`
    pub fn rows(&self) -> usize {
        self.columns
            .iter()
            .fold(0usize, |rows, column| rows.max(column.values.len()))
    }
}

impl DomCodec for Datatable {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_id(&self.id)
            .enter_elem("table");

        // Create a <thead><tr> elem with a <th> row describing each column
        context.enter_elem("thead").enter_elem("tr");
        for column in &self.columns {
            context.enter_elem("th");
            column.to_dom(context);
            context.exit_elem();
        }
        context.exit_elem().exit_elem();

        // Get a name for the type of each column
        let data_types = self
            .columns
            .iter()
            .map(|column| match &column.validator {
                Some(ArrayValidator {
                    items_validator: Some(items_validator),
                    ..
                }) => Some(
                    items_validator
                        .to_string()
                        .trim_end_matches("Validator")
                        .to_lowercase(),
                ),
                _ => None,
            })
            .collect_vec();

        // Create a <tbody> elem with a <td> for each value in each column
        context.enter_elem("tbody");
        for row in 0..self.rows().min(context.max_datatable_rows) {
            context.enter_elem("tr");
            for (column_index, column) in self.columns.iter().enumerate() {
                context.enter_elem("td");

                if let Some(Some(data_type)) = data_types.get(column_index) {
                    context.push_attr("data-type", data_type);
                }

                if let Some(value) = column.values.get(row) {
                    let text = if let Primitive::String(value) = &value {
                        value.clone()
                    } else {
                        serde_json::to_string(value).unwrap_or_default()
                    };
                    context.push_text(&text);
                }
                context.exit_elem();
            }
            context.exit_elem();
        }
        context.exit_elem();

        context.exit_elem().exit_node();
    }
}

impl MarkdownCodec for Datatable {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        // Determine number of columns and rows
        let cols_num = self.columns.len();
        let rows_num = self
            .columns
            .iter()
            .fold(0usize, |max, column| max.max(column.values.len()));

        // Trim, replace inner newlines with <br> & ensure cell has no carriage
        // returns or pipes which will break table
        fn escape_cell(cell: String) -> (String, usize) {
            let cell = cell
                .trim()
                .replace('\n', "<br><br>")
                .replace('\r', " ")
                .replace('|', "\\|");
            let chars = 3.max(cell.chars().count());
            (cell, chars)
        }

        // Do a first iteration over cells to generate the Markdown
        // for each cell (including headers) and determine column widths
        let mut column_widths: Vec<usize> = Vec::new();
        let mut cells: Vec<Vec<String>> = vec![vec![String::new(); cols_num]; rows_num + 1];
        for (col_index, column) in self.columns.iter().enumerate() {
            // Set column header and initialize column width
            let (cell, width) = escape_cell(column.name.clone());
            cells[0][col_index] = cell;

            column_widths.push(width);

            // Fill in cells for this column
            for (row_index, value) in column.values.iter().enumerate() {
                let mut cell_context = MarkdownEncodeContext::default();
                value.to_markdown(&mut cell_context);

                let (cell, width) = escape_cell(cell_context.content);
                cells[row_index + 1][col_index] = cell;

                if let Some(column_width) = column_widths.get_mut(col_index) {
                    if width > *column_width {
                        *column_width = width
                    }
                }

                context.merge_losses(cell_context.losses);
            }
        }

        // Now iterate over rows and encode each to Markdown
        for (row_index, row) in cells.iter().enumerate() {
            // If there is only one row, header row should be empty
            if row_index == 0 && cells.len() == 1 {
                context.push_str("| ");
                for width in &column_widths {
                    context.push_str(&" ".repeat(*width)).push_str(" |");
                }
            }

            // Separator
            if (row_index == 0 && cells.len() == 1) || row_index == 1 {
                context.push_str("|");
                for width in &column_widths {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(*width))
                        .push_str(" |");
                }
                context.newline();
            }

            // Cell content (including header row)
            for (col_index, cell) in row.iter().enumerate() {
                if col_index == 0 {
                    context.push_str("|");
                }

                context.push_str(&format!(
                    " {cell:width$} |",
                    width = column_widths[col_index]
                ));
            }
            context.newline();
        }

        context.exit_node().newline();
    }
}
