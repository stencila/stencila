use std::{ops::Range, time::Duration};

use codec_html_trait::encode::text;
use common::similar::{Algorithm, DiffTag, TextDiff, TextDiffConfig};
use node_store::{
    automerge::{transaction::Transactable, ObjId, ObjType, Prop, Value},
    ReadNode, ReadStore, WriteNode, WriteStore,
};

use crate::{
    cord_provenance::{
        category, display, human_edited, human_written, machine_edited, machine_influence,
        machine_written,
    },
    prelude::*,
    Cord, CordOp, CordRun, ProvenanceCount,
};

impl Cord {
    /// Get the number of authorship runs in the cord
    pub fn runs(&self) -> usize {
        self.runs.len()
    }

    /// Get the count of authors for an authorship run
    pub fn run_count(&self, run: usize) -> u8 {
        self.runs[run].count
    }

    /// Get the list of authors for an authorship run
    pub fn run_authors(&self, run: usize) -> Vec<u8> {
        let CordRun { count, authors, .. } = self.runs[run];
        Self::extract_authors(count, authors)
    }

    /// Get the length of an authorship run
    pub fn run_length(&self, run: usize) -> u32 {
        self.runs[run].length
    }

    /// Update the authors in an authorship run in the cord
    ///
    /// Return `None` if there is no update. Otherwise return updated
    /// count, authors and mi.
    pub fn update_authors(
        count: u8,
        mut authors: u64,
        prov: u8,
        author: u8,
        author_type: AuthorType,
    ) -> Option<(u8, u64, u8)> {
        // Check if update is needed
        let last_author = (authors & 0xFF) as u8;
        if count != 0 && last_author == author {
            return None;
        }

        // Shift authors along
        authors <<= 8;
        authors |= author as u64;

        // Increment the count of authors
        let count = count.saturating_add(1);

        // Update the provenance byte
        let prov = match author_type {
            AuthorType::Human => human_edited(prov),
            AuthorType::Machine => machine_edited(prov),
        };

        Some((count, authors, prov))
    }

    /// Get the list of authors for an authorship run
    pub fn extract_authors(count: u8, mut value: u64) -> Vec<u8> {
        let count = count.min(8) as usize;
        let mut authors = Vec::with_capacity(count);
        for _ in 0..count {
            authors.push((value & 0xFF) as u8);
            value >>= 8;
        }
        authors
    }

    /// Create a JSON array of the authors
    pub fn json_authors(count: u8, value: u64) -> String {
        let mut json = "[".to_string();
        let mut first = true;
        for author in Self::extract_authors(count, value) {
            if !first {
                json.push(',')
            }
            json.push_str(&author.to_string());
            first = false;
        }
        json.push(']');
        json
    }

    /// Set the run of the cord to indicate unknown authorship
    ///
    /// Should normally only be used when `runs` is empty but `string` is not.
    fn unknown_author(&mut self, length: u32) {
        self.runs = vec![CordRun {
            count: 1,
            authors: u8::MAX as u64,
            provenance: 0,
            length,
        }];
    }

    /// Coalesce runs where possible
    fn coalesce_runs(&mut self) {
        if self.runs.len() > 1 {
            let mut coalesced: Vec<CordRun> = Vec::with_capacity(1);
            for run in self.runs.iter() {
                if let Some(last) = coalesced.last_mut() {
                    if (run.count, run.authors) == (last.count, last.authors) {
                        last.length += run.length;
                        continue;
                    }
                }
                coalesced.push(run.clone())
            }
            self.runs = coalesced;
        }
    }

    /// Check that the sum of the run lengths is equal to the number of chars and that there
    /// are no empty runs
    #[cfg(debug_assertions)]
    fn check_runs(&self) {
        let mut runs = 0usize;
        for run in self.runs.iter() {
            assert!(run.length > 0, "run length is zero: {:?}", self.runs);
            runs += run.length as usize;
        }

        let chars = self.string.chars().count();

        assert_eq!(
            runs, chars,
            "sum of run lengths != chars in string: {:?} vs \"{}\"",
            self.runs, self.string
        )
    }

