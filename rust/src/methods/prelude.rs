use serde::Serialize;
use strum::{EnumString, EnumVariantNames, ToString};

#[derive(Debug, EnumString, EnumVariantNames, PartialEq, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Method {
    Build,
    Call,
    Clean,
    Compile,
    Decode,
    Delete,
    Encode,
    Enrich,
    Execute,
    Export,
    Funcs,
    Get,
    Import,
    Pipe,
    Read,
    Reshape,
    Select,
    Set,
    Validate,
    Vars,
    Write,
}
