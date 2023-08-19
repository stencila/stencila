# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array_validator import ArrayValidator
from .boolean_validator import BooleanValidator
from .constant_validator import ConstantValidator
from .date_time_validator import DateTimeValidator
from .date_validator import DateValidator
from .duration_validator import DurationValidator
from .enum_validator import EnumValidator
from .integer_validator import IntegerValidator
from .number_validator import NumberValidator
from .string_validator import StringValidator
from .time_validator import TimeValidator
from .timestamp_validator import TimestampValidator
from .tuple_validator import TupleValidator


Validator = Union[
    ArrayValidator,
    BooleanValidator,
    ConstantValidator,
    DateTimeValidator,
    DateValidator,
    DurationValidator,
    EnumValidator,
    IntegerValidator,
    NumberValidator,
    StringValidator,
    TimeValidator,
    TimestampValidator,
    TupleValidator,
]
"""
Union type for validators.
"""