    /// Create cord operations
    pub fn create_ops(&self, other: &Self) -> Vec<CordOp> {
        // Calculate diff operations
        let diff = TextDiff::configure()
            .algorithm(Algorithm::Patience)
            .timeout(Duration::from_secs(1))
            .diff_chars(self.as_str(), other.as_str());

        // Convert them to `CordOp`s
        let mut cord_ops = Vec::new();
        let mut pos = 0usize;
        for op in diff.ops() {
            match op.tag() {
                DiffTag::Insert => {
                    let value = other
                        .chars()
                        .skip(op.new_range().start)
                        .take(op.new_range().len())
                        .collect();
                    cord_ops.push(CordOp::Insert(pos, value));
                }
                DiffTag::Delete => {
                    let end = pos + op.old_range().len();
                    cord_ops.push(CordOp::Delete(pos..end));
                }
                DiffTag::Replace => {
                    let end = pos + op.old_range().len();
                    let value = other
                        .chars()
                        .skip(op.new_range().start)
                        .take(op.new_range().len())
                        .collect();
                    cord_ops.push(CordOp::Replace(pos..end, value));
                }
                DiffTag::Equal => {}
            }
            pos += op.new_range().len();
        }

        cord_ops
    }

    /// Apply an insert operation
    pub fn apply_insert(
        &mut self,
        position: usize,
        value: &str,
        author: Option<u8>,
        author_type: Option<AuthorType>,
    ) {
        let old_length = self.chars().count();

        // Check for out of bounds pos  or empty value
        if position > old_length || value.is_empty() {
            return;
        }

        // Update the string
        if let Some((index, ..)) = self.char_indices().nth(position) {
            self.insert_str(index, value);
        } else {
            self.push_str(value);
        }

        // If was empty and no author supplied then can return early
        if old_length == 0 && author.is_none() {
            return;
        }

        // Otherwise, we need to add some authorship so if author
        // not supplied then add as unknown author
        let author = author.unwrap_or(u8::MAX);
        let prov = match author_type {
            Some(AuthorType::Machine) => machine_written(),
            Some(AuthorType::Human) | None => human_written(),
        };

        let value_length = value.chars().count() as u32;

        // Shortcut for updating authorship if was empty
        if old_length == 0 {
            self.runs = vec![CordRun::new(1, author as u64, prov, value_length)];
            return;
        }

        // If runs is empty then add a single "unknown author" run for the old_length
        if self.runs.is_empty() && !self.is_empty() {
            self.unknown_author(old_length as u32);
        }

        // Find the run at the insertion position and update authorship
        let history = (1, author as u64);
        let mut run_start = 0usize;
        let mut inserted = false;
        for run in 0..self.runs.len() {
            let CordRun {
                count: run_count,
                authors: run_authors,
                provenance: run_prov,
                length: run_length,
            } = self.runs[run];
            let run_history = (run_count, run_authors);
            let run_length = run_length as usize;
            let run_end = run_start + run_length;

            if run_end < position {
                // Position is after run so continue
            } else if run_start >= position {
                // Position is before run
                if run_history == history {
                    // Same history: extend the existing run
                    self.runs[run].length += value_length;
                } else {
                    // Different author: insert before
                    self.runs
                        .insert(run, CordRun::new(history.0, history.1, prov, value_length));
                }

                inserted = true;
                break;
            } else if run_start <= position && run_end > position {
                // Position is within run
                if run_history == history {
                    // Same history: extend the existing run
                    self.runs[run].length += value_length;
                } else {
                    // Split the run and insert after
                    self.runs[run].length = (position - run_start) as u32;
                    self.runs.insert(
                        run + 1,
                        CordRun::new(
                            run_count,
                            run_authors,
                            run_prov,
                            (run_length - (position - run_start)) as u32,
                        ),
                    );
                    self.runs.insert(
                        run + 1,
                        CordRun::new(history.0, history.1, prov, value_length),
                    );
                }

                inserted = true;
                break;
            }

            run_start += run_length;
        }

        if !inserted {
            let same_as_last = self
                .runs
                .last()
                .map(|run| (run.count, run.authors) == history)
                .unwrap_or_default();
            if same_as_last {
                let last = self.runs.len() - 1;
                self.runs[last].length += value_length;
            } else {
                self.runs
                    .push(CordRun::new(history.0, history.1, prov, value_length));
            }
        }

        #[cfg(debug_assertions)]
        self.check_runs()
    }

