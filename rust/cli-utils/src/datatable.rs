use comfy_table::{Cell, CellAlignment, Color, Row};

use common::{itertools::Itertools, serde_json};
use format::Format;
use schema::{Datatable, Primitive, Validator};

use crate::{table, Code, ToStdout};

impl ToStdout for Datatable {
    fn to_terminal(&self) -> impl std::fmt::Display {
        let mut table = table::new();

        let header = self
            .columns
            .iter()
            .map(|column| {
                let (color, alignment) = match column
                    .validator
                    .as_ref()
                    .and_then(|validator| validator.items_validator.as_deref())
                {
                    Some(Validator::BooleanValidator(..)) => (Color::Blue, CellAlignment::Right),
                    Some(Validator::IntegerValidator(..)) => (Color::Cyan, CellAlignment::Right),
                    Some(Validator::NumberValidator(..)) => (Color::Green, CellAlignment::Right),
                    Some(Validator::StringValidator(..)) => (Color::Yellow, CellAlignment::Left),
                    _ => (Color::Magenta, CellAlignment::Right),
                };

                Cell::new(column.name.as_str())
                    .fg(color)
                    .set_alignment(alignment)
            })
            .collect_vec();
        table.set_header(header);

        let mut rows = Vec::new();
        for row in 0..self.rows() {
            let cells = self.columns.iter().map(|column| {
                let alignment = match column
                    .validator
                    .as_ref()
                    .and_then(|validator| validator.items_validator.as_deref())
                {
                    Some(Validator::StringValidator(..)) => CellAlignment::Left,
                    _ => CellAlignment::Right,
                };

                column
                    .values
                    .get(row)
                    .map_or_else(
                        || Cell::new(""),
                        |value| match value {
                            Primitive::Null(cell) => {
                                Cell::new(cell.to_string()).fg(Color::DarkGrey)
                            }
                            Primitive::Boolean(cell) => Cell::new(cell.to_string()),
                            Primitive::Integer(cell) => Cell::new(cell.to_string()),
                            Primitive::UnsignedInteger(cell) => Cell::new(cell.to_string()),
                            Primitive::Number(cell) => Cell::new(cell.to_string()),
                            Primitive::String(cell) => Cell::new(cell.to_string()),
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
                    .set_alignment(alignment)
            });
            rows.push(Row::from(cells))
        }

        table.add_rows(rows);

        table
    }
}
