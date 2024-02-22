//! # Common dependencies
//!
//! This internal crate simply re-exports dependencies that are commonly used across other internal
//! crates. Rust has a small `std` library (["and that's OK"](https://blog.nindalf.com/posts/rust-stdlib/)).
//! This crate acts as our internal version of a standard library, like [`stdx`](https://github.com/brson/stdx)
//! and others.
//!
//! The primary benefit of this crate is that there is only one place that version numbers for
//! commonly used dependencies need to be updated. Some of these crates are in line to become
//! part of the `std` library (e.g. `once_cell`).

pub use async_recursion;
pub use async_trait;
pub use base64;
pub use bs58;
pub use chrono;
pub use clap;
pub use derivative;
pub use derive_more;
pub use dirs;
pub use eyre;
pub use futures;
pub use glob;
pub use indexmap;
pub use inflector;
pub use itertools;
pub use maplit;
pub use once_cell;
pub use proc_macro2;
pub use quote;
pub use rand;
pub use regex;
pub use reqwest;
pub use serde;
pub use serde_json;
pub use serde_with;
pub use serde_yaml;
pub use similar;
pub use slug;
pub use smart_default;
pub use smol_str;
pub use strum;
pub use syn;
pub use tar;
pub use tempfile;
pub use tokio;
pub use toml;
pub use tracing;
pub use type_safe_id;
pub use uuid;
pub use which;
pub use xz2;
pub use zip;
