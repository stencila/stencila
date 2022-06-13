#![recursion_limit = "256"]
#![forbid(unsafe_code)]

// Objects
//
// Core Stencila objects e.g `File`, `Article`, `Project`

pub mod documents;
pub use kernels;
pub mod projects;
pub mod sessions;
pub mod sources;

// Features
//
// Features that can be turned off

#[cfg(feature = "cli")]
#[allow(deprecated)] // Remove when using clap 4.0 (https://github.com/clap-rs/clap/issues/3822)
pub mod cli;

#[cfg(feature = "upgrade")]
pub mod upgrade;

#[cfg(feature = "server")]
pub mod server;

#[cfg(any(feature = "server"))]
pub mod jwt;

#[cfg(any(feature = "server"))]
pub mod rpc;

// Internal configuration, messaging etc

pub mod config;
pub mod errors;
pub mod logging;
pub mod telemetry;

// Utilities
//
// Usually just small functions that are often wrappers around other crates.
pub mod utils;
