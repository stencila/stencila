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

// The `EnumValidator` is replaceable because it is to difficult to
// work with fine grained DOM patches to `values` (because they are in a <select>).
// Instead the `parameterValidator` proxy knows how to deal with replacement of
// this type of validator.
replaceable_struct!(EnumValidator, values);
