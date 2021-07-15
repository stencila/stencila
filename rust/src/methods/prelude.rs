use schemars::JsonSchema;
use serde::Serialize;
use strum::{Display, EnumString, EnumVariantNames};

/// An enumeration of all methods
#[derive(
    Clone, Copy, Debug, Display, EnumString, EnumVariantNames, PartialEq, JsonSchema, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Method {
    Import,
    Export,
    
    Decode,
    Encode,

    Coerce,
    Reshape,

    Compile,
    Build,
    Execute,
}
