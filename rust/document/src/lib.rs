#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use codecs::PoshMap;
use common::{
    clap::{self, ValueEnum},
    eyre::{bail, eyre, OptionExt, Result},
    serde::{Deserialize, Serialize},
    serde_json,
    smart_default::SmartDefault,
    strum::{Display, EnumString},
    tokio::{
        self,
        fs::read_to_string,
        sync::{mpsc, oneshot, watch, RwLock},
    },
    tracing,
};
use kernels::Kernels;
use node_diagnostics::{diagnostics, Diagnostic, DiagnosticLevel};
use node_find::find;
use node_first::first;
use schema::{
    Article, AuthorRole, Chat, Config, ContentType, ExecutionBounds, File, Node, NodeId,
    NodeProperty, NodeType, Null, Patch, Prompt,
};

#[allow(clippy::print_stderr)]
pub mod cli;

mod config;
mod demo;
mod files;
mod sync_directory;
mod sync_dom;
mod sync_file;
mod sync_format;
mod sync_object;
mod task_command;
mod task_update;
mod track;

// Re-exports for convenience of consuming crates
pub use codecs::{self, DecodeOptions, EncodeOptions, Format, LossesResponse};
pub use node_execute::ExecuteOptions;
pub use schema;
pub use sync_dom::DomPatch;

/// The synchronization mode between documents and external resources
///
/// Examples of external resources which may be synchronized with a document include
/// a file on the local file system or an editor in a web browser. This enum determines
/// whether changes in the document should be reflected in the resource and vice versa.
#[derive(Debug, Display, Default, Clone, Copy, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum SyncDirection {
    In,
    Out,
    #[default]
    InOut,
}

/// An entry in the log of a document
///
/// Made `Serialize` so that, if desired, the log can be obtained
/// as JSON from the CLI.
#[derive(Serialize)]
#[serde(crate = "common::serde")]
pub struct LogEntry {
    pub hash: String,
    pub parents: Vec<String>,
    pub timestamp: i64,
    pub author: String,
    pub message: String,
}

/// A command on a document, or nodes within it
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[strum(serialize_all = "kebab-case")]
#[serde(tag = "command", rename_all = "kebab-case", crate = "common::serde")]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Compile the document
    CompileDocument { config: Config },

    /// Lint the document
    LintDocument {
        format: bool,
        fix: bool,
        config: Config,
    },

    /// Execute the entire document
    ExecuteDocument(ExecuteOptions),

    /// Execute specific nodes within the document
    ExecuteNodes((CommandNodes, ExecuteOptions)),

    /// Interrupt the entire document
    InterruptDocument,

    /// Interrupt specific nodes within the document
    InterruptNodes(CommandNodes),

    /// Patch a node in the document
    PatchNode(Patch),

    /// Patch a node in the document using content in a specific format
    PatchNodeFormat {
        node_id: Option<NodeId>,
        property: NodeProperty,
        format: Format,
        content: String,
        content_type: ContentType,
    },

    /// Patch the document and then execute nodes within it
    ///
    /// This command should be used when it is necessary to patch and then
    /// immediately execute as it avoid race conditions associated with
    /// sending separate patch and execute commands.
    PatchExecuteNodes((Patch, CommandNodes, ExecuteOptions)),

    /// Patch and then execute a chat
    ///
    /// Adds a new user [`ChatMessage`] to the chat and then executes the
    /// chat thereby creating a new model message.
    PatchExecuteChat {
        chat_id: NodeId,
        text: String,
        files: Option<Vec<File>>,
    },
}

/// The node ids for commands that require them
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "common::serde")]
pub struct CommandNodes {
    /// The list of nodes involved in a command
    #[serde(alias = "nodeIds")]
    node_ids: Vec<NodeId>,

    /// The scope for the command
    scope: CommandScope,
}

