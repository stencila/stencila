use crate::blocks;
use crate::other;
use crate::schema::{AdditionalProperties, JsonSchema, refer};
use crate::works;

pub fn simple() -> JsonSchema {
    JsonSchema::object()
        .title("Article")
        .description("A simple article")
        .required(["type", "content"])
        .property("type", JsonSchema::string_const("Article"))
        .property(
            "content",
            JsonSchema::array()
                .description("The content of the article")
                .items(refer(blocks::simple())),
        )
}

pub fn metadata() -> JsonSchema {
    JsonSchema::object()
        .title("Article")
        .description("Article with metadata fields for publication information")
        .additional_properties(AdditionalProperties::Bool(false))
        .required(["type"])
        .property("type", JsonSchema::string_const("Article"))
        .property(
            "doi",
            JsonSchema::string()
                .description("Digital Object Identifier")
                .pattern("^10\\.\\d+/.+$"),
        )
        .property(
            "title",
            JsonSchema::string().description("Title of the article"),
        )
        .property(
            "authors",
            JsonSchema::array()
                .description("Authors of the article")
                .items(JsonSchema::new().any_of(vec![
                    refer(other::person::simple()),
                    refer(other::organization::simple()),
                    JsonSchema::string(),
                ])),
        )
        .property(
            "dateReceived",
            JsonSchema::string()
                .description("Date when article was received")
                .format("date"),
        )
        .property(
            "dateAccepted",
            JsonSchema::string()
                .description("Date when article was accepted")
                .format("date"),
        )
        .property(
            "datePublished",
            JsonSchema::string()
                .description("Date when article was published")
                .format("date"),
        )
        .property(
            "keywords",
            JsonSchema::array()
                .description("Keywords describing the article")
                .items(JsonSchema::string()),
        )
        .property(
            "isPartOf",
            refer(works::periodical::simple()).description("Periodical this article is part of"),
        )
        .property(
            "licenses",
            JsonSchema::array()
                .description("License information")
                .items(JsonSchema::string()),
        )
}
