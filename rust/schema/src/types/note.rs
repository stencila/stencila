//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::note_type::NoteType;
use super::string::String;

/// Additional content which is not part of the main content of a document.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Note {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("Note"),

    /// The identifier for this item
    id: Option<String>,

    /// Determines where the note content is displayed within the document.
    note_type: NoteType,

    /// Content of the note, usually a paragraph.
    content: Vec<Block>,
}
