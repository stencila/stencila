#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    clap::{self, Args},
    eyre::Result,
    serde::{Deserialize, Serialize},
    tokio::sync::{mpsc::UnboundedSender, RwLock, RwLockWriteGuard},
    tracing,
};
use kernels::Kernels;
use prompts::prompt::DocumentContext;
use schema::{
    Block, CompilationDigest, ExecutionMode, Inline, Node, NodeId, NodeProperty, Patch, PatchOp,
    PatchPath, VisitorAsync, WalkControl, WalkNode,
};

type NodeIds = Vec<NodeId>;

mod prelude;

mod article;
mod call_block;
mod code_chunk;
mod code_expression;
mod figure;
mod for_block;
mod heading;
mod if_block;
mod include_block;
mod instruction_block;
mod instruction_inline;
mod math_block;
mod math_inline;
mod paragraph;
mod parameter;
mod prompt;
mod styled_block;
mod styled_inline;
mod table;

/// Walk over a root node and compile it and child nodes
pub async fn compile(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: Option<UnboundedSender<Patch>>,
    node_ids: Option<NodeIds>,
    options: Option<ExecuteOptions>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender, node_ids, options);
    executor.compile(&mut root).await
}

/// Walk over a root node and execute it and child nodes
pub async fn execute(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: Option<UnboundedSender<Patch>>,
    node_ids: Option<NodeIds>,
    options: Option<ExecuteOptions>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender, node_ids, options);
    executor.prepare(&mut root).await?;
    executor.execute(&mut root).await
}

/// Walk over a root node and interrupt it and child nodes
pub async fn interrupt(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: Option<UnboundedSender<Patch>>,
    node_ids: Option<NodeIds>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender, node_ids, None);
    executor.interrupt(&mut root).await
}

/// A trait for an executable node
///
/// Default action does nothing to the node but continues walking
/// over its descendants. Implementation will normally at least
/// override `compile` and/or `execute`. If `execute` is implemented,
/// so to should `pending`
#[allow(unused)]
trait Executable {
    /// Compile the node
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }

    /// Prepare the node, and the executor, for execution
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }

    /// Execute the node
    ///
    /// Note that this method is intentionally infallible because we want
    /// executable nodes to handle any errors associated with their execution
    /// and record them in `execution_messages` so that they are visible
    /// to the user.
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }

    /// Interrupt execution of the node
    async fn interrupt(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }
}

/// A visitor that walks over a tree of nodes and executes them
pub struct Executor {
    /// The home directory of the document being executed
    ///
    /// Used to resolve relative file paths in `IncludeBlock` and `CallBlock` nodes.
    home: PathBuf,

    /// The kernels that will be used for execution
    kernels: Arc<RwLock<Kernels>>,

    /// A sender for a [`NodePatch`] channel
    ///
    /// Patches reflecting the state of nodes during execution should be sent
    /// on this channel.
    patch_sender: Option<UnboundedSender<Patch>>,

    /// The nodes that should be executed
    ///
    /// If `None` then the entire node (usually an `Article`) will be executed.
    node_ids: Option<NodeIds>,

    /// The phase of execution
    phase: Phase,

    /// The variables context for prompts
    document_context: DocumentContext,

    /// The count of `Table`s and `CodeChunk`s with a table `labelType`
    table_count: u32,

    /// The count of `Figure`s and `CodeChunk`s with a figure `labelType`
    figure_count: u32,

    /// The count of `MathBlock`s
    equation_count: u32,

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
    /// Re-execute all node types regardless of current state
    #[arg(long)]
    pub force_all: bool,

    /// Skip executing code
    ///
    /// By default, code-based nodes in the document (e.g. `CodeChunk`, `CodeExpression`, `ForBlock`)
    /// nodes will be executed if they are stale. Use this flag to skip executing all code-based nodes.
    #[arg(long)]
    pub skip_code: bool,

    /// Skip executing instructions
    ///
    /// By default, instructions with no suggestions, or with suggestions that have
    /// been rejected will be executed. Use this flag to skip executing all instructions.
    #[arg(long, alias = "skip-inst")]
    pub skip_instructions: bool,

    /// Re-execute instructions with suggestions that have not yet been reviewed
    ///
    /// By default, an instruction that has a suggestion that has not yet be reviewed
    /// (i.e. has a suggestion status that is empty) will not be re-executed. Use this
    /// flag to force these instructions to be re-executed.
    #[arg(long)]
    pub force_unreviewed: bool,

    /// Re-execute instructions with suggestions that have been accepted.
    ///
    /// By default, an instruction that has a suggestion that has been accepted, will
    /// not be re-executed. Use this flag to force these instructions to be re-executed.
    #[arg(long)]
    pub force_accepted: bool,

    /// Skip re-executing instructions with suggestions that have been rejected
    ///
    /// By default, instructions that have a suggestion that has been rejected, will be
    /// re-executed. Use this flag to skip re-execution of these instructions.
    #[arg(long)]
    pub skip_rejected: bool,

    /// Prepare, but do not actually perform, execution tasks
    ///
    /// Currently only supported by instructions where it is useful for debugging the
    /// rendering of prompts without making a potentially slow generative model API request.
    #[arg(long)]
    pub dry_run: bool,
}

/// A phase of an [`Executor`]
///
/// These phases determine which method of each [`Executable`] is called as
/// the executor walks over the root node.
enum Phase {
    Compile,
    Prepare,
    Execute,
    ExecuteWithoutPatches,
    Interrupt,
}

