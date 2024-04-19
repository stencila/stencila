pub(crate) use automerge::{
    transaction::Transactable, AutoCommit as WriteStore, ObjId, ObjType, Prop,
    ReadDoc as ReadStore, Value,
};

pub(crate) use common::eyre::Result;

pub(crate) use crate::{ReadNode, WriteNode};