impl CommandNodes {
    pub fn new(node_ids: Vec<NodeId>, scope: CommandScope) -> Self {
        Self { node_ids, scope }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum CommandScope {
    /// Only listed nodes
    #[default]
    Only,

    /// Listed nodes and any that appear before them in the document
    PlusBefore,

    /// Listed nodes and any that appear after them in the document
    PlusAfter,

    /// Listed nodes, upstream dependencies and downstream dependents
    PlusUpstreamDownstream,
}

/// The status of a command
#[derive(Clone)]
pub enum CommandStatus {
    Ignored,
    Waiting,
    Running,
    Succeeded,
    Failed(String),
    Interrupted,
}

impl CommandStatus {
    /// Did the command finish successfully
    ///
    /// Returns an `Err` if the command did not succeed.
    pub fn ok(self) -> Result<()> {
        use CommandStatus::*;
        match self {
            Ignored => bail!("Command was ignored"),
            Waiting => bail!("Command is waiting"),
            Running => bail!("Command is running"),
            Succeeded => Ok(()),
            Failed(error) => bail!("Command failed: {error}"),
            Interrupted => bail!("Command was interrupted"),
        }
    }

    /// Did the command succeed?
    pub fn succeeded(&self) -> bool {
        matches!(self, CommandStatus::Succeeded)
    }

    /// Did the command fail?
    pub fn failed(&self) -> bool {
        matches!(self, CommandStatus::Failed(..))
    }

    /// Has the command finished (includes failed or interrupted but not waiting or running)
    pub fn finished(&self) -> bool {
        use CommandStatus::*;
        matches!(self, Ignored | Succeeded | Failed(..) | Interrupted)
    }
}

/// An update to the root node of the document
#[derive(Debug, SmartDefault)]
pub struct Update {
    /// The new value of the node (at present always the `root` of the document)
    pub node: Node,

    /// The source format that generated the update
    ///
    /// If `None` then the update is assumed to be programmatically generated
    /// internally, rather than from a source format.
    pub format: Option<Format>,

    /// The authors of the update
    pub authors: Option<Vec<AuthorRole>>,

    /// Whether to compile the document after the update
    ///
    /// If `lint` is `true` then this will be ignored (since linting
    /// involves compiling the document).
    #[default = true]
    pub compile: bool,

    /// Whether to lint the document after the update
    pub lint: bool,

    /// Whether to execute the document after the update
    pub execute: Option<Vec<NodeId>>,
}

impl Update {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            ..Default::default()
        }
    }
}

type DocumentKernels = Arc<RwLock<Kernels>>;

type DocumentRoot = Arc<RwLock<Node>>;

type DocumentWatchSender = watch::Sender<Node>;
type DocumentWatchReceiver = watch::Receiver<Node>;

type DocumentAckSender = oneshot::Sender<()>;

type DocumentUpdateSender = mpsc::Sender<(Update, Option<DocumentAckSender>)>;
type DocumentUpdateReceiver = mpsc::Receiver<(Update, Option<DocumentAckSender>)>;

type DocumentPatchSender = mpsc::UnboundedSender<(Patch, Option<DocumentAckSender>)>;
type DocumentPatchReceiver = mpsc::UnboundedReceiver<(Patch, Option<DocumentAckSender>)>;

type DocumentCommandStatusSender = mpsc::Sender<CommandStatus>;
type DocumentCommandSender = mpsc::Sender<(Command, Option<DocumentCommandStatusSender>)>;
type DocumentCommandReceiver = mpsc::Receiver<(Command, Option<DocumentCommandStatusSender>)>;

/// A document
#[allow(unused)]
#[derive(Debug)]
pub struct Document {
    /// The document's home directory
    home: PathBuf,

    /// The path to the document's source file
    path: Option<PathBuf>,

    /// Options for decoding the document from its source, and the source of any `IncludeBlocks` within it
    decode_options: Option<DecodeOptions>,

    /// The root node of the document
    root: DocumentRoot,

    /// The document's execution kernels
    kernels: DocumentKernels,

    /// A channel receiver for watching for changes to the root [`Node`]
    watch_receiver: DocumentWatchReceiver,

    /// A channel sender for sending updates to the root [`Node`]
    update_sender: DocumentUpdateSender,

    /// A channel sender for sending patches to the root [`Node`]
    patch_sender: DocumentPatchSender,

