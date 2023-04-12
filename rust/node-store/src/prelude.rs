pub(crate) use automerge::{
    transaction::Transactable, AutoCommit as WriteStore, ObjId, ObjType, Prop,
    ReadDoc as ReadStore, ScalarValue, Value,
};

pub(crate) use common::eyre::Result;

pub(crate) use crate::{Read, Write, SIMILARITY_MAX};
