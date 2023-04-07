pub(crate) use automerge::{
    transaction::Transactable, AutoCommit as WriteStore, ObjId, ObjType, Prop,
    ReadDoc as ReadStore, ScalarValue, Value, ROOT,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};

pub(crate) use common::eyre::Result;

pub(crate) use crate::{Read, Write};

/// The maximum similarity index between to nodes
pub(crate) const SIMILARITY_MAX: usize = 1000;

/// Bail from a function with the type of node included in the message
#[macro_export]
macro_rules! bail_type {
    ($message:literal) => {
        common::eyre::bail!($message, type = std::any::type_name::<Self>())
    };
}

/// Bail from a `load` function with the unexpected and expected type
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

pub(crate) fn dump_new_scalar<S: Into<ScalarValue>>(
    store: &mut WriteStore,
    obj: &ObjId,
    prop: Prop,
    scalar: S,
) -> Result<()> {
    match prop {
        Prop::Map(key) => store.put(obj, key, scalar)?,
        Prop::Seq(index) => store.insert(obj, index, scalar)?,
    };

    Ok(())
}

pub(crate) fn dump_prop_scalar<S: Into<ScalarValue>>(
    store: &mut WriteStore,
    obj: &ObjId,
    prop: Prop,
    scalar: S,
) -> Result<()> {
    store.put(obj, prop, scalar)?;

    Ok(())
}

pub(crate) fn dump_new_object(
    store: &mut WriteStore,
    obj: &ObjId,
    prop: Prop,
    obj_type: ObjType,
) -> Result<ObjId> {
    let id = match prop {
        Prop::Map(key) => store.put_object(obj, key, obj_type)?,
        Prop::Seq(index) => store.insert_object(obj, index, obj_type)?,
    };

    Ok(id)
}

/// Convert an Automerge [`ObjId`] to a Base64 string
pub(crate) fn obj_id_to_base64(obj: &ObjId) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(obj.to_bytes())
}

/// Convert a Base64 string to an Automerge [`ObjId`]
pub(crate) fn obj_id_from_base64<S: AsRef<str>>(base64: S) -> Result<ObjId> {
    let bytes = BASE64_URL_SAFE_NO_PAD.decode(base64.as_ref().as_bytes())?;
    Ok(ObjId::try_from(bytes.as_slice())?)
}
