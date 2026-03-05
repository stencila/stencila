//! Built-in interviewer implementations (§6.4).
//!
//! Re-exports from the [`stencila_interviews`] crate.

pub use stencila_interviews::interviewers::*;

mod awaitable;

pub use awaitable::{AwaitableInterviewer, PendingInterviewSnapshot, SubmittedAnswer};