    /// Apply a delete operation
    pub fn apply_delete(&mut self, range: Range<usize>) {
        let old_length = self.chars().count();

        // Check for out of bounds range or nothing to do
        if range.start >= old_length || range.is_empty() {
            return;
        }

        // Update the string. The following match is conservative in covering all circumstances,
        // but avoid having a panic or other undefined behavior
        match (
            self.char_indices().nth(range.start),
            self.char_indices().nth(range.end),
        ) {
            (Some((start, ..)), Some((end, ..))) => self.replace_range(start..end, ""),
            (Some((start, ..)), None) => self.replace_range(start.., ""),
            (None, Some((end, ..))) => self.replace_range(..end, ""),
            (None, None) => self.replace_range(.., ""),
        }

        // Check if whole string is now empty in which case early return for updating runs
        if self.is_empty() {
            self.runs.clear();
            return;
        }

        // If runs is empty then add a single "unknown author" run for the old_length
        if self.runs.is_empty() && !self.is_empty() {
            self.unknown_author(old_length as u32);
        }

        // Update authorship
        let mut run = 0;
        let mut run_start = 0usize;
        while run < self.runs.len() {
            let &CordRun {
                length: run_length, ..
            } = &self.runs[run];
            let run_length = run_length as usize;
            let run_end = run_start + run_length;

            if run_end < range.start {
                // Run is before delete range so continue
                run += 1;
            } else if run_start > range.end {
                // Run is after delete range so finish
                break;
            } else if run_start == range.start && run_end == range.end {
                // Delete of entire run: remove it
                self.runs.remove(run);
                break;
            } else if run_start <= range.start && run_end >= range.end {
                // Delete within a single run: update length and finish
                self.runs[run].length = (run_length - range.len()) as u32;
                break;
            } else if run_start < range.start {
                // Delete spans multiple runs and starts midway through this one:
                // update length and continue
                self.runs[run].length = (range.start - run_start) as u32;
                run += 1;
            } else if run_start >= range.start && run_end <= range.end {
                // Delete spans multiple runs and spans this one completely:
                // remove it
                self.runs.remove(run);
            } else if run_end == range.end {
                // Delete spans multiple runs and ends at the end of this one:
                // remove it and finish
                self.runs.remove(run);
                break;
            } else if run_end > range.end {
                // Delete spans multiple run and ends midway through this one:
                // update length and finish
                self.runs[run].length = (run_end - range.end) as u32;
                break;
            }

            run_start += run_length;
        }

        self.coalesce_runs();

        #[cfg(debug_assertions)]
        self.check_runs()
    }

