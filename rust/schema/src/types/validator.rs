// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::array_validator::ArrayValidator;
use super::boolean_validator::BooleanValidator;
use super::constant_validator::ConstantValidator;
use super::date_time_validator::DateTimeValidator;
use super::date_validator::DateValidator;
use super::duration_validator::DurationValidator;
use super::enum_validator::EnumValidator;
use super::integer_validator::IntegerValidator;
use super::number_validator::NumberValidator;
use super::string_validator::StringValidator;
use super::time_validator::TimeValidator;
use super::timestamp_validator::TimestampValidator;
use super::tuple_validator::TupleValidator;

/// Union type for validators.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum Validator {
    #[default]
    ArrayValidator(ArrayValidator),

    BooleanValidator(BooleanValidator),

    ConstantValidator(ConstantValidator),

    DateTimeValidator(DateTimeValidator),

    DateValidator(DateValidator),

    DurationValidator(DurationValidator),

    EnumValidator(EnumValidator),

    IntegerValidator(IntegerValidator),

    NumberValidator(NumberValidator),

    StringValidator(StringValidator),

    TimeValidator(TimeValidator),

    TimestampValidator(TimestampValidator),

    TupleValidator(TupleValidator),
}
