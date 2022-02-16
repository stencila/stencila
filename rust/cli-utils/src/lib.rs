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

// Export structopt and async_trait given that usage of this crate requires them...
// and others because they are useful :)
// Note: this structopt can not actually be used for macros yet.
// See https://github.com/TeXitoi/structopt/issues/339
pub use ansi_term;
pub use async_trait;
pub use color_eyre;
pub use structopt;
pub use tracing_subscriber;
