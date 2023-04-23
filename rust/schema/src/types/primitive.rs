use crate::prelude::*;

use super::array::Array;
use super::boolean::Boolean;
use super::integer::Integer;
use super::null::Null;
use super::number::Number;
use super::object::Object;
use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// Union type for all primitives values
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Null(Null::default())"]
pub enum Primitive {
    Null(Null),
    Boolean(Boolean),
    Integer(Integer),
    UnsignedInteger(UnsignedInteger),
    Number(Number),
    String(String),
    Array(Array),
    Object(Object),
}
