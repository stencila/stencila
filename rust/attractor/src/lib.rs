//! Attractor pipeline runner — a DOT-based execution engine for multi-stage AI workflows.
//!
//! Implements the [Attractor Specification][spec].
//!
//! [spec]: https://github.com/strongdm/attractor/blob/main/attractor-spec.md

#![warn(clippy::pedantic)]
#![allow(clippy::result_large_err)]

pub mod checkpoint;
pub mod condition;
pub mod context;
pub mod edge_selection;
pub mod engine;
pub mod error;
pub mod events;
pub mod graph;
pub mod handler;
pub mod handlers;
pub mod parser;
pub mod retry;
pub mod run_directory;
pub mod stylesheet;
pub mod stylesheet_parser;
pub mod transform;
pub mod transforms;
pub mod types;
pub mod validation;

pub use error::{AttractorError, AttractorResult};
pub use graph::{AttrValue, Edge, Graph, Node};
pub use parser::parse_dot;
pub use transform::{Transform, TransformRegistry};
pub use types::*;
