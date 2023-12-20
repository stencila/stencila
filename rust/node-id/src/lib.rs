use std::{fmt, str::FromStr};

use common::{bs58, derive_more::Deref, eyre, uuid::Uuid};

/// A UUID for a node
///
/// This type exists so that we can define a default implementation
/// which creates a random UUID.
#[derive(Debug, Clone, Copy, Deref)]
pub struct NodeUuid(Uuid);

impl Default for NodeUuid {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl PartialEq for NodeUuid {
    fn eq(&self, _other: &Self) -> bool {
        // Node id should not affect node equality
        true
    }
}

/// A unique id for a node include a short nickname
/// for the type of node
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    nick: &'static str,
    uuid: Uuid,
}

impl NodeId {
    pub fn new(nick: &'static str, uuid: &Uuid) -> Self {
        Self { nick, uuid: *uuid }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nick)?;
        f.write_str("_")?;

        let bytes = self.uuid.as_bytes();
        let id = bs58::encode(bytes).into_string();
        f.write_str(&id)
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl FromStr for NodeId {
    type Err = eyre::Report;

    fn from_str(_node_id: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
