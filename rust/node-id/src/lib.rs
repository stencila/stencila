use std::{
    fmt::{self, Write},
    str,
};

#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};

use common::eyre::bail;
#[allow(unused)]
use common::{
    bs58, derive_more::Deref, eyre::Report, once_cell::sync::Lazy, serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay, uuid::Uuid,
};

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
/// - define a `PartialEq` implementation which always returns true so that
///   a node's `uid` is ignored in equality testing.
#[derive(Clone, Deref)]
pub struct NodeUid(Vec<u8>);

/// An atomic counter for deterministic auto-incremented ids
/// during development
///
/// Having deterministic ids is particularly useful for snapshot tests
/// to avoid changes in snapshots due to random ids.
#[cfg(debug_assertions)]
static NODE_UID: Lazy<AtomicU64> = Lazy::new(AtomicU64::default);

impl NodeUid {
    // Reset the `NodeUid` counter during tests
    pub fn testing_only_reset() {
        #[cfg(debug_assertions)]
        NODE_UID.store(0, Ordering::SeqCst)
    }
}

impl Default for NodeUid {
    fn default() -> Self {
        #[cfg(not(debug_assertions))]
        let bytes = Uuid::new_v4().as_bytes().to_vec();

        #[cfg(debug_assertions)]
        let bytes = NODE_UID
            .fetch_add(1, Ordering::SeqCst)
            .to_be_bytes()
            .to_vec();

        Self(bytes)
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
#[derive(Clone, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
#[serde_with(crate = "common::serde_with")]
pub struct NodeId {
    nick: [u8; 3],
    uid: Vec<u8>,
}

impl NodeId {
    /// Create a new node id
    pub fn new(nick: &'static [u8; 3], uid: &[u8]) -> Self {
        Self {
            nick: *nick,
            uid: uid.into(),
        }
    }

    /// Create a new null id
    pub fn null() -> Self {
        Self {
            nick: [0, 0, 0],
            uid: Vec::new(),
        }
    }

    /// Get the node type nickname of the node id
    pub fn nick(&self) -> &str {
        str::from_utf8(&self.nick).expect("node type nicknames should always be utf8")
    }

    /// Get the unique id part of the node id
    ///
    /// Note that this id is unique within a document, not universally.
    pub fn uid(&self) -> &[u8] {
        &self.uid
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.nick[0] as char)?;
        f.write_char(self.nick[1] as char)?;
        f.write_char(self.nick[2] as char)?;

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

impl str::FromStr for NodeId {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        if bytes.len() < 5 || bytes[3] != b'_' {
            bail!("Invalid node id")
        }

        let nick = [bytes[0], bytes[1], bytes[2]];

        let uid = bs58::decode(&bytes[4..]).into_vec()?;

        Ok(Self { nick, uid })
    }
}
