//! Generated file, do not edit

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
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
#[def = "CodeChunk(CodeChunk::default())"]
pub enum ExecutionDependantNode {
    Button(Button),
    Call(Call),
    CodeChunk(CodeChunk),
    CodeExpression(CodeExpression),
    Division(Division),
    File(File),
    Parameter(Parameter),
    Span(Span),
    Variable(Variable),
}
