//! Internal utility functions

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

pub(crate) fn value_error(report: eyre::Report) -> PyErr {
    PyValueError::new_err(report.to_string())
}

pub(crate) fn runtime_error(report: eyre::Report) -> PyErr {
    PyRuntimeError::new_err(report.to_string())
}
