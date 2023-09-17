// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::button::Button;
use super::call::Call;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::division::Division;
use super::file::File;
use super::parameter::Parameter;
use super::span::Span;
use super::variable::Variable;

/// Node types that can be execution dependencies
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, SmartDefault, ReadNode, WriteNode)]
#[serde(untagged, crate = "common::serde")]
pub enum ExecutionDependantNode {
    Button(Button),
    Call(Call),
    #[default]
    CodeChunk(CodeChunk),
    CodeExpression(CodeExpression),
    Division(Division),
    File(File),
    Parameter(Parameter),
    Span(Span),
    Variable(Variable),
}
