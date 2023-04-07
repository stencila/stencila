use schema::Boolean;

use crate::prelude::*;

impl Read for Boolean {
    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(*value)
    }
}

impl Write for Boolean {
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Boolean(value) = *scalar {
                if value == *self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, *self)
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, *self)
    }
}