    // Replace a range in the string with new content and update authorship
    pub fn apply_replace(
        &mut self,
        range: Range<usize>,
        value: &str,
        author_index: Option<u8>,
        author_type: Option<AuthorType>,
    ) {
        let old_length = self.chars().count();

        // Check for out of bounds range or nothing to do
        if range.start >= old_length || range.is_empty() {
            return;
        }

        // Update the string. The following match is conservative in covering all circumstances,
        // but avoid having a panic or other undefined behavior
        match (
            self.char_indices().nth(range.start),
            self.char_indices().nth(range.end),
        ) {
            (Some((start, ..)), Some((end, ..))) => self.replace_range(start..end, value),
            (Some((start, ..)), None) => self.replace_range(start.., value),
            (None, Some((end, ..))) => self.replace_range(..end, value),
            (None, None) => self.replace_range(.., value),
        }

        // If was empty and no author supplied then can return early
        if old_length == 0 && author_index.is_none() {
            return;
        }

        // Otherwise, need to add some authorship, so if author
        // not supplied then add as unknown author
        let author = author_index.unwrap_or(u8::MAX);
        let author_type = author_type.unwrap_or(AuthorType::Human);

        // If runs is empty then add a single "unknown author" run for the old_length
        if self.runs.is_empty() && !self.is_empty() {
            self.unknown_author(old_length as u32);
        }

        // Update authorship
        let value_length = value.chars().count();
        let mut run = 0;
        let mut run_start = 0usize;
        let mut multi_run_length = 0;
        while run < self.runs.len() {
            let &CordRun {
                count: run_count,
                authors: run_authors,
                provenance: run_prov,
                length: run_length,
            } = &self.runs[run];
            let run_length = run_length as usize;
            let run_end = run_start + run_length;

            if run_end < range.start {
                // Run is before replace range so continue
                run += 1;
            } else if run_start > range.end {
                // Run is after replace range so finish
                break;
            } else if run_start == range.start && run_end == range.end {
                // Replace of entire run: update authorship and finish
                if let Some((new_count, new_authors, new_prov)) =
                    Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                {
                    let run = &mut self.runs[run];
                    run.count = new_count;
                    run.authors = new_authors;
                    run.provenance = new_prov;
                }
                self.runs[run].length = value_length as u32;
                break;
            } else if run_start == range.start && run_end > range.end
                || run_start < range.start && run_end == range.end
            {
                // Replace at start or end of run and enclosed within it: update length of this run,
                // create a new run if necessary, and finish
                if let Some((new_count, new_authors, new_prov)) =
                    Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                {
                    self.runs[run].length = (run_length - range.len()) as u32;

                    let new_run = if run_end == range.end { run + 1 } else { run };
                    self.runs.insert(
                        new_run,
                        CordRun::new(new_count, new_authors, new_prov, value_length as u32),
                    )
                } else {
                    self.runs[run].length = (run_length + value_length - range.len()) as u32;
                }
                break;
            } else if run_start < range.start && run_end > range.end {
                // Replace within a single run but not at either end: update length of this run,
                // create two new runs if necessary, and finish
                if let Some((new_count, new_authors, new_prov)) =
                    Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                {
                    self.runs[run].length = (range.start - run_start) as u32;

                    self.runs.insert(
                        run + 1,
                        CordRun::new(new_count, new_authors, new_prov, value_length as u32),
                    );
                    self.runs.insert(
                        run + 2,
                        CordRun::new(
                            run_count,
                            run_authors,
                            run_prov,
                            (run_end - range.end) as u32,
                        ),
                    );
                } else {
                    self.runs[run].length = (run_length + value_length - range.len()) as u32;
                }
                break;
            } else if run_start < range.start && run_end > range.start {
                // Replace spans multiple runs and starts midway through this one:
                // split this run into two if necessary and accumulate remaining run length
                if let Some((new_count, new_authors, new_prov)) =
                    Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                {
                    let new_length = range.start - run_start;
                    self.runs[run].length = new_length as u32;
                    self.runs.insert(
                        run + 1,
                        CordRun::new(
                            new_count,
                            new_authors,
                            new_prov,
                            (run_length - new_length) as u32,
                        ),
                    );

                    run += 2;
                } else {
                    run += 1;
                }
                multi_run_length = run_end - range.start;
            } else if run_start >= range.start && run_end <= range.end {
                // Replace spans multiple runs and spans this one completely:
                // remove if it is no longer needed, otherwise update authors
                // if necessary and accumulate run length
                if multi_run_length >= value_length {
                    self.runs.remove(run);
                } else {
                    if let Some((new_count, new_authors, new_prov)) =
                        Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                    {
                        let run = &mut self.runs[run];
                        run.count = new_count;
                        run.authors = new_authors;
                        run.provenance = new_prov;
                    }
                    multi_run_length += run_length;
                    run += 1;
                }
            } else if run_end == range.end {
                // Replace spans multiple runs and ends at the end of this one:
                // if the accumulated run length is equal to this length of the value then
                // this run can be deleted. Otherwise, update authors if necessary and finish.
                if multi_run_length >= value_length {
                    self.runs.remove(run);
                } else if let Some((new_count, new_authors, new_prov)) =
                    Self::update_authors(run_count, run_authors, run_prov, author, author_type)
                {
                    let run = &mut self.runs[run];
                    run.count = new_count;
                    run.authors = new_authors;
                    run.provenance = new_prov;
                }
                break;
            } else if run_end > range.end {
                // Replace spans multiple run and ends midway through this one:
                // split this run into two if necessary, adjust for accumulated length
                // and then finish
                let new_length = run_end - range.end;
                self.runs[run].length = new_length as u32;

                // If necessary insert a new run before this one for remaining new bytes
                let remaining = (value_length.saturating_sub(multi_run_length)) as u32;
                if remaining > 0 {
                    let prov = match author_type {
                        AuthorType::Human => human_written(),
                        AuthorType::Machine => machine_written(),
                    };
                    self.runs
                        .insert(run, CordRun::new(1, author as u64, prov, remaining))
                }

                break;
            }

            run_start += run_length;
        }

        self.coalesce_runs();

        #[cfg(debug_assertions)]
        self.check_runs()
    }

