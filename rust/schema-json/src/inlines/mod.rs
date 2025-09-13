pub mod math;
pub mod text;

use crate::schema::{refer, JsonSchema};

pub fn simple() -> JsonSchema {
    JsonSchema::new()
        .title("Inline")
        .description("Union type for valid inline content")
        .any_of(vec![refer(text::plain()), refer(math::tex())])
}
