use automerge::iter::ListRangeItem;

use crate::prelude::*;

impl<T> ReadNode for Vec<T>
where
    T: ReadNode + std::fmt::Display,
{
    fn load_list<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        // Load the items into a new vec
        let mut vec = Vec::new();
        for ListRangeItem { index, .. } in crdt.list_range(obj, ..) {
            let node = T::load_prop(crdt, obj, index.into())?;
            vec.push(node);
        }

        Ok(vec)
    }

    fn load_none() -> Result<Self> {
        Ok(Self::default())
    }
}

impl<T> WriteNode for Vec<T>
where
    T: WriteNode + std::fmt::Debug,
{
    fn insert_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        // Create the new list in the CRDT
        let prop_obj_id = match prop {
            Prop::Map(key) => crdt.put_object(obj_id, key, ObjType::List)?,
            Prop::Seq(index) => crdt.insert_object(obj_id, index, ObjType::List)?,
        };

        // Insert each item into that new list
        for (index, node) in self.iter().enumerate() {
            node.insert_prop(crdt, map, &prop_obj_id, index.into())?;
        }

        Ok(())
    }

    fn put_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        // Get the existing object at the property
        let existing = crdt.get(obj_id, prop.clone())?;

        if let Some((Value::Object(ObjType::List), prop_obj_id)) = existing {
            // Existing object is a map so dump to it
            // TODO: correlate nodes with existing ones: create two arrays with unique id
            // (but shared on both sides) then do a patience diff to compare
            for (index, node) in self.iter().enumerate() {
                node.put_prop(crdt, map, &prop_obj_id, index.into())?;
            }

            // Delete any extra items in the CRDT
            for index in self.len()..crdt.length(prop_obj_id.clone()) {
                crdt.delete(prop_obj_id.clone(), Prop::Seq(index))?;
            }
        } else {
            if existing.is_some() {
                crdt.delete(obj_id, prop.clone())?;
            }
            self.insert_prop(crdt, map, obj_id, prop)?;
        }

        Ok(())
    }

    fn similarity<C: ReadCrdt>(&self, crdt: &C, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::List), _prop_obj_id)) = crdt.get(obj, prop)? {
            // TODO
        }

        Ok(0)
    }
}
