//! A crate for diffing, patching and merging (diffing then patching) nodes
//!
//! This crate duplicates some functionality that is already in the sibling
//! `node-patch` and `node-map` crates. It is anticipated that this crate
//! will replace those.

use std::fmt::{self, Debug};

use common::similar::DiffTag;
use common::{
    derive_more::{Deref, DerefMut},
    similar::{
        algorithms::{diff_slices, Algorithm, Capture, Compact},
        DiffOp,
    },
};
use node_id::NodeId;

pub use node_merge_derive::MergeNode;
use node_type::{NodeProperty, NodeType};

/// A trait for diffing, patching and merging nodes
///
/// The node that is being merged in from is usually one that has just been
/// decoded from a lossy source format such a Markdown. The node that is being merged into
/// is usually one that has already been executed and thus has derived properties
/// (e.g `executionStatus` and `outputs`) which we want to preserve.
pub trait MergeNode {
    /// Condense a node into a list of properties that can be diffed
    ///
    /// This default implementation does nothing. Implementors should
    /// call the various methods of the context to collect properties.
    #[allow(unused_variables)]
    fn condense(&self, context: &mut CondenseContext) {}

    /// Diff the node with another
    ///
    /// Calculates the diff operations necessary to make `self` the
    /// same as the `new` node. Does so by condensing each node from a tree
    /// of properties into a list of diff-able properties. Which properties
    /// are treated as diff-able will be dependent on the format from which
    /// the `new` node was decoded. Presently however, only one format,
    /// Markdown is considered.
    fn diff<M>(&self, new: M) -> DiffResult
    where
        M: MergeNode,
    {
        let mut old_context = CondenseContext::new();
        self.condense(&mut old_context);

        let mut new_context = CondenseContext::new();
        new.condense(&mut new_context);

        let old = old_context.diffable_properties();
        let new = new_context.diffable_properties();

        let mut diff_hook = Compact::new(Capture::new(), &old, &new);
        diff_slices(Algorithm::Patience, &mut diff_hook, &old, &new).unwrap();
        let diff_ops = diff_hook.into_inner().into_ops();

        let mut patch_ops: Vec<NodeOp> = Vec::new();
        for op in &diff_ops {
            let (op, old, new) = op.as_tag_tuple();
            match op {
                DiffTag::Insert => {
                    for i in new {
                        let pth = new_context.properties[i].1.clone();
                        let node = new_context.properties[i].3.clone();
                        patch_ops.push(NodeOp::Add((pth, node)));
                    }

                    // patch_ops.push(NodeOp::Add(
                    //     NodePath(vec![NodeSlot::Index(*old_index)]),
                    //     new[*new_index..*new_index + *new_len].join("\n"),
                    // )
                }
                // DiffOp::Delete(..) => {}
                DiffTag::Delete => {
                    for i in old {
                        patch_ops.push(NodeOp::Remove(old_context.properties[i].1.clone()));
                    }
                }
                _ => {}
            }
        }
        // let patch_ops = vec![NodeOp::Replace((
        //     NodePath(vec![NodeSlot::Property((
        //         NodeType::MathBlock,
        //         NodeProperty::Code,
        //     ))]),
        //     "foo".to_string(),
        // ))];

        DiffResult {
            #[cfg(debug_assertions)]
            old_context,
            #[cfg(debug_assertions)]
            new_context,
            diff_ops,
            node_ops: patch_ops,
        }
    }
}

