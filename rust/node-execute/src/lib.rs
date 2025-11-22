#![recursion_limit = "256"]

use std::{collections::HashMap, fmt::Debug, path::PathBuf, str::FromStr, sync::Arc};

use clap::Args;
use eyre::{Result, bail, eyre};
use futures::future::join_all;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockWriteGuard, mpsc, oneshot};

use stencila_codecs::{DecodeOptions, Format};
use stencila_kernels::Kernels;
use stencila_linters::LintingOptions;
use stencila_schema::{
    AuthorRole, AuthorRoleName, Block, CitationGroup, CompilationMessage, Config, ExecutionBounds,
    ExecutionMode, ExecutionRequired, ExecutionStatus, IfBlockClause, Inline, LabelType, Link,
    List, ListItem, ListOrder, Node, NodeId, NodePath, NodeProperty, NodeType, Paragraph, Patch,
    PatchNode, PatchOp, PatchValue, Reference, SuggestionBlock, Timestamp, VisitorAsync,
    WalkControl, WalkNode,
};

type NodeIds = Vec<NodeId>;

mod prelude;

mod appendix_break;
mod article;
mod call_block;
mod chat;
mod citation;
mod citation_group;
mod code_chunk;
mod code_expression;
mod code_utils;
mod datatable;
mod excerpt;
mod figure;
mod for_block;
mod heading;
mod if_block;
mod include_block;
mod instruction_block;
mod instruction_inline;
mod island;
mod link;
mod math_block;
mod math_inline;
mod model_utils;
mod parameter;
mod prompt;
mod prompt_block;
mod raw_block;
mod styled_block;
mod styled_inline;
mod suggestion_block;
mod supplement;
mod table;
mod text;

type PatchSender = mpsc::UnboundedSender<(Patch, Option<oneshot::Sender<()>>)>;

/// Walk over a root node and compile it and child nodes
pub async fn compile(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    config: Config,
    patch_sender: Option<PatchSender>,
    decode_options: Option<DecodeOptions>,
    compile_options: Option<CompileOptions>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender);
    executor.config = Some(config);
    executor.decode_options = decode_options;
    executor.compile_options = compile_options;
    executor.compile(&mut root).await?;
    executor.link(&mut root).await?;
    executor.finalize().await
}

/// Walk over a root node and execute it and child nodes
pub async fn execute(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: Option<PatchSender>,
    node_ids: Option<NodeIds>,
    execute_options: Option<ExecuteOptions>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender);
    executor.node_ids = node_ids;
    executor.execute_options = execute_options;
    executor.prepare(&mut root).await?;
    executor.execute(&mut root).await?;
    executor.finalize().await
}

