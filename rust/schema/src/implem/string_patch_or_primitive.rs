use smol_str::SmolStr;

use common::eyre::Result;
use node_store::{automerge::ObjId, ReadNode, ReadStore};

use crate::{Null, Primitive, StringPatch, StringPatchOrPrimitive};

// It is necessary to implement this because `ModifyOperation.value`
// is required.
impl Default for StringPatchOrPrimitive {
    fn default() -> Self {
        Self::Primitive(Primitive::Null(Null))
    }
}

impl ReadNode for StringPatchOrPrimitive {
    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(StringPatchOrPrimitive::StringPatch(StringPatch::load_map(
            store, obj,
        )?))
    }

    fn load_null() -> Result<Self> {
        Primitive::load_null().map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Primitive::load_boolean(value).map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Primitive::load_int(value).map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Primitive::load_uint(value).map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Primitive::load_f64(value).map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Primitive::load_str(value).map(|value| StringPatchOrPrimitive::Primitive(value))
    }

    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Primitive::load_list(store, obj).map(|value| StringPatchOrPrimitive::Primitive(value))
    }
}
