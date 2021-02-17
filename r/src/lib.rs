#![allow(clippy::not_unsafe_ptr_arg_deref)]

use extendr_api::*;
#[extendr]
fn init(rfunc: &str) -> Result<()> {
    let _robj = call!(rfunc)?;
    Ok(())
}

#[extendr]
fn serve(url: Option<String>, background: Option<bool>) -> Result<()> {
    let background = background.unwrap_or(false);
    match if background {
        stencila::serve::serve_background(url, None)
    } else {
        stencila::serve::serve_blocking(url, None)
    } {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::Other(error.to_string())),
    }
}

extendr_module! {
    mod stencila;
    fn init;
    fn serve;
}
