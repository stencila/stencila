use smol_str::SmolStr;

use common::eyre::Result;
use node_store::{
    automerge::{ObjId, Prop},
    Read, ReadStore, Write, WriteStore,
};

use crate::{Array, Null, Object, Primitive};

impl Read for Primitive {
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

    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Array(Array::load_list(store, obj)?))
    }

    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(Primitive::Object(Object::load_map(store, obj)?))
    }
}

impl Write for Primitive {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.insert_prop(store, obj_id, prop),
            Primitive::Boolean(value) => value.insert_prop(store, obj_id, prop),
            Primitive::Integer(value) => value.insert_prop(store, obj_id, prop),
            Primitive::UnsignedInteger(value) => value.insert_prop(store, obj_id, prop),
            Primitive::Number(value) => value.insert_prop(store, obj_id, prop),
            Primitive::String(value) => value.insert_prop(store, obj_id, prop),
            Primitive::Array(value) => value.insert_prop(store, obj_id, prop),
            Primitive::Object(value) => value.insert_prop(store, obj_id, prop),
        }
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.put_prop(store, obj_id, prop),
            Primitive::Boolean(value) => value.put_prop(store, obj_id, prop),
            Primitive::Integer(value) => value.put_prop(store, obj_id, prop),
            Primitive::UnsignedInteger(value) => value.put_prop(store, obj_id, prop),
            Primitive::Number(value) => value.put_prop(store, obj_id, prop),
            Primitive::String(value) => value.put_prop(store, obj_id, prop),
            Primitive::Array(value) => value.put_prop(store, obj_id, prop),
            Primitive::Object(value) => value.put_prop(store, obj_id, prop),
        }
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        match self {
            Primitive::Null(value) => value.similarity(store, obj_id, prop),
            Primitive::Boolean(value) => value.similarity(store, obj_id, prop),
            Primitive::Integer(value) => value.similarity(store, obj_id, prop),
            Primitive::UnsignedInteger(value) => value.similarity(store, obj_id, prop),
            Primitive::Number(value) => value.similarity(store, obj_id, prop),
            Primitive::String(value) => value.similarity(store, obj_id, prop),
            Primitive::Array(value) => value.similarity(store, obj_id, prop),
            Primitive::Object(value) => value.similarity(store, obj_id, prop),
        }
    }
}
