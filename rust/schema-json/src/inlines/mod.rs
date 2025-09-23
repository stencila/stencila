pub mod math;
pub mod text;

use crate::schema::{JsonSchema, refer};

pub fn simple() -> JsonSchema {
    JsonSchema::new()
        .title("Inline")
        .description("Simple inline content")
        .any_of(vec![refer(text::plain()), refer(math::tex())])
}
