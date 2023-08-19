// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::button::Button;
use super::code_chunk::CodeChunk;
use super::file::File;
use super::parameter::Parameter;
use super::software_source_code::SoftwareSourceCode;
use super::variable::Variable;

/// Node types that can be execution dependencies
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
pub enum ExecutionDependencyNode {
    Button(Button),
    #[default]
    CodeChunk(CodeChunk),
    File(File),
    Parameter(Parameter),
    SoftwareSourceCode(SoftwareSourceCode),
    Variable(Variable),
}
