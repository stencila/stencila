use crate::{Address, Slot};
use schemars::JsonSchema;
use serde::Serialize;
use std::any::type_name;
use thiserror::Error;

/// An enumeration of custom errors returned by this library
///
/// Where possible functions should return one of these errors to provide greater
/// context to the user, in particular regarding actions that can be taken to
/// resolve the error.
#[derive(Error, Debug, JsonSchema, Serialize)]
#[serde(tag = "type")]
#[schemars(deny_unknown_fields)]
pub enum Error {
    /// Used to indicate that two values are not the same (rather than
    /// return `false`, this error allows for convenient early return via `?`).
    #[error("Values are not the same (type and/or their value differ).")]
    NotSame,

    /// Used to indicate that two values are not equal (rather than
    /// return `false`, this error allows for convenient early return via `?`).
    #[error("Values are not equal (type is equal but their value differs")]
    NotEqual,

    /// An address resolved to a type that is not able to be pointed to
    /// (does not have a [`Pointer`] variant)
    #[error("Address `{address}` resolved to a type that can not be pointed to `{type_name}`")]
    UnpointableType { address: Address, type_name: String },

    /// The user attempted to use an an address (e.g. in a patch operation) that
    /// is invalid address for the type.
    ///
    /// Does not include the address because in places that this error is used
    /// that is usually modified e.g. `pop_front`.
    #[error("Invalid node address for type `{type_name}`: {details}")]
    InvalidAddress { type_name: String, details: String },

    /// The user attempted to apply a patch operation that is invalid for
    /// the type.
    #[error("Invalid patch operation `{op}` for type `{type_name}`")]
    InvalidPatchOperation { op: String, type_name: String },

    /// The user attempted to apply a patch operation with an invalid
    /// value for the type.
    #[error("Invalid patch value for type `{type_name}`")]
    InvalidPatchValue { type_name: String },

    /// The user attempted to use a slot with an invalid type for the
    /// type of the object (e.g. a `Slot::Name` on a `Vector`).
    #[error("Invalid slot type `{variant}` for type `{type_name}`")]
    InvalidSlotVariant { variant: String, type_name: String },

    /// The user attempted to use a slot with an invalid `Slot::Name`
    /// for the type (e.g a key for a `HashMap` that is not occupied).
    #[error("Invalid patch address name `{name}` for type `{type_name}`")]
    InvalidSlotName { name: String, type_name: String },

    /// The user attempted to use a slot with an invalid `Slot::Index`
    /// for the type (e.g an index that is greater than the size of a vector).
    #[error("Invalid patch address index `{index}` for type `{type_name}`")]
    InvalidSlotIndex { index: usize, type_name: String },
}

/// Create an `UnpointableType` error
pub fn unpointable_type<Type: ?Sized>(address: &Address) -> Error {
    Error::UnpointableType {
        address: address.clone(),
        type_name: type_name::<Type>().into(),
    }
}

/// Create an `InvalidAddress` error
pub fn invalid_address<Type: ?Sized>(details: &str) -> Error {
    Error::InvalidAddress {
        type_name: type_name::<Type>().into(),
        details: details.into(),
    }
}

/// Create an `InvalidPatchOperation` error
pub fn invalid_patch_operation<Type: ?Sized>(op: &str) -> Error {
    Error::InvalidPatchOperation {
        op: op.into(),
        type_name: type_name::<Type>().into(),
    }
}

/// Create an `InvalidPatchValue` error
pub fn invalid_patch_value<Type: ?Sized>() -> Error {
    Error::InvalidPatchValue {
        type_name: type_name::<Type>().into(),
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
