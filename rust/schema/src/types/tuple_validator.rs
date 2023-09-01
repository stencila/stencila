// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array of heterogeneous items.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TupleValidator {
    /// The type of this item
    pub r#type: MustBe!("TupleValidator"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// An array of validators specifying the constraints on each successive item in the array.
    pub items: Option<Vec<Validator>>,
}
impl TupleValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
