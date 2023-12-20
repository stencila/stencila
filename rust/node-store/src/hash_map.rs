use std::collections::{HashMap, HashSet};

use automerge::iter::MapRangeItem;

use crate::prelude::*;

impl<T> ReadNode for HashMap<String, T>
where
    T: ReadNode,
{
    fn load_map<C: ReadCrdt>(crdt: &C, obj_id: &ObjId) -> Result<Self> {
        let mut map = Self::new();
        for MapRangeItem { key, .. } in crdt.map_range(obj_id, ..) {
            let node = T::load_prop(crdt, obj_id, key.into())?;
            map.insert(key.to_string(), node);
        }

        Ok(map)
    }

    fn load_none() -> Result<Self> {
        Ok(Self::default())
    }
}

impl<T> WriteNode for HashMap<String, T>
where
    T: WriteNode,
{
    fn sync_map(&self, crdt: &mut WriteCrdt, map: &mut StoreMap, obj_id: &ObjId) -> Result<()> {
        // Get all the keys for the map in the CRDT
        let mut keys: HashSet<String> = crdt.keys(obj_id).collect();

        // Update values for keys that are in both map and crdt
        for (key, node) in self.iter() {
            node.put_prop(crdt, map, obj_id, key.into())?;
            keys.remove(key);
        }

        // Remove keys that are in the CRDT but not in map
        for key in keys {
            crdt.delete(obj_id, key.as_str())?;
        }

        Ok(())
    }

    fn insert_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        // Create the new map in the CRDT
        let prop_obj_id = match prop {
            Prop::Map(key) => crdt.put_object(obj_id, key, ObjType::Map)?,
            Prop::Seq(index) => crdt.insert_object(obj_id, index, ObjType::Map)?,
        };

        // Insert each key into that new map
        for (key, node) in self.iter() {
            node.insert_prop(crdt, map, &prop_obj_id, key.into())?;
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

        if let Some((Value::Object(ObjType::Map), prop_obj_id)) = existing {
            // Existing object is a map so sync it
            self.sync_map(crdt, map, &prop_obj_id)
        } else {
            // Remove any existing object of different type
            if existing.is_some() {
                crdt.delete(obj_id, prop.clone())?;
            }

            // Insert a new map object
            self.insert_prop(crdt, map, obj_id, prop)?;

            Ok(())
        }
    }

    fn similarity<C: ReadCrdt>(&self, crdt: &C, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Map), _prop_obj_id)) = crdt.get(obj_id, prop)? {
            // TODO
        }
        Ok(0)
    }
}
