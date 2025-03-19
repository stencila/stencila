use std::{
    collections::VecDeque,
    fmt::{self, Debug},
};

use derive_more::{Deref, DerefMut, IntoIterator};

use common::{
    eyre::{bail, OptionExt, Report, Result},
    serde::{Deserialize, Serialize},
    serde_json::{self},
};
use node_type::NodeProperty;

/// A slot in a node path: either a property identifier or the index of a vector.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum NodeSlot {
    Property(NodeProperty),
    Index(usize),
}

impl From<NodeProperty> for NodeSlot {
    fn from(value: NodeProperty) -> Self {
        Self::Property(value)
    }
}

impl From<usize> for NodeSlot {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl TryFrom<serde_json::Value> for NodeSlot {
    type Error = Report;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        use serde_json::Value::*;
        match value {
            String(name) => Ok(Self::from(NodeProperty::try_from(name.as_str())?)),
            Number(index) => index
                .as_u64()
                .ok_or_eyre("Expected u64")
                .map(|value| Self::from(value as usize)),
            _ => bail!("Unhandled JSON value for `PatchSlot`"),
        }
    }
}

/// A path to reach a node from the root: a vector of [`PatchSlot`]s
///
/// A [`VecDeque`], rather than a [`Vec`] so that when applying operations in
/// a call to `patch` the front of the path can be popped off.
#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    DerefMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
#[serde(crate = "common::serde")]
#[derive(Default)]
pub struct NodePath(VecDeque<NodeSlot>);

impl NodePath {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<NodeProperty> for NodePath {
    fn from(value: NodeProperty) -> Self {
        Self::from(NodeSlot::from(value))
    }
}

impl From<usize> for NodePath {
    fn from(value: usize) -> Self {
        Self::from(NodeSlot::from(value))
    }
}

impl From<NodeSlot> for NodePath {
    fn from(value: NodeSlot) -> Self {
        Self(VecDeque::from([value]))
    }
}

impl<const N: usize> From<[NodeSlot; N]> for NodePath {
    fn from(value: [NodeSlot; N]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<[NodeProperty; N]> for NodePath {
    fn from(value: [NodeProperty; N]) -> Self {
        Self(value.into_iter().map(NodeSlot::from).collect())
    }
}

impl TryFrom<serde_json::Value> for NodePath {
    type Error = Report;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        use serde_json::Value::*;
        match value {
            Number(..) | String(..) => Ok(Self::from(NodeSlot::try_from(value)?)),
            Array(array) => Ok(Self(VecDeque::from(
                array
                    .into_iter()
                    .map(NodeSlot::try_from)
                    .collect::<Result<Vec<_>>>()?,
            ))),
            _ => bail!("Unhandled JSON value for `PatchPath`"),
        }
    }
}

impl Debug for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, slot) in self.iter().enumerate() {
            if index != 0 {
                f.write_str(".")?;
            }
            slot.fmt(f)?;
        }

        Ok(())
    }
}
