use std::{fmt::Display, ops::Range};

use common::{serde::Serialize, serde_with::skip_serializing_none, smol_str::SmolStr};

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
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for MappingEntry {
            range,
            start_offset,
            end_offset,
            node_type,
            node_id,
            property,
        } in &self.entries
        {
            writeln!(
                f,
                "{:<5} {:<5} {start_offset:<5} {end_offset:<5} {node_id} {node_type}{}",
                range.start,
                range.end,
                property
                    .as_ref()
                    .map_or_else(String::new, |prop| format!(".{prop}"))
            )?;
        }

        Ok(())
    }
}
