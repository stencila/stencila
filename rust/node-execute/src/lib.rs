#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    eyre::Result,
    tokio::sync::{RwLock, RwLockWriteGuard},
    tracing,
};
use context::Context;
use kernels::Kernels;
use node_patch::{
    load_property, replace_property, NodePatch, NodePatchSender, Operation, Property, Value,
};
use node_store::{ReadNode, WriteStore};
use schema::{
    walk::{VisitorAsync, WalkControl, WalkNode},
    Block, Inline, Node, NodeId,
};

type NodeIds = Vec<NodeId>;

mod prelude;

mod article;
mod call_block;
mod code_chunk;
mod code_expression;
mod for_block;
mod if_block;
mod include_block;
mod instruction_block;
mod math;
mod styled;

/// Walk over a root node and execute it and child nodes
pub async fn execute(
    home: PathBuf,
    store: Arc<RwLock<WriteStore>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: NodePatchSender,
    node_ids: Option<NodeIds>,
) -> Result<()> {
    let mut root = {
        // This is within a block to ensure that the lock on `store` gets
        // dropped before execution
        let store = store.read().await;
        Node::load(&*store).unwrap()
    };

    let mut executor = Executor::new(home, store, kernels, patch_sender, node_ids);
    executor.pending(&mut root).await?;
    executor.execute(&mut root).await
}

/// Walk over a root node and interrupt it and child nodes
pub async fn interrupt(
    home: PathBuf,
    store: Arc<RwLock<WriteStore>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: NodePatchSender,
    node_ids: Option<NodeIds>,
) -> Result<()> {
    let mut root = {
        // This is within a block to ensure that the lock on `store` gets
        // dropped before execution
        let store = store.read().await;
        Node::load(&*store).unwrap()
    };

    let mut executor = Executor::new(home, store, kernels, patch_sender, node_ids);
    executor.interrupt(&mut root).await
}

/// A trait for an executable node
trait Executable {
    /// Set the execution status of the node to pending
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl;

    /// Execute the node
    ///
    /// Note that this method is intentionally infallible because we want
    /// executable nodes to handle any errors associated with their execution
    /// and record them in `execution_messages` so that they are visible
    /// to the user.
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl;

    /// Interrupt execution of the node
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl;
}

/// A visitor that walks over a tree of nodes and executes them
pub struct Executor {
    /// The home directory of the document being executed
    ///
    /// Used to resolve relative file paths in `IncludeBlock` and `CallBlock` nodes.
    home: PathBuf,

    /// The store of the root node
    store: Arc<RwLock<WriteStore>>,

    /// The kernels that will be used for execution
    kernels: Arc<RwLock<Kernels>>,

    /// A sender for a [`NodePatch`] channel
    ///
    /// Patches reflecting the state of nodes during execution should be sent
    /// on this channel.
    patch_sender: NodePatchSender,

    /// The nodes that should be executed
    ///
    /// If `None` then the entire node (usually an `Article`) will be executed.
    node_ids: Option<NodeIds>,

    /// The phase of execution
    phase: Phase,

    /// The document context
    context: Context,

    /// Whether the current node is the last in a set
    ///
    /// Used for `IfBlock` (and possibly others) to control behavior of execution
    /// of child nodes.
    is_last: bool,
}

/// A phase of an [`Executor`]
///
/// These phases determine which method of each [`Executable`] is called as
/// the executor walks over the root node.
enum Phase {
    Pending,
    Execute,
    Interrupt,
}

impl Executor {
    /// Create a new executor
    fn new(
        home: PathBuf,
        store: Arc<RwLock<WriteStore>>,
        kernels: Arc<RwLock<Kernels>>,
        patch_sender: NodePatchSender,
        node_ids: Option<NodeIds>,
    ) -> Self {
        Self {
            home,
            store,
            kernels,
            patch_sender,
            node_ids,
            phase: Phase::Pending,
            context: Context::default(),
            is_last: false,
        }
    }

    /// Run [`Phase::Pending`]
    async fn pending(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Pending;
        self.is_last = false;
        root.walk_async(self).await
    }

    /// Run [`Phase::Execute`]
    async fn execute(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Execute;
        self.is_last = false;

        // Create a new context before walking the tree. Note that
        // this means that instructions will on "see" the other nodes that
        // precede them in the document.
        self.context = Context::default();

        root.walk_async(self).await
    }

