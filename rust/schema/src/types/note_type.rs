//! Generated file, do not edit

use crate::prelude::*;

/// The type of a `Note` which determines where the note content is displayed within the document..
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
#[def = "Footnote"]
pub enum NoteType {
    Footnote,
    Endnote,
    Sidenote,
}
