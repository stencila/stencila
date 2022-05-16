//! Next iteration of Stencila document server
//! 
//! This is intended to replace the existing implementation in `../stencila/src/server.rs`.
//! The transition is likely to be gradual, at least initially, particularly that the existing
//! implementation relies on modules in the main crate, notably `rpc` (so these will have to be 
//! factored out into their own crates first).
//! 
//! The advantage of having the server in it's own crate (in addition to the usual advantage of
//! reduced compile time) is that the server can be run from sibling crates that require
//! it (e.g. `codec-pdf` and `codec-png`).
//! 
//! For better developer ergonomics, including reduced compile times, this crate uses `axum`.

pub mod errors;
pub mod statics;

mod server;
pub use server::*;

#[cfg(feature = "cli")]
pub mod cli;
