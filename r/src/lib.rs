#![allow(clippy::not_unsafe_ptr_arg_deref)]

use extendr_api::*;
use std::str::FromStr;

#[extendr]
fn init(rfunc: &str) -> Result<()> {
    let _robj = call!(rfunc)?;
    Ok(())
}

#[extendr]
fn serve(
    protocol: Option<String>,
    address: Option<String>,
    port: u16,
    background: Option<bool>,
) -> Result<()> {
    let protocol = protocol.unwrap_or_else(|| "stdio".to_string());
    let protocol = match stencila::protocols::Protocol::from_str(protocol.as_str()) {
        Ok(value) => Some(value),
        Err(_error) => return Err(Error::Other("Invalid protocol".to_string())),
    };

    let background = background.unwrap_or(false);

    match if background {
        stencila::serve::serve_background(protocol, address, Some(port))
    } else {
        stencila::serve::serve_blocking(protocol, address, Some(port))
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
