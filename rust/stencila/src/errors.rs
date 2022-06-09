use std::{any::type_name, collections::HashMap};

use schemars::{gen::SchemaSettings, JsonSchema};
use thiserror::Error;

use common::{eyre::Result, serde::Serialize, serde_json};

/// An enumeration of custom errors returned by this library
///
/// Where possible functions should return one of these errors to provide greater
/// context to the user, in particular regarding actions that can be taken to
/// resolve the error.
#[derive(Error, Debug, JsonSchema, Serialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Error {
    /// An identifier was supplied that does not match the pattern for the
    /// expected family of identifiers.
    #[error("Invalid universal identifier for family `{family}`: {id}")]
    InvalidUUID { family: String, id: String },

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

/// Create an `IncompatibleLanguage` error
pub fn incompatible_language<Type: ?Sized>(language: &str) -> Error {
    Error::IncompatibleLanguage {
        language: language.to_string(),
        kernel_type: type_name::<Type>().into(),
    }
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
