use std::time::Duration;

use codec_html_trait::encode::text;
use common::similar::{capture_diff_deadline, Algorithm, DiffTag, TextDiffConfig};
use node_store::{
    automerge::{transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadNode, ReadStore, WriteNode, WriteStore,
};

use crate::{prelude::*, Cord, CordOp};

impl StripNode for Cord {}

impl PatchNode for Cord {
    fn to_value(&self) -> Result<PatchValue> {
        Ok(PatchValue::String(self.to_string()))
    }

    fn from_value(value: PatchValue) -> Result<Self> {
        match value {
            PatchValue::String(value) => Ok(value.into()),
            _ => bail!("Invalid value for Cord"),
        }
    }

    #[allow(unused_variables)]
    fn similarity(&self, other: &Cord, context: &mut PatchContext) -> Result<f32> {
        // Calculate a difference ratio based on Unicode graphemes rather
        // that chars or bytes since that is more semantically meaningful for user
        // changes
        let diff = TextDiffConfig::default()
            .algorithm(Algorithm::Patience)
            .timeout(Duration::from_secs(1))
            .diff_graphemes(self.as_str(), other.as_str());

        // Note minimum similarity because same types
        // This is important because it means a `Cord` will have non-zero
        // similarity with itself, even if all characters change
        Ok(diff.ratio().max(0.00001))
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        if other != self {
            // Calculate diff operations using bytes since those
            let diff_ops = capture_diff_deadline(
                Algorithm::Patience,
                self.as_bytes(),
                0..self.len(),
                other.as_bytes(),
                0..other.len(),
                None,
            );

            // Convert them to `CordOp`s
            let mut cord_ops = Vec::new();
            let mut pos = 0usize;
            for op in diff_ops {
                match op.tag() {
                    DiffTag::Insert => {
                        let value = other[op.new_range()].to_string();
                        cord_ops.push(CordOp::Insert(pos, value));
                    }
                    DiffTag::Delete => {
                        let end = pos + op.old_range().len();
                        cord_ops.push(CordOp::Delete(pos..end));
                    }
                    DiffTag::Replace => {
                        let end = pos + op.old_range().len();
                        let value = other[op.new_range()].to_string();
                        cord_ops.push(CordOp::Replace(pos..end, value));
                    }
                    DiffTag::Equal => {}
                }
                pos += op.new_range().len();
            }

            context.op_apply(cord_ops);
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        if !path.is_empty() {
            bail!("Invalid path `{path:?}` for Cord");
        }

        let PatchOp::Apply(ops) = op else {
            bail!("Invalid op for Cord");
        };

        for op in ops {
            match op {
                CordOp::Insert(pos, value) => {
                    self.insert_str(pos, &value);
                }
                CordOp::Delete(range) => {
                    self.replace_range(range, "");
                }
                CordOp::Replace(range, value) => {
                    self.replace_range(range, &value);
                }
            }
        }

        Ok(())
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
                .diff_chars(&value, self);

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
