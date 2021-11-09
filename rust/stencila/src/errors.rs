use crate::{
    patches::{Address, Slot},
    utils::schemas,
};
use eyre::Result;
use once_cell::sync::Lazy;
use schemars::{gen::SchemaSettings, JsonSchema};
use serde::Serialize;
use std::{
    any::type_name,
    collections::HashMap,
    sync::{atomic::AtomicBool, Mutex},
};
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
    /// An identifier was supplied that does not match the pattern for the
    /// expected family of identifiers.
    #[error("Invalid universal identifier for family `{family}`: {id}")]
    InvalidUUID { family: String, id: String },

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
    UnpointableType {
        #[schemars(schema_with = "address_schema")]
        address: Address,
        type_name: String,
    },

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

    /// The user attempted to open a document with an unknown format
    #[error("Unknown format `{format}`")]
    UnknownFormat { format: String },

    /// A kernel was asked to execute code in an incompatible programming language
    #[error("Incompatible programming language `{language}` for kernel type `{kernel_type}`")]
    IncompatibleLanguage {
        language: String,
        kernel_type: String,
    },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. However, none of the registered
    /// plugins implement this method. It may be that the user needs to do
    /// `stencila plugins refresh` to fetch the manifests for the plugins.
    /// Or it may be that the method name is just plain wrong.
    #[error("None of the registered plugins implement method `{method}`")]
    UndelegatableMethod { method: String },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. At least one of the plugins implements
    /// this method but not with the values for the supplied parameters e.g.
    /// `decode(content, format)` which `format = 'some-unknown-format'`.
    #[error("None of the registered plugins implement method `{method}` with given parameters")]
    UndelegatableCall {
        method: String,
        params: HashMap<String, serde_json::Value>,
    },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. There is a matching implementation
    /// for the call but plugin which implements it is not yet installed.
    #[error("Plugin `{plugin}` is not yet installed")]
    PluginNotInstalled { plugin: String },

    /// An error of unspecified type
    #[error("{message}")]
    Unspecified { message: String },
}

/// Generate the JSON Schema for the `addresses` property to avoid duplicated types.
fn address_schema(_generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    schemas::typescript("Address", true)
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

/// Create an `IncompatibleLanguage` error
pub fn incompatible_language<Type: ?Sized>(language: &str) -> Error {
    Error::IncompatibleLanguage {
        language: language.to_string(),
        kernel_type: type_name::<Type>().into(),
    }
}

/// A global stack of errors that can be used have "sideband" errors i.e. those that
/// do not cause a function to return early and are not part of the `Result` of a function.
pub static ERRORS: Lazy<Mutex<Vec<Error>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// A flag to indicate if errors are being collected
static COLLECT: AtomicBool = AtomicBool::new(false);

/// Start collecting errors
#[deprecated]
pub fn start() {
    ERRORS.lock().expect("To be able to get lock").clear();
    COLLECT.swap(true, std::sync::atomic::Ordering::Relaxed);
}

/// Report an error
#[deprecated = "Use tracing::error/warn instead"]
pub fn report(error: Error) {
    #[cfg(test)]
    eprintln!("Reported ERROR: {}", error);

    if COLLECT.load(std::sync::atomic::Ordering::Relaxed) {
        ERRORS.lock().expect("To be able to get lock").push(error);
    }
}

/// Record an error result if any
#[deprecated = "Use tracing::error/warn instead"]
pub fn attempt<T>(result: Result<T>) {
    if let Err(error) = result {
        let message = error.to_string();
        match error.downcast::<Error>() {
            Ok(error) => report(error),
            Err(_) => report(Error::Unspecified { message }),
        }
    }
}

/// Stop collecting errors and return those collected
#[deprecated]
pub fn stop() -> Vec<Error> {
    COLLECT.swap(false, std::sync::atomic::Ordering::Relaxed);
    ERRORS.lock().expect("To be able to get lock").split_off(0)
}

/// Get JSON Schema for the `Error` enum.
///
/// This needs to be generated slightly differently to the other schemas
/// in this library because of the way `thiserror` wants enums to be layed out.
pub fn schema() -> Result<serde_json::Value> {
    let settings = SchemaSettings::draft2019_09().with(|settings| {
        settings.option_add_null_type = false;
        settings.inline_subschemas = false;
    });
    let schema = settings.into_generator().into_root_schema_for::<Error>();
    let schema = serde_json::to_value(schema)?;
    Ok(schema)
}
