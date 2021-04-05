#![deny(unsafe_code)]

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "interact")]
pub mod interact;

#[cfg(feature = "open")]
pub mod open;

#[cfg(feature = "plugins")]
pub mod plugins;

#[cfg(feature = "upgrade")]
pub mod upgrade;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "request")]
pub mod request;

#[cfg(feature = "serve")]
pub mod serve;

#[cfg(any(feature = "request", feature = "serve"))]
pub mod delegate;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod jwt;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod methods;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod protocols;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc;
#[cfg(any(feature = "request", feature = "serve"))]
pub mod urls;

// Methods

pub mod read;
pub mod write;

pub mod decode;
pub mod encode;

pub mod export;
pub mod import;

pub mod convert;

pub mod validate;

pub mod execute;

// Utilities

mod util {
    pub mod dirs;
}
pub mod logging;
pub mod nodes;

// Re-export packages (mainly for use by stencila-* language packages in this workspace)

pub use anyhow;
pub use serde_json;
pub use tokio;
pub use tracing;
