// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::directory::Directory;
use super::file::File;
use super::symbolic_link::SymbolicLink;

/// [`File`] or [`SymbolicLink`] or [`Directory`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum FileOrSymbolicLinkOrDirectory {
    #[default]
    File(File),

    SymbolicLink(SymbolicLink),

    Directory(Directory),
}
