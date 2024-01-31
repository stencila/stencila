// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite::Cite;
use super::text::Text;

/// [`Cite`] or [`Text`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum CiteOrText {
    #[default]
    Cite(Cite),

    Text(Text),
}