// Implementation for simple "atomic" types not in schema
macro_rules! atom {
    ($type:ty) => {
        impl MergeNode for $type {
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
impl<T> MergeNode for Box<T>
where
    T: MergeNode,
{
    fn condense(&self, context: &mut CondenseContext) {
        self.as_ref().condense(context)
    }
}

// Implementation for optional properties
impl<T> MergeNode for Option<T>
where
    T: MergeNode,
{
    fn condense(&self, context: &mut CondenseContext) {
        if let Some(value) = self {
            value.condense(context);
        }
    }
}

// Implementation for vector properties
impl<T> MergeNode for Vec<T>
where
    T: MergeNode,
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
pub struct NodePath(Vec<NodeSlot>);

impl NodePath {
    fn ancestor_index(&self, other: &NodePath) -> Option<usize> {
        let mut i: usize = 0;
        for slots in self.iter().zip(other.iter()) {
            if slots.0 != slots.1 {
                break;
            }
            i += 1;
        }
        if i == 0 {
            None
        } else {
            Some(i - 1)
        }
    }

    fn find_ancestor_index(paths: &Vec<NodePath>) -> Option<usize> {
        let split = paths.split_first();
        let mut i: usize = 0;
        if split.is_some() {
            let (first, others) = split.unwrap();
            for o in others {
                let lca = first.ancestor_index(o);
                lca?;
                i = i.min(lca.unwrap());
            }
        }
        Some(i)
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
    properties: Vec<(NodeAncestry, NodePath, NodeSlot, String)>,
}

impl CondenseContext {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            ancestry: NodeAncestry(Vec::new()),
            path: NodePath(Vec::new()),
            properties: Vec::new(),
        }
    }

    /// Get the properties as a diff-able tuple of (slot, value)
    ///
    /// This excludes the ancestry and path of a property since they should
    /// not be considered in the diffing (although both are used for creating
    /// patches from the diff operations)
    fn diffable_properties(&self) -> Vec<(&NodeSlot, &String)> {
        self.properties
            .iter()
            .map(|(.., slot, value)| (slot, value))
            .collect()
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
            |(ancestry_width, path_width, slot_width), (ancestry, path, slot, ..)| {
                (
                    ancestry_width.max(format!("{ancestry:?}").len()),
                    path_width.max(format!("{path:?}").len()),
                    slot_width.max(format!("{slot:?}").len()),
                )
            },
        );

        // Now, output using those widths
        for (ancestry, path, slot, value) in self.properties.iter() {
            let ancestry = format!("{ancestry:?}");
            let path = format!("{path:?}");
            let slot = format!("{slot:?}");
            let value = value.replace('\n', r"\\n");
            f.write_fmt(format_args!(
                "{ancestry:<ancestry_width$}  {path:<path_width$}  {slot:<slot_width$}  \"{value}\"\n",
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
            .push(NodeSlot::Property((*node_type, node_property)));
        self
    }

    /// Exit a property during the walk
    pub fn exit_property(&mut self) -> &mut Self {
        let popped = self.path.pop();
        debug_assert!(matches!(popped, Some(NodeSlot::Property(..))));
        self
    }

    /// Enter an item in a vector during the walk
    pub fn enter_index(&mut self, index: usize) -> &mut Self {
        self.path.push(NodeSlot::Index(index.into()));
        self
    }

    /// Exit an item in a vector during the walk
    pub fn exit_index(&mut self) -> &mut Self {
        let popped = self.path.pop();
        debug_assert!(matches!(popped, Some(NodeSlot::Index(..))));
        self
    }

    /// Collected a property value during the walk
    pub fn collect_value(&mut self, value: &str) -> &mut Self {
        // Clone the last slot in the path to return in `diffable_properties`
        let slot = self
            .path
            .last()
            .cloned()
            .unwrap_or_else(|| NodeSlot::Index(0));

        self.properties.push((
            self.ancestry.clone(),
            self.path.clone(),
            slot,
            value.to_string(),
        ));
        self
    }
}

/// An operation to apply to a node
///
/// Similar, and using the same operation names as operations in
/// JSON Patch (https://jsonpatch.com/) (but using enum and tuples).
#[derive(Debug)]
enum NodeOp {
    Add((NodePath, Node)),
    Remove(NodePath),
    Replace((NodePath, Node)),
    Move((NodePath, NodePath)),
}

// TODO: This should be an actual schema node but for now can not be
// because of circular dependency if we depend on schema
type Node = String;

/// The result from a diff operation
///
/// During development the result includes the two condense contexts and diff ops
/// in addition to the generated patch ops. This is mostly done so that we can use them when creating
/// snapshots tests to understand the algorithm
pub struct DiffResult {
    #[cfg(debug_assertions)]
    old_context: CondenseContext,

    #[cfg(debug_assertions)]
    new_context: CondenseContext,

    diff_ops: Vec<DiffOp>,

    node_ops: Vec<NodeOp>,
}

/// Display the diff result as three sets of tables
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
impl Debug for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.old_context.fmt(f)?;
        writeln!(f)?;
        self.new_context.fmt(f)?;
        writeln!(f)?;

        writeln!(f, "DiffOp       Old range    New range")?;
        for op in &self.diff_ops {
            let (tag, old_range, new_range) = op.as_tag_tuple();
            writeln!(
                f,
                "{:<10}   {}..{}         {}..{}",
                format!("{tag:?}"),
                old_range.start,
                old_range.end,
                new_range.start,
                new_range.end
            )?;
        }

        writeln!(f)?;
        for op in &self.node_ops {
            writeln!(f, "{op:?}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let mut p1 = NodePath(vec![NodeSlot::Property((
            NodeType::Article,
            NodeProperty::Title,
        ))]);
        assert_eq!(p1.ancestor_index(&p1), Some(0));

        let mut p2 = p1.clone();
        p2.push(NodeSlot::Property((
            NodeType::Article,
            NodeProperty::Abstract,
        )));
        assert_eq!(p1.ancestor_index(&p2), Some(0));

        let mut p3 = p2.clone();
        p3.push(NodeSlot::Property((
            NodeType::Article,
            NodeProperty::Abstract,
        )));
        assert_eq!(p2.ancestor_index(&p3), Some(1));
        assert_eq!(p1.ancestor_index(&p3), Some(0));
    }
}
