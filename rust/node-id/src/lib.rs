use std::fmt;

use common::{bs58, derive_more::Deref, serde_with::SerializeDisplay, uuid::Uuid};

/// A unique id for a node
///
/// Stores a vector of unique bytes used to uniquely identify a node within a document.
/// When the node is instantiated in memory the bytes will initialized from a UUID.
/// When the node is loaded from a CRDT the bytes of will be set to the bytes of the
/// CRDT's unique id for the node.
///
/// This type exists as a `newtype` of `Vec<u8>` so that we can:
///
/// - define a `Default` implementation which creates a random UUID, and
/// - define a `PartialEq` implementation which ignores the
#[derive(Clone, Deref)]
pub struct NodeUid(Vec<u8>);

impl Default for NodeUid {
    fn default() -> Self {
        Self(Uuid::new_v4().as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for NodeUid {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl fmt::Debug for NodeUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = bs58::encode(&self.0).into_string();
        f.write_str(&id)
    }
}

impl PartialEq for NodeUid {
    fn eq(&self, _other: &Self) -> bool {
        // Node uid should not affect node equality
        true
    }
}

/// A unique id for a node including a short nickname for the type of node
#[derive(Clone, PartialEq, Eq, Hash, SerializeDisplay)]
#[serde_with(crate = "common::serde_with")]
pub struct NodeId {
    nick: &'static str,
    uid: Vec<u8>,
}

impl NodeId {
    pub fn new(nick: &'static str, uid: &[u8]) -> Self {
        Self {
            nick,
            uid: uid.into(),
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nick)?;
        f.write_str("_")?;

        let id = bs58::encode(&self.uid).into_string();
        f.write_str(&id)
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}
