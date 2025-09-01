// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates the action (create, update or delete) to derive for a `Form`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum FormDeriveAction {
    #[default]
    Create,

    Update,

    Delete,

    UpdateOrDelete,
}
