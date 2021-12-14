use super::prelude::*;
use stencila_schema::*;

impl Patchable for Datatable {
    patchable_struct_is_equal!(columns);
    patchable_struct_hash!(columns);

    fn diff(&self, other: &Self, differ: &mut Differ) {
        // TODO: Implement diffing optimized (semantically and computationally) for datatables
        // e.g. `Add` and `Remove` for entire columns and entire rows,
        // `Replace` for individual cells
        differ.replace(other)
    }
}

patchable_struct!(DatatableColumn, name, validator, values);

patchable_struct!(Parameter, name, validator, value);

patchable_variants!(
    ValidatorTypes,
    ValidatorTypes::ArrayValidator,
    ValidatorTypes::BooleanValidator,
    ValidatorTypes::ConstantValidator,
    ValidatorTypes::EnumValidator,
    ValidatorTypes::IntegerValidator,
    ValidatorTypes::NumberValidator,
    ValidatorTypes::StringValidator,
    ValidatorTypes::TupleValidator
);
patchable_struct!(ArrayValidator);
patchable_struct!(BooleanValidator);
patchable_struct!(ConstantValidator, value);
patchable_struct!(EnumValidator, values);
patchable_struct!(IntegerValidator);
patchable_struct!(
    NumberValidator,
    minimum,
    maximum,
    exclusive_minimum,
    exclusive_maximum,
    multiple_of
);
patchable_struct!(StringValidator, min_length, max_length, pattern);
patchable_struct!(TupleValidator, items);
