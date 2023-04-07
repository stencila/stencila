use schema::Null;

use crate::prelude::*;

impl Read for Null {
    fn load_null() -> Result<Self> {
        Ok(Self {})
    }
}

impl Write for Null {
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Null = *scalar {
                return Ok(SIMILARITY_MAX);
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, ())
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, ())
    }
}
