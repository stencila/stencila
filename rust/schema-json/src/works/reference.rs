use crate::{
    other::{organization, person},
    schema::{JsonSchema, refer},
};

pub fn reference() -> JsonSchema {
    JsonSchema::object()
        .title("Reference")
        .description("A reference to a creative work, including books, articles, chapters, etc.")
        .required(["type", "workType", "authors", "date", "title"])
        .property("type", JsonSchema::string_const("Reference"))
        .property(
            "workType",
            JsonSchema::string_enum([
                "Article",
                "Book", 
                "Chapter",
                "Dataset",
                "Report",
                "Thesis",
                "WebPage"
            ])
            .description("The type of work being referenced"),
        )
        .property(
            "authors",
            JsonSchema::array()
                .description("Authors of the referenced work")
                .items(JsonSchema::new().any_of(vec![
                    refer(person::simple()),
                    refer(organization::simple()),
                ])),
        )
        .property(
            "date",
            JsonSchema::string()
                .description("Year or date of publication")
                .format("date"),
        )
        .property(
            "title",
            JsonSchema::string().description("Title of the work"),
        )
        .property(
            "isPartOf",
            JsonSchema::new().any_of(vec![
                refer(journal()),
                refer(book())
            ])
            .description("A reference to the work that this work is part of. For example, the journal if this is an article, or the book if this is a chapter."),
        )
        .property(
            "publisher",
            JsonSchema::new()
                .any_of(vec![
                    JsonSchema::string(),
                    refer(person::simple()),
                    refer(organization::simple()),
                ])
                .description("Publisher of the referenced work"),
        )
        .property(
            "pageStart",
            JsonSchema::new()
                .any_of(vec![JsonSchema::integer(), JsonSchema::string()])
                .description("Starting page number"),
        )
        .property(
            "pageEnd",
            JsonSchema::new()
                .any_of(vec![JsonSchema::integer(), JsonSchema::string()])
                .description("Ending page number"),
        )
        .property(
            "version",
            JsonSchema::new()
                .any_of(vec![JsonSchema::string(), JsonSchema::number()])
                .description("Version/edition of the referenced work"),
        )
        .property(
            "doi",
            JsonSchema::string()
                .description("Digital Object Identifier")
                .pattern("^10\\.\\d+/.+$"),
        )
        .property(
            "url",
            JsonSchema::string()
                .description("URL of the referenced work")
                .format("uri"),
        )
        .disallow_additional_properties()
}

pub fn book() -> JsonSchema {
    JsonSchema::object()
        .title("Book")
        .description("A book")
        .required(["type", "title"])
        .property("type", JsonSchema::string_const("Reference"))
        .property("workType", JsonSchema::string_const("Book"))
        .property(
            "title",
            JsonSchema::string().description("Title of the book"),
        )
        .property(
            "authors",
            JsonSchema::array()
                .description("Authors of the book")
                .items(JsonSchema::new().any_of(vec![
                    JsonSchema::string(),
                    refer(person::simple()),
                    refer(organization::simple()),
                ])),
        )
        .property(
            "editors",
            JsonSchema::array()
                .description("Editors of the book")
                .items(refer(person::simple())),
        )
        .property(
            "publisher",
            JsonSchema::new()
                .any_of(vec![
                    JsonSchema::string(),
                    refer(person::simple()),
                    refer(organization::simple()),
                ])
                .description("Publisher of the book"),
        )
        .disallow_additional_properties()
}

pub fn journal() -> JsonSchema {
    JsonSchema::object()
        .title("Journal")
        .description("A journal")
        .required(["type", "title"])
        .property("type", JsonSchema::string_const("Reference"))
        .property("workType", JsonSchema::string_const("Periodical"))
        .property(
            "title",
            JsonSchema::string().description("Title of the journal"),
        )
        .property(
            "volumeNumber",
            JsonSchema::new()
                .any_of(vec![JsonSchema::integer(), JsonSchema::string()])
                .description("Volume number"),
        )
        .property(
            "issueNumber",
            JsonSchema::new()
                .any_of(vec![JsonSchema::integer(), JsonSchema::string()])
                .description("Issue number"),
        )
        .disallow_additional_properties()
}
