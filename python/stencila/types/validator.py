# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ArrayValidator = ForwardRef("ArrayValidator")
BooleanValidator = ForwardRef("BooleanValidator")
ConstantValidator = ForwardRef("ConstantValidator")
DateTimeValidator = ForwardRef("DateTimeValidator")
DateValidator = ForwardRef("DateValidator")
DurationValidator = ForwardRef("DurationValidator")
EnumValidator = ForwardRef("EnumValidator")
IntegerValidator = ForwardRef("IntegerValidator")
NumberValidator = ForwardRef("NumberValidator")
StringValidator = ForwardRef("StringValidator")
TimeValidator = ForwardRef("TimeValidator")
TimestampValidator = ForwardRef("TimestampValidator")
TupleValidator = ForwardRef("TupleValidator")


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
