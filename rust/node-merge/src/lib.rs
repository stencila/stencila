use std::fmt::{self, Display};

use common::derive_more::{Deref, DerefMut};
use node_id::NodeId;

pub use node_merge_derive::MergeNode;
use node_type::{NodeProperty, NodeType};

/// A trait for merging a node into another
///
/// The node that is being merged in from is usually one that has just been
/// decoded from a lossy source format such a Markdown. The node that is being merged into
/// is usually one that has already been executed and thus has derived properties (e.g `executionStatus`
/// and `outputs`) which we want to preserve.
pub trait MergeNode {
    /// Flatten a nodes source properties into a vector of strings
    ///
    /// This default implementation does nothing. Implementors should
    /// call the method of the context to add source properties.
    #[allow(unused_variables)]
    fn condense(&self, context: &mut CondenseContext) {}
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

/// A list of node ids in the path to a property
///
/// This list of ids is stored for each property so that we can determine whether
/// an entire node is to be deleted or moved.
#[derive(Clone, Deref, DerefMut)]
pub struct NodeHandle(Vec<NodeId>);

/// Display the handle as a forward slash separated list
///
/// Intended only for testing and debugging during development.
impl Display for NodeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, id) in self.iter().enumerate() {
            if index != 0 {
                f.write_str("/")?;
            }
            id.fmt(f)?;
        }

        Ok(())
    }
}

/// A part of a node address: either a property identifier or the index of a vector.
#[derive(Clone)]
pub enum NodeSlot {
    Property((NodeType, NodeProperty)),
    Index(usize),
}

/// Display the slot
///
/// Intended only for testing and debugging during development.
impl Display for NodeSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeSlot::Property((node_type, node_prop)) => {
                f.write_fmt(format_args!("{node_type}:{node_prop}"))
            }
            NodeSlot::Index(index) => index.fmt(f),
        }
    }
}

///
#[derive(Clone, Deref, DerefMut)]
pub struct NodeAddress(Vec<NodeSlot>);

/// Display the address as a dot separated list
///
/// Intended only for testing and debugging during development.
impl Display for NodeAddress {
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
/// node ids and [`NodeSlot`]s which are used to build up the `handles`
/// and `properties` of which there is one for each property collected
/// during the walk.
///
/// Whether a property is collected or not is determined by whether it
/// has the `#[merge(format = "xxx")` attribute as declared in the schema YAML files.
pub struct CondenseContext {
    /// The stack of node types in the current walk
    types: Vec<NodeType>,

    /// The current handle (stack of node ids) in the current walk
    handle: NodeHandle,

    /// The current address (stack of node slots) in the current walk
    address: NodeAddress,

    /// The handle for each property collected in the walk
    handles: Vec<NodeHandle>,

    /// The address and value of each property collected in the walk
    properties: Vec<(NodeAddress, String)>,
}

impl CondenseContext {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            handle: NodeHandle(Vec::new()),
            address: NodeAddress(Vec::new()),
            handles: Vec::new(),
            properties: Vec::new(),
        }
    }
}

/// Display the context in tabular format
///
/// Intended only for testing and debugging during development.
impl Display for CondenseContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.properties.is_empty() {
            f.write_str("No source properties")?;
        }

        // Find the maximum widths in each column
        let (handle_width, address_width) = self.handles.iter().zip(self.properties.iter()).fold(
            (0usize, 0usize),
            |(handle_width, address_width), (handle, (address, ..))| {
                (
                    handle_width.max(handle.to_string().len()),
                    address_width.max(address.to_string().len()),
                )
            },
        );

        // Now, output using those widths
        for (handle, (address, source)) in self.handles.iter().zip(self.properties.iter()) {
            let handle = handle.to_string();
            let address = address.to_string();
            let source = source.replace('\n', r"\\n");
            f.write_fmt(format_args!(
                "{handle:<handle_width$}  {address:<address_width$}  \"{source}\"\n",
            ))?;
        }

        Ok(())
    }
}

impl CondenseContext {
    /// Enter a node during the walk
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.types.push(node_type);
        self.handle.push(node_id);
        self
    }

    /// Exit a node during the walk
    pub fn exit_node(&mut self) -> &mut Self {
        self.types.pop();
        self.handle.pop();
        self
    }

    /// Enter a property during the walk
    pub fn enter_property(&mut self, node_property: NodeProperty) -> &mut Self {
        let node_type = self
            .types
            .last()
            .expect("only called after entering a node");
        self.address
            .push(NodeSlot::Property((*node_type, node_property)));
        self
    }

    /// Exit a property during the walk
    pub fn exit_property(&mut self) -> &mut Self {
        let popped = self.address.pop();
        debug_assert!(matches!(popped, Some(NodeSlot::Property(..))));
        self
    }

    /// Enter an item in a vector during the walk
    pub fn enter_index(&mut self, index: usize) -> &mut Self {
        self.address.push(NodeSlot::Index(index.into()));
        self
    }

    /// Exit an item in a vector during the walk
    pub fn exit_index(&mut self) -> &mut Self {
        let popped = self.address.pop();
        debug_assert!(matches!(popped, Some(NodeSlot::Index(..))));
        self
    }

    /// Collected a property value during the walk
    pub fn collect_value(&mut self, value: &str) -> &mut Self {
        self.handles.push(self.handle.clone());
        self.properties
            .push((self.address.clone(), value.to_string()));
        self
    }
}
