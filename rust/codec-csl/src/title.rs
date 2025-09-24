use serde::Deserialize;
use stencila_codec::stencila_schema::{Inline, shortcuts::t};

/// Represents a title in CSL items
#[derive(Deserialize)]
#[serde(untagged)]
pub enum TitleField {
    String(String),
    Array(Vec<String>),
}

impl From<TitleField> for String {
    fn from(value: TitleField) -> Self {
        match value {
            TitleField::String(value) => value.to_string(),
            TitleField::Array(values) => values.join(""),
        }
    }
}

impl From<TitleField> for Vec<Inline> {
    fn from(value: TitleField) -> Self {
        match value {
            TitleField::String(value) => vec![t(value)],
            TitleField::Array(values) => values.into_iter().map(t).collect(),
        }
    }
}
