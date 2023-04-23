use crate::prelude::*;

use super::button::Button;
use super::code_chunk::CodeChunk;
use super::file::File;
use super::parameter::Parameter;
use super::software_source_code::SoftwareSourceCode;
use super::variable::Variable;

/// Node types that can be execution dependencies
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "CodeChunk(CodeChunk::default())"]
pub enum ExecutionDependencyNode {
    Button(Button),
    CodeChunk(CodeChunk),
    File(File),
    Parameter(Parameter),
    SoftwareSourceCode(SoftwareSourceCode),
    Variable(Variable),
}
