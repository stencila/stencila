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
    Read,
    Write,

    Decode,
    Encode,

    Validate,

    Upcast,
    Downcast,

    Import,
    Export,

    Compile,
    Build,
    Execute,
}
