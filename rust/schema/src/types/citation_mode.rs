// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The mode of a `Cite`.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum CitationMode {
    #[default]
    Parenthetical,
    Narrative,
    NarrativeAuthor,
}
