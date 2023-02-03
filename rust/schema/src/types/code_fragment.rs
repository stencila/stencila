//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// Inline code.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeFragment {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("CodeFragment"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The code.
    pub code: String,

    /// The programming language of the code.
    pub programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CodeFragmentOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeFragmentOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    pub media_type: Option<String>,
}