/// Walk over a root node and interrupt it and child nodes
pub async fn interrupt(
    home: PathBuf,
    root: Arc<RwLock<Node>>,
    kernels: Arc<RwLock<Kernels>>,
    patch_sender: Option<PatchSender>,
    node_ids: Option<NodeIds>,
) -> Result<()> {
    let mut root = root.read().await.clone();
    let mut executor = Executor::new(home, kernels, patch_sender);
    executor.node_ids = node_ids;
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

    /// Link the node
    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }

    /// Prepare the node, and the executor, for execution
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        WalkControl::Continue
    }

    /// Execute the node
    ///
    /// Note that this method is required to be infallible because we want
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

    /// The decoding options used when compiling `IncludeBlock`s
    decode_options: Option<DecodeOptions>,

    /// The options used when compiling nodes
    compile_options: Option<CompileOptions>,

    /// The kernels that will be used for execution
    kernels: Arc<RwLock<Kernels>>,

    /// A sender for a [`NodePatch`] channel
    ///
    /// Patches reflecting the state of nodes during execution should be sent
    /// on this channel.
    patch_sender: Option<PatchSender>,

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

    /// The position (relative index of block of inline node) in the walk
    ///
    /// Used to set the `currentPosition` in any `docsql` and `docsdb` kernels
    walk_position: u64,

    /// A list of the ancestors node types in the current walk
    ///
    /// Used to skip headings within figures, tables and code chunk captions.
    walk_ancestors: Vec<NodeType>,

    /// Information on the headings in the document
    headings: Vec<HeadingInfo>,

    /// The count of level 1 `Heading` nodes after the first `AppendixBreak`
    appendix_count: Option<u32>,

    /// The count of `Table` nodes and `CodeChunk` nodes with a table `labelType`
    table_count: u32,

    /// The count of `Figure` nodes and `CodeChunk` nodes with a figure `labelType`
    figure_count: u32,

    /// The count of `MathBlock` nodes
    equation_count: u32,

    /// The count of `Supplement` nodes
    supplement_count: u32,

    /// Labels that may be the target of internal `Link`s
    labels: HashMap<String, (LabelType, String)>,

    /// References that may be the `target` of citations
    ///
    /// All references that the document "knows about", including existing
    /// references, excerpts, and references cited with those excerpts should be
    /// be included in this list
    bibliography: HashMap<String, Reference>,

    /// Citations and citation groups collected while walking over the the root node
    ///
    /// Used to render content for both the citations and a reference list.
    citations: IndexMap<NodeId, (CitationGroup, Option<Vec<Inline>>)>,

    /// The last programming language used
    programming_language: Option<String>,

    /// Information about nodes, their code and language, and whether they have changed,
    /// used for linting
    linting_context: Vec<(Option<NodeId>, String, Option<String>, bool)>,

    /// Whether to force execution of nodes
    ///
    /// Used for `IfBlock` (and possibly others in the future) to ensure re-execution
    /// of the content of clauses that were previously executed but which are now stale.
    /// This may not be necessary when we have fully functioning dependency analysis
    /// so that we are able to determine what is stale and needs re-execution.
    /// See https://github.com/stencila/stencila/issues/2562.
    ///
    /// Equivalent to setting `execute_options.force_all`.
    force_all: bool,

    /// Whether the current node is the last in a set
    ///
    /// Used for `IfBlock` (and possibly others in the future) to control behavior of
    /// execution of child nodes.
    is_last: bool,

    /// Configuration options
    config: Option<Config>,

    /// Execution options
    execute_options: Option<ExecuteOptions>,
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
        if let Some(previous) = headings.last()
            && level < previous.level
        {
            let mut children: Vec<HeadingInfo> = Vec::new();
            while let Some(mut previous) = headings.pop() {
                if let Some(child) = children.last()
                    && previous.level < child.level
                {
                    previous.children.append(&mut children);
                }
                children.insert(0, previous);

                if let Some(last) = headings.last()
                    && level >= last.level
                {
                    break;
                }
            }

            if let Some(previous) = headings.last_mut() {
                previous.children = children;
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
#[serde(default)]
pub struct CompileOptions {
    /// Should lint the document
    pub should_lint: bool,

    /// If should lint, should also apply formatting corrections
    pub should_format: bool,

    /// If should lint, should also fix warnings and errors where possible
    pub should_fix: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Args)]
#[serde(default)]
pub struct ExecuteOptions {
    /// Ignore any errors while executing document
    #[arg(long, help_heading = "Execution Options")]
    pub ignore_errors: bool,

    /// Re-execute all node types regardless of current state
    #[arg(long, help_heading = "Execution Options")]
    pub force_all: bool,

    /// Skip executing code
    ///
    /// By default, code-based nodes in the document (e.g. `CodeChunk`, `CodeExpression`, `ForBlock`)
    /// nodes will be executed if they are stale. Use this flag to skip executing all code-based nodes.
    #[arg(long, help_heading = "Execution Options")]
    pub skip_code: bool,

    /// Skip executing instructions
    ///
    /// By default, instructions with no suggestions, or with suggestions that have
    /// been rejected will be executed. Use this flag to skip executing all instructions.
    #[arg(long, alias = "skip-inst", help_heading = "Execution Options")]
    pub skip_instructions: bool,

    /// Retain existing suggestions for instructions
    ///
    /// By default, when you execute an instruction, the existing suggestions for the instruction
    /// are deleted. Use this flag to retain existing suggestions, for example, so that you can
    /// use a previous one if a revision is worse.
    #[arg(long, help_heading = "Execution Options")]
    pub retain_suggestions: bool,

    /// Re-execute instructions with suggestions that have not yet been reviewed
    ///
    /// By default, an instruction that has a suggestion that has not yet be reviewed
    /// (i.e. has a suggestion status that is empty) will not be re-executed. Use this
    /// flag to force these instructions to be re-executed.
    #[arg(long, help_heading = "Execution Options")]
    pub force_unreviewed: bool,

    /// Re-execute instructions with suggestions that have been accepted.
    ///
    /// By default, an instruction that has a suggestion that has been accepted, will
    /// not be re-executed. Use this flag to force these instructions to be re-executed.
    #[arg(long, help_heading = "Execution Options")]
    pub force_accepted: bool,

    /// Skip re-executing instructions with suggestions that have been rejected
    ///
    /// By default, instructions that have a suggestion that has been rejected, will be
    /// re-executed. Use this flag to skip re-execution of these instructions.
    #[arg(long, help_heading = "Execution Options")]
    pub skip_rejected: bool,

    /// Prepare, but do not actually perform, execution tasks
    ///
    /// Currently only supported by instructions where it is useful for debugging the
    /// rendering of prompts without making a potentially slow generative model API request.
    #[arg(long, help_heading = "Execution Options")]
    pub dry_run: bool,
}

/// A phase of an [`Executor`]
///
/// These phases determine which method of each [`Executable`] is called as
/// the executor walks over the root node.
#[derive(Clone)]
enum Phase {
    Compile,
    Link,
    Prepare,
    Execute,
    Interrupt,
}

impl Executor {
    /// Create a new executor
    fn new(
        home: PathBuf,
        kernels: Arc<RwLock<Kernels>>,
        patch_sender: Option<PatchSender>,
    ) -> Self {
        Self {
            directory_stack: vec![home],
            decode_options: None,
            compile_options: None,
            kernels,
            patch_sender,
            node_ids: None,
            phase: Phase::Prepare,
            execution_status: ExecutionStatus::Pending,
            walk_position: 0,
            walk_ancestors: Default::default(),
            headings: Vec::new(),
            appendix_count: None,
            table_count: 0,
            figure_count: 0,
            equation_count: 0,
            supplement_count: 0,
            labels: Default::default(),
            bibliography: Default::default(),
            citations: Default::default(),
            programming_language: None,
            linting_context: Vec::new(),
            force_all: false,
            is_last: false,
            config: None,
            execute_options: None,
        }
    }

    /// Create a fork of the executor for supplementary works
    ///
    /// Resets counters etc so that the supplemental work has separate series for
    /// tables, figures etc.
    fn fork_for_supplement(&self) -> Self {
        Self {
            walk_position: 0,
            walk_ancestors: Default::default(),
            headings: Vec::new(),
            appendix_count: None,
            table_count: 0,
            figure_count: 0,
            equation_count: 0,
            supplement_count: 0,
            labels: Default::default(),
            bibliography: Default::default(),
            citations: Default::default(),
            ..self.clone()
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
        Ok(Self {
            phase: Phase::Execute,
            kernels: self.replicate_kernels(ExecutionBounds::Fork, None).await?,
            ..self.clone()
        })
    }

    /// Create a fork of the executor's kernels
    async fn replicate_kernels(
        &self,
        bounds: ExecutionBounds,
        lang: Option<&str>,
    ) -> Result<Arc<RwLock<Kernels>>> {
        let kernels = self.kernels().await.replicate(bounds, lang).await?;
        Ok(Arc::new(RwLock::new(kernels)))
    }

    /// Run [`Phase::Compile`]
    async fn compile<N: WalkNode + PatchNode + Debug>(&mut self, root: &mut N) -> Result<()> {
        self.phase = Phase::Compile;
        self.appendix_count = None;
        self.table_count = 0;
        self.figure_count = 0;
        self.equation_count = 0;
        self.supplement_count = 0;
        self.linting_context.clear();
        self.walk_position = 0;
        self.walk_ancestors.clear();
        root.walk_async(self).await?;

        if self
            .compile_options
            .as_ref()
            .map(|opts| opts.should_lint)
            .unwrap_or_default()
        {
            self.lint(root).await?;
        }

        Ok(())
    }

    /// Run [`Phase::Link`]
    async fn link<N: WalkNode + PatchNode + Debug>(&mut self, root: &mut N) -> Result<()> {
        self.phase = Phase::Link;
        self.walk_position = 0;
        self.walk_ancestors.clear();
        root.walk_async(self).await?;

        Ok(())
    }

    /// Run [`Phase::Compile`] and [`Phase::Link`] on a node
    async fn compile_link<N: WalkNode + PatchNode + Debug>(&mut self, node: &mut N) -> Result<()> {
        self.compile(node).await?;
        self.link(node).await?;

        Ok(())
    }

    /// Add a variable declaration to the linting context
    ///
    /// This is used when it is necessary to declare some variable as being part of
    /// the context but not part of a specific node's code.
    pub(crate) fn linting_variable(
        &mut self,
        name: &str,
        lang: &Option<String>,
        has_changed: bool,
    ) {
        let format = lang.as_deref().map(Format::from_name);

        use Format::*;
        let declaration = match format {
            Some(JavaScript) => format!("var {name} = null;\n"),
            // Add `Any` type hint so that linter does not complain
            Some(Python) => format!("{name}: Any = None\n"),
            Some(R) => format!("{name} <- NULL\n"),
            _ => format!("{name} = 0;\n"),
        };

        self.linting_context
            .push((None, declaration, lang.clone(), has_changed));
    }

    /// Add code to the linting context
    ///
    /// Note that for code chunks, this needs to be done regardless of whether the code chunk
    /// has changed or not because we need to collect all the code in the document for linting.
    pub(crate) fn linting_code(
        &mut self,
        node_id: &NodeId,
        code: &str,
        lang: &Option<String>,
        has_changed: bool,
    ) {
        self.linting_context.push((
            Some(node_id.clone()),
            code.to_string(),
            lang.clone(),
            has_changed,
        ));
    }

    /// Lint code that was collected while compiling
    async fn lint<P: PatchNode + Debug>(&mut self, node: &mut P) -> Result<()> {
        let CompileOptions {
            should_format,
            should_fix,
            ..
        } = self.compile_options.clone().unwrap_or_default();

        // Skip linting if formatting or fixing is not required and
        // none of the codes have changed
        if !should_format
            && !should_fix
            && !self
                .linting_context
                .iter()
                .any(|(.., has_changed)| *has_changed)
        {
            return Ok(());
        }

        const BEGIN: &str = "STENCILA-NODE-BEGIN";
        const END: &str = "STENCILA-NODE-END";

        // Get the single-line comment characters for a language
        fn format_comment(format: &Option<Format>) -> &'static str {
            match format {
                Some(Format::Python | Format::R) => "#",
                Some(Format::JavaScript) => "//",
                _ => "//",
            }
        }

        // Collate code for each language with comments delimiting each node.
        // Record which nodes are associated with each language so we only extract
        // code and messages for relevant nodes for each language
        let mut format_codes: HashMap<Option<Format>, String> = HashMap::new();
        let mut format_nodes: HashMap<Option<Format>, Vec<NodeId>> = HashMap::new();
        for (node_id, code, language, ..) in &self.linting_context {
            let format = language.as_deref().map(Format::from_name);

            if let Some(node_id) = node_id {
                let comment = format_comment(&format);
                let code =
                    format!("{comment} {BEGIN} {node_id}\n{code}\n{comment} {END} {node_id}\n");

                if let Some(existing) = format_codes.get_mut(&format) {
                    existing.push_str(&code);
                } else {
                    format_codes.insert(format.clone(), code);
                }

                format_nodes
                    .entry(format)
                    .or_default()
                    .push(node_id.clone());
            } else if let Some(existing) = format_codes.get_mut(&format) {
                existing.push_str(code);
            } else {
                format_codes.insert(format.clone(), code.clone());
            }
        }

        let dir = self
            .directory_stack
            .first()
            .ok_or_else(|| eyre!("Executor has no top directory"))?;

        // Run linting for each of the collected languages concurrently
        let futures = format_codes
            .clone()
            .into_iter()
            .map(|(format, code)| async move {
                let format = format?;

                match stencila_linters::lint(
                    &code,
                    Some(dir),
                    LintingOptions {
                        format: Some(format.clone()),
                        should_format,
                        should_fix,
                        ..Default::default()
                    },
                )
                .await
                {
                    Ok(output) => Some((format, output)),
                    Err(error) => {
                        tracing::debug!("While linting {format}: {error}");
                        None
                    }
                }
            });
        let mut outputs: HashMap<Format, _> =
            join_all(futures).await.into_iter().flatten().collect();

        let mut node_codes: HashMap<NodeId, String> = HashMap::new();
        let mut node_messages: HashMap<NodeId, Vec<CompilationMessage>> = HashMap::new();
        let mut node_authors: HashMap<NodeId, Vec<AuthorRole>> = HashMap::new();
        for (format, code) in format_codes {
            let Some(format) = format else {
                continue;
            };
            let Some(output) = outputs.remove(&format) else {
                continue;
            };

            let format = Some(format);
            let this_format_nodes = format_nodes.remove(&format).unwrap_or_default();

            if let Some(authors) = output.authors {
                // Get the formatter and linter authors
                // This could be refined to only add formatter role to nodes
                // that changed
                for node_id in &this_format_nodes {
                    node_authors.insert(node_id.clone(), authors.clone());
                }
            }

            // Get the line comment prefix for the format
            let comment_prefix = format_comment(&format);

            // If there is output code then it needs to be used to patch nodes
            // and also message line numbers need to be calculated based on it
            let (code, code_changed) = match output.content {
                Some(code) => (code, true),
                None => (code.clone(), false),
            };

            // If there is a code change then extract the new code for each node
            // and compare to see if it has changed
            if code_changed {
                let lines = code.lines().collect_vec();
                let mut line_index = 0;
                for (node_id, old_code, ..) in &self.linting_context {
                    // Continue if code is not part of the node
                    let Some(node_id) = node_id else {
                        continue;
                    };

                    // Continue if node does not have code for this language
                    if !this_format_nodes.contains(node_id) {
                        continue;
                    }

                    // Move over lines looking for code beginning and ending
                    let begin = format!("{comment_prefix} {BEGIN} {node_id}");
                    let end = format!("{comment_prefix} {END} {node_id}");
                    let mut new_code = String::new();
                    let mut begun = false;
                    while line_index < lines.len() {
                        let Some(line) = lines.get(line_index) else {
                            break;
                        };

                        if *line == begin {
                            begun = true;
                        } else if *line == end {
                            break;
                        } else if begun {
                            new_code.push_str(line);
                            new_code.push('\n');
                        }

                        line_index += 1;
                    }

                    // This should not happen because we only look for the nodes using this
                    // language. But if it does, then generate a debug message.
                    if !begun {
                        tracing::debug!(
                            "Could not find formatted and/or fixed code for node `{node_id}`"
                        );
                        continue;
                    }

                    // Ignore ending whitespace space
                    new_code.truncate(new_code.trim_end().len());
                    let old_code = old_code.trim_end();

                    if new_code != old_code {
                        node_codes.insert(node_id.clone(), new_code);
                    }
                }
            }

            // If there are any messages then map them back to the nodes by finding the
            // BEGIN line immediately before the location of the message
            if let Some(messages) = output.messages {
                let lines = code.lines().collect_vec();
                for mut message in messages {
                    // Get the line of the message as the starting line
                    let Some(start_line) = message
                        .code_location
                        .as_ref()
                        .and_then(|loc| loc.start_line)
                    else {
                        tracing::trace!("Message has no start line");
                        continue;
                    };

                    // Find the BEGIN line for the message
                    let mut node_begin_line = None;
                    let start_line = (start_line as usize).min(lines.len().saturating_sub(1));
                    for line_index in (0..start_line).rev() {
                        let Some(line) = lines.get(line_index) else {
                            continue;
                        };

                        // Split line into 3 parts
                        let parts: Option<(&str, &str, &str)> =
                            line.split_whitespace().next_tuple();

                        // If we find an END line already then abort the search
                        // (this can happen if the message is related to "anonymous"
                        // code injected for variable declarations etc)
                        if let Some((first, second, ..)) = parts
                            && first == comment_prefix
                            && second == END
                        {
                            break;
                        };

                        // If not a BEGIN line, continue
                        let Some((_, BEGIN, node_id)) = parts else {
                            continue;
                        };

                        // If not able to parse a node id form last part, continue
                        let Ok(node_id) = NodeId::from_str(node_id) else {
                            tracing::debug!("Invalid node id: {node_id}");
                            continue;
                        };

                        // Previous BEGIN line found and node id parsed
                        node_begin_line = Some((node_id, line_index));
                        break;
                    }

                    // Adjust line numbers so that they are relative to BEGIN line
                    // Note that if no BEGIN line was found then the message will
                    // NOT be included since we can not associate it with a node.
                    if let Some((node_id, line_index)) = node_begin_line {
                        if let Some(loc) = message.code_location.as_mut() {
                            if let Some(start_line) = loc.start_line.as_mut() {
                                *start_line = start_line
                                    .saturating_sub(line_index as u64)
                                    .saturating_sub(1);
                            }
                            if let Some(end_line) = loc.end_line.as_mut() {
                                *end_line =
                                    end_line.saturating_sub(line_index as u64).saturating_sub(1);
                            }
                        }

                        node_messages.entry(node_id).or_default().push(message);
                    } else {
                        tracing::debug!("No {BEGIN} line for message");
                    }
                }
            }
        }

        // Send a patch for each of the nodes in the linting context
        for (node_id, ..) in &self.linting_context {
            // Continue if code is not part of the node
            let Some(node_id) = node_id else {
                continue;
            };

            let mut ops = Vec::new();

            if let Some(code) = node_codes.get(node_id) {
                ops.push((
                    NodePath::from(NodeProperty::Code),
                    PatchOp::Set(code.to_value()?),
                ));
            };

            let messages = match node_messages.get(node_id) {
                Some(messages) => messages.to_value()?,
                None => PatchValue::None,
            };
            ops.push((
                NodePath::from(NodeProperty::CompilationMessages),
                PatchOp::Set(messages),
            ));

            let authors = node_authors.remove(node_id);

            let patch = Patch {
                node_id: Some(node_id.clone()),
                ops,
                authors,
                ..Default::default()
            };

            // Send the patch to be applied to the do
            self.send_patch(patch.clone());

            // Apply the patch to the local copy of the code so the any further
            // operations on it e.g. execution have the correct line numberings etc
            // Currently just debug level logging of errors but that may need
            // to be revisited later?
            if let Err(error) = stencila_schema::patch(node, patch.clone()) {
                tracing::debug!("While applying local linting patch: {error}");
            }
        }

        Ok(())
    }

    /// Run [`Phase::Prepare`]
    async fn prepare(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Prepare;
        self.walk_position = 0;
        self.walk_ancestors.clear();
        root.walk_async(self).await
    }

    /// Run [`Phase::Execute`]
    async fn execute(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Execute;
        self.walk_position = 0;
        self.walk_ancestors.clear();
        root.walk_async(self).await?;

        Ok(())
    }

    /// Run [`Phase::Interrupt`]
    async fn interrupt(&mut self, root: &mut Node) -> Result<()> {
        self.phase = Phase::Interrupt;
        self.walk_position = 0;
        self.walk_ancestors.clear();
        root.walk_async(self).await
    }

    /// Run the prepare and execute phases on content
    async fn prepare_execute<W: WalkNode>(&mut self, node: &mut W) -> Result<()> {
        for phase in [Phase::Prepare, Phase::Execute] {
            self.phase = phase;
            node.walk_async(self).await?;
        }

        Ok(())
    }

    /// Run the compile, prepare and execute phases on content
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
    pub async fn kernels(&self) -> RwLockWriteGuard<'_, Kernels> {
        self.kernels.write().await
    }

    /// Updates and returns the current programming language of the executor
    ///
    /// Allows users to not have to specify the language on executable code,
    /// particularly inline expressions, for and if blocks, and call arguments.
    pub fn programming_language(&mut self, lang: &Option<String>) -> Option<String> {
        if let Some(lang) = lang {
            self.programming_language = Some(lang.clone());
        }
        self.programming_language.clone()
    }

    /// Get the current appendix label or an empty string if appendices are not currently active
    pub fn appendix_label(&self) -> String {
        match self.appendix_count {
            None => String::new(),
            Some(index) => {
                // Convert number to alphabetic label (A, B, C... Z, AA, AB...)
                let mut label = String::new();
                let mut num = index;

                while num > 0 {
                    let remainder = (num - 1) % 26;
                    label.insert(0, (b'A' + remainder as u8) as char);
                    num = (num - 1) / 26;
                }

                label
            }
        }
    }

    /// Updates the figure count and returns the current figure label
    pub fn figure_label(&mut self) -> String {
        self.figure_count += 1;

        [self.appendix_label(), self.figure_count.to_string()].concat()
    }

    /// Updates the table count and returns the current table label
    pub fn table_label(&mut self) -> String {
        self.table_count += 1;

        [self.appendix_label(), self.table_count.to_string()].concat()
    }

    /// Updates the equation count and returns the current equation label
    pub fn equation_label(&mut self) -> String {
        self.equation_count += 1;

        [self.appendix_label(), self.equation_count.to_string()].concat()
    }

    /// Updates the supplement count and returns the current supplement label
    pub fn supplement_label(&mut self) -> String {
        self.supplement_count += 1;

        self.supplement_count.to_string()
    }

    /// Get the execution status for a node based on state of node
    /// and options of the executor
    pub fn node_execution_status(
        &self,
        node_type: NodeType,
        node_id: &NodeId,
        execution_mode: &Option<ExecutionMode>,
        execution_required: &Option<ExecutionRequired>,
    ) -> Option<ExecutionStatus> {
        // If the node is locked then do not execute
        // A locked node should never be executed, not even if force_all is true
        if matches!(execution_mode, Some(ExecutionMode::Lock)) {
            return Some(ExecutionStatus::Locked);
        }

        let ExecuteOptions {
            force_all,
            skip_instructions,
            skip_code,
            ..
        } = self.execute_options.clone().unwrap_or_default();

        // If either force all is on then mark as pending
        if force_all || self.force_all {
            return Some(ExecutionStatus::Pending);
        }

        // If the executor has any node ids then the current
        // node id must be amongst them
        if let Some(node_ids) = &self.node_ids {
            return if node_ids.contains(node_id) {
                Some(ExecutionStatus::Pending)
            } else {
                None
            };
        }

        // If the node is only to be executed on demand, and there are no node
        // ids, or the node is not among them (checked above) then do not execute
        if matches!(execution_mode, Some(ExecutionMode::Demand)) {
            return Some(ExecutionStatus::Skipped);
        }

        if matches!(
            node_type,
            NodeType::InstructionBlock | NodeType::InstructionInline
        ) {
            if skip_instructions {
                return Some(ExecutionStatus::Skipped);
            }
        } else if skip_code {
            return Some(ExecutionStatus::Skipped);
        }

        // If no `skip_` options applied and node is always to be execute then execute
        if matches!(execution_mode, Some(ExecutionMode::Always)) {
            return Some(ExecutionStatus::Pending);
        }

        // Only remaining execution variant to be checked for is `Need`, so check that
        // node needs to be executed
        if !matches!(execution_required, Some(ExecutionRequired::No)) {
            // If the node has never been executed (both digests are none),
            // or if the digest has changed since last executed, then return
            // `self.execution_status` (usually Pending)
            Some(self.execution_status)
        } else {
            // No change to execution status required
            None
        }
    }

    /// Get the [`AuthorRole`] for a kernel instance with the current timestamp as `last_modified`
    pub async fn node_execution_author_role(&self, instance: &str) -> Option<AuthorRole> {
        if let Some(instance) = self.kernels().await.get_instance(instance).await
            && let Ok(app) = instance.lock().await.info().await
        {
            let mut role = AuthorRole::software(app, AuthorRoleName::Executor);
            role.last_modified = Some(Timestamp::now());
            return Some(role);
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
    pub fn patch_with_authors<P>(&self, node_id: &NodeId, authors: Vec<AuthorRole>, pairs: P)
    where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        self.send_patch_ops(node_id, Some(authors), pairs)
    }

    /// Send patch operations reflecting a change in the state of a node during execution
    fn send_patch_ops<P>(&self, node_id: &NodeId, authors: Option<Vec<AuthorRole>>, pairs: P)
    where
        P: IntoIterator<Item = (NodeProperty, PatchOp)>,
    {
        let Some(sender) = &self.patch_sender else {
            return;
        };

        let ops = pairs
            .into_iter()
            .map(|(property, op)| (NodePath::from(property), op))
            .collect();

        let patch = Patch {
            node_id: Some(node_id.clone()),
            format: None,
            authors,
            ops,
            ..Default::default()
        };

        if let Err(error) = sender.send((patch, None)) {
            tracing::error!("When sending execution node patch: {error}")
        }
    }

    /// Send a patch reflecting a change in the state of a node during execution
    fn send_patch(&self, patch: Patch) {
        let Some(sender) = &self.patch_sender else {
            return;
        };

        if let Err(error) = sender.send((patch, None)) {
            tracing::error!("When sending execution node patch: {error}")
        }
    }

    /// Finalize an operation by sending an empty patch with an acknowledgement
    ///
    /// Call this when the executor has walked over a node, possibly sending multiple patches as it goes,
    /// and you want to wait until all patches have been applied.
    ///
    /// Sends an empty patch with an acknowledgment sender and waits for the acknowledgment. This
    /// ensures that any patches sent previously have been applied before proceeding.
    async fn finalize(&self) -> Result<()> {
        let Some(sender) = &self.patch_sender else {
            bail!("No patch sender for this executor");
        };

        let (ack_sender, ack_receiver) = oneshot::channel();
        if let Err(error) = sender.send((Patch::default(), Some(ack_sender))) {
            tracing::error!("When sending execution node patch: {error}")
        }

        tracing::trace!("Waiting for finalization patch");
        ack_receiver.await?;

        Ok(())
    }

    /// Visit an executable node and call the appropriate method for the phase
    async fn visit_executable<E: Executable>(&mut self, node: &mut E) -> WalkControl {
        match self.phase {
            Phase::Compile => node.compile(self).await,
            Phase::Link => node.link(self).await,
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
            Prompt(node) => self.visit_executable(node).await,
            Chat(node) => self.visit_executable(node).await,
            // Visit nodes in outputs of code chunks
            CodeChunk(node) => self.visit_executable(node).await,
            Excerpt(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_block(&mut self, block: &mut Block) -> Result<WalkControl> {
        self.walk_position += 1;

        use Block::*;
        Ok(match block {
            AppendixBreak(node) => self.visit_executable(node).await,
            CallBlock(node) => self.visit_executable(node).await,
            Chat(node) => self.visit_executable(node).await,
            CodeChunk(node) => self.visit_executable(node).await,
            Figure(node) => self.visit_executable(node).await,
            ForBlock(node) => self.visit_executable(node).await,
            Heading(node) => self.visit_executable(node).await,
            IfBlock(node) => self.visit_executable(node).await,
            IncludeBlock(node) => self.visit_executable(node).await,
            InstructionBlock(node) => self.visit_executable(node).await,
            Island(node) => self.visit_executable(node).await,
            MathBlock(node) => self.visit_executable(node).await,
            PromptBlock(node) => self.visit_executable(node).await,
            RawBlock(node) => self.visit_executable(node).await,
            StyledBlock(node) => self.visit_executable(node).await,
            SuggestionBlock(node) => self.visit_executable(node).await,
            Supplement(node) => self.visit_executable(node).await,
            Table(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    async fn visit_suggestion_block(&mut self, block: &mut SuggestionBlock) -> Result<WalkControl> {
        Ok(self.visit_executable(block).await)
    }

    async fn visit_if_block_clause(&mut self, block: &mut IfBlockClause) -> Result<WalkControl> {
        Ok(self.visit_executable(block).await)
    }

    async fn visit_inline(&mut self, inline: &mut Inline) -> Result<WalkControl> {
        self.walk_position += 1;

        use Inline::*;
        Ok(match inline {
            Citation(node) => self.visit_executable(node).await,
            CitationGroup(node) => self.visit_executable(node).await,
            CodeExpression(node) => self.visit_executable(node).await,
            InstructionInline(node) => self.visit_executable(node).await,
            MathInline(node) => self.visit_executable(node).await,
            Link(node) => self.visit_executable(node).await,
            Parameter(node) => self.visit_executable(node).await,
            StyledInline(node) => self.visit_executable(node).await,
            Text(node) => self.visit_executable(node).await,
            _ => WalkControl::Continue,
        })
    }

    fn enter_struct(&mut self, node_type: NodeType, _node_id: NodeId) -> WalkControl {
        self.walk_ancestors.push(node_type);
        WalkControl::Continue
    }

    fn exit_struct(&mut self) {
        self.walk_ancestors.pop();
    }
}
