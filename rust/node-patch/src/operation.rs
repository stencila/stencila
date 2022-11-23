use std::fmt::Debug;

use schemars::JsonSchema;

use common::{
    defaults::Defaults,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    strum::Display,
};
use node_address::Address;

use crate::value::Value;

/// The operations within a patch
///
/// These are the same operations as described in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of and `test`).
///
/// In addition, there is a `Transform` operation which can be used describe the transformation
/// of a node to another type that has a similar structure. Examples includes:
///
/// - a `String` to an `Emphasis`
/// - a `Paragraph` to a `QuoteBlock`
/// - a `CodeChunk` to a `CodeBlock`
///
/// Note that `Replace`, `Move` and `Copy` could be represented by combinations of `Remove` and `Add`.
/// They are included as a means of providing more semantically meaningful patches, and more
/// space efficient serializations (e.g. it is not necessary to represent the value being moved or copied).
///
/// The structure of these operations differs from JSON Patch operations:
///
/// - they have an `address` property (an array of sting or integer "slots"), rather than a
///   forward slash separated string `path`
///
/// - the `Remove`, `Replace`, `Move` and `Copy` operations have an `items` property which
///   allows several items in a string or an array to be operated on by a single operation
///
/// The `length` field on `Add` and `Replace` is not necessary for applying operations, but
/// is useful for generating them and for determining if there are conflicts between two patches
/// without having to downcast the `value`.
///
/// Note that for `String`s the integers in `address`, `items` and `length` all refer to Unicode
/// graphemes, not bytes.
#[skip_serializing_none]
#[derive(Clone, Debug, Display, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Operation {
    Add(Add),
    Remove(Remove),
    Replace(Replace),
    Move(Move),
    Copy(Copy),
    Transform(Transform),
}

impl Operation {
    /// Create an add operation
    pub fn add(address: Address, value: Value, length: usize) -> Self {
        Operation::Add(Add {
            address,
            value,
            length,
            html: None,
        })
    }

    /// Create an add operation to add one item
    pub fn add_one(address: Address, value: Value) -> Self {
        Operation::Add(Add {
            address,
            value,
            length: 1,
            html: None,
        })
    }

    /// Create a remove operation
    pub fn remove(address: Address, items: usize) -> Self {
        Operation::Remove(Remove { address, items })
    }

    /// Create remove operation to remove one item
    pub fn remove_one(address: Address) -> Self {
        Operation::Remove(Remove { address, items: 1 })
    }

    /// Create a replace operation
    pub fn replace(address: Address, items: usize, value: Value, length: usize) -> Self {
        Operation::Replace(Replace {
            address,
            items,
            value,
            length,
            html: None,
        })
    }

    /// Create a replace operation to replace one item
    pub fn replace_one(address: Address, value: Value) -> Self {
        Operation::Replace(Replace {
            address,
            value,
            items: 1,
            length: 1,
            html: None,
        })
    }

    /// Create a move operation
    pub fn mov(from: Address, items: usize, to: Address) -> Self {
        Operation::Move(Move { from, items, to })
    }

    /// Create a copy operation
    pub fn copy(from: Address, items: usize, to: Address) -> Self {
        Operation::Copy(Copy { from, items, to })
    }

    /// Create a transform operation
    pub fn transform(address: Address, from: &str, to: &str) -> Self {
        Operation::Transform(Transform {
            address,
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

/// Add a value
#[skip_serializing_none]
#[derive(Clone, Debug, Defaults, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", default, crate = "common::serde")]
pub struct Add {
    /// The address to which to add the value
    pub address: Address,

    /// The value to add
    #[schemars(skip)]
    pub value: Value,

    /// The number of items added
    #[def = "1"]
    pub length: usize,

    /// The HTML encoding of `value`
    pub html: Option<String>,
}

/// Remove one or more values
#[skip_serializing_none]
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Remove {
    /// The address from which to remove the value(s)
    pub address: Address,

    /// The number of items to remove
    pub items: usize,
}

/// Replace one or more values
#[skip_serializing_none]
#[derive(Clone, Debug, Defaults, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", default, crate = "common::serde")]
pub struct Replace {
    /// The address which should be replaced
    pub address: Address,

    /// The number of items to replace
    pub items: usize,

    /// The replacement value
    #[schemars(skip)]
    pub value: Value,

    /// The number of items added
    #[def = "1"]
    pub length: usize,

    /// The HTML encoding of `value`
    pub html: Option<String>,
}

/// Move a value from one address to another
#[skip_serializing_none]
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Move {
    /// The address from which to remove the value
    pub from: Address,

    /// The number of items to move
    pub items: usize,

    /// The address to which to add the items
    pub to: Address,
}

/// Copy a value from one address to another
#[skip_serializing_none]
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Copy {
    /// The address from which to copy the value
    pub from: Address,

    /// The number of items to copy
    pub items: usize,

    /// The address to which to copy the items
    pub to: Address,
}

/// Transform a value from one type to another
#[skip_serializing_none]
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Transform {
    /// The address of the `Node` to transform
    pub address: Address,

    /// The type of `Node` to transform from
    pub from: String,

    /// The type of `Node` to transform to
    pub to: String,
}
