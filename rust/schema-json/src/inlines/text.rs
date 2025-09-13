use crate::schema::JsonSchema;

pub fn plain() -> JsonSchema {
    JsonSchema::object()
        .title("Text")
        .description("Plain text content")
        .required(["type", "value"])
        .property("type", JsonSchema::string_const("Text"))
        .property(
            "value",
            JsonSchema::string().description("The plain text content"),
        )
        .disallow_additional_properties()
}
