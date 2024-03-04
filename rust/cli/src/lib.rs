#![recursion_limit = "256"]

mod cli;
pub use crate::cli::Cli;

pub mod display;
pub mod errors;
pub mod logging;
mod uninstall;
pub mod upgrade;
