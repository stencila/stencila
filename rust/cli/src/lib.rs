#![recursion_limit = "256"]

mod cli;
pub use crate::cli::{Cli, Command};

pub mod errors;
pub mod logging;
pub mod preview;
mod uninstall;
pub mod upgrade;
