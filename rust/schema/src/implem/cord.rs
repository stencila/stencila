use std::{ops::Range, time::Duration};

use codec_html_trait::encode::text;
use common::similar::{capture_diff_deadline, Algorithm, DiffTag, TextDiffConfig};
use node_store::{
    automerge::{transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadNode, ReadStore, WriteNode, WriteStore,
};

use crate::{prelude::*, Cord, CordOp};

impl Cord {
    /// Apply an insert operation
    pub fn apply_insert(&mut self, position: usize, value: &str, author_id: u16) {
        // Check for out of bounds pos
        if position > self.len() {
            return;
        }

        // Update the string
        let current_length = self.len();
        self.insert_str(position, value);

        // Early return if authorship does not need updating
        if value.is_empty() {
            return;
        };

        // If authorship is empty then fill it a single "anon" run
        if self.authorship.is_empty() && !self.is_empty() {
            self.authorship = vec![(u16::MAX, current_length)];
        }

        // Find the run at the insertion position and update authorship
        let mut run_start = 0;
        let mut inserted = false;
        for run in 0..self.authorship.len() {
            let (run_author, run_length) = self.authorship[run];
            let run_end = run_start + run_length;

            if run_end < position {
                // Position is after run so continue
            } else if run_start >= position {
                // Position is before run
                if run_author == author_id {
                    // Same author: extend the existing run
                    self.authorship[run].1 += value.len();
                } else {
                    // Different author: insert before
                    self.authorship.insert(run, (author_id, value.len()));
                }

                inserted = true;
                break;
            } else if run_start <= position && run_end >= position {
                // Position is within run
                if run_author == author_id {
                    // Same author: extend the existing run
                    self.authorship[run].1 += value.len();
                } else {
                    // Split the run and insert after
                    self.authorship[run].1 = position - run_start;
                    let remaining = run_length - (position - run_start);
                    if remaining > 0 {
                        self.authorship.insert(run + 1, (run_author, remaining));
                    }
                    self.authorship.insert(run + 1, (author_id, value.len()));
                }

                inserted = true;
                break;
            }

            run_start += run_length;
        }

        if !inserted {
            let run = (author_id, value.len());
            if position == 0 {
                let is_first = self
                    .authorship
                    .first()
                    .map(|&(author, ..)| author == author_id)
                    .unwrap_or_default();
                if is_first {
                    self.authorship[0].1 += value.len();
                } else {
                    self.authorship.insert(0, run)
                }
            } else {
                let is_last = self
                    .authorship
                    .last()
                    .map(|&(author, ..)| author == author_id)
                    .unwrap_or_default();
                if is_last {
                    let last = self.authorship.len();
                    self.authorship[last].1 += value.len();
                } else {
                    self.authorship.push(run);
                }
            }
        }
    }

    /// Apply a delete operation
    pub fn apply_delete(&mut self, range: Range<usize>) {
        // Check for out of bounds range
        if range.start >= self.len() {
            return;
        }

        // Update the string
        self.replace_range(range.clone(), "");

        // Update authorship
        let mut run = 0;
        let mut run_start = 0;
        while run < self.authorship.len() {
            let (.., run_length) = self.authorship[run];
            let run_end = run_start + run_length;

            if run_end < range.start {
                // Run is before delete range so continue
                run += 1;
            } else if run_start >= range.end {
                // Run is after delete range so finish
                break;
            } else if run_start == range.start && run_end == range.end {
                // Delete of entire run
                self.authorship.remove(run);
                break;
            } else if run_start <= range.start && run_end >= range.end {
                // Delete within a single run
                self.authorship[run].1 = run_length - range.len();
                break;
            } else if run_start == range.start {
                // Delete spans multiple runs and starts at start of this one
                self.authorship.remove(run);
            } else if run_start < range.start {
                // Delete spans multiple runs and starts midway through this one
                self.authorship[run].1 = range.start - run_start;
                run += 1;
            } else if run_start > range.start && run_end <= range.end {
                // Delete spans multiple runs and spans this one completely
                self.authorship.remove(run);
            } else if run_end == range.end {
                // Delete spans multiple runs and ends at the end of this one
                self.authorship.remove(run);
                break;
            } else if run_end > range.end {
                // Delete spans multiple run and ends midway through this one
                self.authorship[run].1 = run_end - range.end;
                break;
            }

            run_start += run_length;
        }
    }

    // Replace a range in the string with new content and update authorship
    pub fn apply_replace(&mut self, range: Range<usize>, value: &str, author_id: u16) {
        self.apply_delete(range.clone());
        self.apply_insert(range.start, value, author_id);
    }

    // Apply operations
    pub fn apply_ops(&mut self, ops: Vec<CordOp>, author_id: u16) {
        for op in ops {
            match op {
                CordOp::Insert(pos, value) => self.apply_insert(pos, &value, author_id),
                CordOp::Delete(range) => self.apply_delete(range),
                CordOp::Replace(range, value) => self.apply_replace(range, &value, author_id),
            }
        }
    }
}

impl StripNode for Cord {}

impl PatchNode for Cord {
    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        self.authorship = vec![(context.author_id(), self.len())];

        Ok(())
    }

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

        self.apply_ops(ops, context.author_id());

        Ok(())
    }
}

