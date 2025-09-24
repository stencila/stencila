use serde::Deserialize;

use stencila_codec::stencila_schema::StringOrNumber;

/// Represents ordinary fields in CSL items
///
/// Ordinary variables in CSL-JSON can contain strings, numbers, or mixed content.
/// This enum provides flexible parsing while preserving the original data type.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables (Standard and Number Variables)
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#ordinary-variables
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OrdinaryField {
    Integer(i64),
    Float(f64),
    String(String),
}

impl From<OrdinaryField> for StringOrNumber {
    fn from(value: OrdinaryField) -> Self {
        match value {
            OrdinaryField::Integer(value) => StringOrNumber::Number(value as f64),
            OrdinaryField::Float(value) => StringOrNumber::Number(value),
            OrdinaryField::String(value) => StringOrNumber::String(value),
        }
    }
}
