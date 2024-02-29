use common::eyre::Result;
use kernels::Kernels;
use schema::{
    walk::{VisitorAsync, WalkControl, WalkNode},
    Block, Inline, Node, NodeId,
};

mod prelude;

mod article;
mod call_block;
mod code_chunk;
mod code_expression;
mod for_block;
mod if_block;
mod include_block;
mod instruction;
mod math;
mod styled;

/// Walk over a node and execute it and all its child nodes
pub async fn execute<T: WalkNode>(
    node: &mut T,
    kernels: &mut Kernels,
    node_ids: Option<Vec<NodeId>>,
) -> Result<()> {
    let mut executor = Executor { kernels, node_ids };
    node.walk_async(&mut executor).await
}

/// A trait for an executable node
trait Executable {
    /// Execute the node
    ///
    /// Note that this method is intentionally infallible because we want
    /// executable nodes to handle any errors associated with their execution
    /// and record them in `execution_messages` so that they are visible
    /// to the user.
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl;
}

/// A visitor that walks over a tree of nodes and collects
/// execution tasks from nodes

struct Executor<'lt> {
    /// The kernels that will be used for execution
    kernels: &'lt mut Kernels,

    /// The nodes that should be executed
    ///
    /// If `None` then the entire document will be executed.
    node_ids: Option<Vec<NodeId>>,
}

impl<'lt> VisitorAsync for Executor<'lt> {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &node.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        use Node::*;
        Ok(match node {
            Article(node) => node.execute(self).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &block.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        use Block::*;
        Ok(match block {
            // TODO: CallBlock(node) => node.execute(self).await,
            CodeChunk(node) => node.execute(self).await,
            ForBlock(node) => node.execute(self).await,
            IfBlock(node) => node.execute(self).await,
            // TODO: IncludeBlock(node) => node.execute(self).await,
            // TODO: InstructionBlock(node) => node.execute(self).await,
            // TODO: MathBlock(node) => node.execute(self).await,
            // TODO: StyledBlock(node) => node.execute(self).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &inline.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        use Inline::*;
        Ok(match inline {
            CodeExpression(node) => node.execute(self).await,
            // TODO: InstructionInline(node) => node.execute(self).await,
            // TODO: MathInline(node) => node.execute(self).await,
            // TODO: Parameter(node) => node.execute(self).await,
            // TODO: StyledInline(node) => node.execute(self).await,
            _ => WalkControl::Continue,
        })
    }
}
