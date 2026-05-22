// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::action::Action;
use super::convert_action::ConvertAction;
use super::create_action::CreateAction;
use super::execute_action::ExecuteAction;

/// An action associated with a graph edge.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum GraphAction {
    /// A generic concrete activity.
    #[default]
    Action(Action),

    /// A concrete activity that created the edge target.
    CreateAction(CreateAction),

    /// A concrete conversion activity from source to target.
    ConvertAction(ConvertAction),

    /// A concrete execution activity that generated the target.
    ExecuteAction(ExecuteAction),
}
