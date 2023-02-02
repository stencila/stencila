//! Generated file, do not edit

use crate::prelude::*;

use super::boolean::Boolean;
use super::integer::Integer;
use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct ArrayValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("ArrayValidator"),

    /// The identifier for this item
    id: String,

    /// Whether items can have the value `Node::Null`
    items_nullable: Boolean,

    /// Another validator node specifying the constraints on all items in the array.
    items_validator: Option<Box<Validator>>,

    /// An array node is valid if at least one of its items is valid against the `contains` schema.
    contains: Option<Box<Validator>>,

    /// An array node is valid if its size is greater than, or equal to, this value.
    min_items: Option<Integer>,

    /// An array node is valid if its size is less than, or equal to, this value.
    max_items: Option<Integer>,

    /// A flag to indicate that each value in the array should be unique.
    unique_items: Option<Boolean>,
}
