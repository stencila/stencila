// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, strum::EnumString, ReadNode)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum AutomaticExecution {
    Never,

    #[default]
    Needed,

    Always,
}
