use std::collections::HashSet;

use codec_html_trait::encode::{attr, elem};
use node_store::{
    automerge::{iter::MapRangeItem, transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadCrdt, ReadNode, StoreMap, WriteCrdt, WriteNode,
};

use crate::{prelude::*, Object, Primitive};

impl StripNode for Object {}

impl ReadNode for Object {
    fn load_map<C: ReadCrdt>(crdt: &C, obj_id: &ObjId) -> Result<Self> {
        let mut map = Self::new();
        for MapRangeItem { key, .. } in crdt.map_range(obj_id, ..) {
            let node = Primitive::load_prop(crdt, obj_id, key.into())?;
            map.insert(key.to_string(), node);
        }

        Ok(map)
    }
}

impl WriteNode for Object {
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

impl HtmlCodec for Object {
    fn to_html_parts(&self, context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        // Uses spans, rather than say <ul>/<li> because needs to be
        // include e.g for output of a `CodeExpression`
        (
            "stencila-object",
            vec![],
            self.iter()
                .map(|(key, value)| {
                    elem(
                        "stencila-object-item",
                        &[attr("key", key)],
                        &[value.to_html(context)],
                    )
                })
                .collect_vec(),
        )
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl JatsCodec for Object {
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        let (content, losses) = self.to_text();
        (String::new(), Vec::new(), content, losses)
    }
}

impl MarkdownCodec for Object {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let (text, losses) = self.to_text();
        context.push_str(&text);
        context.merge_losses(losses);
    }
}

impl TextCodec for Object {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::one("Object#");

        for value in self.values() {
            if !text.is_empty() {
                text.push(' ');
            }

            let (value_text, value_losses) = value.to_text();
            text.push_str(&value_text);
            losses.merge(value_losses);
        }

        if !text.is_empty() {
            text.push(' ');
        }

        (text, losses)
    }
}
