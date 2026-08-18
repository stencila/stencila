#![recursion_limit = "256"]

use pyo3::prelude::*;

mod compare;
mod convert;
mod credentials;
mod graph;
mod utilities;

#[pymodule]
#[pyo3(name = "_stencila")]
fn stencila(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    compare::module(stencila)?;
    convert::module(stencila)?;
    graph::module(stencila)?;
    credentials::module(stencila)?;

    Ok(())
}