    /// Run [`Phase::Interrupt`]
    async fn interrupt(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Interrupt;
        self.is_last = false;
        root.walk_async(self).await
    }

    /// Get the home directory of the executor
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Obtain a write lock to the kernels
    ///
    /// Used by [`Executable`] nodes to execute and evaluate code and manage variables.
    pub async fn kernels(&self) -> RwLockWriteGuard<Kernels> {
        self.kernels.write().await
    }

    /// Get the document context
    ///
    /// Returns the nodes collected during walking the root node
    /// and updates it with kernels.
    ///
    /// Used by [`Executable`] nodes to pass to assistants to be used
    /// in their system prompts.
    pub async fn context(&mut self) -> Context {
        let kernels = self.kernels().await.kernel_contexts().await;
        self.context.kernels = kernels;
        self.context.clone()
    }

    /// Load a property of a node from the store
    ///
    /// Creates and sends a patch with a single `ReplaceProperty` operation.
    pub async fn swap_property<T>(
        &self,
        node_id: &NodeId,
        property: Property,
        value: Value,
    ) -> Result<T>
    where
        T: ReadNode,
    {
        let mut store = self.store.write().await;
        replace_property(&mut store, node_id, property, value)?;
        load_property(&*store, node_id, property)
    }

    /// Replace several properties of a node
    pub fn replace_properties<P>(&self, node_id: &NodeId, pairs: P)
    where
        P: IntoIterator<Item = (Property, Value)>,
    {
        self.send_patch(NodePatch {
            node_id: node_id.clone(),
            ops: pairs
                .into_iter()
                .map(|(property, value)| Operation::replace_property(property, value))
                .collect(),
        })
    }

    /// Send a patch reflecting a change in the state of a node during execution
    pub fn send_patch(&self, patch: NodePatch) {
        if let Err(error) = self.patch_sender.send(patch) {
            tracing::error!("When sending execution node patch: {error}")
        }
    }

    /// Visit an executable node and call the appropriate method for the phase
    async fn visit_executable<E: Executable>(&mut self, node: &mut E) -> WalkControl {
        match self.phase {
            Phase::Pending => node.pending(self).await,
            Phase::Execute => node.execute(self).await,
            Phase::Interrupt => node.interrupt(self).await,
        }
    }
}

impl VisitorAsync for Executor {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        // If the executor has node ids (i.e. is only executing some nodes, not the entire
        // document) then do not execute this node if it is not in the node ids.
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &node.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        use Node::*;
        let control = match node {
            Article(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        use Block::*;

        // If the block is of a type that is collected in the execution context
        // then do that. Executable nodes should not be added here. Instead they should
        // only be added to the context at the end of their `execute` method is successful
        if let Paragraph(paragraph) = &block {
            self.context.push_paragraph(paragraph)
        }

        // If the executor has node ids (i.e. is only executing some nodes, not the entire
        // document) then do not execute this block if it is not in the node ids.
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &block.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        let control = match block {
            // TODO: CallBlock(node) => self.visit_executable(node).await,
            CodeChunk(node) => self.visit_executable(node).await,
            ForBlock(node) => self.visit_executable(node).await,
            IfBlock(node) => self.visit_executable(node).await,
            IncludeBlock(node) => self.visit_executable(node).await,
            InstructionBlock(node) => self.visit_executable(node).await,
            // TODO: MathBlock(node) => self.visit_executable(node).await,
            // TODO: StyledBlock(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        use Inline::*;

        // If the inline is of a type that is collected in the execution context
        // then do that. Executable nodes should not be added here. Instead they should
        // only be added to the context at the end of their `execute` method is successful
        if let Text(text) = &inline {
            self.context.push_text(text)
        }

        // If the executor has node ids (i.e. is only executing some nodes, not the entire
        // document) then do not execute this inline if it is not in the node ids.
        if let Some(node_ids) = &self.node_ids {
            if let Some(node_id) = &inline.node_id() {
                if !node_ids.contains(node_id) {
                    return Ok(WalkControl::Continue);
                }
            }
        }

        let control = match inline {
            CodeExpression(node) => self.visit_executable(node).await,
            // TODO: InstructionInline(node) => self.visit_executable(node).await,
            // TODO: MathInline(node) => self.visit_executable(node).await,
            // TODO: Parameter(node) => self.visit_executable(node).await,
            // TODO: StyledInline(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }
}