    /// A channel sender for sending commands to the document
    command_sender: DocumentCommandSender,
}

impl Document {
    /// Initialize a new document
    ///
    /// Initializes the document's "watch", "update", "patch", and "command" channels, and
    /// starts the corresponding background tasks.
    #[tracing::instrument]
    pub fn init(
        home: PathBuf,
        path: Option<PathBuf>,
        decode_options: Option<DecodeOptions>,
        root: Option<Node>,
        node_type: Option<NodeType>,
    ) -> Result<Self> {
        let root = if let Some(root) = root {
            // Use the supplied root node
            root
        } else {
            // Create the default root node type, if there is no sidecar file
            // The default node type itself defaults to none, because that is used in the
            // merge function to signal that root should be written over.
            let root_default = || match node_type.unwrap_or(NodeType::Null) {
                NodeType::Article => Node::Article(Article::default()),
                NodeType::Chat => Node::Chat(Chat::default()),
                NodeType::Prompt => Node::Prompt(Prompt::default()),
                _ => Node::Null(Null),
            };

            // Create the root node from the store or an empty article
            match &path {
                Some(path) => Document::restore(path).ok().unwrap_or_else(&root_default),
                None => root_default(),
            }
        };

        // Create channels
        let (watch_sender, watch_receiver) = watch::channel(root.clone());
        let root = Arc::new(RwLock::new(root));
        let (update_sender, update_receiver) = mpsc::channel(8);
        let (patch_sender, patch_receiver) = mpsc::unbounded_channel();
        let (command_sender, command_receiver) = mpsc::channel(256);

        // Create the document's kernels with the same home directory
        let kernels = Kernels::new(ExecutionBounds::Main, &home, Some(watch_receiver.clone()));
        let kernels = Arc::new(RwLock::new(kernels));

        // Start the update task
        {
            let root = root.clone();
            let path = path.clone();
            let command_sender = command_sender.clone();
            tokio::spawn(async move {
                Self::update_task(
                    update_receiver,
                    patch_receiver,
                    root,
                    path,
                    watch_sender,
                    command_sender,
                )
                .await
            });
        }

        // Start the command task
        {
            let home = home.clone();
            let root = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();
            let decode_options = decode_options.clone();
            tokio::spawn(async move {
                Self::command_task(
                    command_receiver,
                    home,
                    root,
                    kernels,
                    patch_sender,
                    decode_options,
                )
                .await
            });
        }

        Ok(Self {
            home,
            path,
            decode_options,
            root,
            kernels,
            watch_receiver,
            update_sender,
            patch_sender,
            command_sender,
        })
    }

    /// Create a new in-memory document of a specific node type
    pub async fn new(node_type: NodeType) -> Result<Self> {
        let home = std::env::current_dir()?;
        Self::init(home, None, None, None, Some(node_type))
    }

    /// Create a new document from an existing root, node
    pub async fn from(
        root: Node,
        path: Option<PathBuf>,
        decode_options: Option<DecodeOptions>,
    ) -> Result<Self> {
        let home = match &path {
            Some(path) => path
                .parent()
                .ok_or_eyre("path has no parent")?
                .to_path_buf(),
            None => std::env::current_dir()?,
        };
        Self::init(home, path, decode_options, Some(root), None)
    }

    /// Initialize a document at a path
    ///
    /// Note that this simply associates the document with the path.
    /// It does not read the document from the path. Use `open`
    /// or `synced` for that.
    async fn at(
        path: &Path,
        decode_options: Option<DecodeOptions>,
        node_type: Option<NodeType>,
    ) -> Result<Self> {
        let home = path
            .parent()
            .ok_or_else(|| eyre!("path has no parent; is it a file?"))?
            .to_path_buf();

        Self::init(
            home,
            Some(path.to_path_buf()),
            decode_options,
            None,
            node_type,
        )
    }

