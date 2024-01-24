// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::grant::Grant;
use super::monetary_grant::MonetaryGrant;

/// [`Grant`] or [`MonetaryGrant`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum GrantOrMonetaryGrant {
    #[default]
    Grant(Grant),

    MonetaryGrant(MonetaryGrant),
}
