use automerge::iter::ListRangeItem;
use common::eyre::Result;

use crate::prelude::*;

impl<T> ReadNode for Vec<T>
where
    T: ReadNode + std::fmt::Display,
{
    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        // Load the items into a new vec
        let mut vec = Vec::new();
        for ListRangeItem { index, .. } in store.list_range(obj, ..) {
            let node = T::load_prop(store, obj, index.into())?;
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
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        // Create the new list in the store
        let prop_obj_id = match prop {
            Prop::Map(key) => store.put_object(obj_id, key, ObjType::List)?,
            Prop::Seq(index) => store.insert_object(obj_id, index, ObjType::List)?,
        };

        // Insert each item into that new list
        for (index, node) in self.iter().enumerate() {
            node.insert_prop(store, &prop_obj_id, index.into())?;
        }

        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        // Get the existing object at the property
        let existing = store.get(obj_id, prop.clone())?;

        if let Some((Value::Object(ObjType::List), prop_obj_id)) = existing {
            // Existing object is a map so dump to it
            // TODO: correlate nodes with existing ones: create two arrays with unique id
            // (but shared on both sides) then do a patience diff to compare
            for (index, node) in self.iter().enumerate() {
                node.put_prop(store, &prop_obj_id, index.into())?;
            }

            // Delete any extra items in the store
            for index in self.len()..store.length(prop_obj_id.clone()) {
                store.delete(prop_obj_id.clone(), Prop::Seq(index))?;
            }
        } else {
            if existing.is_some() {
                store.delete(obj_id, prop.clone())?;
            }
            self.insert_prop(store, obj_id, prop)?;
        }

        Ok(())
    }
}
