use std::fmt::Display;

use common::serde::Serialize;

use crate::ToStdout;

/// A message for a user
#[derive(Serialize)]
#[serde(crate = "common::serde")]
pub struct Message(pub String);

#[macro_export]
macro_rules! message {
    ($str:literal, $($arg:tt)*) => {
        Message(format!($str, $($arg)*))
    };
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

impl ToStdout for Message {
    fn to_terminal(&self) -> impl std::fmt::Display {
        &self.0
    }
}
