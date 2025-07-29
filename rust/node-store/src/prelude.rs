pub(crate) use automerge::{
    AutoCommit as WriteStore, ObjId, ObjType, Prop, ReadDoc as ReadStore, Value,
    transaction::Transactable,
};

pub(crate) use common::eyre::Result;

pub(crate) use crate::{ReadNode, WriteNode};
