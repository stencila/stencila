use crate::inlines;
use crate::schema::{JsonSchema, refer};

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Paragraph")
        .description("A paragraph")
        .required(["type", "content"])
        .property("type", JsonSchema::string_const("Paragraph"))
        .property(
            "content",
            JsonSchema::array()
                .description("The contents of the paragraph")
                .items(refer(inlines::simple())),
        )
        .disallow_additional_properties()
}
