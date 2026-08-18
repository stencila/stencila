//! Exposes format conversion functionality provided by Rust codecs
//!
//! The codecs are asynchronous, but these bindings are not: they release the
//! interpreter lock and drive the codecs on the shared runtime. That keeps the module
//! usable from an ordinary script or notebook without an event loop, and consistent
//! with the other modules in this extension. A caller who is already inside an event
//! loop should dispatch these functions with `asyncio.to_thread`.

use std::path::PathBuf;

use pyo3::prelude::*;

use stencila_codecs::Format;

use crate::utilities::{runtime, runtime_error, value_error};

pub fn module(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    let convert = PyModule::new(stencila.py(), "convert")?;

    convert.add_function(wrap_pyfunction!(to_string, &convert)?)?;
    convert.add_function(wrap_pyfunction!(to_path, &convert)?)?;
    convert.add_function(wrap_pyfunction!(from_string, &convert)?)?;
    convert.add_function(wrap_pyfunction!(from_path, &convert)?)?;
    convert.add_function(wrap_pyfunction!(from_to, &convert)?)?;

    stencila.add("convert", convert)
}

/// The decoding options a caller asked for
fn decode_options(format: Option<String>) -> stencila_codecs::DecodeOptions {
    stencila_codecs::DecodeOptions {
        format: format.as_deref().map(Format::from_name),
        ..Default::default()
    }
}

/// The encoding options a caller asked for
fn encode_options(
    format: Option<String>,
    standalone: Option<bool>,
    compact: Option<bool>,
) -> stencila_codecs::EncodeOptions {
    stencila_codecs::EncodeOptions {
        format: format.as_deref().map(Format::from_name),
        standalone,
        compact,
        ..Default::default()
    }
}

/// Serialize a decoded node for the Python layer
fn node_to_json(node: &stencila_schema::Node) -> PyResult<String> {
    serde_json::to_string(node)
        .map_err(eyre::Report::new)
        .map_err(value_error)
}

/// Deserialize a node supplied by the Python layer
fn node_from_json(json: &str) -> PyResult<stencila_schema::Node> {
    serde_json::from_str(json)
        .map_err(eyre::Report::new)
        .map_err(value_error)
}

/// Decode a Stencila Schema node from a string
#[pyfunction]
#[pyo3(signature = (string, format=None))]
fn from_string(py: Python, string: String, format: Option<String>) -> PyResult<String> {
    py.detach(move || {
        let node = runtime()
            .block_on(stencila_codecs::from_str(
                &string,
                Some(decode_options(format)),
            ))
            .map_err(runtime_error)?;

        node_to_json(&node)
    })
}

/// Decode a Stencila Schema node from a file system path
#[pyfunction]
#[pyo3(signature = (path, format=None))]
fn from_path(py: Python, path: String, format: Option<String>) -> PyResult<String> {
    py.detach(move || {
        let node = runtime()
            .block_on(stencila_codecs::from_path(
                &PathBuf::from(path),
                Some(decode_options(format)),
            ))
            .map_err(runtime_error)?;

        node_to_json(&node)
    })
}

/// Encode a Stencila Schema node to a string
#[pyfunction]
#[pyo3(signature = (json, format=None, standalone=None, compact=None))]
fn to_string(
    py: Python,
    json: String,
    format: Option<String>,
    standalone: Option<bool>,
    compact: Option<bool>,
) -> PyResult<String> {
    py.detach(move || {
        let node = node_from_json(&json)?;

        runtime()
            .block_on(stencila_codecs::to_string(
                &node,
                Some(encode_options(format, standalone, compact)),
            ))
            .map_err(runtime_error)
    })
}

/// Encode a Stencila Schema node to a file system path
#[pyfunction]
#[pyo3(signature = (json, path, format=None, standalone=None, compact=None))]
fn to_path(
    py: Python,
    json: String,
    path: String,
    format: Option<String>,
    standalone: Option<bool>,
    compact: Option<bool>,
) -> PyResult<()> {
    py.detach(move || {
        let node = node_from_json(&json)?;

        runtime()
            .block_on(stencila_codecs::to_path(
                &node,
                &PathBuf::from(path),
                Some(encode_options(format, standalone, compact)),
            ))
            .map_err(runtime_error)?;

        Ok(())
    })
}

/// Convert a document from one format to another
#[pyfunction]
#[pyo3(signature = (
    input=None, output=None, from_format=None, to_format=None,
    to_standalone=None, to_compact=None
))]
fn from_to(
    py: Python,
    input: Option<String>,
    output: Option<String>,
    from_format: Option<String>,
    to_format: Option<String>,
    to_standalone: Option<bool>,
    to_compact: Option<bool>,
) -> PyResult<String> {
    py.detach(move || {
        let input = input.map(PathBuf::from);
        let output = output.map(PathBuf::from);

        runtime()
            .block_on(stencila_codecs::convert(
                input.as_deref(),
                output.as_deref(),
                Some(decode_options(from_format)),
                Some(encode_options(to_format, to_standalone, to_compact)),
            ))
            .map_err(runtime_error)
    })
}
