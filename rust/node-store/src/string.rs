use common::smol_str::SmolStr;

use crate::prelude::*;

impl ReadNode for String {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(value.to_string())
    }

    fn load_none() -> Result<Self> {
        Ok(Self::default())
    }
}

impl WriteNode for String {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match prop {
            Prop::Map(key) => store.put(obj_id, key, self)?,
            Prop::Seq(index) => store.insert(obj_id, index, self)?,
        };
        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        Ok(store.put(obj_id, prop, self)?)
    }
}
