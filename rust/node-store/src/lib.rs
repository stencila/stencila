//! Interface between Stencila Schema and Automerge

use std::path::Path;
use std::time::SystemTime;

use automerge::ROOT;
use smol_str::SmolStr;

use common::{
    async_trait::async_trait,
    eyre::{bail, Context, Result},
    tokio::fs::{read, write},
};
use node_strip::StripNode;

pub use automerge::{
    self, AutoCommit as WriteStore, ChangeHash as CommitHash, ObjId, ObjType, Prop,
    ReadDoc as ReadStore,
};
pub(crate) use automerge::{transaction::CommitOptions, ScalarValue, Value};

pub use node_store_derive::{ReadNode, WriteNode};

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

/// The maximum similarity index between to nodes
pub const SIMILARITY_MAX: usize = 1000;

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
            "Unexpected Automerge `{unexpected}` while attempting to load `{type}` from store",
            unexpected = $unexpected,
            type = std::any::type_name::<Self>()
        )
    };
}

/// A trait for reading Stencila Schema nodes from an Automerge store
#[async_trait]
pub trait ReadNode: StripNode + Sized {
    /// Read a Stencila Schema node from an Automerge file
    async fn read(path: &Path) -> Result<(WriteStore, Self)> {
        let store = load_store(path).await?;

        // If the following call to `Self::load` fails it can be useful to use `inspect_store(&store)?`
        // to inspect the shape of the data in the store
        inspect_store(&store)?;

        let node = Self::load(&store)?;

        Ok((store, node))
    }

    /// Load a Stencila Schema node from an Automerge store
    ///
    /// Because Automerge stores must have a map at the root, this method calls
    /// the `load_map` method. As such it will fail if that method is not
    /// implemented for the node type (e.g. `Number` and `Vec` nodes).
    fn load<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_map(store, &ROOT)
    }

    /// Load the Stencila Schema node from a property for an object in an Automerge store
    fn load_from<S: ReadStore>(&mut self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<()> {
        *self = Self::load_prop(store, obj_id, prop)?;

        Ok(())
    }

    /// Load a new Stencila Schema node from a property for an object in an Automerge store
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

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Null`]
    fn load_null() -> Result<Self> {
        bail_load_unexpected!("Null")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Boolean`]
    fn load_boolean(_value: &bool) -> Result<Self> {
        bail_load_unexpected!("Boolean")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Int`]
    fn load_int(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Int")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Uint`]
    fn load_uint(_value: &u64) -> Result<Self> {
        bail_load_unexpected!("Uint")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::F64`]
    fn load_f64(_value: &f64) -> Result<Self> {
        bail_load_unexpected!("F64")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Str`]
    fn load_str(_value: &SmolStr) -> Result<Self> {
        bail_load_unexpected!("Str")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Counter`]
    fn load_counter(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Counter")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Timestamp`]
    fn load_timestamp(_value: &i64) -> Result<Self> {
        bail_load_unexpected!("Timestamp")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Bytes`]
    fn load_bytes(_value: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Bytes")
    }

    /// Load a Stencila Schema node from an [`automerge::ScalarValue::Unknown`]
    fn load_unknown(_type_code: u8, _bytes: &[u8]) -> Result<Self> {
        bail_load_unexpected!("Unknown")
    }

    /// Load a Stencila Schema node from an [`automerge::ObjType::Text`] value
    fn load_text<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Text")
    }

    /// Load a Stencila Schema node from an [`automerge::ObjType::List`]
    fn load_list<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("List")
    }

    /// Load a Stencila Schema node from an [`automerge::ObjType::Map`]
    fn load_map<S: ReadStore>(_store: &S, _obj_id: &ObjId) -> Result<Self> {
        bail_load_unexpected!("Map")
    }

    /// Load a Stencila Schema node from a `None` value
    fn load_none() -> Result<Self> {
        bail_load_unexpected!("None")
    }
}

/// A trait for writing a Stencila node to an Automerge store
#[async_trait]
pub trait WriteNode {
    /// Write a Stencila node to an Automerge store
    async fn write(
        &self,
        store: &mut WriteStore,
        path: &Path,
        message: &str,
    ) -> Result<CommitHash> {
        self.dump(store)?;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards?!")
            .as_secs() as i64;

        // Create commit options: `CommitOptions` is not `Clone`,
        // so this closure is just to keep the following DRY
        let options = || {
            CommitOptions::default()
                .with_time(time)
                .with_message(message)
        };

        let change = store.commit_with(options()).unwrap_or_else(|| {
            // If there were no changes to commit, then
            // create an "empty change"
            store.empty_change(options())
        });

        let bytes = store.save();
        write(path, bytes).await?;

        Ok(change)
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
pub async fn load_store(path: &Path) -> Result<WriteStore> {
    if !path.exists() {
        bail!("Path `{}` does not exist", path.display());
    }
    if path.is_dir() {
        bail!("Path `{}` is a directory; expected a file", path.display());
    }

    let bytes = read(path).await?;
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

/// Get the `NodeType` of an object in an Automerge store
pub fn get_type<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Option<String>> {
    // This function is normally only be called for Stencila struct types (not for primitives)
    // However, if the Automerge object is not a `Map` the following `get` call will panic!
    // So its important to do this check, and return the closest Stencila type to the
    // Automerge type.
    match store.object_type(obj_id)? {
        ObjType::List => return Ok(Some("Array".to_string())),
        ObjType::Text => return Ok(Some("String".to_string())),
        _ => {}
    };

    let Some((value,..)) = store.get(obj_id, Prop::from("type"))? else {
        return Ok(None)
    };

    let Value::Scalar(value) = value else {
        bail!("Expected `type` property to be a scalar");
    };

    let ScalarValue::Str(value) = value.as_ref() else {
        bail!("Expected `type` property to be a string");
    };

    Ok(Some(value.to_string()))
}
