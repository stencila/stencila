//! Attractor pipeline runner — a DOT-based execution engine for multi-stage AI workflows.
//!
//! Implements the [Attractor Specification][spec].
//!
//! [spec]: https://github.com/strongdm/attractor/blob/main/attractor-spec.md

#![warn(clippy::pedantic)]
#![allow(clippy::result_large_err)]

pub mod checkpoint;
pub mod context;
pub mod error;
pub mod graph;
pub mod parser;
pub mod types;

pub use error::{AttractorError, AttractorResult};
pub use graph::{AttrValue, Edge, Graph, Node};
pub use parser::parse_dot;
pub use types::*;
