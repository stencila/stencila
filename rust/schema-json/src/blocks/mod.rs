pub mod math;
pub mod paragraph;
pub mod table;

use crate::schema::{JsonSchema, refer};

pub fn simple() -> JsonSchema {
    JsonSchema::new()
        .title("Block")
        .description("Simple block content")
        .any_of(vec![
            refer(paragraph::simple()),
            refer(table::simple()),
            refer(math::tex()),
        ])
}
