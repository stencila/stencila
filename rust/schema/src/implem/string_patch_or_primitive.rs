use smol_str::SmolStr;

use common::eyre::Result;
use node_store::{automerge::ObjId, ReadNode, ReadStore};

use crate::{Primitive, StringPatch, StringPatchOrPrimitive};

impl ReadNode for StringPatchOrPrimitive {
    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(StringPatchOrPrimitive::StringPatch(StringPatch::load_map(
            store, obj,
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

    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Primitive::load_list(store, obj).map(StringPatchOrPrimitive::Primitive)
    }
}
