use schemars::JsonSchema;

use flagset::{flags, FlagSet};

use common::{
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    strum::Display,
};
use node_address::Address;

use crate::value::{Value, Values};

/// The operations within a patch
///
/// Most of the operations in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of and `test`) are included here.
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
/// - they have a a `type` property, instead of an `op` property, and use title-case names,
///   rather than lowercase names
///
/// - they have an `address` property (an array of sting or integer "slots"), rather than a forward slash
///   separated string `path` (Note that for `String`s the integers in `address`refer to Unicode
///   graphemes, not bytes)
///
/// - the `Add` and `Replace` operations have a `html` property which may include the HTML
///   representation of the added or replacement node
///
/// - the `Move` and `Copy` operations have `from` and `to` properties, rather than `from` and `path`
///
#[skip_serializing_none]
#[derive(Clone, Debug, Display, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Operation {
    Add(Add),
    AddMany(AddMany),
    Remove(Remove),
    RemoveMany(RemoveMany),
    Replace(Replace),
    ReplaceMany(ReplaceMany),
    Move(Move),
    Copy(Copy),
    Transform(Transform),
}

flags! {
    /// A set of flags for controlling which operations are allowed in various algorithms
    pub enum OperationFlag: u64 {
        Add,
        AddMany,
        Remove,
        RemoveMany,
        Replace,
        ReplaceMany,
        Move,
        Copy,
        Transform,
    }
}

pub type OperationFlagSet = FlagSet<OperationFlag>;

impl OperationFlag {
    pub fn all() -> FlagSet<OperationFlag> {
        OperationFlag::Add
            | OperationFlag::AddMany
            | OperationFlag::Remove
            | OperationFlag::RemoveMany
            | OperationFlag::Replace
            | OperationFlag::ReplaceMany
            | OperationFlag::Move
            | OperationFlag::Copy
            | OperationFlag::Transform
    }
}

impl Operation {
    /// Create an `Add` operation
    pub fn add(address: Address, value: Value) -> Self {
        Operation::Add(Add {
            address,
            value,
            html: None,
        })
    }

    /// Create an `AddMany` operation
    pub fn add_many(address: Address, values: Values) -> Self {
        Operation::AddMany(AddMany {
            address,
            values,
            html: None,
        })
    }

    /// Create a `Remove` operation
    pub fn remove(address: Address) -> Self {
        Operation::Remove(Remove { address })
    }

    /// Create a `RemoveMany` operation
    pub fn remove_many(address: Address, items: usize) -> Self {
        Operation::RemoveMany(RemoveMany { address, items })
    }

    /// Create a `Replace` operation
    pub fn replace(address: Address, value: Value) -> Self {
        Operation::Replace(Replace {
            address,
            value,
            html: None,
        })
    }

    /// Create a `ReplaceMany` operation
    pub fn replace_many(address: Address, items: usize, values: Values) -> Self {
        Operation::ReplaceMany(ReplaceMany {
            address,
            items,
            values,
            html: None,
        })
    }

    /// Create a move operation
    pub fn mov(from: Address, to: Address) -> Self {
        Operation::Move(Move { from, to })
    }

    /// Create a copy operation
    pub fn copy(from: Address, to: Address) -> Self {
        Operation::Copy(Copy { from, to })
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
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Add {
    /// The address to which to add the value
    pub address: Address,

    /// The value to add
    #[schemars(skip)]
    pub value: Value,

    /// The HTML encoding of the added value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Add more than one value
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct AddMany {
    /// The address to which to add the values
    pub address: Address,

    /// The values to add
    #[schemars(skip)]
    pub values: Values,

    /// The HTML encoding of the added values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Remove a value
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Remove {
    /// The address from which to remove the value
    pub address: Address,
}

/// Remove more than one value
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct RemoveMany {
    /// The address from which to remove the values
    pub address: Address,

    /// The number of items to remove
    pub items: usize,
}

/// Replace a value
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Replace {
    /// The address which should be replaced
    pub address: Address,

    /// The replacement value
    #[schemars(skip)]
    pub value: Value,

    /// The HTML encoding of the replacement value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Replace more than one value
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ReplaceMany {
    /// The address which should be replaced
    pub address: Address,

    /// The number of items to be replaced (can be more or less than the length of values)
    pub items: usize,

    /// The replacement values
    #[schemars(skip)]
    pub values: Values,

    /// The HTML encoding of the replacement value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Move a value from one address to another
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Move {
    /// The address from which to remove the value
    pub from: Address,

    /// The address to which to add the items
    pub to: Address,
}

/// Copy a value from one address to another
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Copy {
    /// The address from which to copy the value
    pub from: Address,

    /// The address to which to copy the items
    pub to: Address,
}

/// Transform a value from one type to another
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Transform {
    /// The address of the `Node` to transform
    pub address: Address,

    /// The type of `Node` to transform from
    pub from: String,

    /// The type of `Node` to transform to
    pub to: String,
}
