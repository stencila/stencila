use std::any::type_name;

use schemars::JsonSchema;
use thiserror::Error;

use common::serde::Serialize;

/// An enumeration of custom errors returned by this library
///
/// Where possible functions should return one of these errors to provide greater
/// context to the user, in particular regarding actions that can be taken to
/// resolve the error.
#[derive(Error, Debug, JsonSchema, Serialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Error {
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
