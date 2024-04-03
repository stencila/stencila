// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::array::Array;
use super::boolean::Boolean;
use super::integer::Integer;
use super::null::Null;
use super::number::Number;
use super::object::Object;
use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// Union type for all primitives values.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, MergeNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum Primitive {
    #[default]
    Null(Null),

    Boolean(Boolean),

    Integer(Integer),

    UnsignedInteger(UnsignedInteger),

    Number(Number),

    String(String),

    Array(Array),

    Object(Object),
}
