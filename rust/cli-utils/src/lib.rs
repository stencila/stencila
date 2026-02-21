//! Utility functions and types for command line interfaces
//!
//! This crate exists as a place to put CLI related things we want to re-use in
//! sibling crates for both convenience and consistency.
//!
//! See also the sibling `ask` crate for getting user inputs from the CLI.

use std::fmt::Display;
use std::io::IsTerminal;

// Modules for parsing CLI arguments and printing extended help messages

mod parsers;
pub use parsers::*;

pub use color_print;
pub use stencila_format;

// Modules for diagnostics to stderr
// Consider using `tracing::warn!` and `tracing::error` instead for
// modules that are not specifically CLI related.

mod message;
pub use message::*;

// Modules for outputs to stdout
// These implement the `ToStdout` trait (below)

mod code;
pub use code::*;

mod datatable;

pub mod tabulated;
pub use tabulated::Tabulated;

// Progress bars
pub mod progress;

// Utilities
pub use terminal_size;

mod utils;
pub use utils::*;

/// A trait for displaying an object to stdout
pub trait ToStdout: Display {
    /// Print the object to stdout
    ///
    /// This is intended to be the only function in this workspace
    /// that can print to stdout. It allows us to be more intentional
    /// about the use of stdout (e.g. by searching for use of this method).
    ///
    /// This is important in the context of protocols such as LSP and MCP
    /// where the protocol involves communication over stdin/stdout and
    /// randomly printing to stdout can break that.
    #[allow(clippy::print_stdout)]
    fn to_stdout(&self) {
        if std::io::stdout().is_terminal() {
            println!("{}", self.to_terminal())
        } else {
            println!("{self}")
        }
    }

    /// Print the object to a terminal
    fn to_terminal(&self) -> impl Display;
}
