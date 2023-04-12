use common::eyre::Result;

use crate::prelude::*;

impl<T> Read for Option<T>
where
    T: Read,
{
    fn load_prop<S: ReadStore>(store: &S, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        match store.get(obj_id, prop.clone())? {
            // There is a value in the store for the property so load using type
            Some(..) => Ok(Some(T::load_prop(store, obj_id, prop)?)),
            // There is no value in the store for the property so return None
            None => Ok(None),
        }
    }
}

impl<T> Write for Option<T>
where
    T: Write,
{
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match self {
            // There is a value so insert it into the store
            Some(value) => value.insert_prop(store, obj_id, prop),
            // There is no value so do nothing
            None => Ok(()),
        }
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match self {
            // There is a value so put it in the store
            Some(value) => value.put_prop(store, obj_id, prop),
            None => {
                // There is no value so remove it from the store
                store.delete(obj_id, prop)?;
                Ok(())
            }
        }
    }
}
