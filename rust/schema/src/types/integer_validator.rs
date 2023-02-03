//! Generated file, do not edit

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// A validator specifying the constraints on an integer node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct IntegerValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("IntegerValidator"),

    /// The identifier for this item
    #[key]
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
