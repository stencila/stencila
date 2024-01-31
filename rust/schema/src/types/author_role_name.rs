// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A `roleName` for an `AuthorRole`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum AuthorRoleName {
    #[default]
    Writer,

    Instructor,

    Prompter,

    Generator,
}
