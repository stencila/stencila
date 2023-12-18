use std::{fmt, str::FromStr};

use common::{bs58, eyre, uuid::Uuid};

use crate::NodeType;

/// A trait for entity node types to get their type and id
pub trait Entity {
    /// Get the [`NodeType`] of the entity
    fn node_type() -> NodeType;

    /// Get the `node_id` of the entity
    fn node_id(&self) -> &NodeId;
}

/// The unique id for a node
#[derive(Debug, Clone)]
pub struct NodeId(Uuid);

impl Default for NodeId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl PartialEq for NodeId {
    fn eq(&self, _other: &Self) -> bool {
        // Node id should not affect node equality
        true
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("node_")?;

        let bytes = self.0.as_bytes();
        let id = bs58::encode(bytes).into_string();
        f.write_str(&id)
    }
}

impl FromStr for NodeId {
    type Err = eyre::Report;

    fn from_str(_node_id: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
