use crate::prelude::*;

impl<T> Read for Box<T>
where
    T: Read,
{
    fn load_prop<S: ReadStore>(store: &S, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        Ok(Box::new(T::load_prop(store, obj_id, prop)?))
    }
}

impl<T> Write for Box<T>
where
    T: Write,
{
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.as_ref().insert_prop(store, obj_id, prop)
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.as_ref().put_prop(store, obj_id, prop)
    }
}
