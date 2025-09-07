use indexmap::IndexMap;
use stencila_codec_info::lost_options;

use crate::{
    ArrayValidator, Datatable, DatatableColumn, Object, Primitive, Table, TableRowType, prelude::*,
};

use stencila_codec_text_trait::to_text;

impl Datatable {
    /// Get the number of rows in the [`Datatable`]
    pub fn rows(&self) -> usize {
        self.columns
            .iter()
            .fold(0usize, |rows, column| rows.max(column.values.len()))
    }

    /// Create a datatable from raw column data with type inference
    ///
    /// Takes column names and corresponding raw text values, performs type
    /// inference on each column, and creates a properly typed datatable with
    /// validators.
    ///
    /// - Input: Vec of (column_name, column_values) tuples
    /// - Output: Datatable with typed columns and appropriate ArrayValidators
    ///
    /// Each column's string values are analyzed to determine the most
    /// appropriate primitive type (Boolean, Integer, Number, or String) and
    /// converted accordingly.
    pub fn from_string_columns(columns: Vec<(String, Vec<String>)>) -> Self {
        let columns = columns
            .into_iter()
            .map(|(name, values)| DatatableColumn::from_strings(name, values))
            .collect();

        Datatable::new(columns)
    }

    /// Create a datatable from JSON column data with type inference
    ///
    /// Takes column names and corresponding JSON values, performs type
    /// inference on each column, and creates a properly typed datatable with
    /// validators.
    ///
    /// - Input: Vec of (column_name, json_values) tuples  
    /// - Output: Datatable with typed columns and appropriate ArrayValidators
    ///
    /// Each column's JSON values are analyzed to determine the most appropriate
    /// primitive type while preserving JSON semantics (numbers stay numbers,
    /// etc.).
    pub fn from_json_columns(columns: Vec<(String, Vec<serde_json::Value>)>) -> Self {
        let columns = columns
            .into_iter()
            .map(|(name, values)| DatatableColumn::from_json_values(name, values))
            .collect();

        Datatable::new(columns)
    }

    /// Try to create a datatable from a table with uniformity checks
    ///
    /// Converts a table to a datatable only if it meets strict criteria for
    /// uniformity and simplicity. This is the same validation logic used in
    /// node-structuring for selective table-to-datatable conversion.
    ///
    /// - Input: Table that may or may not meet conversion criteria
    /// - Output: Option<Datatable> with conversion or None if invalid
    ///
    /// # Validation Criteria
    /// - Table must have uniform shape (all rows same cell count)
    /// - No cells can have rowspan or colspan attributes
    /// - All cells must contain simple text content (single paragraph only)
    /// - At least one row must exist for headers
    ///
    /// # Success Case
    /// Returns Some(datatable) with proper type inference and validators.
    ///
    /// # Failure Cases
    /// Returns None for:
    /// - Empty tables
    /// - Inconsistent row lengths
    /// - Cells with span attributes
    /// - Cells with complex content (non-paragraph or multiple blocks)
    pub fn from_table_if_uniform(table: &Table) -> Option<Self> {
        // Return early if table has no rows
        if table.rows.is_empty() {
            return None;
        }

        // Check for uniform shape - all rows must have the same number of cells
        let expected_cell_count = table.rows[0].cells.len();
        if expected_cell_count == 0 {
            return None;
        }

        for row in table.rows.iter() {
            // Check cell count consistency
            if row.cells.len() != expected_cell_count {
                return None;
            }

            // Check for column/row spans
            for cell in row.cells.iter() {
                if cell.options.column_span.is_some() || cell.options.row_span.is_some() {
                    return None;
                }
            }
        }

        // Validate that all cells can be converted to simple text
        for row in table.rows.iter() {
            for cell in row.cells.iter() {
                // Cell must have exactly one block that is a paragraph
                if cell.content.len() != 1 {
                    return None;
                }
                if !matches!(cell.content[0], crate::Block::Paragraph(..)) {
                    return None;
                }
            }
        }

        // All validation passed - use the coercive From implementation
        Some(Datatable::from(table))
    }
}

/// Create a vector of Stencila [`Object`]s from a [`Datatable`]
///
/// Converts a column-oriented datatable to a row-oriented vector of objects.
/// Each object in the returned vector represents one row, with keys being
/// column names and values being the primitive values from that row.
///
/// - Input: Datatable with N columns and M rows
/// - Output: Vec<Object> with M elements, each containing N key-value pairs
///
/// # Example
///
/// A datatable with columns "name" and "age":
///
/// ```markdown
/// | name  | age |
/// |-------|-----|
/// | Alice | 30  |
/// | Bob   | 25  |
/// ```
///
/// Becomes:
///
/// ```json
/// [
///     {"name": "Alice", "age": 30},
///     {"name": "Bob", "age": 25}
/// ]
/// ```
impl From<Datatable> for Vec<Object> {
    fn from(datatable: Datatable) -> Self {
        (0..datatable.rows())
            .map(|row| {
                let pairs = datatable
                    .columns
                    .iter()
                    .map(|column| (column.name.clone(), column.values[row].clone()))
                    .collect();
                Object(pairs)
            })
            .collect()
    }
}

