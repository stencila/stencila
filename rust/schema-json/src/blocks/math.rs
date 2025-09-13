use crate::schema::JsonSchema;

pub fn tex() -> JsonSchema {
    JsonSchema::object()
        .title("MathBlock")
        .description("Display math equation")
        .required(["type", "code"])
        .property("type", JsonSchema::string_enum(["MathBlock"]))
        .property("code", JsonSchema::string().description("TeX/LaTeX code"))
        .property(
            "label",
            JsonSchema::string().description("A label for the equation(e.g. '1')"),
        )
}
