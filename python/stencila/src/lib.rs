use pyo3::prelude::*;

mod convert;
mod utilities;

#[pymodule]
#[pyo3(name = "_stencila")]
fn stencila(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    convert::module(stencila)?;

    Ok(())
}
