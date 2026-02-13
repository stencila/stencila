//! Built-in handler implementations (§4.3–4.4, §4.7).
//!
//! These handlers cover the structural node types that require no
//! external interaction:
//! - [`StartHandler`] — pipeline entry point
//! - [`ExitHandler`] — pipeline termination
//! - [`ConditionalHandler`] — routing decision point

mod conditional;
mod exit;
mod start;

pub use conditional::ConditionalHandler;
pub use exit::ExitHandler;
pub use start::StartHandler;
