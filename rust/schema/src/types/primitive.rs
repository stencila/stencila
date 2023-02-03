//! Generated file, do not edit

use crate::prelude::*;

use super::array::Array;
use super::boolean::Boolean;
use super::integer::Integer;
use super::null::Null;
use super::number::Number;
use super::object::Object;
use super::string::String;

/// Union type for all primitives values
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]
#[def = "Null(Null::default())"]
pub enum Primitive {
    Null(Null),
    Boolean(Boolean),
    Integer(Integer),
    Number(Number),
    String(String),
    Array(Array),
    Object(Object),
}
