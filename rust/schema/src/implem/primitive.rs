use common::smol_str::SmolStr;
use node_store::{automerge::ObjId, ReadCrdt, ReadNode};

use crate::{prelude::*, Array, Null, Object, Primitive};

impl ReadNode for Primitive {
    fn load_null() -> Result<Self> {
        Ok(Primitive::Null(Null {}))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(Primitive::Boolean(*value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Ok(Primitive::Integer(*value))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Ok(Primitive::UnsignedInteger(*value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(Primitive::Number(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(Primitive::String(value.to_string()))
    }

    fn load_list<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Array(Array::load_list(crdt, obj)?))
    }

    fn load_map<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Object(Object::load_map(crdt, obj)?))
    }
}
