//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::string::String;

/// A clause within a `If` node
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct IfClause {
    /// The code.
    code: String,

    /// The programming language of the code.
    programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    guess_language: Boolean,

    /// The content to render if the result is true-thy
    content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<IfClauseOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct IfClauseOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    media_type: Option<String>,

    /// Whether this clause is the active clause in the parent `If` node
    is_active: Option<Boolean>,
}
