//! Internal utility functions

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use tokio::runtime::Runtime;

/// Share one async runtime across every synchronous binding.
///
/// Reusing the runtime already created for the asynchronous bindings avoids
/// per-call setup and keeps blocking entry points on the same executor.
pub(crate) fn runtime() -> &'static Runtime {
    pyo3_async_runtimes::tokio::get_runtime()
}

/// Report a caller mistake, such as an unusable argument or malformed payload.
///
/// Accepting anything convertible into a report lets call sites pass native
/// errors directly instead of converting at every boundary.
pub(crate) fn value_error(error: impl Into<eyre::Report>) -> PyErr {
    PyValueError::new_err(error.into().to_string())
}

/// Report a failure encountered while performing the requested work.
pub(crate) fn runtime_error(error: impl Into<eyre::Report>) -> PyErr {
    PyRuntimeError::new_err(error.into().to_string())
}

/// Serialize a native result for the Python layer to decode.
pub(crate) fn to_json(value: &impl Serialize) -> PyResult<String> {
    serde_json::to_string(value).map_err(value_error)
}

/// Deserialize a payload supplied by the Python layer.
pub(crate) fn from_json<T: DeserializeOwned>(json: &str) -> PyResult<T> {
    serde_json::from_str(json).map_err(value_error)
}
