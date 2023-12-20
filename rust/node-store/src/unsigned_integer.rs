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
    fn insert_prop(
        &self,
        crdt: &mut WriteCrdt,
        _map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        match prop {
            Prop::Map(key) => crdt.put(obj_id, key, *self)?,
            Prop::Seq(index) => crdt.insert(obj_id, index, *self)?,
        };
        Ok(())
    }

    fn put_prop(
        &self,
        crdt: &mut WriteCrdt,
        _map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        Ok(crdt.put(obj_id, prop, *self)?)
    }

    fn similarity<C: ReadCrdt>(&self, crdt: &C, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = crdt.get(obj_id, prop)? {
            if let ScalarValue::Uint(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }
}
