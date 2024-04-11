use std::{
    collections::VecDeque,
    fmt::{self, Debug},
};

use common::derive_more::{Deref, DerefMut};
use node_id::NodeId;
use node_type::{NodeProperty, NodeType};

use crate::Node;

/// A trait for condensing a node into a list of diff-able and merge-able properties
pub trait PatchNode {
    /// Condense a node into a list of properties that can be diffed
    ///
    /// This default implementation does nothing. Implementors should
    /// call the various methods of the `context` to collect properties.
    #[allow(unused_variables)]
    fn condense(&self, context: &mut CondenseContext) {}

    #[allow(unused_variables)]
    fn get_path(&self, path: &mut NodePath) -> Option<Node> {
        None
    }

    #[allow(unused_variables)]
    fn set_path(&mut self, path: &mut NodePath, node: Node) {}

    #[allow(unused_variables)]
    fn insert_path(&mut self, path: &mut NodePath, node: Node) {}

    #[allow(unused_variables)]
    fn remove_path(&mut self, path: &mut NodePath) {}
}

// Implementation for simple "atomic" types not in schema
macro_rules! atom {
    ($type:ty) => {
        impl PatchNode for $type {
            fn condense(&self, context: &mut CondenseContext) {
                context.collect_value(&self.to_string());
            }
        }
    };
}
atom!(bool);
atom!(i64);
atom!(u64);
atom!(f64);
atom!(String);

// Implementation for boxed properties
impl<T> PatchNode for Box<T>
where
    T: PatchNode,
{
    fn condense(&self, context: &mut CondenseContext) {
        self.as_ref().condense(context)
    }
}

// Implementation for optional properties
impl<T> PatchNode for Option<T>
where
    T: PatchNode,
{
    fn condense(&self, context: &mut CondenseContext) {
        if let Some(value) = self {
            value.condense(context);
        }
    }
}

// Implementation for vector properties
impl<T> PatchNode for Vec<T>
where
    T: PatchNode,
{
    fn condense(&self, context: &mut CondenseContext) {
        for (index, item) in self.iter().enumerate() {
            context.enter_index(index);
            item.condense(context);
            context.exit_index();
        }
    }
}

/// A list of ancestor node ids for a property
///
/// This list of ids is stored for each property so that we can combine
/// adjacent diff operations on properties into an operation to insert,
/// delete, or move an entire node. This is done by finding the highest
/// common ancestor for adjacent properties.
#[derive(Clone, Deref, DerefMut)]
pub struct NodeAncestry(Vec<NodeId>);

impl Default for NodeAncestry {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl NodeAncestry {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Display the ancestry as a dot separated list
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
impl Debug for NodeAncestry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, id) in self.iter().enumerate() {
            if index != 0 {
                f.write_str(".")?;
            }
            id.fmt(f)?;
        }

        Ok(())
    }
}

/// A slot in a node path: either a property identifier or the index of a vector.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeSlot {
    Property((NodeType, NodeProperty)),
    Index(usize),
}

/// Display the slot
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
impl Debug for NodeSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeSlot::Property((node_type, node_prop)) => {
                f.write_fmt(format_args!("{node_type}:{node_prop}"))
            }
            NodeSlot::Index(index) => index.fmt(f),
        }
    }
}

/// A path to reach a node from the root: a vector of [`NodeSlot`]s
///
/// Used when applying a patch to a node to traverse directly to the
/// branch of the tree that a patch operation should be applied.
/// Similar to the `path` of JSON Patch (https://jsonpatch.com/).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
pub struct NodePath(VecDeque<NodeSlot>);

