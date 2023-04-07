use smol_str::SmolStr;

use common::eyre::Result;

use crate::prelude::*;

impl Read for String {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(value.to_string())
    }
}

impl Write for String {
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj, prop)? {
            if let ScalarValue::Str(value) = scalar.as_ref() {
                if value == self {
                    return Ok(SIMILARITY_MAX);
                }
            }
        }
        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_new_scalar(store, obj, prop, self)
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        dump_prop_scalar(store, obj, prop, self)
    }
}
