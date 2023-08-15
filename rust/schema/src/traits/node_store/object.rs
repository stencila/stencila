use std::collections::HashSet;

use common::eyre::Result;
use node_store::{
    automerge::{iter::MapRangeItem, transaction::Transactable, ObjId, ObjType, Prop, Value},
    Read, ReadStore, Write, WriteStore,
};

use crate::{Object, Primitive};

impl Read for Object {
    fn load_map<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        let mut map = Self::new();
        for MapRangeItem { key, .. } in store.map_range(obj_id, ..) {
            let node = Primitive::load_prop(store, obj_id, key.into())?;
            map.insert(key.to_string(), node);
        }

        Ok(map)
    }
}

impl Write for Object {
    fn sync_map(&self, store: &mut WriteStore, obj_id: &ObjId) -> Result<()> {
        // Get all the keys for the map in the store
        let mut keys: HashSet<String> = store.keys(obj_id).collect();

        // Update values for keys that are in both map and store
        for (key, node) in self.iter() {
            node.put_prop(store, obj_id, key.into())?;
            keys.remove(key);
        }

        // Remove keys that are in the store but not in map
        for key in keys {
            store.delete(obj_id, key.as_str())?;
        }

        Ok(())
    }

    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        // Create the new map in the store
        let prop_obj_id = match prop {
            Prop::Map(key) => store.put_object(obj_id, key, ObjType::Map)?,
            Prop::Seq(index) => store.insert_object(obj_id, index, ObjType::Map)?,
        };

        // Insert each key into that new map
        for (key, node) in self.iter() {
            node.insert_prop(store, &prop_obj_id, key.into())?;
        }

        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        // Get the existing object at the property
        let existing = store.get(obj_id, prop.clone())?;

        if let Some((Value::Object(ObjType::Map), prop_obj_id)) = existing {
            // Existing object is a map so sync it
            self.sync_map(store, &prop_obj_id)
        } else {
            // Remove any existing object of different type
            if existing.is_some() {
                store.delete(obj_id, prop.clone())?;
            }

            // Insert a new map object
            self.insert_prop(store, obj_id, prop)?;

            Ok(())
        }
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Map), _prop_obj_id)) = store.get(obj_id, prop)? {
            // TODO
        }
        Ok(0)
    }
}
