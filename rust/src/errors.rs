use std::collections::HashMap;

use crate::{methods::Method, utils::schemas};
use eyre::Result;
use schemars::{gen::SchemaSettings, schema_for, JsonSchema};
use serde::Serialize;
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
    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. However, none of the registered
    /// plugins implement this method. It may be that the use needs to do
    /// `stencila plugins refresh` to fetch the manifests for the plugins.
    /// Or it may be that the method name is just plain wrong.
    #[error("None of the registered plugins implement method '{method}'")]
    UndelegatableMethod { method: Method },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. At least one of the plugins implements
    /// this method but not with the values for the supplied parameters e.g.
    /// `decode(content, format)` which `format = 'some-unknown-format'`.
    #[error("None of the registered plugins implement method '{method}' with given parameters")]
    UndelegatableCall {
        method: Method,
        params: HashMap<String, serde_json::Value>,
    },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. There is a matching implementation
    /// for the call but plugin which implements it is not yet installed.
    #[error("Plugin '{plugin}' is not yet installed")]
    PluginNotInstalled { plugin: String },

    /// Another, unspecified, error type
    #[error("An unknown error occurred")]
    Unknown,
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
