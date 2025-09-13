pub mod math;
pub mod paragraph;

use crate::schema::{JsonSchema, refer};

pub fn simple() -> JsonSchema {
    JsonSchema::new()
        .title("Block")
        .description("Simple block content")
        .any_of(vec![refer(paragraph::simple()), refer(math::tex())])
}
