use crate::prelude::*;

impl<T> ReadNode for Box<T>
where
    T: ReadNode,
{
    fn load_prop<S: ReadStore>(store: &S, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        Ok(Box::new(T::load_prop(store, obj_id, prop)?))
    }
}

impl<T> WriteNode for Box<T>
where
    T: WriteNode,
{
    fn sync_map(&self, store: &mut WriteStore, obj_id: &ObjId) -> Result<()> {
        self.as_ref().sync_map(store, obj_id)
    }

    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.as_ref().insert_prop(store, obj_id, prop)
    }

    fn insert_into(&self, store: &mut WriteStore, obj_id: &ObjId) -> Result<()> {
        self.as_ref().insert_into(store, obj_id)
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.as_ref().put_prop(store, obj_id, prop)
    }
}
