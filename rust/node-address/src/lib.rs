use std::{
    any::type_name,
    collections::{BTreeMap, VecDeque},
    fmt::{self, Debug},
    iter::FromIterator,
};

use schemars::JsonSchema;
use thiserror::Error;

use common::{
    derive_more::{Constructor, Deref, DerefMut},
    eyre::Result,
    inflector::cases::{camelcase::to_camel_case, snakecase::to_snake_case},
    serde::{self, Deserialize, Deserializer, Serialize},
    strum::AsRefStr,
};

/// A slot, used as part of an [`Address`], to locate a value within a `Node` tree.
///
/// Slots can be used to identify a part of a larger object.
///
/// The `Name` variant can be used to identify:
///
/// - the property name of a `struct`
/// - the key of a `HashMap<String, ...>`
///
/// The `Integer` variant can be used to identify:
///
/// - the index of a `Vec`
/// - the index of a Unicode character in a `String`
///
/// The `None` variant is used in places where a `Slot` is required
/// but none applies to the particular type or use case.
///
/// In contrast to JSON Patch, which uses a [JSON Pointer](http://tools.ietf.org/html/rfc6901)
/// to describe the location of additions and removals, slots offer improved performance and
/// type safety.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema, Serialize, Deserialize, AsRefStr,
)]
#[serde(untagged, crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Slot {
    Index(usize),
    Name(String),
}

impl fmt::Display for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Name(name) => write!(formatter, "{}", name),
            Slot::Index(index) => write!(formatter, "{}", index),
        }
    }
}

impl From<usize> for Slot {
    fn from(index: usize) -> Slot {
        Slot::Index(index)
    }
}

impl From<&str> for Slot {
    fn from(name: &str) -> Slot {
        Slot::Name(name.to_string())
    }
}

/// The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
///
/// Implemented as a double-ended queue. Given that addresses usually have less than
/// six slots it may be more performant to use a stack allocated `tinyvec` here instead.
///
/// Note: This could instead have be called a "Path", but that name was avoided because
/// of potential confusion with file system paths.
#[derive(
    Debug,
    Clone,
    Default,
    Constructor,
    Deref,
    DerefMut,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    JsonSchema,
)]
#[schemars(deny_unknown_fields)]
pub struct Address(VecDeque<Slot>);

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let me = self
            .iter()
            .map(|slot| slot.to_string())
            .collect::<Vec<String>>()
            .join(".");
        write!(formatter, "{}", me)
    }
}

impl Serialize for Address {
    /// Custom serialization to convert `Name` slots to camelCase
    ///
    /// This is done here for consistency with how Stencila Schema nodes are
    /// serialized using the Serde option `#[serde(rename_all = "camelCase")]`.
    ///
    /// It avoids incompatability with patches sent to the `web` module and the
    /// camelCase convention used for both DOM element attributed and JSON/JavaScript
    /// property names.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let camel_cased: Vec<Slot> = self
            .iter()
            .map(|slot| match slot {
                Slot::Index(..) => slot.clone(),
                Slot::Name(name) => Slot::Name(to_camel_case(name)),
            })
            .collect();
        camel_cased.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Address {
    /// Custom deserialization to convert `Name` slots to snake_case
    ///
    /// See notes for `impl Serialize for Address`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let slots: Vec<Slot> = Deserialize::deserialize(deserializer)?;
        let snake_cased = slots.into_iter().map(|slot| match slot {
            Slot::Index(..) => slot,
            Slot::Name(name) => Slot::Name(to_snake_case(&name)),
        });
        Ok(Address(VecDeque::from_iter(snake_cased)))
    }
}

impl From<usize> for Address {
    fn from(index: usize) -> Address {
        Address(VecDeque::from_iter([Slot::Index(index)]))
    }
}

impl From<&str> for Address {
    fn from(name: &str) -> Address {
        Address(VecDeque::from_iter([Slot::Name(name.to_string())]))
    }
}

impl Address {
    /// Create an empty address
    pub fn empty() -> Self {
        Self::default()
    }

    /// Prepend an address with another
    pub fn prepend(&mut self, other: &Self) {
        *self = other.concat(self)
    }

    /// Concatenate an address with another
    pub fn concat(&self, other: &Self) -> Self {
        let mut concat = self.clone();
        concat.append(&mut other.clone());
        concat
    }

    /// Add a name slot to an address
    pub fn add_name(&self, name: &str) -> Self {
        let mut added = self.clone();
        added.push_back(Slot::Name(name.to_string()));
        added
    }

    /// Add an index slot to an address
    pub fn add_index(&self, index: usize) -> Self {
        let mut added = self.clone();
        added.push_back(Slot::Index(index));
        added
    }
}

/// A map of node ids to their address
///
/// Used to enable faster access to a node based on it's id.
/// A `BTreeMap` is used instead of a `HashMap` for determinism in order
/// of entries.
pub type AddressMap = BTreeMap<String, Address>;

/// An enumeration of custom errors returned by this library
///
/// Where possible functions should return one of these errors to provide greater
/// context to the user, in particular regarding actions that can be taken to
/// resolve the error.
#[derive(Error, Debug, JsonSchema, Serialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Error {
    /// The user attempted to use an an address (e.g. in a patch operation) that
    /// is invalid address for the type.
    ///
    /// Does not include the address because in places that this error is used
    /// that is usually modified e.g. `pop_front`.
    #[error("Invalid address for node of type `{type_name}`: {details}")]
    InvalidAddress { type_name: String, details: String },

    /// The user attempted to use a slot with an invalid type for the
    /// node type (e.g. a `Slot::Name` on a `Vector`).
    #[error("Invalid slot type `{variant}` for node of type `{type_name}`")]
    InvalidSlotVariant { variant: String, type_name: String },

    /// The user attempted to use a slot with an invalid `Slot::Name`
    /// for the node type (e.g a key for a `HashMap` that is not occupied).
    #[error("Invalid address slot name `{name}` for node of type `{type_name}`")]
    InvalidSlotName { name: String, type_name: String },

    /// The user attempted to use a slot with an invalid `Slot::Index`
    /// for the node type (e.g an index that is greater than the size of a vector).
    #[error("Invalid address slot index `{index}` for node of type `{type_name}`")]
    InvalidSlotIndex { index: usize, type_name: String },
}

/// Create an `InvalidAddress` error
pub fn invalid_address<Type: ?Sized>(details: &str) -> Error {
    Error::InvalidAddress {
        type_name: type_name::<Type>().into(),
        details: details.into(),
    }
}

/// Create an `InvalidSlotVariant` error
pub fn invalid_slot_variant<Type: ?Sized>(slot: Slot) -> Error {
    Error::InvalidSlotVariant {
        variant: slot.as_ref().into(),
        type_name: type_name::<Type>().into(),
    }
}

/// Create an `InvalidSlotName` error
pub fn invalid_slot_name<Type: ?Sized>(name: &str) -> Error {
    Error::InvalidSlotName {
        name: name.into(),
        type_name: type_name::<Type>().into(),
    }
}

/// Create an `InvalidSlotIndex` error
pub fn invalid_slot_index<Type: ?Sized>(index: usize) -> Error {
    Error::InvalidSlotIndex {
        index,
        type_name: type_name::<Type>().into(),
    }
}
