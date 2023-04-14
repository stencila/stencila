use crate::prelude::*;

/// The mode of a `Cite`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Parenthetical"]
pub enum CitationMode {
    Parenthetical,
    Narrative,
    NarrativeAuthor,
}
