use std::{fmt::Display, ops::Range};

use common::{
    derive_more::{Deref, DerefMut},
    smol_str::SmolStr,
};

pub use node_id::NodeId;
pub use node_type::NodeType;

/// A mapping between UTF-8 character positions and nodes and their properties
#[derive(Default, Deref, DerefMut)]
pub struct Mapping {
    inner: Vec<MappingEntry>,
}

/// An entry in a [`Mapping`]
#[derive(Debug, Clone)]
pub struct MappingEntry {
    /// The range of positions the node spans
    pub range: Range<usize>,

    /// The type of the node
    pub node_type: NodeType,

    /// The id of the node
    pub node_id: NodeId,

    /// The name of the node property, if applicable
    pub property: Option<SmolStr>,
}

impl MappingEntry {
    /// Create a new mapping entry
    pub fn new(
        range: Range<usize>,
        node_type: NodeType,
        node_id: NodeId,
        property: Option<SmolStr>,
    ) -> Self {
        Self {
            range,
            node_type,
            node_id,
            property,
        }
    }
}

impl Mapping {
    /// Create an empty mapping
    ///
    /// Equivalent to [`Mapping::default`] but provided to make it more explicit
    /// when a codec provides no mapping (i.e. it returns `Mapping::none()`)
    pub fn none() -> Self {
        Self::default()
    }

    /// Find the node that is closest to a position
    pub fn closest(&self, position: usize) -> Option<MappingEntry> {
        for entry in self.iter() {
            if entry.range.contains(&position) {
                return Some(entry.clone());
            }
        }
        None
    }

    /// Find the node that is closest to a position and for which the predicate function returns true
    pub fn closest_where<F>(&self, position: usize, predicate: F) -> Option<MappingEntry>
    where
        F: Fn(&MappingEntry) -> bool,
    {
        for entry in self.iter() {
            if entry.range.contains(&position) && predicate(entry) {
                return Some(entry.clone());
            }
        }
        None
    }
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for MappingEntry {
            range,
            node_type,
            node_id,
            property,
        } in &self.inner
        {
            writeln!(
                f,
                "{:<5} {:<5} {} {}{}",
                range.start,
                range.end,
                node_id,
                node_type,
                property
                    .as_ref()
                    .map_or_else(String::new, |prop| format!(".{prop}"))
            )?;
        }

        Ok(())
    }
}
