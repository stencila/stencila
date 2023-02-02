//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A code block.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CodeBlock {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("CodeBlock"),

    /// The identifier for this item
    id: String,

    /// The code.
    code: String,

    /// The programming language of the code.
    programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<CodeBlockOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CodeBlockOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    media_type: Option<String>,
}
