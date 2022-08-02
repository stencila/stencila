use node_validate::Validator;
use stencila_schema::*;

use super::prelude::*;

impl Patchable for Datatable {
    fn diff(&self, other: &Self, differ: &mut Differ) {
        // TODO: Implement diffing optimized (semantically and computationally) for datatables
        // e.g. `Add` and `Remove` for entire columns and entire rows,
        // `Replace` for individual cells
        differ.replace(other)
    }
}

patchable_struct!(DatatableColumn, name, validator, values);

/// Implement `Patchable` for `Parameter` to ensure that values of
/// `default` and `value` fields (which can be any `Node`) meet the
/// requirements of the `validator`.
impl Patchable for Parameter {
    patchable_struct_diff!(name, validator, default, value);

    // Presently, only `apply_replace` is overridden (because those are the operations sent
    // by the web client). In the future, the other apply_* operations probably need overriding.
    patchable_struct_apply_add!(name, validator, default, value);
    patchable_struct_apply_remove!(name, validator, default, value);
    patchable_struct_apply_move!(name, validator, default, value);
    patchable_struct_apply_transform!(name, validator, default, value);

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        if let Some(Slot::Name(name)) = address.pop_front() {
            match name.as_str() {
                "name" => self.name.apply_replace(address, items, value),
                "validator" => self.validator.apply_replace(address, items, value),
                "default" => self.default.apply_replace(address, items, value),
                "value" => self.value.apply_replace(address, items, value),
                _ => bail!(invalid_slot_name::<Self>(&name)),
            }?;

            // Ensure that the parameters `value` and `default` are valid for any
            // validator by coercing them to it
            if let Some(validator) = self.validator.as_deref() {
                if let Some(value) = self.value.as_deref() {
                    let node = validator.coerce(value);
                    self.value = Some(Box::new(node));
                }
                if let Some(default) = self.default.as_deref() {
                    let node = validator.coerce(default);
                    self.default = Some(Box::new(node));
                }
            }

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be a name"))
        }
    }
}

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

// The `EnumValidator` is replaceable because it is to difficult to
// work with fine grained DOM patches to `values` (because they are in a <select>).
// Instead the `parameterValidator` proxy knows how to deal with replacement of
// this type of validator.
replaceable_struct!(EnumValidator, values);
