use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use stencila::{
    anyhow::{bail, Result},
    delegate::DELEGATOR,
    logging,
    methods::Method,
    nodes::Node,
    serde_json, tracing,
};

/// init(manifest, dispatch)
/// ---
///
/// Initialize a plugin.
///
/// When the plugin is initialized the `manifest` function is called with arguments
/// describing the current system and should return the plugin's manifest as a Python
/// dictionary. The function should have the signature:
///   
///     def manifest(os: str, **kwargs) -> dict
///
/// The `os` argument is a string describing the current operating system
/// e.g. "windows", "macos", ""linux"
#[pyfunction]
fn init(_manifest: PyObject, dispatch: PyObject, log_level: Option<String>) -> PyResult<()> {
    // Initialize logging
    match logging::init(log_level) {
        Ok(_) => {}
        Err(error) => return Err(PyRuntimeError::new_err(error.to_string())),
    };

    // Call the manifest function with system information and store
    // the result so that it can be used in `manifest` and `register`
    // functions.
    //let manifest =

    // Create a delegator: a function that calls the Python `dispatch`
    // function for each request with the request method and parameters
    // and returns a node.
    let python_delegator = Box::new(
        move |method: Method, params: serde_json::Value| -> Result<Node> {
            let _span = tracing::trace_span!("python_delegator");

            // Obtain the Python Global Interpreter Lock
            let gil = Python::acquire_gil();
            let py = gil.python();

            // Call the dispatch function the the name of the method and it's
            // parameters as a dictionary
            let method = method.to_string();
            let params = serde_json::to_string(&params)?;
            match dispatch.call(py, (method, params), None) {
                // Convert the returned JSON string into a `Node` (a `serde_json::Value`)
                Ok(json) => {
                    let json = json.to_string();
                    let node = serde_json::from_str(&json)?;
                    Ok(node)
                }
                // Convert any raised Python error into an `anyhow:Error`
                Err(error) => bail!(error),
            }
        },
    );

    let result = DELEGATOR.set(python_delegator);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(PyRuntimeError::new_err("Failed to set delegator")),
    }
}

/// serve(protocol = "ws", address = "127.0.0.1", port = 9000, background = True)
/// --
///
/// Run a server using `protocol` (e.g. "ws", "http", "stdio") and listening
/// on `address` and `port`.
#[pyfunction]
fn serve(py: Python, url: Option<String>, background: Option<bool>) -> PyResult<()> {
    let background = background.unwrap_or(false);
    // When creating threads that will aquire the GIL, or doing any blocking,
    // it is necessary to call `py.allow_threads`
    py.allow_threads(|| {
        match if background {
            stencila::serve::serve_background(url, None)
        } else {
            stencila::serve::serve_blocking(url, None)
        } {
            Ok(_) => Ok(()),
            Err(error) => Err(PyRuntimeError::new_err(error.to_string())),
        }
    })
}

/// Define the `stencila` Python module
#[pymodule]
fn stencila(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(serve, m)?)?;

    Ok(())
}
