// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::delete_block::DeleteBlock;
use super::insert_block::InsertBlock;
use super::modify_block::ModifyBlock;
use super::replace_block::ReplaceBlock;

/// Union type for all types that are descended from `SuggestionBlock`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum SuggestionBlockType {
    #[default]
    DeleteBlock(DeleteBlock),

    InsertBlock(InsertBlock),

    ModifyBlock(ModifyBlock),

    ReplaceBlock(ReplaceBlock),
}
