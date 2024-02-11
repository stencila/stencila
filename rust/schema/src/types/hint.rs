// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::array_hint::ArrayHint;
use super::boolean::Boolean;
use super::function::Function;
use super::integer::Integer;
use super::number::Number;
use super::object_hint::ObjectHint;
use super::string_hint::StringHint;
use super::unknown::Unknown;

/// Union type for hints of the value and/or structure of data.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum Hint {
    #[default]
    ArrayHint(ArrayHint),

    Function(Function),

    ObjectHint(ObjectHint),

    StringHint(StringHint),

    Unknown(Unknown),

    Boolean(Boolean),

    Integer(Integer),

    Number(Number),
}
