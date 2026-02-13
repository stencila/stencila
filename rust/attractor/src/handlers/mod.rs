//! Built-in handler implementations (§4.3–4.6, §4.8–4.10).
//!
//! These handlers cover the structural and execution node types:
//! - [`StartHandler`] — pipeline entry point
//! - [`ExitHandler`] — pipeline termination
//! - [`ConditionalHandler`] — routing decision point
//! - [`CodergenHandler`] — LLM code generation
//! - [`ToolHandler`] — shell command execution
//! - [`WaitForHumanHandler`] — human-in-the-loop gate
//! - [`ParallelHandler`] — parallel fan-out execution
//! - [`FanInHandler`] — parallel result consolidation

mod codergen;
mod conditional;
mod exit;
mod fan_in;
mod parallel;
mod start;
mod tool;
mod wait_human;

pub use codergen::{CodergenBackend, CodergenHandler, CodergenResponse};
pub use conditional::ConditionalHandler;
pub use exit::ExitHandler;
pub use fan_in::FanInHandler;
pub use parallel::{DEFAULT_MAX_PARALLEL, ParallelHandler};
pub use start::StartHandler;
pub use tool::ToolHandler;
pub use wait_human::{WaitForHumanHandler, parse_accelerator_key};
