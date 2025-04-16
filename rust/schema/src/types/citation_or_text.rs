// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::citation::Citation;
use super::text::Text;

/// [`Citation`] or [`Text`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum CitationOrText {
    #[default]
    Citation(Citation),

    Text(Text),
}
