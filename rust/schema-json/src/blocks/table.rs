use crate::{
    blocks::paragraph,
    schema::{JsonSchema, refer},
};

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Table")
        .description("A table")
        .required(["type", "rows"])
        .property("type", JsonSchema::string_const("Table"))
        .property(
            "rows",
            JsonSchema::array()
                .description("Rows of cells in the table")
                .items(refer(table_row())),
        )
        .disallow_additional_properties()
}

pub fn table_row() -> JsonSchema {
    JsonSchema::object()
        .title("TableRow")
        .description("A row within a table")
        .required(["type", "cells"])
        .property("type", JsonSchema::string_const("TableRow"))
        .property(
            "cells",
            JsonSchema::array()
                .description("Cells in the row")
                .items(refer(table_cell())),
        )
        .property(
            "rowType",
            JsonSchema::string_enum(["HeaderRow", "BodyRow", "FooterRow"])
                .description("The type of row"),
        )
        .disallow_additional_properties()
}

pub fn table_cell() -> JsonSchema {
    JsonSchema::object()
        .title("TableCell")
        .description("A cell within a table.")
        .required(["type", "content"])
        .property("type", JsonSchema::string_const("TableCell"))
        .property(
            "content",
            JsonSchema::array()
                .description("Content of the cell. Usually a single paragraph. If the cell content is math then use a single MathInline object within the content of the paragraph. Otherwise use a single Text object.")
                .items(refer(paragraph::simple())),
        )
        .property(
            "columnSpan",
            JsonSchema::integer()
                .description("How many columns the cell extends")
                .minimum(0.0)
                .maximum(1000.0),
        )
        .property(
            "rowSpan",
            JsonSchema::integer()
                .description("How many rows the cell extends")
                .minimum(0.0)
                .maximum(100.0),
        )
        .property(
            "horizontalAlignment",
            JsonSchema::string_enum(["AlignLeft", "AlignCenter", "AlignRight", "AlignJustify"])
                .description("The horizontal alignment of the content of the table cell"),
        )
        .disallow_additional_properties()
}
