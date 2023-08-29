// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Note` which determines where the note content is displayed within the document..
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, HtmlCodec, TextCodec)]
#[serde(crate = "common::serde")]
pub enum NoteType {
    #[default]
    Footnote,
    Endnote,
    Sidenote,
}
