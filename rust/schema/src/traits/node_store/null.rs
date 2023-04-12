use common::eyre::Result;
use node_store::{
    automerge::{transaction::Transactable, ObjId, Prop, ScalarValue, Value},
    Read, ReadStore, Write, WriteStore, SIMILARITY_MAX,
};

use crate::Null;

impl Read for Null {
    fn load_null() -> Result<Self> {
        Ok(Self {})
    }
}

impl Write for Null {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match prop {
            Prop::Map(key) => store.put(obj_id, key, ScalarValue::Null)?,
            Prop::Seq(index) => store.insert(obj_id, index, ScalarValue::Null)?,
        };
        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        Ok(store.put(obj_id, prop, ScalarValue::Null)?)
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj_id, prop)? {
            if let ScalarValue::Null = *scalar {
                return Ok(SIMILARITY_MAX);
            }
        }
        Ok(0)
    }
}
