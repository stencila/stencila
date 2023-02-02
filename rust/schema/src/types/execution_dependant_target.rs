//! Generated file, do not edit

use crate::prelude::*;

use super::call::Call;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::division::Division;
use super::file::File;
use super::for_::For;
use super::if_::If;
use super::span::Span;
use super::variable::Variable;

/// Node types that can be execution dependants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]

pub enum ExecutionDependantTarget {
    Call(Call),
    CodeChunk(CodeChunk),
    CodeExpression(CodeExpression),
    Division(Division),
    If(If),
    File(File),
    For(For),
    Span(Span),
    Variable(Variable),
}
