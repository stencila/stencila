// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite::Cite;
use super::text::Text;

/// [`Cite`] or [`Text`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum CiteOrText {
    #[default]
    Cite(Cite),

    Text(Text),
}
