use std::future::Future;

use common::{async_recursion::async_recursion, eyre::Result};
use node_id::NodeId;
use node_type::{NodeProperty, NodeType};

use crate::{
    Array, Block, Boolean, CreativeWorkType, IfBlockClause, Inline, Integer, ListItem, Node, Null,
    Number, Object, SuggestionBlock, SuggestionInline, TableCell, TableRow, UnsignedInteger,
    WalkthroughStep,
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

    /// Visit a `SuggestionBlock` node type
    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `SuggestionInline` node type
    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit an `IfBlockClause` node
    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `ListItem` node
    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `TableRow` node
    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `TableCell` node
    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit a `WalkthroughStep` node
    fn visit_walkthrough_step(&mut self, step: &WalkthroughStep) -> WalkControl {
        WalkControl::Continue
    }

    /// Enter a struct
    fn enter_struct(&mut self, node_type: NodeType, node_id: NodeId) {}

    /// Exit a struct
    fn exit_struct(&mut self) {}

    /// Enter a property
    fn enter_property(&mut self, property: NodeProperty) {}

    /// Exit a property
    fn exit_property(&mut self) {}

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) {}

    /// Exit a node at an index
    fn exit_index(&mut self) {}
}

/// A mutating node visitor
///
/// Unlike [`Visitor`], the methods of [`VisitorMut`] are able to mutate both the visitor,
/// and the visited node.
#[allow(unused_variables)]
pub trait VisitorMut: Sized {
    /// Visit, and potentially mutate, a node
    fn visit<T: WalkNode>(&mut self, node: &mut T) {
        node.walk_mut(self)
    }

    /// Visit, and potentially mutate, a `Node` node type
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `CreativeWork` node type
    fn visit_work(&mut self, work: &mut CreativeWorkType) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `Block` node type
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, an `Inline` node type
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `SuggestionBlock` node type
    fn visit_suggestion_block(&mut self, block: &mut SuggestionBlock) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `SuggestionInline` node type
    fn visit_suggestion_inline(&mut self, clause: &mut SuggestionInline) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, an `IfBlockClause` node
    fn visit_if_block_clause(&mut self, inline: &mut IfBlockClause) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `ListItem` node
    fn visit_list_item(&mut self, list_item: &mut ListItem) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `TableRow` node
    fn visit_table_row(&mut self, table_row: &mut TableRow) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `TableCell` node
    fn visit_table_cell(&mut self, table_cell: &mut TableCell) -> WalkControl {
        WalkControl::Continue
    }

    /// Visit, and potentially mutate, a `WalkthroughStep` node
    fn visit_walkthrough_step(&mut self, step: &mut WalkthroughStep) -> WalkControl {
        WalkControl::Continue
    }

    /// Enter a struct
    fn enter_struct(&mut self, node_type: NodeType, node_id: NodeId) {}

    /// Exit a struct
    fn exit_struct(&mut self) {}

    /// Enter a property
    fn enter_property(&mut self, property: NodeProperty) {}

    /// Exit a property
    fn exit_property(&mut self) {}

    /// Enter a node at an index
    fn enter_index(&mut self, index: usize) {}

    /// Exit a node at an index
    fn exit_index(&mut self) {}
}

/// A mutating node visitor with asynchronous methods
///
/// Like [`VisitorMut`] but with async and fallible `visit_*` methods.
#[allow(unused_variables, async_fn_in_trait)]
pub trait VisitorAsync: Send + Sync {
    /// Visit, and potentially mutate, a `Node` node type
    fn visit_node(&mut self, node: &mut Node) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `CreativeWork` node type
    fn visit_work(
        &mut self,
        work: &mut CreativeWorkType,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `Block` node type
    fn visit_block(
        &mut self,
        block: &mut Block,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, an `Inline` node type
    fn visit_inline(
        &mut self,
        inline: &mut Inline,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `SuggestionBlock` node type
    fn visit_suggestion_block(
        &mut self,
        block: &mut SuggestionBlock,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `SuggestionInline` node type
    fn visit_suggestion_inline(
        &mut self,
        inline: &mut SuggestionInline,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, an `IfBlockClause` node
    fn visit_if_block_clause(
        &mut self,
        clause: &mut IfBlockClause,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `ListItem` node
    fn visit_list_item(
        &mut self,
        list_item: &mut ListItem,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `TableRow` node
    fn visit_table_row(
        &mut self,
        table_row: &mut TableRow,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `TableCell` node
    fn visit_table_cell(
        &mut self,
        table_cell: &mut TableCell,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Visit, and potentially mutate, a `WalkthroughStep` node
    fn visit_walkthrough_step(
        &mut self,
        step: &WalkthroughStep,
    ) -> impl Future<Output = Result<WalkControl>> + Send {
        async { Ok(WalkControl::Continue) }
    }

    /// Enter a struct
    fn enter_struct(&mut self, node_type: NodeType, node_id: NodeId) {}

    /// Exit a struct
    fn exit_struct(&mut self) {}

    /// Enter a property
    fn enter_property(&mut self, property: NodeProperty) {}

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
#[allow(unused_variables, async_fn_in_trait)]
pub trait WalkNode {
    /// Walk over a node's children
    fn walk<V: Visitor>(&self, visitor: &mut V) {}

    /// Walk over, and potentially mutate, a node's children
    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {}

    /// Walk over, and potentially mutate, a node's children asynchronously and fallibly
    #[async_recursion]
    async fn walk_async<V>(&mut self, visitor: &mut V) -> Result<()>
    where
        V: VisitorAsync,
    {
        Ok(())
    }
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
    T: WalkNode + Send,
{
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        self.as_ref().walk(visitor)
    }

    fn walk_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        self.as_mut().walk_mut(visitor)
    }

    #[async_recursion]
    async fn walk_async<V>(&mut self, visitor: &mut V) -> Result<()>
    where
        V: VisitorAsync,
    {
        self.as_mut().walk_async(visitor).await
    }
}

impl<T> WalkNode for Option<T>
where
    T: WalkNode + Send,
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

    #[async_recursion]
    async fn walk_async<V>(&mut self, visitor: &mut V) -> Result<()>
    where
        V: VisitorAsync,
    {
        if let Some(value) = self {
            value.walk_async(visitor).await
        } else {
            Ok(())
        }
    }
}

impl<T> WalkNode for Vec<T>
where
    T: WalkNode + Send,
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

    #[async_recursion]
    async fn walk_async<V>(&mut self, visitor: &mut V) -> Result<()>
    where
        V: VisitorAsync,
    {
        for (index, node) in self.iter_mut().enumerate() {
            visitor.enter_index(index);
            node.walk_async(visitor).await?;
            visitor.exit_index();
        }

        Ok(())
    }
}