impl Executor {
    /// Create a new executor
    fn new(
        home: PathBuf,
        kernels: Arc<RwLock<Kernels>>,
        patch_sender: Option<UnboundedSender<Patch>>,
        node_ids: Option<NodeIds>,
        options: Option<ExecuteOptions>,
    ) -> Self {
        Self {
            home,
            kernels,
            patch_sender,
            node_ids,
            phase: Phase::Prepare,
            document_context: DocumentContext::default(),
            table_count: 0,
            figure_count: 0,
            equation_count: 0,
            is_last: false,
            options: options.unwrap_or_default(),
        }
    }

    /// Run [`Phase::Compile`]
    async fn compile(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Compile;
        self.table_count = 0;
        self.figure_count = 0;
        self.equation_count = 0;
        root.walk_async(self).await
    }

    /// Run [`Phase::Prepare`]
    async fn prepare(&mut self, root: &mut Node) -> Result<()> {
        // Create a new context before walking the tree to avoid
        // having hangover information from the last time the prepare
        // phase was run.
        self.document_context = DocumentContext::default();

        self.phase = Phase::Prepare;
        root.walk_async(self).await
    }

    /// Run [`Phase::Execute`]
    async fn execute(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Execute;
        root.walk_async(self).await
    }

    /// Run [`Phase::Interrupt`]
    async fn interrupt(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Interrupt;
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

    /// Should the executor execute a node
    pub fn should_execute(
        &self,
        node_id: &NodeId,
        execution_mode: &Option<ExecutionMode>,
        compilation_digest: &Option<CompilationDigest>,
        execution_digest: &Option<CompilationDigest>,
    ) -> bool {
        if self.options.force_all || matches!(execution_mode, Some(ExecutionMode::Always)) {
            return true;
        }

        if matches!(execution_mode, Some(ExecutionMode::Locked)) {
            return false;
        }

        if let Some(node_ids) = &self.node_ids {
            return node_ids.contains(node_id);
        }

        if self.options.skip_code {
            return false;
        }

        // If the node has never been executed (both digests are none),
        // or if the digest has changed since last executed, then execute the node
        (compilation_digest.is_none() && execution_digest.is_none())
            || compilation_digest != execution_digest
    }

    /// Should the executor execute an `Instruction`
    #[allow(unreachable_code, unused_variables)]
    pub fn should_execute_instruction(
        &self,
        node_id: &NodeId,
        execution_mode: &Option<ExecutionMode>,
        compilation_digest: &Option<CompilationDigest>,
        execution_digest: &Option<CompilationDigest>,
    ) -> bool {
        if self.options.force_all || matches!(execution_mode, Some(ExecutionMode::Always)) {
            return true;
        }

        if matches!(execution_mode, Some(ExecutionMode::Locked)) {
            return false;
        }

        if let Some(node_ids) = &self.node_ids {
            return node_ids.contains(node_id);
        }

        if self.options.skip_instructions {
            return false;
        }

        // If the node has never been executed (both digests are none),
        // or if the digest has changed since last executed, then execute the node
        (compilation_digest.is_none() && execution_digest.is_none())
            || compilation_digest != execution_digest
    }

    /// Patch several properties of a node
    pub fn patch<P>(&self, node_id: &NodeId, pairs: P)
    where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        self.send_patch(node_id, None, pairs)
    }

    /// Patch several properties of a node and attribute authorship
    pub fn patch_with_authors<P>(
        &self,
        node_id: &NodeId,
        authors: Vec<schema::AuthorRole>,
        pairs: P,
    ) where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        self.send_patch(node_id, Some(authors), pairs)
    }

    /// Send a patch reflecting a change in the state of a node during execution
    fn send_patch<P>(&self, node_id: &NodeId, authors: Option<Vec<schema::AuthorRole>>, pairs: P)
    where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        let Some(sender) = &self.patch_sender else {
            return;
        };

        let ops = pairs
            .into_iter()
            .map(|(property, op)| (PatchPath::from(property), op))
            .collect();

        let patch = Patch {
            node_id: Some(node_id.clone()),
            format: None,
            authors,
            ops,
        };

        if let Err(error) = sender.send(patch) {
            tracing::error!("When sending execution node patch: {error}")
        }
    }

    /// Visit an executable node and call the appropriate method for the phase
    async fn visit_executable<E: Executable>(&mut self, node: &mut E) -> WalkControl {
        match self.phase {
            Phase::Compile => node.compile(self).await,
            Phase::Prepare => node.prepare(self).await,
            Phase::Execute | Phase::ExecuteWithoutPatches => node.execute(self).await,
            Phase::Interrupt => node.interrupt(self).await,
        }
    }
}

impl VisitorAsync for Executor {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        use Node::*;
        Ok(match node {
            Article(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        use Block::*;
        Ok(match block {
            CallBlock(node) => self.visit_executable(node).await,
            CodeChunk(node) => self.visit_executable(node).await,
            Figure(node) => self.visit_executable(node).await,
            ForBlock(node) => self.visit_executable(node).await,
            Heading(node) => self.visit_executable(node).await,
            IfBlock(node) => self.visit_executable(node).await,
            IncludeBlock(node) => self.visit_executable(node).await,
            InstructionBlock(node) => self.visit_executable(node).await,
            MathBlock(node) => self.visit_executable(node).await,
            Paragraph(node) => self.visit_executable(node).await,
            StyledBlock(node) => self.visit_executable(node).await,
            Table(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        use Inline::*;
        Ok(match inline {
            CodeExpression(node) => self.visit_executable(node).await,
            InstructionInline(node) => self.visit_executable(node).await,
            MathInline(node) => self.visit_executable(node).await,
            Parameter(node) => self.visit_executable(node).await,
            StyledInline(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }
}
