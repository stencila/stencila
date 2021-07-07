use crate::methods::Method;
use eyre::Result;
use once_cell::sync::Lazy;
use schemars::{gen::SchemaSettings, JsonSchema};
use serde::Serialize;
use std::{
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
    /// The user attempted to open a document with an unknown format
    #[error("Unknown format '{format}'")]
    UnknownFormat { format: String },

    /// The user attempted to call a method that is not implemented internally
    /// and so must be delegated to a plugin. However, none of the registered
    /// plugins implement this method. It may be that the user needs to do
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

/// A global stack of errors that can be used have "sideband" errors i.e. those that
/// do not cause a function to return early and are not part of the `Result` of a function.
pub static ERRORS: Lazy<Mutex<Vec<Error>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// A flag to indicate if errors are being collected
static COLLECT: AtomicBool = AtomicBool::new(false);

/// Start collecting errors
pub fn start() {
    ERRORS.lock().expect("To be able to get lock").clear();
    COLLECT.swap(true, std::sync::atomic::Ordering::Relaxed);
}

/// Push an error
pub fn push_error(error: Error) {
    if COLLECT.load(std::sync::atomic::Ordering::Relaxed) {
        ERRORS.lock().expect("To be able to get lock").push(error);
    }
}

/// Push a error report
pub fn push_report(report: eyre::Report) {
    match report.downcast::<Error>() {
        Ok(error) => push_error(error),
        Err(_) => push_error(Error::Unknown),
    }
}

/// Stop collecting errors and return those collected
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
