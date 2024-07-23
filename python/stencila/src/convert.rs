// SPDX-FileCopyrightText: 2024 Nokome Bentley
//
// SPDX-License-Identifier: Apache-2.0

//! Exposes format conversion functionality provided by Rust codecs

use std::path::PathBuf;

use pyo3::prelude::*;

use codecs::Format;
use common::{eyre, serde_json};

use crate::utilities::{runtime_error, value_error};

pub fn module(py: Python<'_>) -> PyResult<&PyModule> {
    let convert = PyModule::new(py, "convert")?;

    convert.add_function(wrap_pyfunction!(to_string, convert)?)?;
    convert.add_function(wrap_pyfunction!(to_path, convert)?)?;
    convert.add_function(wrap_pyfunction!(from_string, convert)?)?;
    convert.add_function(wrap_pyfunction!(from_path, convert)?)?;
    convert.add_function(wrap_pyfunction!(from_to, convert)?)?;

    Ok(convert)
}

/// Decoding options
#[derive(FromPyObject)]
struct DecodeOptions {
    /// The format to be decode from
    #[pyo3(item)]
    format: Option<String>,
}

impl TryInto<codecs::DecodeOptions> for DecodeOptions {
    type Error = PyErr;

    fn try_into(self) -> PyResult<codecs::DecodeOptions> {
        Ok(codecs::DecodeOptions {
            format: self.format.as_ref().map(|format| Format::from_name(format)),
            ..Default::default()
        })
    }
}

/// Encoding options
#[derive(FromPyObject)]
struct EncodeOptions {
    /// The format to encode to
    #[pyo3(item)]
    format: Option<String>,

    /// Whether to encode as a standalone document
    ///
    /// Unless specified otherwise, this is the default when encoding to a file
    /// (as opposed to a string).
    #[pyo3(item)]
    standalone: Option<bool>,

    /// Whether to encode in compact form
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" (e.g. indented) forms.
    #[pyo3(item)]
    compact: Option<bool>,
}

impl TryInto<codecs::EncodeOptions> for EncodeOptions {
    type Error = PyErr;

    fn try_into(self) -> PyResult<codecs::EncodeOptions> {
        Ok(codecs::EncodeOptions {
            format: self.format.as_ref().map(|format| Format::from_name(format)),
            standalone: self.standalone,
            compact: self.compact,
            ..Default::default()
        })
    }
}

/// Decode a Stencila Schema node from a string
#[pyfunction]
fn from_string(py: Python, input: String, options: DecodeOptions) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let options = Some(options.try_into()?);

        let node = codecs::from_str(&input, options)
            .await
            .map_err(runtime_error)?;

        serde_json::to_string(&node)
            .map_err(eyre::Report::new)
            .map_err(value_error)
    })
}

/// Decode a Stencila Schema node from a file system path
#[pyfunction]
fn from_path(py: Python, path: String, options: DecodeOptions) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let path = PathBuf::from(path);
        let options = Some(options.try_into()?);

        let node = codecs::from_path(&path, options)
            .await
            .map_err(runtime_error)?;

        serde_json::to_string(&node)
            .map_err(eyre::Report::new)
            .map_err(value_error)
    })
}

/// Encode a Stencila Schema node to a string
#[pyfunction]
fn to_string(py: Python, json: String, options: EncodeOptions) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let options = Some(options.try_into()?);

        let node = serde_json::from_str(&json)
            .map_err(eyre::Report::new)
            .map_err(value_error)?;

        codecs::to_string(&node, options)
            .await
            .map_err(runtime_error)
    })
}

/// Encode a Stencila Schema node to a file system path
#[pyfunction]
fn to_path(py: Python, json: String, path: String, options: EncodeOptions) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let path = PathBuf::from(path);
        let options = Some(options.try_into()?);

        let node = serde_json::from_str(&json)
            .map_err(eyre::Report::new)
            .map_err(value_error)?;

        codecs::to_path(&node, &path, options)
            .await
            .map_err(runtime_error)
    })
}

/// Convert a document from one format to another
#[pyfunction]
fn from_to(
    py: Python,
    input: String,
    output: String,
    decode_options: DecodeOptions,
    encode_options: EncodeOptions,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let input = if input.is_empty() {
            None
        } else {
            Some(PathBuf::from(input))
        };
        let output = if output.is_empty() {
            None
        } else {
            Some(PathBuf::from(output))
        };
        let decode_options = Some(decode_options.try_into()?);
        let encode_options = Some(encode_options.try_into()?);

        codecs::convert(
            input.as_deref(),
            output.as_deref(),
            decode_options,
            encode_options,
        )
        .await
        .map_err(runtime_error)
    })
}
