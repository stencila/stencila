//! Attractor pipeline runner — a DOT-based execution engine for multi-stage AI workflows.
//!
//! Implements the [Attractor Specification][spec].
//!
//! # Concurrency and lock-order guidance
//!
//! Some subsystems in this crate share state via locks, most notably the
//! SQLite-backed [`context::Context`] / [`sqlite_backend::SqliteBackend`]
//! integration which uses a shared `Arc<Mutex<rusqlite::Connection>>`.
//!
//! To avoid deadlocks:
//!
//! - keep lock scopes as small as possible
//! - do not hold a raw `SQLite` connection lock while calling higher-level
//!   abstractions such as [`context::Context`] methods, artifact storage,
//!   interview coordination, or event emission
//! - avoid calling into another lock-owning subsystem while holding a local
//!   `Mutex` or `RwLock` guard; collect the data you need, drop the guard,
//!   then perform the follow-up work
//!
//! When touching resume, persistence, or workflow coordination code, prefer
//! the pattern of buffering database results first and applying context or
//! cross-subsystem updates only after the relevant lock has been released.
//!
//! [spec]: https://github.com/strongdm/attractor/blob/main/attractor-spec.md

#![warn(clippy::pedantic)]
#![allow(clippy::result_large_err)]

pub mod artifact;
pub mod checkpoint;
pub mod condition;
pub mod context;
pub mod definition_snapshot;
pub mod edge_selection;
pub mod engine;
pub mod envelope;
pub mod error;
pub mod events;
pub mod fidelity;
pub mod graph;
pub mod handler;
pub mod handlers;
pub mod interpolation;
pub mod interviewer;
pub mod interviewers;
pub mod parser;
pub mod resume;
pub mod retry;
pub mod sqlite_backend;
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
