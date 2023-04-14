use common::eyre::Result;
use node_store::{
    automerge::{ObjId, Prop},
    Read, ReadStore, Write, WriteStore,
};

use crate::{Array, Primitive};

impl Read for Array {
    fn load_list<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_list(store, obj_id)?))
    }

    fn load_none() -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_none()?))
    }
}

impl Write for Array {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.0.insert_prop(store, obj_id, prop)
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.0.put_prop(store, obj_id, prop)
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        self.0.similarity(store, obj_id, prop)
    }
}
