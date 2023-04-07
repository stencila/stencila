use common::eyre::Result;
use schema::Integer;

use crate::prelude::*;

impl Read for Integer {
    fn load_int(value: &i64) -> Result<Self> {
        Ok(*value)
    }
}

impl Write for Integer {
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Int(value) = *scalar {
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
