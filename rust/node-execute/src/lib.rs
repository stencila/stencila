#![recursion_limit = "256"]

use std::{path::PathBuf, sync::Arc};

use common::{
    clap::{self, Args},
    eyre::Result,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    tokio::sync::{mpsc::UnboundedSender, RwLock, RwLockWriteGuard},
    tracing,
};
use kernels::Kernels;
use prompts::prompt::{DocumentContext, InstructionContext};
use schema::{
    AuthorRole, AuthorRoleName, Block, CompilationDigest, ExecutionBounds, ExecutionMode,
    ExecutionStatus, Inline, Link, List, ListItem, ListOrder, Node, NodeId, NodeProperty, NodeType,
    Paragraph, Patch, PatchOp, PatchPath, Timestamp, VisitorAsync, WalkControl, WalkNode,
};

type NodeIds = Vec<NodeId>;

mod prelude;

mod article;
mod call_block;
mod chat;
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
mod prompt_block;
mod raw_block;
mod section;
mod styled_block;
mod styled_inline;
mod suggestion_block;
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
#[derive(Clone)]
pub struct Executor {
    /// The stack of directories being executed, included or called
    ///
    /// Used to resolve relative file paths in `IncludeBlock` and `CallBlock` nodes.
    /// Needs to be a stack for nested includes and calls (i.e. those inside documents
    /// that have themselves been included or called).
    directory_stack: Vec<PathBuf>,

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

    /// The execution status to apply to nodes
    ///
    /// Currently, only used during [`Phase::Prepare`] and defaults
    /// to pending.
    execution_status: ExecutionStatus,

    /// The bounds on execution
    execution_bounds: ExecutionBounds,

    /// The document context for prompts
    document_context: DocumentContext,

    /// The instruction context for prompts
    instruction_context: Option<InstructionContext>,

    /// Information on the headings in the document
    headings: Vec<HeadingInfo>,

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

/// Records information about a heading in order to created
/// a nested list of headings for a document.
#[derive(Debug, Clone)]
pub struct HeadingInfo {
    /// The level of the heading
    level: i64,

    /// The node id of the heading (used to create a link to it)
    node_id: NodeId,

    /// The content of the heading
    content: Vec<Inline>,

    /// The headings nested under the heading
    children: Vec<HeadingInfo>,
}

impl HeadingInfo {
    /// Collapse headings deeper that the current level into their parents
    fn collapse(level: i64, headings: &mut Vec<HeadingInfo>) {
        if let Some(previous) = headings.last() {
            if level < previous.level {
                let mut children: Vec<HeadingInfo> = Vec::new();
                while let Some(mut previous) = headings.pop() {
                    if let Some(child) = children.last() {
                        if previous.level < child.level {
                            previous.children.append(&mut children);
                        }
                    }
                    children.insert(0, previous);

                    if let Some(last) = headings.last() {
                        if level >= last.level {
                            break;
                        }
                    }
                }

                if let Some(previous) = headings.last_mut() {
                    previous.children = children;
                }
            }
        }
    }

    /// Create a [`ListItem`] from a [`HeadingInfo`]
    fn into_list_item(self) -> ListItem {
        let mut content = vec![Block::Paragraph(Paragraph::new(vec![Inline::Link(
            Link::new(self.content, ["#", &self.node_id.to_string()].concat()),
        )]))];

        if !self.children.is_empty() {
            content.push(Block::List(Self::into_list(self.children)));
        }

        ListItem::new(content)
    }

    /// Create a [`List`] from a vector of [`HeadingInfo`]
    fn into_list(headings: Vec<HeadingInfo>) -> List {
        List::new(
            headings
                .into_iter()
                .map(|info| info.into_list_item())
                .collect_vec(),
            ListOrder::Ascending,
        )
    }
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

