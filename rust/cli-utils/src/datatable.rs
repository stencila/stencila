use comfy_table::{Cell, CellAlignment, Color, Row};

use common::{itertools::Itertools, serde_json};
use format::Format;
use schema::{Datatable, Primitive};

use crate::{table, Code, ToStdout};

impl ToStdout for Datatable {
    fn to_terminal(&self) -> impl std::fmt::Display {
        let mut table = table::new();

        let header = self
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect_vec();
        table.set_header(header);

        let mut rows = Vec::new();
        for row in 0..self.rows() {
            let cells = self.columns.iter().map(|column| {
                column.values.get(row).map_or_else(
                    || Cell::new(""),
                    |value| match value {
                        Primitive::Null(cell) => Cell::new(cell.to_string())
                            .fg(Color::DarkGrey)
                            .set_alignment(CellAlignment::Right),
                        Primitive::Boolean(cell) => Cell::new(cell.to_string())
                            .fg(Color::Magenta)
                            .set_alignment(CellAlignment::Right),
                        Primitive::Integer(cell) => Cell::new(cell.to_string())
                            .fg(Color::Green)
                            .set_alignment(CellAlignment::Right),
                        Primitive::UnsignedInteger(cell) => Cell::new(cell.to_string())
                            .fg(Color::Cyan)
                            .set_alignment(CellAlignment::Right),
                        Primitive::Number(cell) => Cell::new(cell.to_string())
                            .fg(Color::Blue)
                            .set_alignment(CellAlignment::Right),
                        Primitive::String(cell) => Cell::new(cell.to_string())
                            .fg(Color::DarkYellow)
                            .set_alignment(CellAlignment::Left),
                        Primitive::Array(cell) => Cell::new(
                            Code::new(
                                Format::Json,
                                &serde_json::to_string(cell).unwrap_or_default(),
                            )
                            .to_terminal(),
                        ),
                        Primitive::Object(cell) => Cell::new(
                            Code::new(
                                Format::Json,
                                &serde_json::to_string(cell).unwrap_or_default(),
                            )
                            .to_terminal(),
                        ),
                    },
                )
            });
            rows.push(Row::from(cells))
        }

        table.add_rows(rows);

        table
    }
}
