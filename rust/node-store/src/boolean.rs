use crate::prelude::*;

impl Read for bool {
    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(*value)
    }

    fn load_none() -> Result<Self> {
        Ok(Self::default())
    }
}

impl Write for bool {
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

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj_id, prop)? {
            if let ScalarValue::Boolean(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }
}
