use crate::schema::JsonSchema;

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Organization")
        .description("An organization such as a university, company, or institution")
        .required(["type", "name"])
        .property("type", JsonSchema::string_const("Organization"))
        .property(
            "name",
            JsonSchema::string().description("Name of the organization"),
        )
        .property(
            "ror",
            JsonSchema::string()
                .description("Research Organization Registry ID")
                .pattern("^0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$"),
        )
        .property(
            "address",
            JsonSchema::string().description("Postal address"),
        )
        .disallow_additional_properties()
}
