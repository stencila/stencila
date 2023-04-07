use common::eyre::Result;
use schema::{Array, Null, Object, Primitive};
use smol_str::SmolStr;

use crate::{ObjId, Prop, Read, ReadStore, Write, WriteStore};

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
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        match self {
            Primitive::Null(value) => value.similarity(store, obj, prop),
            Primitive::Boolean(value) => value.similarity(store, obj, prop),
            Primitive::Integer(value) => value.similarity(store, obj, prop),
            Primitive::UnsignedInteger(value) => value.similarity(store, obj, prop),
            Primitive::Number(value) => value.similarity(store, obj, prop),
            Primitive::String(value) => value.similarity(store, obj, prop),
            Primitive::Array(value) => value.similarity(store, obj, prop),
            Primitive::Object(value) => value.similarity(store, obj, prop),
        }
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.dump_new(store, obj, prop),
            Primitive::Boolean(value) => value.dump_new(store, obj, prop),
            Primitive::Integer(value) => value.dump_new(store, obj, prop),
            Primitive::UnsignedInteger(value) => value.dump_new(store, obj, prop),
            Primitive::Number(value) => value.dump_new(store, obj, prop),
            Primitive::String(value) => value.dump_new(store, obj, prop),
            Primitive::Array(value) => value.dump_new(store, obj, prop),
            Primitive::Object(value) => value.dump_new(store, obj, prop),
        }
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Primitive::Null(value) => value.dump_prop(store, obj, prop),
            Primitive::Boolean(value) => value.dump_prop(store, obj, prop),
            Primitive::Integer(value) => value.dump_prop(store, obj, prop),
            Primitive::UnsignedInteger(value) => value.dump_prop(store, obj, prop),
            Primitive::Number(value) => value.dump_prop(store, obj, prop),
            Primitive::String(value) => value.dump_prop(store, obj, prop),
            Primitive::Array(value) => value.dump_prop(store, obj, prop),
            Primitive::Object(value) => value.dump_prop(store, obj, prop),
        }
    }
}
