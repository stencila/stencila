//! Built-in handler implementations (§4.3–4.6, §4.8–4.10).
//!
//! These handlers cover the structural and execution node types:
//! - [`StartHandler`] — pipeline entry point
//! - [`ExitHandler`] — pipeline termination
//! - [`FailHandler`] — explicit pipeline failure
//! - [`ConditionalHandler`] — routing decision point
//! - [`CodergenHandler`] — LLM code generation
//! - [`ShellHandler`] — shell command execution
//! - [`WaitForHumanHandler`] — human-in-the-loop gate
//! - [`ParallelHandler`] — parallel fan-out execution
//! - [`FanInHandler`] — parallel result consolidation

mod codergen;
mod conditional;
mod exit;
mod fail;
mod fan_in;
mod parallel;
mod shell;
mod start;
mod wait_human;

pub use codergen::{CodergenBackend, CodergenHandler, CodergenOutput};
pub use conditional::ConditionalHandler;
pub use exit::ExitHandler;
pub use fail::FailHandler;
pub use fan_in::FanInHandler;
pub use parallel::{DEFAULT_MAX_PARALLEL, ParallelHandler};
pub use shell::ShellHandler;
pub use start::StartHandler;
pub use wait_human::{WaitForHumanHandler, parse_accelerator_key};
