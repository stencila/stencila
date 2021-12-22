use super::prelude::*;
use node_coerce::coerce_to_validator;
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

/// Implement `Patchable` for `Parameter` to ensure that values of
/// `default` and `value` fields (which can be any `Node`) meet the
/// requirements of the `validator`.
impl Patchable for Parameter {
    patchable_struct_is_equal!(name, validator, default, value);
    patchable_struct_hash!(name, validator, default, value);
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
            parameter_coerce(self)
        } else {
            bail!(invalid_address::<Self>("first slot should be a name"))
        }
    }
}

// This may get moved into a `Coerceable` trait and `.coerce()` called in all
// of the `apply_*` methods of the `Patchable` trait.
fn parameter_coerce(par: &mut Parameter) -> Result<()> {
    if let Some(validator) = par.validator.as_deref() {
        if let Some(value) = par.value.as_deref() {
            let node = coerce_to_validator(value, validator)?;
            par.value = Some(Box::new(node));
        }
        if let Some(default) = par.default.as_deref() {
            let node = coerce_to_validator(default, validator)?;
            par.default = Some(Box::new(node));
        }
    }
    Ok(())
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