    /// Retain existing suggestions for instructions
    ///
    /// By default, when you execute an instruction, the existing suggestions for the instruction
    /// are deleted. Use this flag to retain existing suggestions, for example, so that you can
    /// use a previous one if a revision is worse.
    #[arg(long)]
    pub retain_suggestions: bool,

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
#[derive(Clone)]
enum Phase {
    Compile,
    Prepare,
    Execute,
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
            directory_stack: vec![home],
            kernels,
            patch_sender,
            node_ids,
            phase: Phase::Prepare,
            execution_status: ExecutionStatus::Pending,
            execution_bounds: ExecutionBounds::Main,
            document_context: DocumentContext::default(),
            instruction_context: None,
            headings: Vec::new(),
            table_count: 0,
            figure_count: 0,
            equation_count: 0,
            is_last: false,
            options: options.unwrap_or_default(),
        }
    }

    /// Create a fork of the executor that has `node_ids: None`
    ///
    /// This allows the newly forked executor to execute nodes that are not
    /// listed in the `node_ids` of the parent executor, specifically within
    /// newly created suggestions.
    fn fork_for_all(&self) -> Self {
        Self {
            node_ids: None,
            ..self.clone()
        }
    }

    /// Create a fork of the executor for [`Phase::Compile`]
    ///
    /// This allows the executor to compile nodes within parts of the document,
    /// specifically within rejected or proposed suggestions, without changing
    /// the main executor's:
    ///
    /// - headings list
    /// - table, figure and equation counts
    /// - document context
    fn fork_for_compile(&self) -> Self {
        Self {
            phase: Phase::Compile,
            ..self.clone()
        }
    }

    /// Create a fork of the executor for [`Phase::Prepare`]
    ///
    /// This allows the executor to prepare nodes within parts of the document,
    /// specifically within rejected or proposed suggestions, and mark them
    /// as [`ExecutionStatus::Rejected`] rather than [`ExecutionStatus::Pending`].
    fn fork_for_prepare(&self, execution_status: ExecutionStatus) -> Self {
        Self {
            phase: Phase::Prepare,
            execution_status,
            ..self.clone()
        }
    }

