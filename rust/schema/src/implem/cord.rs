use std::time::Duration;

use codec_html_trait::encode::text;
use common::similar::{Algorithm, DiffTag, TextDiffConfig};
use node_store::{
    automerge::{transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadCrdt, ReadNode, StoreMap, WriteCrdt, WriteNode, SIMILARITY_MAX,
};

use crate::{prelude::*, Cord};

impl StripNode for Cord {}

impl ReadNode for Cord {
    fn load_text<C: ReadCrdt>(crdt: &C, obj_id: &ObjId) -> Result<Self> {
        Ok(Self::new(crdt.text(obj_id)?))
    }
}

impl WriteNode for Cord {
    fn insert_prop(
        &self,
        crdt: &mut WriteCrdt,
        _map: &mut StoreMap,
        obj_id: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        // Create the new text object in the CRDT
        let prop_obj_id = match prop {
            Prop::Map(key) => crdt.put_object(obj_id, key, ObjType::Text)?,
            Prop::Seq(index) => crdt.insert_object(obj_id, index, ObjType::Text)?,
        };

        // Splice in all of the new text
        crdt.splice_text(prop_obj_id, 0, 0, self)?;

        Ok(())
    }

    fn put_prop(
        &self,
        crdt: &mut WriteCrdt,
        map: &mut StoreMap,
        obj: &ObjId,
        prop: Prop,
    ) -> Result<()> {
        // Get the existing object at the property
        let existing = crdt.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Text), prop_obj)) = existing {
            // Existing property is text, so get its value, diff it with the
            // current value and apply diff operations as `splice_text` operations
            let value = crdt.text(&prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, self);

            let mut pos = 0usize;
            for op in diff.ops() {
                match op.tag() {
                    DiffTag::Insert => {
                        let insert = &self[op.new_range()];
                        crdt.splice_text(&prop_obj, pos, 0, insert)?;
                    }
                    DiffTag::Delete => {
                        let delete = op.old_range().len() as isize;
                        crdt.splice_text(&prop_obj, pos, delete, "")?;
                    }
                    DiffTag::Replace => {
                        let delete = op.old_range().len() as isize;
                        let insert = &self[op.new_range()];
                        crdt.splice_text(&prop_obj, pos, delete, insert)?;
                    }
                    DiffTag::Equal => {}
                }
                pos += op.new_range().len();
            }
        } else {
            // Remove any existing property of different type
            if existing.is_some() {
                crdt.delete(obj, prop.clone())?;
            }

            // Insert a new `Text` object
            self.insert_prop(crdt, map, obj, prop)?;
        }

        Ok(())
    }

    fn similarity<C: ReadCrdt>(&self, crdt: &C, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Text), prop_obj)) = crdt.get(obj, prop)? {
            let value = crdt.text(prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, self);

            return Ok((diff.ratio() * SIMILARITY_MAX as f32) as usize);
        }

        Ok(0)
    }
}

impl HtmlCodec for Cord {
    fn to_html(&self, _context: &mut HtmlEncodeContext) -> String {
        text(self)
    }

    fn to_html_parts(&self, _context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        unreachable!("should not be called for text value")
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl MarkdownCodec for Cord {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.push_str(&self.to_string());
    }
}

impl TextCodec for Cord {
    fn to_text(&self) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}