/// Create a datatable from a vector of JSON objects
///
/// Converts a row-oriented vector of JSON objects to a column-oriented
/// datatable. Each JSON object represents one row, with keys becoming column
/// names and values being analyzed for type inference and converted to
/// primitives.
///
/// - Input: Vec of M JSON objects, each with potentially different keys
/// - Output: Datatable with N columns (union of all keys) and M rows
///
/// # Example
///
/// ```json
/// [
///     {"name": "Alice", "age": 30},
///     {"name": "Bob", "city": "NYC"}
/// ]
/// ```
///
/// Becomes a datatable with columns "name", "age", "city" where missing values
/// are filled with nulls:
///
/// ```markdown
/// | name  | age  | city |
/// |-------|------|------|
/// | Alice | 30   | null |
/// | Bob   | null | NYC  |
/// ```
impl From<Vec<serde_json::Map<String, serde_json::Value>>> for Datatable {
    fn from(rows: Vec<serde_json::Map<String, serde_json::Value>>) -> Self {
        if rows.is_empty() {
            return Datatable::new(Vec::new());
        }

        // First pass: collect all unique column names to ensure consistent ordering
        let mut column_names = IndexMap::new();
        for row in &rows {
            for name in row.keys() {
                column_names.entry(name.clone()).or_insert(());
            }
        }
        let column_names: Vec<String> = column_names.into_keys().collect();

        // Second pass: collect JSON values for each column
        let column_data: Vec<(String, Vec<serde_json::Value>)> = column_names
            .into_iter()
            .map(|column_name| {
                let values: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|row| {
                        row.get(&column_name)
                            .cloned()
                            .unwrap_or(serde_json::Value::Null)
                    })
                    .collect();

                (column_name, values)
            })
            .collect();

        Datatable::from_json_columns(column_data)
    }
}

/// Create a datatable from a table
///
/// Converts a row-oriented table to a column-oriented datatable. Unlike the
/// selective table-to-datatable conversion in node-structuring, this
/// implementation coerces any table structure and handles irregularities:
///
/// - Input: Table with potentially irregular structure
/// - Output: Datatable with consistent columns and type inference
///
/// # Behavior
/// - Uses first row as headers, or generates "Column N" names if no headers
/// - Handles rows with different cell counts by padding with empty strings
/// - Extracts text content from complex cells (falls back to empty string)
/// - Applies type inference to determine appropriate primitive types
impl From<&Table> for Datatable {
    fn from(table: &Table) -> Self {
        if table.rows.is_empty() {
            return Datatable::new(Vec::new());
        }

        // Find header row - either explicitly marked or use first row
        let header_row_index = table
            .rows
            .iter()
            .position(|row| matches!(row.row_type, Some(TableRowType::HeaderRow)))
            .unwrap_or(0);

        let header_row = &table.rows[header_row_index];
        let num_columns = table
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);

        // Extract column names from header row or generate defaults
        let column_names: Vec<String> = (0..num_columns)
            .map(|i| {
                header_row
                    .cells
                    .get(i)
                    .map(|cell| {
                        let name = to_text(&cell.content).trim().to_string();
                        if name.is_empty() {
                            format!("Column {}", i + 1)
                        } else {
                            name
                        }
                    })
                    .unwrap_or_else(|| format!("Column {}", i + 1))
            })
            .collect();

        // Extract data from all rows (including header row for now, we'll skip it in collection)
        let mut column_data: Vec<Vec<String>> = vec![Vec::new(); num_columns];

        for (row_index, row) in table.rows.iter().enumerate() {
            // Skip the header row for data collection
            if row_index == header_row_index {
                continue;
            }

            for (column_index, column) in column_data.iter_mut().enumerate().take(num_columns) {
                let cell_text = row
                    .cells
                    .get(column_index)
                    .map(|cell| to_text(&cell.content).trim().to_string())
                    .unwrap_or_else(String::new);

                column.push(cell_text);
            }
        }

        // Create column tuples and use shared functionality
        let columns: Vec<(String, Vec<String>)> =
            column_names.into_iter().zip(column_data).collect();

        Datatable::from_string_columns(columns)
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

                if let Some(column_width) = column_widths.get_mut(col_index)
                    && width > *column_width
                {
                    *column_width = width
                }

                context.merge_losses(cell_context.losses);
            }
        }

        // Iterate over rows and encode each to Markdown
        for (row_index, row) in cells.iter().enumerate() {
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

            // Separator after first row
            if row_index == 0 {
                context.push_str("|");
                for width in &column_widths {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(*width))
                        .push_str(" |");
                }
                context.newline();
            }
        }

        context.exit_node().newline();
    }
}