impl ReadNode for Cord {
    fn load_text<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Self::from(store.text(obj_id)?))
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

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn insert_at_start() {
        let mut cord = Cord {
            string: "world!".to_string(),
            authorship: vec![(2, 6)],
        };
        cord.apply_insert(0, "Hello, ", 1);
        assert_eq!(cord.string, "Hello, world!");
        assert_eq!(cord.authorship, vec![(1, 7), (2, 6)]);
    }

    #[test]
    fn insert_at_end() {
        let mut cord = Cord {
            string: "Hello".to_string(),
            authorship: vec![(1, 5)],
        };
        cord.apply_insert(5, ", world!", 1);
        assert_eq!(cord.string, "Hello, world!");
        assert_eq!(cord.authorship, vec![(1, 13)]);
    }

    #[test]
    fn insert_at_middle() {
        let mut cord = Cord {
            string: "Hello world!".to_string(),
            authorship: vec![(1, 5), (2, 7)],
        };
        cord.apply_insert(5, ", beautiful", 3);
        assert_eq!(cord.string, "Hello, beautiful world!");
        assert_eq!(cord.authorship, vec![(1, 5), (3, 11), (2, 7)]);
    }

    #[test]
    fn insert_nothing() {
        let mut cord = Cord {
            string: "Hello".to_string(),
            authorship: vec![(1, 5)],
        };
        cord.apply_insert(3, "", 1); // Empty string insertion
        assert_eq!(cord.string, "Hello");
        assert_eq!(cord.authorship, vec![(1, 5)]);
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut cord = Cord {
            string: "Hello".to_string(),
            authorship: vec![(1, 5)],
        };
        cord.apply_insert(10, " world", 1); // Beyond the length of the string
        assert_eq!(cord.string, "Hello");
        assert_eq!(cord.authorship, vec![(1, 5)]); // No change
    }

    #[test]
    fn delete_entire_run() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 7), (2, 6)],
        };
        cord.apply_delete(0..7);
        assert_eq!(cord.string, "world!");
        assert_eq!(cord.authorship, vec![(2, 6)]);
    }

    #[test]
    fn delete_within_run() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 13)],
        };
        cord.apply_delete(0..6);
        assert_eq!(cord.string, " world!");
        assert_eq!(cord.authorship, vec![(1, 7)]);

        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 13)],
        };
        cord.apply_delete(5..13);
        assert_eq!(cord.string, "Hello");
        assert_eq!(cord.authorship, vec![(1, 5)]);

        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 13)],
        };
        cord.apply_delete(1..12);
        assert_eq!(cord.string, "H!");
        assert_eq!(cord.authorship, vec![(1, 2)]);
    }

    #[test]
    fn delete_across_runs() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 6), (2, 7)],
        };
        cord.apply_delete(5..12);
        assert_eq!(cord.string, "Hello!");
        assert_eq!(cord.authorship, vec![(1, 5), (2, 1)]);

        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 3), (2, 2), (3, 8)],
        };
        cord.apply_delete(1..12);
        assert_eq!(cord.string, "H!");
        assert_eq!(cord.authorship, vec![(1, 1), (3, 1)]);
    }

    #[test]
    fn delete_at_edges() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 7), (2, 6)],
        };
        cord.apply_delete(0..5); // Beginning edge
        assert_eq!(cord.string, ", world!");
        assert_eq!(cord.authorship, vec![(1, 2), (2, 6)]);

        cord.apply_delete(5..8); // End edge
        assert_eq!(cord.string, ", wor");
        assert_eq!(cord.authorship, vec![(1, 2), (2, 3)]);
    }

    #[test]
    fn delete_no_effect() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 7), (2, 6)],
        };
        cord.apply_delete(14..20); // Beyond string length
        assert_eq!(cord.string, "Hello, world!");
        assert_eq!(cord.authorship, vec![(1, 7), (2, 6)]);
    }

    #[test]
    fn delete_empty_range() {
        let mut cord = Cord {
            string: "Hello, world!".to_string(),
            authorship: vec![(1, 7), (2, 6)],
        };
        cord.apply_delete(5..5); // Empty range should do nothing
        assert_eq!(cord.string, "Hello, world!");
        assert_eq!(cord.authorship, vec![(1, 7), (2, 6)]);
    }

    #[test]
    fn delete_from_empty() {
        let mut cord = Cord {
            string: "".to_string(),
            authorship: Vec::new(),
        };
        cord.apply_delete(0..1); // Deleting from an empty string
        assert_eq!(cord.string, "");
        assert_eq!(cord.authorship, Vec::new());
    }
}
