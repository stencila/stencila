#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    clap::{self, Args},
    eyre::Result,
    serde::{Deserialize, Serialize},
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
    Block, Inline, InstructionBlock, InstructionInline, Node, NodeId, SuggestionBlockType,
    SuggestionInlineType, SuggestionStatus,
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
mod instruction_inline;
mod math_block;
mod math_inline;
mod styled;

/// Walk over a root node and execute it and child nodes
pub async fn execute(
    home: PathBuf,
    store: Arc<RwLock<WriteStore>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: NodePatchSender,
    node_ids: Option<NodeIds>,
    options: Option<ExecuteOptions>,
) -> Result<()> {
    let mut root = {
        // This is within a block to ensure that the lock on `store` gets
        // dropped before execution
        let store = store.read().await;
        Node::load(&*store).unwrap()
    };

    let mut executor = Executor::new(home, store, kernels, patch_sender, node_ids, options);
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

    let mut executor = Executor::new(home, store, kernels, patch_sender, node_ids, None);
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

    /// Options for execution
    options: ExecuteOptions,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Args)]
#[serde(default, crate = "common::serde")]
pub struct ExecuteOptions {
    /// Skip executing code
    ///
    /// By default, code-based nodes in the document (e.g. `CodeChunk`, `CodeExpression`, `ForBlock`)
    /// nodes will be executed if they are stale. Use this flag to skip executing all code-based nodes.
    #[arg(long)]
    skip_code: bool,

    /// Skip executing instructions
    ///
    /// By default, instructions with no suggestions, or with suggestions that have
    /// been rejected will be executed. Use this flag to skip executing all instructions.
    #[arg(long, alias = "skip-inst")]
    skip_instructions: bool,

    /// Re-execute instructions with suggestions that have not yet been reviewed
    ///
    /// By default, an instruction that has a suggestion that has not yet be reviewed
    /// (i.e. has a suggestion status that is empty) will not be re-executed. Use this
    /// flag to force these instructions to be re-executed.
    #[arg(long)]
    force_unreviewed: bool,

    /// Re-execute instructions with suggestions that have been accepted.
    ///
    /// By default, an instruction that has a suggestion that has been accepted, will
    /// not be re-executed. Use this flag to force these instructions to be re-executed.
    #[arg(long)]
    force_accepted: bool,

    /// Skip re-executing instructions with suggestions that have been rejected
    ///
    /// By default, instructions that have a suggestion that has been rejected, will be
    /// re-executed. Use this flag to skip re-execution of these instructions.
    #[arg(long)]
    skip_rejected: bool,

    /// Prepare, but do not actually perform, execution tasks
    ///
    /// Currently only supported by assistants where is is useful for debugging the
    /// rendering of system prompts without making a potentially slow API request.
    #[arg(long)]
    dry_run: bool,
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
        options: Option<ExecuteOptions>,
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
            options: options.unwrap_or_default(),
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

    /// Should the executor execute a code-based node (derived from `CodeExecutable`)
    pub fn should_execute_code(&self) -> bool {
        !self.options.skip_code
    }

    /// Should the executor execute an `Instruction` based on the the
    /// status of its suggestion.
    pub fn should_execute_instruction(&self, status: &Option<SuggestionStatus>) -> bool {
        let Some(status) = status else {
            // Re-execute unreviewed only if `force_unreviewed`
            return self.options.force_unreviewed;
        };

        use SuggestionStatus::*;
        match status {
            // Re-execute proposed only if `force_unreviewed`
            Proposed => self.options.force_unreviewed,
            // Re-execute accepted only if `force_accepted`
            Accepted => self.options.force_accepted,
            // Re-execute rejected unless `skip_reject`
            Rejected => self.options.skip_rejected,
        }
    }

    /// Should the executor execute an `InstructionBlock`
    pub fn should_execute_instruction_block(&self, instruction: &InstructionBlock) -> bool {
        let suggestion = &instruction.options.suggestion;

        // Respect `skip_instructions`
        if self.options.skip_instructions {
            return false;
        }

        let Some(suggestion) = suggestion else {
            // Execute instructions without suggestions
            return true;
        };

        use SuggestionBlockType::*;
        let status = match suggestion {
            DeleteBlock(block) => &block.suggestion_status,
            InsertBlock(block) => &block.suggestion_status,
            ModifyBlock(block) => &block.suggestion_status,
            ReplaceBlock(block) => &block.suggestion_status,
        };

        self.should_execute_instruction(status)
    }

    /// Should the executor execute an `InstructionInline`
    pub fn should_execute_instruction_inline(&self, instruction: &InstructionInline) -> bool {
        let suggestion = &instruction.options.suggestion;

        // Respect `skip_instructions`
        if self.options.skip_instructions {
            return false;
        }

        let Some(suggestion) = suggestion else {
            // Execute instructions without suggestions
            return true;
        };

        use SuggestionInlineType::*;
        let status = match suggestion {
            DeleteInline(inline) => &inline.suggestion_status,
            InsertInline(inline) => &inline.suggestion_status,
            ModifyInline(inline) => &inline.suggestion_status,
            ReplaceInline(inline) => &inline.suggestion_status,
        };

        self.should_execute_instruction(status)
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

    /// Replace a property of a node
    pub fn replace_property(&self, node_id: &NodeId, property: Property, value: Value) {
        self.send_patch(NodePatch {
            node_id: node_id.clone(),
            ops: vec![Operation::replace_property(property, value)],
        })
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

        // If the block is of a type that is collected in the execution context then do that.
        match block {
            CodeChunk(node) => self.context.push_code_chunk(node),
            InstructionBlock(node) => self.context.push_instruction_block(node),
            MathBlock(node) => self.context.push_math_block(node),
            Paragraph(node) => self.context.push_paragraph(node),
            _ => {}
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
            MathBlock(node) => self.visit_executable(node).await,
            // TODO: StyledBlock(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        use Inline::*;

        // If the inline is of a type that is collected in the execution context then do that.
        match inline {
            CodeExpression(node) => self.context.push_code_expression(node),
            InstructionInline(node) => self.context.push_instruction_inline(node),
            MathInline(node) => self.context.push_math_inline(node),
            Text(node) => self.context.push_text(node),
            _ => {}
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
            InstructionInline(node) => self.visit_executable(node).await,
            MathInline(node) => self.visit_executable(node).await,
            // TODO: Parameter(node) => self.visit_executable(node).await,
            // TODO: StyledInline(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }
}
