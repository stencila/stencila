//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array of heterogeneous items.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TupleValidator {
    /// The type of this item
    pub r#type: MustBe!("TupleValidator"),

    /// The identifier for this item
    pub id: Option<String>,

    /// An array of validators specifying the constraints on each successive item in the array.
    pub items: Option<Vec<Validator>>,
}

impl TupleValidator {
    pub fn new() -> Self {
        Self{
            
            ..Default::default()
        }
    }
}

