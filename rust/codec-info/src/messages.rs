use std::fmt::Display;

use common::derive_more::{Deref, DerefMut};

/// The severity level of a message.
///
/// This is very similar to `schema::MessageLevel` but that can not be used
/// here due to circular dependencies
#[derive(Default, Debug, PartialEq, Eq)]
pub enum MessageLevel {
    /// A trace message
    Trace,

    /// A debug message
    Debug,

    /// An informational message
    Info,

    /// A warning message
    Warning,

    /// An error message
    #[default]
    Error,
}

/// An error, warning or log message generated during compilation.
///
/// This is similar to the `CompilationMessage` and `ExecutionMessage`
/// in the Stencila Schema but to avoid adding a dependency is
/// implemented as a separate struct.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Message {
    /// The severity level of the message.
    pub level: MessageLevel,

    /// The text of the message.
    pub message: String,

    /// The 0-based index of the first line of the location in the contents.
    pub start_line: Option<usize>,

    /// The 0-based index of the first column of the location in the contents.
    pub start_column: Option<usize>,

    /// The 0-based index of the last line of the location in the contents.
    pub end_line: Option<usize>,

    /// The 0-based index of the last column of the location in the contents.
    pub end_column: Option<usize>,
}

/// Decoding and encoding messages
#[derive(Default, Debug, Deref, DerefMut, PartialEq, Eq)]
pub struct Messages(pub Vec<Message>);

impl Messages {
    /// Create an empty set of messages
    pub fn none() -> Self {
        Self::default()
    }
}

#[cfg(debug_assertions)]
impl Display for Messages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for message in self.iter() {
            f.write_fmt(format_args!(
                "{} {}\n",
                message.start_line.unwrap_or(999),
                message.message
            ))?;
        }
        f.write_str("")
    }
}
