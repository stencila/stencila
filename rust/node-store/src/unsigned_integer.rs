use crate::prelude::*;

impl ReadNode for u64 {
    fn load_uint(value: &u64) -> Result<Self> {
        Ok(*value)
    }

    fn load_none() -> Result<Self> {
        Ok(Self::default())
    }
}

impl WriteNode for u64 {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match prop {
            Prop::Map(key) => store.put(obj_id, key, *self)?,
            Prop::Seq(index) => store.insert(obj_id, index, *self)?,
        };
        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        Ok(store.put(obj_id, prop, *self)?)
    }
}
