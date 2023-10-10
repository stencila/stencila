// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The mode of a `Cite`.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, SmartDefault, ReadNode, WriteNode)]
#[serde(crate = "common::serde")]
pub enum CitationMode {
    #[default]
    Parenthetical,

    Narrative,

    NarrativeAuthor,
}
