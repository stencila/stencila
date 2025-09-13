use crate::{
    other::organization,
    schema::{JsonSchema, refer},
};

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Periodical")
        .description("A periodical publication such as a journal or magazine")
        .required(["type", "name"])
        .property("type", JsonSchema::string_const("Periodical"))
        .property(
            "name",
            JsonSchema::string().description("Name of the periodical"),
        )
        .property(
            "publisher",
            refer(organization::simple()).description("Publisher of the periodical"),
        )
        .disallow_additional_properties()
}
