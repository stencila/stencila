use std::{fmt::Display, ops::Range};

use common::{itertools::Itertools, serde::Serialize, serde_with::skip_serializing_none};
pub use node_id::NodeId;
pub use node_type::{NodeProperty, NodeType};

/// A mapping between UTF-8 character indices and nodes and their properties
#[derive(Default, Clone, PartialEq, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct Mapping {
    entries: Vec<MappingEntry>,
}

/// An entry in a [`Mapping`]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct MappingEntry {
    /// The range of UTF-8 character indices for the entry
    #[serde(skip)]
    pub range: Range<usize>,

    /// The offset of the start this entry from the start of the previous entry
    #[serde(rename = "start")]
    pub start_offset: i32,

    /// The offset of the end this entry from the end of the previous entry
    #[serde(rename = "end")]
    pub end_offset: i32,

    /// The type of the node
    pub node_type: NodeType,

    /// The id of the node
    pub node_id: NodeId,

    /// The node property for property entries
    pub property: Option<NodeProperty>,

    /// The authorship (`count`, `authors`, and `provenance`) for `Cord` runs
    pub authorship: Option<(u8, u64, u8)>,
}

impl Mapping {
    /// Create an empty mapping
    ///
    /// Equivalent to [`Mapping::default`] but provided to make it more explicit
    /// when a codec provides no mapping (i.e. it returns `Mapping::none()`)
    pub fn none() -> Self {
        Self::default()
    }

    /// Add a new mapping entry
    pub fn add(
        &mut self,
        start: usize,
        end: usize,
        node_type: NodeType,
        node_id: NodeId,
        property: Option<NodeProperty>,
        authorship: Option<(u8, u64, u8)>,
    ) {
        let last = match self.entries.last() {
            Some(entry) => &entry.range,
            None => &(0..0),
        };
        let entry = MappingEntry {
            range: start..end,
            start_offset: start as i32 - last.start as i32,
            end_offset: end as i32 - last.end as i32,
            node_type,
            node_id,
            property,
            authorship,
        };
        self.entries.push(entry)
    }

    /// Replace an entry for with a new node type and id
    pub fn replace(&mut self, node_id: NodeId, new_node_type: NodeType, new_node_id: NodeId) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.node_id == node_id)
        {
            entry.node_type = new_node_type;
            entry.node_id = new_node_id;
        }
    }

    /// Extend an entry to the end of another
    pub fn extend(&mut self, first_node_id: NodeId, second_node_id: NodeId) {
        // Get the second entry
        if let Some((second_index, second_entry)) = self
            .entries
            .iter()
            .find_position(|entry| entry.node_id == second_node_id)
        {
            let second_start = second_entry.range.start;
            let second_end = second_entry.range.end;

            // Find the first entry
            if let Some((first_index, ..)) = self
                .entries
                .iter()
                .find_position(|entry| entry.node_id == first_node_id)
            {
                // Remove the first entry
                let MappingEntry {
                    range: Range { start, .. },
                    node_type,
                    node_id,
                    property,
                    authorship,
                    ..
                } = self.entries.remove(first_index);

                // Add a new entry after the second with appropriate offsets
                let entry = MappingEntry {
                    range: start..second_end,
                    start_offset: start as i32 - second_start as i32,
                    end_offset: 0,
                    node_type,
                    node_id,
                    property,
                    authorship,
                };
                self.entries.insert(second_index, entry);
            }
        }
    }

    /// Remove an entry
    pub fn remove(&mut self, node_id: NodeId) {
        self.entries.retain(|entry| entry.node_id != node_id)
    }

    /// Get the entries in the mapping
    pub fn entries(&self) -> &Vec<MappingEntry> {
        &self.entries
    }

    /// Get the entry, if any, at a UTF-8 character index
    pub fn entry_at(&self, index: usize) -> Option<&MappingEntry> {
        self.entries
            .iter()
            .find(|&entry| index >= entry.range.start && index < entry.range.end)
    }

    /// Get the id of the node, if any, at a UTF-8 character index
    pub fn node_id_at(&self, index: usize) -> Option<&NodeId> {
        self.entry_at(index).map(|entry| &entry.node_id)
    }

    /// Get the node property, if any, at a UTF-8 character index
    pub fn property_at(&self, index: usize) -> Option<&NodeProperty> {
        self.entry_at(index)
            .and_then(|entry| entry.property.as_ref())
    }

    /// Get the authorship of the `Cord` run, if any, at a UTF-8 character index
    pub fn authorship_at(&self, index: usize) -> Option<&(u8, u64, u8)> {
        self.entry_at(index)
            .and_then(|entry| entry.authorship.as_ref())
    }

    /// Get the range of UTF-8 character indices, if any, of a node
    pub fn range_of_node(&self, node_id: &NodeId) -> Option<Range<usize>> {
        for entry in self.entries.iter() {
            if &entry.node_id == node_id {
                return Some(entry.range.clone());
            }
        }
        None
    }

    /// Get the range of UTF-8 character indices, if any, of a node property
    pub fn range_of_property(
        &self,
        node_id: &NodeId,
        node_property: NodeProperty,
    ) -> Option<Range<usize>> {
        for entry in self.entries.iter() {
            if &entry.node_id == node_id && entry.property == Some(node_property) {
                return Some(entry.range.clone());
            }
        }
        None
    }
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Previously we included node_ids in display but because they are non-deterministic
        // for any one test (e.g. depend on test order) is seems best to avoid that
        writeln!(
            f,
            // For `insta` inline snapshots, the first column header should be
            // left aligned to avoid indentation reformatting on insert of snapshot
            "start     end        offsets   node_type+property                   authorship"
        )?;

        for MappingEntry {
            range: Range { start, end },
            start_offset,
            end_offset,
            node_type,
            property,
            authorship,
            ..
        } in &self.entries
        {
            let offsets = format!("({start_offset}, {end_offset})");

            let node_type_prop = property.as_ref().map_or_else(
                || format!("{node_type}"),
                |prop| format!("{node_type}.{prop}"),
            );

            let authorship = authorship
                .as_ref()
                .map_or_else(String::new, |authorship| format!("{authorship:?}"));

            let line =
                format!("{start:>6} {end:>6} {offsets:>14}   {node_type_prop:<36} {authorship}");
            let line = line.trim_end();

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}
