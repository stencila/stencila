#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use codec_text_trait::TextCodec;
use common::{
    clap::{self, Args},
    eyre::Result,
    serde::{Deserialize, Serialize},
    tokio::sync::{mpsc::UnboundedSender, RwLock, RwLockWriteGuard},
    tracing,
};
use context::Context;
use kernels::Kernels;
use schema::{
    AutomaticExecution, Block, CompilationDigest, Inline, InstructionBlock, InstructionInline,
    Node, NodeId, NodeProperty, Patch, PatchOp, PatchPath, VisitorAsync, WalkControl, WalkNode,
};

type NodeIds = Vec<NodeId>;

mod prelude;

mod article;
mod call_block;
mod code_chunk;
mod code_expression;
mod figure;
mod for_block;
mod if_block;
mod include_block;
mod instruction_block;
mod instruction_inline;
mod math_block;
mod math_inline;
mod styled_block;
mod styled_inline;
mod table;

/// Walk over a root node and compile it and child nodes
pub async fn compile(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: UnboundedSender<Patch>,
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
    patch_sender: UnboundedSender<Patch>,
    node_ids: Option<NodeIds>,
    options: Option<ExecuteOptions>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender, node_ids, options);
    executor.pending(&mut root).await?;
    executor.execute(&mut root).await
}

/// Walk over a root node and interrupt it and child nodes
pub async fn interrupt(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: UnboundedSender<Patch>,
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

    /// Set the execution status of the node to pending
    async fn pending(&mut self, executor: &mut Executor) -> WalkControl {
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
    patch_sender: UnboundedSender<Patch>,

    /// The nodes that should be executed
    ///
    /// If `None` then the entire node (usually an `Article`) will be executed.
    node_ids: Option<NodeIds>,

    /// The phase of execution
    phase: Phase,

    /// The document context
    context: Context,

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
    /// Currently only supported by assistants where it is useful for debugging the
    /// rendering of system prompts without making a potentially slow API request.
    #[arg(long)]
    pub dry_run: bool,
}

/// A phase of an [`Executor`]
///
/// These phases determine which method of each [`Executable`] is called as
/// the executor walks over the root node.
enum Phase {
    Compile,
    Pending,
    Execute,
    Interrupt,
}

impl Executor {
    /// Create a new executor
    fn new(
        home: PathBuf,
        kernels: Arc<RwLock<Kernels>>,
        patch_sender: UnboundedSender<Patch>,
        node_ids: Option<NodeIds>,
        options: Option<ExecuteOptions>,
    ) -> Self {
        Self {
            home,
            kernels,
            patch_sender,
            node_ids,
            phase: Phase::Pending,
            context: Context::default(),
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

    /// Run [`Phase::Pending`]
    async fn pending(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Pending;
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

    /// Should the executor execute a code-based node (a node derived from `CodeExecutable`)
    pub fn should_execute_code(
        &self,
        node_id: &NodeId,
        auto_exec: &Option<AutomaticExecution>,
        compilation_digest: &Option<CompilationDigest>,
        execution_digest: &Option<CompilationDigest>,
    ) -> bool {
        if self.options.force_all {
            return true;
        }

        if let Some(node_ids) = &self.node_ids {
            return node_ids.contains(node_id);
        }

        if self.options.skip_code {
            return false;
        }

        // If the node has never been executed (both digests are none),
        // or if the digest has changed since last executed, then execute
        // the node
        (compilation_digest.is_none() && execution_digest.is_none())
            || compilation_digest != execution_digest
    }

    /// Should the executor execute an `InstructionBlock`ss
    #[allow(unreachable_code, unused_variables)]
    pub fn should_execute_instruction_block(
        &self,
        node_id: &NodeId,
        instruction: &InstructionBlock,
    ) -> bool {
        // TODO: reinstate the logic of this function
        return true;

        if self.options.force_all {
            return true;
        }

        if let Some(node_ids) = &self.node_ids {
            return node_ids.contains(node_id);
        }

        // Respect `skip_instructions`
        if self.options.skip_instructions {
            return false;
        }

        instruction
            .suggestions
            .as_ref()
            .map_or(true, |suggestions| suggestions.is_empty())
    }

    /// Should the executor execute an `InstructionInline`
    pub fn should_execute_instruction_inline(
        &self,
        node_id: &NodeId,
        instruction: &InstructionInline,
    ) -> bool {
        if self.options.force_all {
            return true;
        }

        if let Some(node_ids) = &self.node_ids {
            return node_ids.contains(node_id);
        }

        // Respect `skip_instructions`
        if self.options.skip_instructions {
            return false;
        }

        instruction
            .suggestions
            .as_ref()
            .map_or(true, |suggestions| suggestions.is_empty())
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

        if let Err(error) = self.patch_sender.send(patch) {
            tracing::error!("When sending execution node patch: {error}")
        }
    }

    /// Visit an executable node and call the appropriate method for the phase
    async fn visit_executable<E: Executable>(&mut self, node: &mut E) -> WalkControl {
        match self.phase {
            Phase::Compile => node.compile(self).await,
            Phase::Pending => node.pending(self).await,
            Phase::Execute => node.execute(self).await,
            Phase::Interrupt => node.interrupt(self).await,
        }
    }
}

impl VisitorAsync for Executor {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        // Collect the node into the context if appropriate
        if let Node::Article(article) = node {
            if let Some(title) = &article.title {
                self.context.set_title(&title.to_text().0);
            }
            if let Some(genre) = &article.genre {
                self.context.set_genre(&genre.to_text().0);
            }
            if let Some(keywords) = &article.keywords {
                self.context.set_keywords(keywords);
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
            Heading(node) => self.context.push_heading(node),
            Paragraph(node) => self.context.push_paragraph(node),
            _ => {}
        }

        let control = match block {
            // TODO: CallBlock(node) => self.visit_executable(node).await,
            CodeChunk(node) => self.visit_executable(node).await,
            Figure(node) => self.visit_executable(node).await,
            ForBlock(node) => self.visit_executable(node).await,
            IfBlock(node) => self.visit_executable(node).await,
            IncludeBlock(node) => self.visit_executable(node).await,
            InstructionBlock(node) => self.visit_executable(node).await,
            MathBlock(node) => self.visit_executable(node).await,
            StyledBlock(node) => self.visit_executable(node).await,
            Table(node) => self.visit_executable(node).await,
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

        let control = match inline {
            CodeExpression(node) => self.visit_executable(node).await,
            InstructionInline(node) => self.visit_executable(node).await,
            MathInline(node) => self.visit_executable(node).await,
            // TODO: Parameter(node) => self.visit_executable(node).await,
            StyledInline(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        };

        Ok(control)
    }
}
