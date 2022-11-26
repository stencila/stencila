//! Patching for [`ValidatorTypes`] nodes

use stencila_schema::*;

use super::prelude::*;

patchable_variants!(
    ValidatorTypes,
    ValidatorTypes::BooleanValidator,
    ValidatorTypes::ConstantValidator,
    ValidatorTypes::EnumValidator,
    ValidatorTypes::IntegerValidator,
    ValidatorTypes::NumberValidator,
    ValidatorTypes::StringValidator,
    ValidatorTypes::DateValidator,
    ValidatorTypes::TimeValidator,
    ValidatorTypes::DateTimeValidator,
    ValidatorTypes::TimestampValidator,
    ValidatorTypes::DurationValidator,
    ValidatorTypes::ArrayValidator,
    ValidatorTypes::TupleValidator
);

patchable_struct!(BooleanValidator);
patchable_struct!(ConstantValidator, value);
patchable_struct!(EnumValidator, values);

patchable_struct!(
    IntegerValidator,
    minimum,
    maximum,
    exclusive_minimum,
    exclusive_maximum,
    multiple_of
);
patchable_struct!(
    NumberValidator,
    minimum,
    maximum,
    exclusive_minimum,
    exclusive_maximum,
    multiple_of
);

patchable_struct!(StringValidator, min_length, max_length, pattern);

patchable_struct!(DateValidator, minimum, maximum);
patchable_struct!(TimeValidator, minimum, maximum);
patchable_struct!(DateTimeValidator, minimum, maximum);

patchable_struct!(TimestampValidator, minimum, maximum, time_units);
patchable_struct!(DurationValidator, minimum, maximum, time_units);

patchable_struct!(ArrayValidator);
patchable_struct!(TupleValidator, items);
