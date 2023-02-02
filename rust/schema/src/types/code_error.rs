//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// An error that occurred when parsing, compiling or executing a Code node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CodeError {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("CodeError"),

    /// The identifier for this item
    id: Option<String>,

    /// The error message or brief description of the error.
    error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    error_type: Option<String>,

    /// Stack trace leading up to the error.
    stack_trace: Option<String>,
}
