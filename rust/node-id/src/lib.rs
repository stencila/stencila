use std::{
    fmt::{self, Write},
    str,
};

use derive_more::Deref;
use rand::{RngExt, distr::Alphanumeric, rng};

use eyre::{Report, bail};
use serde_with::{DeserializeFromStr, SerializeDisplay};

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

impl Default for NodeUid {
    fn default() -> Self {
        // The following alphabet/length has 62^24 ~= 1.0x10^43 random possibilities.
        // Compare to UUIDv4 which has 2^122 ~= 5.3×10^36.
        // For collision probabilities see https://alex7kom.github.io/nano-nanoid-cc/?alphabet=ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789&size=24&speed=1000&speedUnit=second
        let bytes: Vec<u8> = rng().sample_iter(&Alphanumeric).take(24).collect();

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
        let uid = str::from_utf8(&self.0).expect("node uid should always be utf8");
        f.write_str(uid)
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
#[serde_with(crate = "serde_with")]
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

    /// Create a new random id for a nick
    pub fn random(nick: [u8; 3]) -> Self {
        Self {
            nick,
            uid: NodeUid::default().0,
        }
    }

    /// Get the node type nickname of the node id
    pub fn nick(&self) -> &str {
        str::from_utf8(&self.nick).expect("node type nicknames should always be utf8")
    }

    /// Get the unique id part of the node id as bytes
    pub fn uid(&self) -> &[u8] {
        &self.uid
    }

    /// Get the unique id part of the node id as a string
    pub fn uid_str(&self) -> &str {
        str::from_utf8(&self.uid).expect("node uid should always be utf8")
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.nick[0] as char)?;
        f.write_char(self.nick[1] as char)?;
        f.write_char(self.nick[2] as char)?;

        f.write_str("_")?;

        let uid = str::from_utf8(&self.uid).expect("node uid should always be utf8");
        f.write_str(uid)
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
        let uid = bytes[4..].to_vec();

        Ok(Self { nick, uid })
    }
}