    // Apply operations
    pub fn apply_ops(
        &mut self,
        ops: Vec<CordOp>,
        author_id: Option<u8>,
        author_type: Option<AuthorType>,
    ) {
        for op in ops {
            match op {
                CordOp::Insert(pos, value) => {
                    self.apply_insert(pos, &value, author_id, author_type)
                }
                CordOp::Delete(range) => self.apply_delete(range),
                CordOp::Replace(range, value) => {
                    self.apply_replace(range, &value, author_id, author_type)
                }
            }
        }
    }
}

impl StripNode for Cord {}

impl PatchNode for Cord {
    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        if let Some(author_index) = context.author_index() {
            let mi = match context.author_type() {
                Some(AuthorType::Machine) => machine_written(),
                _ => human_written(),
            };

            self.runs = vec![CordRun::new(1, author_index as u64, mi, self.len() as u32)];
        }

        Ok(())
    }

    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        if self.runs.is_empty() {
            None
        } else {
            Some(
                self.runs
                    .iter()
                    .map(|run| ProvenanceCount::new(category(run.provenance), run.length as u64))
                    .collect(),
            )
        }
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

    fn similarity(&self, other: &Cord, _context: &mut PatchContext) -> Result<f32> {
        // Calculate a difference ratio using chars as we do for generating diffs
        let diff = TextDiffConfig::default()
            .algorithm(Algorithm::Patience)
            .timeout(Duration::from_secs(1))
            .diff_chars(self.as_str(), other.as_str());

        // Note minimum similarity because same types
        // This is important because it means a `Cord` will have non-zero
        // similarity with itself, even if all characters change
        Ok(diff.ratio().max(self.minimum_similarity()))
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        if other != self {
            let cord_ops = self.create_ops(other);
            context.op_apply(cord_ops);
        }

        Ok(())
    }

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        if patch.node_id.is_some() {
            return Ok(false);
        }

        patch.apply(self, context)
    }

    fn apply(
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

        self.apply_ops(ops, context.author_index(), context.author_type());

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

impl DomCodec for Cord {
    /// Encode a cord to DOM HTML as a series of <stencila-authorship> elements
    ///
    /// For use when the cord is a `Text::value`.
    fn to_dom(&self, context: &mut DomEncodeContext) {
        if self.runs.is_empty() {
            context.push_text(&self.string);
        } else {
            let chars = self.string.chars();

            let mut start = 0;
            for run in &self.runs {
                let text = chars
                    .clone()
                    .skip(start)
                    .take(run.length as usize)
                    .collect::<String>();
                context
                    .enter_elem_attrs(
                        "stencila-authorship",
                        [
                            ("count", &run.count.to_string()),
                            ("authors", &Self::json_authors(run.count, run.authors)),
                            ("provenance", &display(run.provenance)),
                            ("mi", &machine_influence(run.provenance).to_string()),
                        ],
                    )
                    .push_text(&text)
                    .exit_elem();
                start += run.length as usize;
            }
        }
    }

    /// Encode a cord to DOM HTML as a HTML attribute
    ///
    /// For use when the cord is a `CodeChunk::code`, `MathBlock::code` etc.
    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
        context.push_attr(name, &self.string);

        if !self.runs.is_empty() {
            let mut json = "[".to_string();
            let mut start = 0;
            let mut first = true;
            for run in &self.runs {
                if !first {
                    json.push(',');
                }
                let end = start + run.length;

                json.push('[');
                json.push_str(&start.to_string());
                json.push(',');
                json.push_str(&end.to_string());
                json.push(',');
                json.push_str(&run.count.to_string());
                json.push(',');
                json.push_str(&Self::json_authors(run.count, run.authors));
                json.push_str(",\"");
                json.push_str(&display(run.provenance));
                json.push_str("\",");
                json.push_str(&machine_influence(run.provenance).to_string());
                json.push(']');

                first = false;
                start = end;
            }
            json.push(']');
            context.push_attr(&[name, "-authorship"].concat(), &json);
        }
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
        if self.runs.is_empty() {
            context.push_str(self.as_str());
        } else {
            let runs = self.runs.iter().map(CordRun::as_tuple).collect_vec();
            context.push_authored_str(&runs, self.as_str());
        }
    }
}

impl TextCodec for Cord {
    fn to_text(&self) -> (String, Losses) {
        (self.to_string(), Losses::none())
    }
}
