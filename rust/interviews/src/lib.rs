//! Human-in-the-loop interview infrastructure.
//!
//! Provides the [`Interviewer`] trait, question/answer types, stateless
//! built-in implementations, and (with the `sqlite` feature) the
//! `interviews` SQLite table schema and [`PersistentInterviewer`] decorator.
//!
//! This crate exists as a shared dependency so that both the `attractor`
//! (pipeline engine) and `agents` crates can use the same interview types
//! without creating a circular dependency.

pub mod interviewer;
pub mod interviewers;

#[cfg(feature = "sqlite")]
mod persistent;

#[cfg(feature = "sqlite")]
pub use persistent::{PersistentInterviewer, delete_interviews_for_context};

// ---------------------------------------------------------------------------
// Migrations (sqlite feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlite")]
pub use stencila_db::migration::Migration;

#[cfg(feature = "sqlite")]
pub static INTERVIEW_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("migrations/001_initial.sql"),
}];
