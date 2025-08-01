use codec::common::serde::Deserialize;

/// Represents ordinary fields in CSL items
///
/// Ordinary variables in CSL-JSON can contain strings, numbers, or mixed content.
/// This enum provides flexible parsing while preserving the original data type.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables (Standard and Number Variables)
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#ordinary-variables
#[derive(Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
pub enum OrdinaryField {
    Integer(i64),
    Float(f64),
    String(String),
}
