use pyo3::prelude::*;

mod convert;
mod utilities;

#[pymodule]
#[pyo3(name = "_stencila")]
fn stencila(py: Python<'_>, stencila: &PyModule) -> PyResult<()> {
    let convert = convert::module(py)?;
    stencila.add_submodule(convert)?;

    Ok(())
}
