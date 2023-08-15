//! Interface between Stencila Schema and Automerge

use std::{
    fs::{read, write},
    path::Path,
    time::SystemTime,
};

use automerge::ROOT;
use common::eyre::{bail, Context, Result};
use smol_str::SmolStr;

pub use automerge::{self, AutoCommit as WriteStore, ObjId, ObjType, Prop, ReadDoc as ReadStore};
pub(crate) use automerge::{transaction::CommitOptions, ScalarValue, Value};

/// The maximum similarity index between to nodes
pub const SIMILARITY_MAX: usize = 1000;

mod prelude;

mod boolean;
mod r#box;
mod hash_map;
mod integer;
mod number;
mod option;
mod string;
mod unsigned_integer;
mod vec;

/// Bail from a function with the type of node included in the message
#[macro_export]
macro_rules! bail_type {
    ($message:literal) => {
        common::eyre::bail!($message, type = std::any::type_name::<Self>())
    };
}

/// Bail from a `load` function with the unexpected and expected type in the message
#[macro_export]
macro_rules! bail_load_unexpected {
    ($unexpected:literal) => {
        common::eyre::bail!(
            "unexpected Automerge `{unexpected}` while attempting to load `{type}` from store",
            unexpected = $unexpected,
            type = std::any::type_name::<Self>()
        )
    };
}

/// A trait for reading Stencila document nodes from an Automerge store
pub trait Read: Sized {
    /// Read a Stencila document node from an Automerge file
    fn read(path: &Path) -> Result<(WriteStore, Self)> {
        let store = load_store(path)?;

        // If the following call to `Self::load` fails it can be useful to use `inspect_store(&store)?`
        // to inspect the shape of the data in the store
        inspect_store(&store)?;

        let node = Self::load(&store)?;

        Ok((store, node))
    }

