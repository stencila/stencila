use common::eyre::Result;

use crate::prelude::*;

impl<T> Read for Vec<T>
where
    T: Read + std::fmt::Debug,
{
    fn load_list<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        // Load the items into a new vec
        let mut vec = Vec::new();
        for (index, ..) in store.list_range(obj, ..) {
            let node = T::load_prop(store, obj, index.into())?;
            vec.push(node);
        }

        Ok(vec)
    }

    fn load_none() -> Result<Self> {
        // If None where vec expected return empty vec
        Ok(Vec::new())
    }
}

impl<T> Write for Vec<T>
where
    T: Write + std::fmt::Debug,
{
    fn insert_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        // Create the new list in the store
        let prop_obj_id = match prop {
            Prop::Map(key) => store.put_object(obj, key, ObjType::List)?,
            Prop::Seq(index) => store.insert_object(obj, index, ObjType::List)?,
        };

        // Insert each item into that new list
        for (index, node) in self.iter().enumerate() {
            node.insert_prop(store, &prop_obj_id, index.into())?;
        }

        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        // Get the existing object at the property
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::List), prop_obj)) = existing {
            // Existing object is a map so dump to it
            // TODO: correlate nodes with existing ones: create two arrays with unique id
            // (but shared on both sides) then do a patience diff to compare
            for (index, node) in self.iter().enumerate() {
                node.put_prop(store, &prop_obj, index.into())?;
            }
        } else {
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }
            self.insert_prop(store, obj, prop)?;
        }

        Ok(())
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::List), _prop_obj_id)) = store.get(obj, prop)? {
            // TODO
        }

        Ok(0)
    }
}
