// Features

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "request")]
pub mod request;

#[cfg(feature = "serve")]
pub mod serve;

#[cfg(any(feature = "request", feature = "serve"))]
pub mod delegate;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod methods;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod protocols;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc;

// Methods

//pub mod convert;
pub mod decode;
pub mod encode;
pub mod validate;

// Utilities

pub mod nodes;
