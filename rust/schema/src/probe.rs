use eyre::{Result, bail};

use crate::{Array, Node, NodePath, NodeSlot, Null, Object};

/// Get the node, or nodes at a path within another node
pub fn get<T: ProbeNode>(node: &T, mut path: NodePath) -> Result<NodeSet> {
    node.duplicate(&mut path)
}

/// A trait to efficiently access a node within another
///
/// Traverse directly to a node at a given [`NodePath`]. If a path to
/// a node is available this is more efficient than walking the entire node
/// tree looking for a node with a [`NodeId`] as does the `node_find::find` function.
pub trait ProbeNode: Clone {
    /// Create a duplicate of the node at the path
    #[allow(unused_variables)]
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet>;
}

pub enum NodeSet {
    One(Node),
    Many(Vec<Node>),
}

impl ProbeNode for Null {
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        if !path.is_empty() {
            bail!("Node path should be empty")
        }
        Ok(NodeSet::One(Node::Null(self.clone())))
    }
}

macro_rules! atom {
    ($type:ty, $node:ident) => {
        impl ProbeNode for $type {
            fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
                if !path.is_empty() {
                    bail!("Node path should be empty")
                }
                Ok(NodeSet::One(Node::$node(*self)))
            }
        }
    };
}
atom!(bool, Boolean);
atom!(i64, Integer);
atom!(u64, UnsignedInteger);
atom!(f64, Number);

impl ProbeNode for String {
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        if !path.is_empty() {
            bail!("Node path should be empty for `String`")
        }
        Ok(NodeSet::One(Node::String(self.clone())))
    }
}

impl ProbeNode for Object {
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        if path.is_empty() {
            Ok(NodeSet::One(Node::Object(self.clone())))
        } else {
            bail!("Probing with `Object` nodes not yet supported")
        }
    }
}

impl ProbeNode for Array {
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        if path.is_empty() {
            Ok(NodeSet::One(Node::Array(self.clone())))
        } else {
            self.0.duplicate(path)
        }
    }
}

impl<T> ProbeNode for Box<T>
where
    T: ProbeNode,
{
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        self.as_ref().duplicate(path)
    }
}

impl<T> ProbeNode for Option<T>
where
    T: ProbeNode,
{
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        match self {
            Some(node) => node.duplicate(path),
            None => Ok(NodeSet::One(Node::Null(crate::Null))),
        }
    }
}

impl<T> ProbeNode for Vec<T>
where
    T: ProbeNode + Clone,
{
    fn duplicate(&self, path: &mut NodePath) -> Result<NodeSet> {
        if path.is_empty() {
            let nodes = self
                .iter()
                .filter_map(|item| match item.duplicate(&mut NodePath::new()) {
                    Ok(NodeSet::One(node)) => Some(node),
                    _ => None,
                })
                .collect();
            return Ok(NodeSet::Many(nodes));
        }

        let Some(NodeSlot::Index(index)) = path.pop_front() else {
            bail!("Node path should have index at front for `Vec`")
        };

        let Some(item) = self.get(index) else {
            bail!("Invalid index for `Vec`")
        };

        item.duplicate(path)
    }
}
