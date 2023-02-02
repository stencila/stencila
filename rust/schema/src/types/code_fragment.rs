//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// Inline code.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CodeFragment {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("CodeFragment"),

    /// The identifier for this item
    id: String,

    /// The code.
    code: String,

    /// The programming language of the code.
    programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<CodeFragmentOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CodeFragmentOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    media_type: Option<String>,
}
