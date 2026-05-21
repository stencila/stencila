// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The status of an action.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum ActionStatusType {
    /// The action is proposed or possible but has not started.
    PotentialActionStatus,

    /// The action is currently in progress.
    ActiveActionStatus,

    /// The action has completed successfully.
    #[default]
    CompletedActionStatus,

    /// The action failed.
    FailedActionStatus,
}
