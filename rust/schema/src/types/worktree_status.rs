// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The status of a source worktree relative to a commit.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum WorktreeStatus {
    /// The source worktree matched the recorded commit with no known uncommitted or untracked changes.
    #[default]
    Clean,

    /// The source worktree had tracked changes that were not represented by the recorded commit.
    Dirty,

    /// The source file or relevant worktree content was not tracked by version control.
    Untracked,
}