    /// Load a Stencila document node from an Automerge store
    ///
    /// Because Automerge stores must have a map at the root, this method calls
    /// the `load_map` method. As such it will fail if that method is not
    /// implemented for the node type (e.g. `Number` and `Vec` nodes).
    fn load<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_map(store, &ROOT)
    }

    /// Load the Stencila document node from a property for an object in an Automerge store
    fn load_from<S: ReadStore>(&mut self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<()> {
        *self = Self::load_prop(store, obj_id, prop)?;

        Ok(())
    }

    /// Load a new Stencila document node from a property for an object in an Automerge store
    fn load_prop<S: ReadStore>(store: &S, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        match store.get(obj_id, prop)? {
            Some((Value::Scalar(scalar), ..)) => match scalar.as_ref() {
                ScalarValue::Null => Self::load_null(),
                ScalarValue::Boolean(value) => Self::load_boolean(value),
                ScalarValue::Int(value) => Self::load_int(value),
                ScalarValue::Uint(value) => Self::load_uint(value),
                ScalarValue::F64(value) => Self::load_f64(value),
                ScalarValue::Str(value) => Self::load_str(value),
                ScalarValue::Counter(value) => Self::load_counter(&value.try_into()?),
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

    /// Load a Stencila document node from an [`automerge::ScalarValue::Null`]
    fn load_null() -> Result<Self> {
        bail_load_unexpected!("Null")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Boolean`]
    fn load_boolean(_value: &bool) -> Result<Self> {
        bail_load_unexpected!("Boolean")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Int`]
    fn load_int(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Int")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Uint`]
    fn load_uint(_value: &u64) -> Result<Self> {
        bail_load_unexpected!("Uint")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::F64`]
    fn load_f64(_value: &f64) -> Result<Self> {
        bail_load_unexpected!("F64")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Str`]
    fn load_str(_value: &SmolStr) -> Result<Self> {
        bail_load_unexpected!("Str")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Counter`]
    fn load_counter(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Counter")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Timestamp`]
    fn load_timestamp(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Timestamp")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Bytes`]
    fn load_bytes(_value: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Bytes")
    }

    /// Load a Stencila document node from an [`automerge::ScalarValue::Unknown`]
    fn load_unknown(_type_code: u8, _bytes: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Unknown")
    }

    /// Load a Stencila document node from an [`automerge::ObjType::Text`] value
    fn load_text<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Text")
    }

    /// Load a Stencila document node from an [`automerge::ObjType::List`]
    fn load_list<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("List")
    }

    /// Load a Stencila document node from an [`automerge::ObjType::Map`]
    fn load_map<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Map")
    }

    /// Load a Stencila document node from a `None` value
    fn load_none() -> Result<Self> {
        bail_load_unexpected!("None")
    }
}

/// A trait for writing a Stencila node to an Automerge store
pub trait Write {
    /// Write a Stencila node to an Automerge store
    fn write(&self, store: &mut WriteStore, path: &Path, message: &str) -> Result<()> {
        self.dump(store)?;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards?!")
            .as_secs() as i64;

        let options = CommitOptions::default()
            .with_time(time)
            .with_message(message);
        store.commit_with(options);

        let bytes = store.save();
        write(path, bytes)?;

        Ok(())
    }

    /// Dump a Stencila node to an Automerge store
    ///
    /// Because Automerge stores must have a map at the root, this method calls
    /// the `dump_map` method. As such it will fail if that method is not
    /// implemented for the node type (e.g. `Number` and `Vec` nodes).
    fn dump(&self, store: &mut WriteStore) -> Result<()> {
        self.sync_map(store, &ROOT)
    }

    /// Dump a Stencila node to a map in an Automerge store
    ///
    /// This method is used to dump a node to an existing object in an Automerge
    /// store. It only needs to be implemented for node types that are represented
    /// as maps in an Automerge store.
    fn sync_map(&self, _store: &mut WriteStore, _obj_id: &ObjId) -> Result<()> {
        bail_type!("method `Write::sync_map` not implemented for type `{type}`")
    }

    /// Insert a node into a new property of an Automerge store
    fn insert_prop(&self, _store: &mut WriteStore, _obj_id: &ObjId, _prop: Prop) -> Result<()> {
        bail_type!("method `Write::insert_prop` not implemented for type `{type}`")
    }

    /// Put a node into an existing property of an Automerge store
    fn put_prop(&self, _store: &mut WriteStore, _obj_id: &ObjId, _prop: Prop) -> Result<()> {
        bail_type!("method `Write::put_prop` not implemented for type `{type}`")
    }

    /// Calculate the similarity index between the current node and a property in an Automerge store
    ///
    /// The similarity index is used as part of the diffing algorithm.
    fn similarity<S: ReadStore>(&self, _store: &S, _obj_id: &ObjId, _prop: Prop) -> Result<usize> {
        bail_type!("method `Write::similarity` not implemented for type `{type}`")
    }
}

/// Load an Automerge store into memory
pub fn load_store(path: &Path) -> Result<WriteStore> {
    if !path.exists() {
        bail!("Path `{}` does not exist", path.display());
    }
    if path.is_dir() {
        bail!("Path `{}` is a directory; expected a file", path.display());
    }

    let bytes = read(path)?;
    let store = WriteStore::load(&bytes)
        .wrap_err_with(|| format!("Unable to open file `{}`", path.display()))?;
    Ok(store)
}

/// Inspect an Automerge store by serializing it to JSON
pub fn inspect_store<S: ReadStore>(store: &S) -> Result<String> {
    Ok(common::serde_json::to_string_pretty(
        &automerge::AutoSerde::from(store),
    )?)
}

/// Get the `type` property of an object in an Automerge store
pub fn get_type<T, S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<String> {
    let type_name = std::any::type_name::<T>();

    let Some((value,..)) = store.get(obj_id, Prop::from("type"))? else {
        bail!("No `type` property in Automerge store for type `{type_name}`");
    };

    let Value::Scalar(value) = value else {
        bail!("Expected `type` property for type `{type_name}` in Automerge store to be a scalar");
    };

    let ScalarValue::Str(value) = value.as_ref() else {
        bail!("Expected `type` property for type `{type_name}` in Automerge store to be a string");
    };

    Ok(value.to_string())
}

/// Serialize an Automerge object id as a Base64 string
pub fn id_to_base64(obj_id: &ObjId) -> String {
    use common::base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};

    let bytes = obj_id.to_bytes();
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

/// Deserialize a Base64 string to an Automerge object id
pub fn base64_to_id(base64: &str) -> Result<ObjId> {
    use common::base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};

    let bytes = BASE64_URL_SAFE_NO_PAD.decode(base64)?;
    let id = ObjId::try_from(bytes.as_slice())?;
    Ok(id)
}
