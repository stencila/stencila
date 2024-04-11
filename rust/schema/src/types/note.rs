// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::note_type::NoteType;
use super::string::String;

/// Additional content which is not part of the main content of a document.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Note")]
#[jats(elem = "fn", attribs(fn__type = "custom"))]
pub struct Note {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Note"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Determines where the note content is displayed within the document.
    #[serde(alias = "note-type", alias = "note_type")]
    #[merge(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"NoteType::Footnote"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"NoteType::Footnote"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"NoteType::arbitrary()"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"NoteType::arbitrary()"#))]
    #[jats(attr = "custom-type")]
    pub note_type: NoteType,

    /// Content of the note, usually a paragraph.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[merge(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![p([t("Note paragraph")])]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"vec![p([t("Note paragraph")])]"#))]
    #[cfg_attr(feature = "proptest-high", proptest(value = r#"vec![p([t("Note paragraph")])]"#))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"vec![p([t("Note paragraph")])]"#))]
    #[dom(elem = "aside")]
    pub content: Vec<Block>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Note {
    const NICK: [u8; 3] = [110, 111, 116];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Note
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(note_type: NoteType, content: Vec<Block>) -> Self {
        Self {
            note_type,
            content,
            ..Default::default()
        }
    }
}
