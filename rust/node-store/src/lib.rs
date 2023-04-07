use std::path::Path;

use smol_str::SmolStr;

use common::eyre::Result;

pub(crate) use automerge::{
    AutoCommit as WriteStore, ObjId, ObjType, Prop, ReadDoc as ReadStore, ScalarValue, Value,
};
pub type Store = WriteStore;

mod prelude;

mod boolean;
mod index_map;
mod integer;
mod node;
mod null;
mod number;
mod primitive;
mod string;
mod text;
mod unsigned_integer;
mod vec;

pub trait Read: Sized {
    /// Read a Stencila Schema node from an Automerge file
    fn read(_path: &Path) -> Result<Self> {
        todo!() // Self::load(store)
    }

    /// Load a Stencila Schema node from an Automerge document
    fn load<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_root(store)
    }

    fn load_root<S: ReadStore>(_store: &S) -> Result<Self> {
        bail_type!("attempting to load node of type `{type}` from store; only maps allowed at root")
    }

    fn load_prop<S: ReadStore>(store: &S, obj: &ObjId, prop: Prop) -> Result<Self> {
        match store.get(obj, prop)? {
            Some((Value::Scalar(scalar), ..)) => match scalar.as_ref() {
                ScalarValue::Null => Self::load_null(),
                ScalarValue::Boolean(value) => Self::load_boolean(value),
                ScalarValue::Int(value) => Self::load_int(value),
                ScalarValue::Uint(value) => Self::load_uint(value),
                ScalarValue::F64(value) => Self::load_f64(value),
                ScalarValue::Str(value) => Self::load_str(value),
                ScalarValue::Counter(..) => Self::load_counter(),
                ScalarValue::Timestamp(value) => Self::load_timestamp(value),
                ScalarValue::Bytes(value) => Self::load_bytes(value),
                ScalarValue::Unknown { type_code, bytes } => Self::load_unknown(*type_code, bytes),
            },
            Some((Value::Object(ObjType::Text), id)) => Self::load_text(store, &id),
            Some((Value::Object(ObjType::List), id)) => Self::load_list(store, &id),
            Some((Value::Object(ObjType::Map), id)) | Some((Value::Object(ObjType::Table), id)) => {
                Self::load_map(store, &id)
            }
            None => Self::load_none(),
        }
    }

    fn load_null() -> Result<Self> {
        bail_load_unexpected!("Null")
    }

    fn load_boolean(_value: &bool) -> Result<Self> {
        bail_load_unexpected!("Boolean")
    }

    fn load_int(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Int")
    }

    fn load_uint(_value: &u64) -> Result<Self> {
        bail_load_unexpected!("Uint")
    }

    fn load_f64(_value: &f64) -> Result<Self> {
        bail_load_unexpected!("F64")
    }

    fn load_str(_value: &SmolStr) -> Result<Self> {
        bail_load_unexpected!("Str")
    }

    fn load_counter() -> Result<Self> {
        bail_load_unexpected!("Counter")
    }

    fn load_timestamp(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Timestamp")
    }

    fn load_bytes(_value: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Bytes")
    }

    fn load_unknown(_type_code: u8, _bytes: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Unknown")
    }

    fn load_text<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Text")
    }

    fn load_list<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("List")
    }

    fn load_map<S: ReadStore>(_store: &S, _obj: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Map")
    }

    fn load_none() -> Result<Self> {
        bail_load_unexpected!("None")
    }
}

pub trait Write {
    /// Write a Stencila Schema node to an Automerge file
    fn write(&self, _path: &Path) -> Result<()> {
        todo!() // self.dump(store)
    }

    /// Dump a Stencila Schema node to an Automerge document
    fn dump(&self, store: &mut WriteStore) -> Result<()> {
        self.dump_root(store)
    }

    fn dump_root(&self, _store: &mut WriteStore) -> Result<()> {
        bail_type!("attempting to dump node of type `{type}` to store; only maps allowed at root")
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize>;

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()>;

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()>;
}
