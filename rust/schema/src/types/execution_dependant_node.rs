// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::button::Button;
use super::call::Call;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::division::Division;
use super::file::File;
use super::function::Function;
use super::parameter::Parameter;
use super::styled_inline::StyledInline;
use super::variable::Variable;

/// Node types that can be execution dependencies.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum ExecutionDependantNode {
    Button(Button),

    Call(Call),

    #[default]
    CodeChunk(CodeChunk),

    CodeExpression(CodeExpression),

    Division(Division),

    File(File),

    Function(Function),

    Parameter(Parameter),

    StyledInline(StyledInline),

    Variable(Variable),
}
