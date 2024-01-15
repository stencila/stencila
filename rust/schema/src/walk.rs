use crate::{
    Array, Block, Boolean, CreativeWorkType, Inline, Integer, Node, Null, Number, Object,
    UnsignedInteger,
};

/// Controls whether to continue walking over a node or not
///
/// Similar to `std::ops::ControlFlow` but not a generic, which
/// makes this slightly more ergonomic to use.
///
/// When the `Try` trait is stabilized this should implement it
/// to be able to use the `?` operator on it.
/// https://doc.rust-lang.org/std/ops/trait.Try.html
pub enum WalkControl {
    Break,
    Continue,
}

impl WalkControl {
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break)
    }

    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

/// A node visitor
///
/// The methods of this trait are called while walking over nodes in a node tree.
/// They return `true` to indicate that the walk should continue downwards through
/// the tree and `false` otherwise.
///
/// The methods are able to mutate the visitor, but not the visited node. Use
/// `VisitorMut` when it is necessary to modify the visited node.
#[allow(unused_variables)]
pub trait Visitor: Sized {
    /// Visit a node
    fn visit<T: WalkNode>(&mut self, node: &T) {
        node.walk(self)
    }

    /// Visit a `Node` node type
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `CreativeWork` node type
    fn visit_work(&mut self, work: &CreativeWorkType) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `Block` node type
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit an `Inline` node type
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        WalkControl::Continue
    }

    /// Enter a property
    fn enter_property(&mut self, name: &str) {}

    /// Exit a property
    fn exit_property(&mut self) {}

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) {}

    /// Exit a node at an index
    fn exit_index(&mut self) {}
}

/// A mutating node visitor
///
/// Unlink [`Visitor`], the methods of [`VisitorMut`] are able to mutate both the visitor,
/// and the visited node.
#[allow(unused_variables)]
pub trait VisitorMut: Sized {
    /// Visit, and potentially mutate, a node
    fn visit_mut<T: WalkNode>(&mut self, node: &mut T) {
        node.walk_mut(self)
    }

    /// Visit, and potentially mutate, a `Node` node type
    fn visit_node_mut(&mut self, node: &mut Node) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `CreativeWork` node type
    fn visit_work_mut(&mut self, work: &mut CreativeWorkType) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `Block` node type
    fn visit_block_mut(&mut self, block: &mut Block) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, an `Inline` node type
    fn visit_inline_mut(&mut self, inline: &mut Inline) -> WalkControl {
        WalkControl::Continue
    }

    /// Enter a property
    fn enter_property(&mut self, name: &str) {}

    /// Exit a property
    fn exit_property(&mut self) {}

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) {}

    /// Exit a node at an index
    fn exit_index(&mut self) {}
}

/// A trait for walking over a node's children
///
/// The default implementation of both `walk` and `walk_mut`
/// do nothing.
#[allow(unused_variables)]
pub trait WalkNode {
    /// Walk over a node's children
    fn walk<V: Visitor>(&self, visitor: &mut V) {}

    /// Walk over, and potentially mutate, a node's children
    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {}
}

impl WalkNode for Null {}
impl WalkNode for Boolean {}
impl WalkNode for Integer {}
impl WalkNode for UnsignedInteger {}
impl WalkNode for Number {}
impl WalkNode for String {}
impl WalkNode for Array {}
impl WalkNode for Object {}

impl<T> WalkNode for Box<T>
where
    T: WalkNode,
{
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        self.as_ref().walk(visitor)
    }

    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        self.as_mut().walk_mut(visitor)
    }
}

impl<T> WalkNode for Option<T>
where
    T: WalkNode,
{
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        if let Some(value) = self {
            value.walk(visitor);
        }
    }

    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        if let Some(value) = self {
            value.walk_mut(visitor);
        }
    }
}

impl<T> WalkNode for Vec<T>
where
    T: WalkNode,
{
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        for (index, node) in self.iter().enumerate() {
            visitor.enter_index(index);
            node.walk(visitor);
            visitor.exit_index();
        }
    }

    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        for (index, node) in self.iter_mut().enumerate() {
            visitor.enter_index(index);
            node.walk_mut(visitor);
            visitor.exit_index();
        }
    }
}
