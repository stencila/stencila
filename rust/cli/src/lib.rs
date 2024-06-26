#![recursion_limit = "256"]

mod cli;
pub use crate::cli::{Cli, Command};

pub mod errors;
pub mod logging;
mod uninstall;
pub mod upgrade;
