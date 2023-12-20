pub(crate) use automerge::{
    transaction::Transactable, AutoCommit as WriteCrdt, ObjId, ObjType, Prop, ReadDoc as ReadCrdt,
    ScalarValue, Value,
};

pub(crate) use common::{eyre::Result, smol_str::SmolStr};

pub(crate) use crate::{ReadNode, StoreMap, WriteNode, SIMILARITY_MAX};
