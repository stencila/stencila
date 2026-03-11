//! Human-in-the-loop interview infrastructure.
//!
//! Provides the [`Interviewer`] trait, question/answer types, stateless
//! built-in implementations, the `interviews` SQLite table schema, and
//! the [`PersistentInterviewer`] decorator.
//!
//! This crate exists as a shared dependency so that both the `attractor`
//! (pipeline engine) and `agents` crates can use the same interview types
//! without creating a circular dependency.

pub mod condition;
pub mod conduct;
pub mod interviewer;
pub mod interviewers;
pub mod spec;

mod persistent;

pub use persistent::{
    PendingInterviewRecord, PersistentInterviewer, delete_interviews_for_context,
    find_pending_interview, insert_pending_interview, update_interview_answer,
};

// ---------------------------------------------------------------------------
// Migrations
// ---------------------------------------------------------------------------

pub use stencila_db::migration::Migration;

pub static INTERVIEW_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("migrations/001_initial.sql"),
}];
