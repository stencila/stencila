use std::collections::HashSet;

use common::indexmap::IndexMap;

use crate::prelude::*;

impl<T> Read for IndexMap<String, T>
where
    T: Read,
{
    fn load_root<S: ReadStore>(store: &S) -> Result<Self> {
        Self::load_map(store, &ROOT)
    }

    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let mut map = Self::new();
        for (key, ..) in store.map_range(obj, ..) {
            let node = T::load_prop(store, obj, key.into())?;
            map.insert(key.to_string(), node);
        }

        Ok(map)
    }
}

impl<T> Write for IndexMap<String, T>
where
    T: Write,
{
    fn dump_root(&self, store: &mut WriteStore) -> Result<()> {
        map_dump_existing(store, self, &ROOT)
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Map), _prop_obj)) = store.get(obj, prop)? {}
        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::Map)?;
        for (key, node) in self {
            node.dump_new(store, &prop_obj, key.into())?;
        }

        Ok(())
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Map), prop_obj)) = existing {
            // Existing object is a map so dump to it
            map_dump_existing(store, self, &prop_obj)
        } else {
            // Remove any existing property of different type
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }

            // Insert a new `Map` object
            self.dump_new(store, obj, prop)?;

            Ok(())
        }
    }
}

// Dump a `IndexMap` into an existing store `Map`
fn map_dump_existing<T: Write>(
    store: &mut WriteStore,
    map: &IndexMap<String, T>,
    obj: &ObjId,
) -> Result<()> {
    // Get all the keys for the root map of the store
    let mut keys: HashSet<String> = store.keys(obj).collect();

    // Insert or dump key that are in both map and store
    for (key, node) in map {
        node.dump_prop(store, obj, key.into())?;
        keys.remove(key);
    }

    // Remove keys that are in the store but not in map
    for key in keys {
        store.delete(obj, key.as_str())?;
    }

    Ok(())
}
