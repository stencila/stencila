#![recursion_limit = "256"]
#![deny(unsafe_code)]

// Objects
//
// Core Stencila objects e.g `File`, `Article`, `Project`

pub mod documents;
pub mod files;
pub mod projects;

// Methods
//
// Core functions that operate on Stencila objects and which
// may be delegated to plugins

pub mod methods {
    pub mod prelude;
    pub use prelude::*;

    pub mod read;
    pub mod write;

    pub mod decode;
    pub mod encode;
    #[cfg(feature = "format-html")]
    pub mod encode_html;

    pub mod export;
    pub mod import;

    pub mod validate;

    pub mod compile;
    pub mod execute;
}

// Features
//
// Features that can be turned off

#[cfg(feature = "cli")]
pub mod cli;
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
pub mod jwt;

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc;

// Internal messaging

pub mod logging;
pub mod pubsub;

// Utilities
//
// Usually just small functions that are often wrappers around other crates.

pub mod utils {
    pub mod fs;
    pub mod schemas;
    pub mod urls;
    pub mod uuids;
}

// Re-export packages
//
// Mainly for use by stencila-* language packages in this workspace

pub use eyre;
pub use once_cell;
pub use regex;
pub use serde;
pub use serde_json;
#[cfg(feature = "format-yaml")]
pub use serde_yaml;
pub use strum;
pub use tokio;
pub use tracing;
