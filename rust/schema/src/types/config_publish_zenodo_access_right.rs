// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The access right type
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ConfigPublishZenodoAccessRight {
    /// Open access right.
    #[default]
    #[serde(rename = "open")]
    #[serde(alias = "Open")]
    Open,

    /// Embargoed access right.
    #[serde(rename = "embargoed")]
    #[serde(alias = "Embargoed")]
    Embargoed,

    /// Restricted access right.
    #[serde(rename = "restricted")]
    #[serde(alias = "Restricted")]
    Restricted,

    /// Closed access right.
    #[serde(rename = "closed")]
    #[serde(alias = "Closed")]
    Closed,
}
