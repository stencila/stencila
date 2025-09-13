use crate::blocks;
use crate::schema::{JsonSchema, refer};

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Article")
        .description("A simple article")
        .required(["type", "content"])
        .property("type", JsonSchema::string_enum(["Article"]))
        .property(
            "content",
            JsonSchema::array()
                .description("The content of the article")
                .items(refer(blocks::simple())),
        )
}
