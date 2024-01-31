// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::creative_work_type::CreativeWorkType;
use super::text::Text;

/// [`CreativeWorkType`] or [`Text`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum CreativeWorkTypeOrText {
    #[default]
    CreativeWorkType(CreativeWorkType),

    Text(Text),
}
