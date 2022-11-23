pub use std::any::type_name;

pub use common::eyre::{bail, eyre, Result};
pub use node_address::{
    invalid_address, invalid_slot_index, invalid_slot_name, invalid_slot_variant, Address, Slot,
};

pub use crate::{
    differ::Differ,
    errors::{invalid_patch_operation, invalid_patch_value, Error},
    operation::*,
    patch::Patch,
    patchable::Patchable,
    value::Value,
};
