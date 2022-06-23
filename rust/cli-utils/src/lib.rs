//! Utilities for command line interfaces

pub mod args;

mod command;
pub use command::Run;

pub mod result;
pub use result::Result;

#[cfg(feature = "progress")]
pub mod progress;

#[cfg(feature = "interact")]
pub mod interact;

mod outputs;
pub use outputs::*;

pub mod table;

pub use ansi_term;
pub use clap;
pub use cli_table;
pub use color_eyre;
pub use common;
pub use tracing_subscriber;
