use crate::prelude::*;

/// The type of a `Note` which determines where the note content is displayed within the document..
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Footnote"]
pub enum NoteType {
    Footnote,
    Endnote,
    Sidenote,
}
