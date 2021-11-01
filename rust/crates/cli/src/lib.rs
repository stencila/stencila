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

// Export structopt given that all usage of this crate requires it...
// and others because they are useful :)
pub use structopt;
pub use color_eyre;
