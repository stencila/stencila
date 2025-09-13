use crate::{
    other::organization,
    schema::{JsonSchema, refer},
};

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Person")
        .description("A person with basic information and affiliations")
        .required(["type"])
        .property("type", JsonSchema::string_const("Person"))
        .property(
            "givenNames",
            JsonSchema::array()
                .description("Given names (first names) or initials")
                .items(JsonSchema::string()),
        )
        .property(
            "familyNames",
            JsonSchema::array()
                .description("Family names (last names)")
                .items(JsonSchema::string()),
        )
        .property(
            "orcid",
            JsonSchema::string()
                .description("ORCID identifier")
                .pattern("^\\d{4}-\\d{4}-\\d{4}-\\d{3}[0-9X]$"),
        )
        .property(
            "email",
            JsonSchema::string()
                .description("Email address")
                .format("email"),
        )
        .property(
            "affiliations",
            JsonSchema::array()
                .description("Organizations the person is affiliated with")
                .items(refer(organization::simple())),
        )
        .disallow_additional_properties()
}
