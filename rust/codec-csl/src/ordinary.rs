use codec::common::serde::{Deserialize, Serialize};

/// Represents ordinary fields in CSL items
///
/// Ordinary variables in CSL-JSON can contain strings, numbers, or mixed content.
/// This enum provides flexible parsing while preserving the original data type.
///
/// See https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#ordinary-variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
pub enum OrdinaryField {
    Float(f64),
    Integer(i64),
    String(String),
}
