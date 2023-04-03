//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A point in time recurring on multiple days
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Time {
    /// The type of this item
    pub r#type: MustBe!("Time"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
    pub value: String,
}

impl Time {
    #[rustfmt::skip]
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
