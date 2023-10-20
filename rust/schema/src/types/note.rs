// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::note_type::NoteType;
use super::string::String;

/// Additional content which is not part of the main content of a document.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
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
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[jats(attr = "custom-type")]
    pub note_type: NoteType,

    /// Content of the note, usually a paragraph.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![Block::Paragraph(crate::Paragraph::new(vec![crate::Inline::Text(crate::Text::from("Note paragraph"))]))]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"vec![Block::Paragraph(crate::Paragraph::new(vec![crate::Inline::Text(crate::Text::from("Note paragraph"))]))]"#))]
    #[cfg_attr(feature = "proptest-high", proptest(value = r#"vec![Block::Paragraph(crate::Paragraph::new(vec![crate::Inline::Text(crate::Text::from("Note paragraph"))]))]"#))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"vec![Block::Paragraph(crate::Paragraph::new(vec![crate::Inline::Text(crate::Text::from("Note paragraph"))]))]"#))]
    pub content: Vec<Block>,
}

impl Note {
    pub fn new(note_type: NoteType, content: Vec<Block>) -> Self {
        Self {
            note_type,
            content,
            ..Default::default()
        }
    }
}
