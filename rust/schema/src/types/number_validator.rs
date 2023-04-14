// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// A validator specifying the constraints on a numeric node.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct NumberValidator {
    /// The type of this item
    pub r#type: MustBe!("NumberValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The inclusive lower limit for a numeric node.
    pub minimum: Option<Number>,

    /// The exclusive lower limit for a numeric node.
    pub exclusive_minimum: Option<Number>,

    /// The inclusive upper limit for a numeric node.
    pub maximum: Option<Number>,

    /// The exclusive upper limit for a numeric node.
    pub exclusive_maximum: Option<Number>,

    /// A number that a numeric node must be a multiple of.
    pub multiple_of: Option<Number>,
}

impl NumberValidator {
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