    /// Create a new document at a path
    #[tracing::instrument]
    pub async fn create(path: &Path, force: bool, node_type: NodeType) -> Result<Self> {
        // Check for existing file
        if path.exists() {
            if !force {
                bail!("File already exists; remove the file or use the `--force` option")
            } else {
                // TODO: untrack and delete the existing file
            }
        }

        // Create a document at the path and track it (which will create a tracking file)
        // and save it (which create the file itself)
        let doc = Self::at(path, None, Some(node_type)).await?;
        doc.save().await?;
        doc.track(None).await?;
        Ok(doc)
    }

    /// Open an existing document
    ///
    /// Restores the document if possible (ie. if there is a .stencila/store file for it)
    /// and then imports the path (merging in any differences).
    #[tracing::instrument]
    pub async fn open(path: &Path, decode_options: Option<DecodeOptions>) -> Result<Self> {
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        let doc = Self::at(path, decode_options.clone(), None).await?;
        doc.import(path, decode_options, None).await?;

        Ok(doc)
    }

    /// Open an existing document with syncing
    #[tracing::instrument]
    pub async fn synced(path: &Path, sync: SyncDirection) -> Result<Self> {
        let doc = Self::open(path, None).await?;

        doc.sync_file(path, sync, None, None).await?;

        Ok(doc)
    }

    /// Get the path of the document
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Get the parent directory of the document
    pub fn directory(&self) -> &Path {
        self.path
            .as_ref()
            .and_then(|path| path.parent())
            .unwrap_or_else(|| self.home.as_ref())
    }

    /// Get the file name of the document
    pub fn file_name(&self) -> Option<&str> {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
    }

    /// The format of the document
    ///
    /// Will return `Format::Unknown` if the document has no path
    pub fn format(&self) -> Format {
        match &self.path {
            Some(path) => Format::from_path(path),
            None => Format::Unknown,
        }
    }

    /// Inspect the root node of the document using a function or closure
    ///
    /// See [`Document::mutate`] for an alternative if it is necessary to have
    /// a mutable reference to the root node.
    pub async fn inspect<F, R>(&self, func: F) -> R
    where
        F: Fn(&Node) -> R,
    {
        func(&*self.root.read().await)
    }

    /// Mutate the root node of the document using a function or closure
    ///
    /// See [`Document::inspect`] for an alternative if it is not necessary to have
    /// a mutable reference to the root node. When using this function, note that
    /// no watchers will be notified of the update.
    pub async fn mutate<F, R>(&self, func: F) -> R
    where
        F: Fn(&mut Node) -> R,
    {
        func(&mut *self.root.write().await)
    }

    /// Get a clone of the root node of the document
    ///
    /// Clone's the entire root node, so should only be used when necessary.
    /// Consider using [`Document::inspect`] when you just need to obtain
    /// some part of the document tree.
    pub async fn root(&self) -> Node {
        self.root.read().await.clone()
    }

    /// Find and clone a node within the root node of the document
    ///
    /// Consider using [`Document::inspect`] if it is not necessary to
    /// obtain a clone of the entire node.
    pub async fn find(&self, node_id: NodeId) -> Option<Node> {
        find(&*self.root.read().await, node_id)
    }

    /// Load a string, in a given format, into a new, or existing, document
    ///
    /// The format must be specified in the `format` option.
    ///
    /// Sends the loaded node to the update task so it can be assigned or merged
    /// and watchers notified. Does NOT compile the node as part of the update.
    #[tracing::instrument(skip(self, source))]
    pub async fn load(
        &self,
        source: &str,
        options: Option<DecodeOptions>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        let Some(format) = options.as_ref().and_then(|opts| opts.format.clone()) else {
            bail!("The `format` option must be provided")
        };

        let node = codecs::from_str(source, options).await?;

        self.update(Update {
            node,
            format: Some(format),
            authors,
            compile: false,
            ..Default::default()
        })
        .await
    }

    /// Import a file into a new, or existing, document
    ///
    /// By default, the format of the `source` file is inferred from its extension but
    /// this can be overridden by providing the `format` option.
    ///
    /// Sends the imported node to the update task so it can be assigned or merged
    /// and watchers notified. Does NOT compile the node as part of the update.
    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        source: &Path,
        options: Option<DecodeOptions>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        let options = options.or(self.decode_options.clone());

