use common::{eyre::Result, smol_str::SmolStr};
use node_store::{automerge::ObjId, ReadCrdt, ReadNode};

use crate::{Null, Primitive, StringPatch, StringPatchOrPrimitive};

// It is necessary to implement this because `ModifyOperation.value`
// is required.
impl Default for StringPatchOrPrimitive {
    fn default() -> Self {
        Self::Primitive(Primitive::Null(Null))
    }
}

impl ReadNode for StringPatchOrPrimitive {
    fn load_map<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        Ok(StringPatchOrPrimitive::StringPatch(StringPatch::load_map(
            crdt, obj,
        )?))
    }

    fn load_null() -> Result<Self> {
        Primitive::load_null().map(StringPatchOrPrimitive::Primitive)
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Primitive::load_boolean(value).map(StringPatchOrPrimitive::Primitive)
    }

    fn load_int(value: &i64) -> Result<Self> {
        Primitive::load_int(value).map(StringPatchOrPrimitive::Primitive)
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Primitive::load_uint(value).map(StringPatchOrPrimitive::Primitive)
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Primitive::load_f64(value).map(StringPatchOrPrimitive::Primitive)
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Primitive::load_str(value).map(StringPatchOrPrimitive::Primitive)
    }

    fn load_list<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        Primitive::load_list(crdt, obj).map(StringPatchOrPrimitive::Primitive)
    }
}
