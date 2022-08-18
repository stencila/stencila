#![recursion_limit = "256"]
#![forbid(unsafe_code)]

pub use kernels;
pub mod projects;
pub mod sessions;

// Features
//
// Features that can be turned off

#[cfg(feature = "cli")]
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
