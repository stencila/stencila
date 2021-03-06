#![recursion_limit = "256"]
#![deny(unsafe_code)]

// Objects
//
// Core Stencila objects e.g `File`, `Article`, `Project`

pub mod documents;
pub mod files;
pub mod formats;
pub mod projects;
pub mod sources;

// Methods
//
// Core functions that operate on Stencila objects and which
// may be delegated to plugins

pub mod methods {
    pub mod prelude;
    pub use prelude::*;

    pub mod import;

    pub mod decode;
    pub mod encode;

    pub mod coerce;
    pub mod reshape;

    pub mod compile;
    pub mod execute;
}

// Traits
//
// Helper traits for `Node` structs and vectors are useful across
// methods and elsewhere

pub mod traits {
    mod to_vec_inline_content;
    pub use to_vec_inline_content::ToVecInlineContent;
}

// Features
//
// Features that can be turned off

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "binaries")]
pub mod binaries;

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

// Internal messaging etc

pub mod errors;
pub mod logging;
pub mod pubsub;
pub mod telemetry;

// Utilities
//
// Usually just small functions that are often wrappers around other crates.

pub mod utils {
    pub mod fs;
    pub mod http;
    pub mod schemas;
    pub mod urls;
    pub mod uuids;

    #[cfg(test)]
    pub mod tests;
}

// Re-export packages
//
// Mainly for use by stencila-* language packages in this workspace

pub use eyre;
pub use once_cell;
pub use regex;
pub use serde;
pub use serde_json;
#[cfg(feature = "serde_yaml")]
pub use serde_yaml;
pub use strum;
pub use tokio;
pub use tracing;
pub use validator;
