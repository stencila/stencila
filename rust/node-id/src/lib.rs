use std::fmt;

use common::{bs58, derive_more::Deref, uuid::Uuid};

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
#[derive(Debug, Clone, Deref)]
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

impl PartialEq for NodeUid {
    fn eq(&self, _other: &Self) -> bool {
        // Node uid should not affect node equality
        true
    }
}

/// A unique id for a node including a short nickname for the type of node
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NodeId<'uid> {
    nick: &'static str,
    uid: &'uid [u8],
}

impl<'uid> NodeId<'uid> {
    pub fn new(nick: &'static str, uid: &'uid [u8]) -> Self {
        Self { nick, uid }
    }
}

impl<'uid> fmt::Display for NodeId<'uid> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nick)?;
        f.write_str("_")?;

        let id = bs58::encode(&self.uid).into_string();
        f.write_str(&id)
    }
}

impl<'uid> fmt::Debug for NodeId<'uid> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}
