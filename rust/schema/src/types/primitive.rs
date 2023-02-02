//! Generated file, do not edit

use crate::prelude::*;

use super::array::Array;
use super::boolean::Boolean;
use super::date::Date;
use super::date_time::DateTime;
use super::duration::Duration;
use super::integer::Integer;
use super::null::Null;
use super::number::Number;
use super::object::Object;
use super::string::String;
use super::time::Time;
use super::timestamp::Timestamp;

/// Union type for all primitives values
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[def = "Null(Null::default())"]
pub enum Primitive {
    Null(Null),
    Boolean(Boolean),
    Integer(Integer),
    Number(Number),
    String(String),
    Date(Date),
    Time(Time),
    DateTime(DateTime),
    Timestamp(Timestamp),
    Duration(Duration),
    Object(Object),
    Array(Array),
}