        let format = options
            .as_ref()
            .and_then(|opts| opts.format.clone())
            .or_else(|| Some(Format::from_path(source)));
        let node = codecs::from_path(source, options).await?;

        self.update(Update {
            node,
            format,
            authors,
            compile: false,
            ..Default::default()
        })
        .await
    }

    /// Dump a document to a string in a specified format
    ///
    /// The format argument is required (rather than using the `format` in `options`)
    /// because it is probably an error to call this without a format specified.
    #[tracing::instrument(skip(self))]
    pub async fn dump(&self, format: Format, options: Option<EncodeOptions>) -> Result<String> {
        let root = self.root.read().await;

        let options = EncodeOptions {
            format: Some(format),
            ..options.unwrap_or_default()
        };
        codecs::to_string(&root, Some(options)).await
    }

    /// Export a document to a file
    ///
    /// By default the format is inferred from the extension of the `dest` file
    /// but this can be overridden by providing the `format` option.
    ///
    /// Returns a boolean indicating whether the export was completed or not.
    #[tracing::instrument(skip(self))]
    pub async fn export(&self, dest: &Path, options: Option<EncodeOptions>) -> Result<bool> {
        let root = self.root.read().await;

        codecs::to_path(&root, dest, options).await
    }

    /// Save the document to its source file
    #[tracing::instrument(skip(self))]
    pub async fn save(&self) -> Result<()> {
        tracing::trace!("Saving document");

        let Some(path) = self.path() else {
            bail!("Unable to save document; it has no path yet");
        };

        self.export(
            path,
            Some(EncodeOptions {
                // Ignore losses because lossless tracking storage file is encoded next.
                losses: LossesResponse::Ignore,
                ..Default::default()
            }),
        )
        .await?;

        Ok(())
    }

    /// Subscribe to updates to the document's root node
    pub fn watch(&self) -> watch::Receiver<Node> {
        self.watch_receiver.clone()
    }

    /// Update the root node of the document with a new node value
    ///
    /// This is usually the best way to update the document's root node, rather than
    /// assigning to it directly because watchers will be notified. Waits for acknowledgment
    /// that the update was applied.
    pub async fn update(&self, update: Update) -> Result<()> {
        tracing::trace!("Sending document update");

        let (sender, receiver) = oneshot::channel();
        self.update_sender.send((update, Some(sender))).await?;
        receiver.await?;

        Ok(())
    }

    /// Send a command to the document without waiting for it or subscribing to its status
    #[tracing::instrument(skip(self))]
    pub async fn command_send(&self, command: Command) -> Result<()> {
        tracing::trace!("Sending document command");

        Ok(self.command_sender.send((command, None)).await?)
    }

    /// Send a command to the document and obtain a command id and
    /// a receiver to receive updates on its status
    #[tracing::instrument(skip(self))]
    pub async fn command_subscribe(
        &self,
        command: Command,
    ) -> Result<mpsc::Receiver<CommandStatus>> {
        tracing::trace!("Sending document command and returning status subscription");

        let (sender, receiver) = mpsc::channel(24);
        self.command_sender.send((command, Some(sender))).await?;

        Ok(receiver)
    }

    /// Send a command to the document and wait for it to complete
    #[tracing::instrument(skip(self))]
    pub async fn command_wait(&self, command: Command) -> Result<()> {
        tracing::trace!("Sending document command and waiting for it to finish");

        // Send command
        let (sender, mut receiver) = mpsc::channel(24);
        self.command_sender.send((command, Some(sender))).await?;

        // Wait for the command status to be finished
        tracing::trace!("Waiting for command to finish");
        while let Some(status) = receiver.recv().await {
            if status.finished() {
                tracing::trace!("Command finished");

                // Bail if command failed
                if let CommandStatus::Failed(error) = status {
                    bail!("Command failed: {error}")
                }

                break;
            }
        }

        Ok(())
    }

    /// Compile the document
    ///
    /// Note that this does not do any linting. Use the [`Document::lint`] function for that.
    #[tracing::instrument(skip(self))]
    pub async fn compile(&self) -> Result<()> {
        tracing::trace!("Compiling document");

        let config = self.config().await?;
        self.command_wait(Command::CompileDocument { config }).await
    }

    /// Lint the document
    #[tracing::instrument(skip(self))]
    pub async fn lint(&self, format: bool, fix: bool) -> Result<()> {
        tracing::trace!("Linting document");

        let config = self.config().await?;
        self.command_wait(Command::LintDocument {
            format,
            fix,
            config,
        })
        .await
    }

    /// Execute the document
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, options: ExecuteOptions) -> Result<()> {
        tracing::trace!("Executing document");

        self.command_wait(Command::ExecuteDocument(options)).await
    }

    /// Call the document
    #[tracing::instrument(skip(self))]
    pub async fn call(&self, arguments: &[(&str, &str)], options: ExecuteOptions) -> Result<()> {
        tracing::trace!("Calling document");

        // If there are no arguments then just execute the document
        if arguments.is_empty() {
            return self.command_wait(Command::ExecuteDocument(options)).await;
        }

        // Get the default language of the document. Currently this is just the first
        // language used in any `CodeExecutable` node (usually will be a `CodeChunk`)
        let language = self
            .inspect(|root| {
                first(
                    root,
                    &[
                        NodeType::CodeChunk,
                        NodeType::CodeExpression,
                        NodeType::ForBlock,
                        NodeType::IfBlockClause,
                    ],
                )
                .and_then(|node| match node {
                    Node::CodeChunk(node) => node.programming_language,
                    Node::CodeExpression(node) => node.programming_language,
                    Node::ForBlock(node) => node.programming_language,
                    Node::IfBlockClause(node) => node.programming_language,
                    _ => None,
                })
            })
            .await;

        // Set each argument in the kernels (will use the first programming language kernel)
        // In block to ensure lock on kernels is dropped
        {
            let mut kernels = self.kernels.write().await;

            for (name, value) in arguments {
                let value =
                    serde_json::from_str(value).unwrap_or_else(|_| Node::String(value.to_string()));
                kernels.set(name, &value, language.as_deref()).await?;
            }

            drop(kernels);
        }

        self.command_wait(Command::ExecuteDocument(ExecuteOptions {
            // Force re-execution
            // TODO: when dependency analysis is implemented, this needs
            // to be reconsidered. Some code chunks may not need to be re-executed
            // if the variable value did not change.
            force_all: true,
            ..options
        }))
        .await
    }

    /// Get diagnostics for the document
    pub async fn diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics(&*self.root.read().await)
    }

    /// Print diagnostics for the document
    ///
    /// Returns the count or error, warning, and advice diagnostics.
    pub async fn diagnostics_print(&self) -> Result<(usize, usize, usize)> {
        let diagnostics = self.diagnostics().await;

        // No diagnostics so return early
        if diagnostics.is_empty() {
            return Ok((0, 0, 0));
        }

        // If the document has a path read it so that diagnostics can be printed
        // with the original document source as context.
        let mut source = String::new();
        let generated;
        let (path, poshmap) = if let Some(path) = self.path.as_ref() {
            source = read_to_string(path).await?;

            let format = Format::from_path(path);

            let result = codecs::to_string_with_info(
                &*self.root.read().await,
                Some(EncodeOptions {
                    format: Some(format),
                    ..Default::default()
                }),
            )
            .await?;
            generated = result.0;
            let mapping = result.1.mapping;

            let poshmap = PoshMap::new(&source, &generated, mapping);

            (path.to_string_lossy().to_string(), Some(poshmap))
        } else {
            (String::from("<file>"), None)
        };

        // Print each of the diagnostics to stderr
        let mut errors = 0;
        let mut warnings = 0;
        let mut advice = 0;
        for diagnostic in diagnostics {
            match diagnostic.level {
                DiagnosticLevel::Error => errors += 1,
                DiagnosticLevel::Warning => warnings += 1,
                DiagnosticLevel::Advice => advice += 1,
            }

            diagnostic.to_stderr_pretty(&path, &source, &poshmap)?;
        }

        Ok((errors, warnings, advice))
    }
}
