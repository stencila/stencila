// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::directory::Directory;
use super::file::File;

/// [`File`] or [`Directory`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum FileOrDirectory {
    #[default]
    File(File),

    Directory(Directory),
}