    /// Create a fork of the executor for [`Phase::Execute`]
    ///
    /// Create a clone of the executor, except for having a fork of its [`Kernels`].
    /// This allows the executor to execute nodes within a document,
    /// without effecting the main kernel processes. Specifically, this
    /// is used to execute suggestions.
    async fn fork_for_execute(&self) -> Result<Self> {
        let kernels = self.kernels().await.fork().await?;
        let kernels = Arc::new(RwLock::new(kernels));

        Ok(Self {
            phase: Phase::Execute,
            execution_bounds: ExecutionBounds::Fork,
            kernels,
            ..self.clone()
        })
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

    /// Run the compile, prepare and execute phases on am executable node
    ///
    /// Used when recursively executing new content that has not necessarily been compiled
    /// or prepared yet (e.g. a suggestion or for loop iteration).
    /// If this is not done, the execution status, digests etc of the node may not be correct
    /// when it is executed.
    async fn compile_prepare_execute<W: WalkNode>(&mut self, node: &mut W) -> Result<()> {
        for phase in [Phase::Compile, Phase::Prepare, Phase::Execute] {
            self.phase = phase;
            node.walk_async(self).await?;
        }

        Ok(())
    }

    /// Obtain a write lock to the kernels
    ///
    /// Used by [`Executable`] nodes to execute and evaluate code and manage variables.
    pub async fn kernels(&self) -> RwLockWriteGuard<Kernels> {
        self.kernels.write().await
    }

    /// Get the execution status for a node based on state of node
    /// and options of the executor
    pub fn node_execution_status(
        &self,
        node_type: NodeType,
        node_id: &NodeId,
        execution_mode: &Option<ExecutionMode>,
        compilation_digest: &Option<CompilationDigest>,
        execution_digest: &Option<CompilationDigest>,
    ) -> Option<ExecutionStatus> {
        if self.options.force_all {
            return Some(ExecutionStatus::Pending);
        }

        if matches!(execution_mode, Some(ExecutionMode::Lock)) {
            return Some(ExecutionStatus::Locked);
        }

        if let Some(node_ids) = &self.node_ids {
            // If the executor has any node ids then the current
            // node id must be amongst them
            return if node_ids.contains(node_id) {
                Some(ExecutionStatus::Pending)
            } else {
                None
            };
        }

        if matches!(
            node_type,
            NodeType::InstructionBlock | NodeType::InstructionInline
        ) {
            if self.options.skip_instructions {
                return Some(ExecutionStatus::Skipped);
            }
        } else if self.options.skip_code {
            return Some(ExecutionStatus::Skipped);
        }

        // Check execution mode of node after `skip_` options
        if matches!(execution_mode, Some(ExecutionMode::Always)) {
            return Some(ExecutionStatus::Pending);
        }

        if (compilation_digest.is_none() && execution_digest.is_none())
            || compilation_digest != execution_digest
        {
            // If the node has never been executed (both digests are none),
            // or if the digest has changed since last executed, then return
            // `self.execution_status` (usually Pending)
            Some(self.execution_status.clone())
        } else {
            // No change to execution status required
            None
        }
    }

    /// Get the [`AuthorRole`] for the kernel instance if it is different from the current
    pub async fn node_execution_instance_author(
        &self,
        instance: &String,
        execution_instance: &Option<String>,
    ) -> Option<AuthorRole> {
        if execution_instance.as_ref() != Some(instance) {
            if let Some(instance) = self.kernels().await.get_instance(instance).await {
                if let Ok(app) = instance.lock().await.info().await {
                    let mut role = AuthorRole::software(app, AuthorRoleName::Executor);
                    role.last_modified = Some(Timestamp::now());
                    return Some(role);
                }
            }
        }

        None
    }

    /// Patch several properties of a node
    pub fn patch<P>(&self, node_id: &NodeId, pairs: P)
    where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        self.send_patch_ops(node_id, None, pairs)
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
        self.send_patch_ops(node_id, Some(authors), pairs)
    }

    /// Send patch operations reflecting a change in the state of a node during execution
    fn send_patch_ops<P>(
        &self,
        node_id: &NodeId,
        authors: Option<Vec<schema::AuthorRole>>,
        pairs: P,
    ) where
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

    /// Send a patch reflecting a change in the state of a node during execution
    fn send_patch(&self, patch: Patch) {
        let Some(sender) = &self.patch_sender else {
            return;
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
            Phase::Execute => node.execute(self).await,
            Phase::Interrupt => node.interrupt(self).await,
        }
    }
}

impl VisitorAsync for Executor {
    async fn visit_node(&mut self, node: &mut Node) -> Result<WalkControl> {
        use Node::*;
        Ok(match node {
            Article(node) => self.visit_executable(node).await,
            Chat(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_suggestion_block(
        &mut self,
        block: &mut schema::SuggestionBlock,
    ) -> Result<WalkControl> {
        Ok(self.visit_executable(block).await)
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        use Block::*;
        Ok(match block {
            CallBlock(node) => self.visit_executable(node).await,
            Chat(node) => self.visit_executable(node).await,
            CodeChunk(node) => self.visit_executable(node).await,
            Figure(node) => self.visit_executable(node).await,
            ForBlock(node) => self.visit_executable(node).await,
            Heading(node) => self.visit_executable(node).await,
            IfBlock(node) => self.visit_executable(node).await,
            IncludeBlock(node) => self.visit_executable(node).await,
            InstructionBlock(node) => self.visit_executable(node).await,
            MathBlock(node) => self.visit_executable(node).await,
            Paragraph(node) => self.visit_executable(node).await,
            PromptBlock(node) => self.visit_executable(node).await,
            RawBlock(node) => self.visit_executable(node).await,
            Section(node) => self.visit_executable(node).await,
            StyledBlock(node) => self.visit_executable(node).await,
            SuggestionBlock(node) => self.visit_executable(node).await,
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