impl Default for NodePath {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl NodePath {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> From<[NodeSlot; N]> for NodePath {
    fn from(value: [NodeSlot; N]) -> Self {
        Self(value.into())
    }
}

/// Display the address as a dot separated list
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
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

/// A property collected when condensing a node
#[derive(Debug, Clone)]
pub struct CondenseProperty {
    pub ancestry: NodeAncestry,
    pub path: NodePath,
    pub slot: NodeSlot,
    pub value: String,
}

/// A context for the `condense` method of the `MergeNode` trait
///
/// This context is passed to the `condense` method as we perform
/// a depth first traversal of a node. It maintains stacks of node types,
/// node ids and [`NodeSlot`]s which are used to build up the `ancestries`
/// and `properties` of which there is one for each property collected
/// during the walk.
///
/// Whether a property is collected or not is determined by whether it
/// has the `#[merge(format = "xxx")` attribute as declared in the schema YAML files.
pub struct CondenseContext {
    /// The stack of node types in the current walk
    ///
    /// Required so that the type can be associated with a `NodeSlot::Property`
    /// variant when we enter a node.
    types: Vec<NodeType>,

    /// The current ancestry (stack of node ids) in the walk
    ancestry: NodeAncestry,

    /// The current path (stack of node slots) in the walk
    path: NodePath,

    /// The slot and value of each property collected in the walk
    ///
    /// The `NodeSlot` is included in the property tuple to help disambiguate
    /// properties which have the same value, but which are for entirely different
    /// properties on different types.
    ///
    /// Currently, a `String` is used to store the value of the property.
    /// Most diff-able properties are strings but some are not e.g. integers, enums
    /// It may be better to use a `Primitive` instead of a `String` to avoid
    /// unnecessary string-ification and de-string-ification.
    pub properties: Vec<CondenseProperty>,
}

impl Default for CondenseContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CondenseContext {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            ancestry: NodeAncestry::new(),
            path: NodePath::new(),
            properties: Vec::new(),
        }
    }
}

/// Display the context in tabular format
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
impl Debug for CondenseContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.properties.is_empty() {
            f.write_str("No source properties")?;
        }

        // Find the maximum widths in each column
        let (ancestry_width, path_width, slot_width) = self.properties.iter().fold(
            (0, 0, 0),
            |(ancestry_width, path_width, slot_width), node| {
                (
                    ancestry_width.max(format!("{:?}", node.ancestry).len()),
                    path_width.max(format!("{:?}", node.path).len()),
                    slot_width.max(format!("{:?}", node.slot).len()),
                )
            },
        );

        // Now, output using those widths
        for (i, node) in self.properties.iter().enumerate() {
            let ancestry = format!("{:?}", node.ancestry);
            let path = format!("{:?}", node.path);
            let slot = format!("{:?}", node.slot);
            let value = node.value.replace('\n', r"\\n");
            f.write_fmt(format_args!(
                "{i:<3}  {ancestry:<ancestry_width$}  {path:<path_width$}  {slot:<slot_width$}  \"{value}\"\n",
            ))?;
        }

        Ok(())
    }
}

impl CondenseContext {
    /// Enter a node during the walk
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.types.push(node_type);
        self.ancestry.push(node_id);
        self
    }

    /// Exit a node during the walk
    pub fn exit_node(&mut self) -> &mut Self {
        self.types.pop();
        self.ancestry.pop();
        self
    }

    /// Enter a property during the walk
    pub fn enter_property(&mut self, node_property: NodeProperty) -> &mut Self {
        let node_type = self
            .types
            .last()
            .expect("only called after entering a node");
        self.path
            .push_back(NodeSlot::Property((*node_type, node_property)));
        self
    }

    /// Exit a property during the walk
    pub fn exit_property(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(NodeSlot::Property(..))));
        self
    }

    /// Enter an item in a vector during the walk
    pub fn enter_index(&mut self, index: usize) -> &mut Self {
        self.path.push_back(NodeSlot::Index(index));
        self
    }

    /// Exit an item in a vector during the walk
    pub fn exit_index(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(NodeSlot::Index(..))));
        self
    }

    /// Collected a property value during the walk
    pub fn collect_value(&mut self, value: &str) -> &mut Self {
        // Clone the last slot in the path to return in `diffable_properties`
        let slot = self.path.back().cloned().unwrap_or(NodeSlot::Index(0));

        self.properties.push(CondenseProperty {
            ancestry: self.ancestry.clone(),
            path: self.path.clone(),
            slot,
            value: value.to_string(),
        });
        self
    }
}
