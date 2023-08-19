// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::call::Call;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::division::Division;
use super::file::File;
use super::r#for::For;
use super::r#if::If;
use super::span::Span;
use super::variable::Variable;

/// Node types that can be execution dependants
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
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
