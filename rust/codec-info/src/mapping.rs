use std::{fmt::Display, ops::Range};

use common::{
    itertools::Itertools, serde::Serialize, serde_with::skip_serializing_none, smol_str::SmolStr,
};

pub use node_id::NodeId;
pub use node_type::NodeType;

/// A mapping between UTF-8 character positions and nodes and their properties
#[derive(Default, Clone, PartialEq, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct Mapping {
    entries: Vec<MappingEntry>,
}

impl Mapping {
    pub fn entries(&self) -> &Vec<MappingEntry> {
        &self.entries
    }
}

/// An entry in a [`Mapping`]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct MappingEntry {
    /// The range of positions
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

    /// The name of the node property, if applicable
    pub property: Option<SmolStr>,
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
        property: Option<SmolStr>,
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
                };
                self.entries.insert(second_index, entry);
            }
        }
    }

    /// Remove an entry
    pub fn remove(&mut self, node_id: NodeId) {
        self.entries.retain(|entry| entry.node_id != node_id)
    }
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:>6} {:>6} {:>10}   {:<24} node_type+property",
            "start", "end", "offsets", "node_id"
        )?;

        for MappingEntry {
            range: Range { start, end },
            start_offset,
            end_offset,
            node_type,
            node_id,
            property,
        } in &self.entries
        {
            let offsets = format!("{start_offset}/{end_offset}");

            let node_id = node_id.to_string();

            let prop = property
                .as_ref()
                .map_or_else(String::new, |prop| format!(".{prop}"));

            writeln!(
                f,
                "{start:>6} {end:>6} {offsets:>10}   {node_id:<24} {node_type}{prop}"
            )?;
        }

        Ok(())
    }
}
