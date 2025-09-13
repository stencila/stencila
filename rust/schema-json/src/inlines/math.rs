use crate::schema::JsonSchema;

pub fn tex() -> JsonSchema {
    JsonSchema::object()
        .title("MathInline")
        .description("Inline TeX/LaTeX math")
        .required(["type", "code"])
        .property("type", JsonSchema::string_const("MathInline"))
        .property("code", JsonSchema::string().description("TeX/LaTeX code"))
        .disallow_additional_properties()
}
