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

    /// The user attempted to apply a patch operation that is invalid for
    /// the type.
    #[error("Invalid patch operation `{op}` for type `{type_name}`")]
    InvalidPatchOperation { op: String, type_name: String },

    /// The user attempted to apply a patch operation with an invalid
    /// value for the type.
    #[error("Invalid patch value for type `{type_name}`")]
    InvalidPatchValue { type_name: String },
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
