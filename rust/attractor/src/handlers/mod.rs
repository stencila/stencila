//! Built-in handler implementations (§4.3–4.5, §4.7, §4.10).
//!
//! These handlers cover the structural and execution node types:
//! - [`StartHandler`] — pipeline entry point
//! - [`ExitHandler`] — pipeline termination
//! - [`ConditionalHandler`] — routing decision point
//! - [`CodergenHandler`] — LLM code generation
//! - [`ToolHandler`] — shell command execution

mod codergen;
mod conditional;
mod exit;
mod start;
mod tool;

pub use codergen::{CodergenBackend, CodergenHandler, CodergenResponse};
pub use conditional::ConditionalHandler;
pub use exit::ExitHandler;
pub use start::StartHandler;
pub use tool::ToolHandler;
