use crate::prelude::*;

impl<T> ReadNode for Option<T>
where
    T: ReadNode,
{
    fn load_prop<C: ReadCrdt>(crdt: &C, obj_id: &ObjId, prop: Prop) -> Result<Self> {
        match crdt.get(obj_id, prop.clone())? {
            // There is a value in the CRDT for the property so load using type
            Some(..) => Ok(Some(T::load_prop(crdt, obj_id, prop)?)),
            // There is no value in the CRDT for the property so return None
            None => Ok(None),
        }
    }
}

impl<T> WriteNode for Option<T>
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
        match self {
            // There is a value so insert it into the CRDT
            Some(value) => value.insert_prop(crdt, map, obj_id, prop),
            // There is no value so do nothing
            None => Ok(()),
        }
    }

    fn put_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        match self {
            // There is a value so put it in the CRDT
            Some(value) => value.put_prop(crdt, map, obj_id, prop),
            None => {
                // There is no value so remove it from the CRDT
                crdt.delete(obj_id, prop)?;
                Ok(())
            }
        }
    }
}
