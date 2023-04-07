use std::time::Duration;

use automerge::ObjType;
use similar::{Algorithm, DiffTag, TextDiffConfig};

use common::eyre::Result;
use schema::Text;

use crate::prelude::*;

impl Read for Text {
    fn load_text<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        let id = Some(obj_id_to_base64(obj));
        let value = store.text(obj)?;

        Ok(Self {
            id,
            value,
            ..Default::default()
        })
    }
}

impl Write for Text {
    fn similarity<S: ReadStore>(&self, store: &S, obj: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Object(ObjType::Text), prop_obj)) = store.get(obj, prop)? {
            if let Some(id) = self.id.as_deref() {
                if obj_id_from_base64(id)? == *obj {
                    return Ok(SIMILARITY_MAX);
                }
            }

            let value = store.text(prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, &self.value);

            return Ok((diff.ratio() * SIMILARITY_MAX as f32) as usize);
        }

        Ok(0)
    }

    fn dump_new(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        let prop_obj = dump_new_object(store, obj, prop, ObjType::Text)?;
        store.splice_text(prop_obj, 0, 0, &self.value)?;

        Ok(())
    }

    fn dump_prop(&self, store: &mut WriteStore, obj: &ObjId, prop: Prop) -> Result<()> {
        let existing = store.get(obj, prop.clone())?;

        if let Some((Value::Object(ObjType::Text), prop_obj)) = existing {
            // Existing property is text, so get its value, diff it with the
            // current value and apply diff operations as `splice_text` operations
            let value = store.text(&prop_obj)?;

            let diff = TextDiffConfig::default()
                .algorithm(Algorithm::Patience)
                .timeout(Duration::from_secs(15))
                .diff_graphemes(&value, &self.value);

            let mut pos = 0usize;
            for op in diff.ops() {
                match op.tag() {
                    DiffTag::Insert => {
                        let insert = &self.value[op.new_range()];
                        store.splice_text(&prop_obj, pos, 0, insert)?;
                    }
                    DiffTag::Delete => {
                        let delete = op.old_range().len();
                        store.splice_text(&prop_obj, pos, delete, "")?;
                    }
                    DiffTag::Replace => {
                        let delete = op.old_range().len();
                        let insert = &self.value[op.new_range()];
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
            self.dump_new(store, obj, prop)?;
        }

        Ok(())
    }
}
