//! Utility functions and types for command line interfaces
//!
//! This little crate exists as a place to put CLI related things we
//! want to re-use in sibling crates for both convenience and consistency.

use std::fmt::Display;

use std::io::IsTerminal;

pub use rpassword;

mod code;
pub use code::*;

mod hint;
pub use hint::*;

mod message;
pub use message::*;

mod parsers;
pub use parsers::*;

pub mod table;

/// A trait for displaying an object to stdout
pub trait ToStdout: Display {
    /// Print the object to stdout
    #[allow(clippy::print_stdout)]
    fn to_stdout(&self) {
        if std::io::stdout().is_terminal() {
            println!("{}", self.to_terminal())
        } else {
            println!("{}", self)
        }
    }

    /// Print the object to a terminal
    fn to_terminal(&self) -> impl Display;
}
