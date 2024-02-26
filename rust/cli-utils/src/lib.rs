//! Utility functions and types for command line interfaces
//!
//! This little crate exists as a place to put CLI related things we
//! want to re-use in sibling crates for both convenience and consistency.

use std::fmt::Display;

use is_terminal::IsTerminal;

use common::{serde::Serialize, serde_json};

pub use rpassword;

mod message;
pub use message::*;

pub mod table;

/// A trait for displaying an object to stdout
pub trait ToStdout: Serialize {
    /// Print the object to stdout
    fn to_stdout(&self) {
        if std::io::stdout().is_terminal() {
            println!("{}", self.to_terminal())
        } else {
            println!("{}", serde_json::to_string_pretty(self).unwrap_or_default())
        }
    }

    /// Print the object to a terminal
    fn to_terminal(&self) -> impl Display;
}
