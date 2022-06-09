pub use std::any::{type_name, Any};

pub use common::eyre::{bail, Result};
pub use node_address::{
    invalid_address, invalid_slot_index, invalid_slot_name, invalid_slot_variant, Address, Slot,
};

pub use crate::{
    errors::{invalid_patch_operation, invalid_patch_value, Error},
    Differ, Operation, Patch, Patchable, Value,
};
