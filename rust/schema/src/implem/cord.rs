use std::time::Duration;

use codec_html_trait::encode::text;
use common::similar::{Algorithm, DiffTag, TextDiffConfig};
use node_store::{
    automerge::{transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadNode, ReadStore, WriteNode, WriteStore, SIMILARITY_MAX,
};

use crate::{prelude::*, Cord};

impl StripNode for Cord {}

impl PatchNode for Cord {
    fn condense(&self, context: &mut CondenseContext) {
        context.collect_value(&self.to_string());
    }
}

impl ReadNode for Cord {
    fn load_text<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Self::new(store.text(obj_id)?))
    }
}

impl WriteNode for Cord {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        // Create the new text object in the store
        let prop_obj_id = match prop {
            Prop::Map(key) => store.put_object(obj_id, key, ObjType::Text)?,
            Prop::Seq(index) => store.insert_object(obj_id, index, ObjType::Text)?,
        };

        // Splice in all of the new text
        store.splice_text(prop_obj_id, 0, 0, self)?;

        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        // Get the existing object at the property
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Text), prop_obj)) = existing {
            // Existing property is text, so get its value, diff it with the
            // current value and apply diff operations as `splice_text` operations
            let value = store.text(&prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, self);

            let mut pos = 0usize;
            for op in diff.ops() {
                match op.tag() {
                    DiffTag::Insert => {
                        let insert = &self[op.new_range()];
                        store.splice_text(&prop_obj, pos, 0, insert)?;
                    }
                    DiffTag::Delete => {
                        let delete = op.old_range().len() as isize;
                        store.splice_text(&prop_obj, pos, delete, "")?;
                    }
                    DiffTag::Replace => {
                        let delete = op.old_range().len() as isize;
                        let insert = &self[op.new_range()];
                        store.splice_text(&prop_obj, pos, delete, insert)?;
                    }
                    DiffTag::Equal => {}
                }
                pos += op.new_range().len();
            }
        } else {
            // Remove any existing property of different type
            if existing.is_some() {
                store.delete(obj, prop.clone())?;
            }

            // Insert a new `Text` object
            self.insert_prop(store, obj, prop)?;
        }

        Ok(())
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Text), prop_obj)) = store.get(obj, prop)? {
            let value = store.text(prop_obj)?;

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
