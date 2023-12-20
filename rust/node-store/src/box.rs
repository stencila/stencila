use crate::prelude::*;

impl<T> ReadNode for Box<T>
where
    T: ReadNode,
{
    fn load_prop<C: ReadCrdt>(crdt: &C, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        Ok(Box::new(T::load_prop(crdt, obj_id, prop)?))
    }
}

impl<T> WriteNode for Box<T>
where
    T: WriteNode,
{
    fn insert_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        self.as_ref().insert_prop(crdt, map, obj_id, prop)
    }

    fn put_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        self.as_ref().put_prop(crdt, map, obj_id, prop)
    }
}
