#![recursion_limit = "256"]

use std::{
    io,
    path::{Path, PathBuf},
    sync::{atomic::AtomicU64, Arc},
    time::{Duration, SystemTime},
};

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, eyre, Result},
    serde::{Deserialize, Serialize},
    strum::{Display, EnumString},
    tokio::{
        self,
        sync::{broadcast, mpsc, watch, RwLock},
        time::sleep,
    },
    tracing,
    type_safe_id::{StaticType, TypeSafeId},
};
use kernels::Kernels;
use node_execute::ExecuteOptions;
use node_find::find;
use schema::{
    authorship, Article, AuthorRole, Chat, File, Node, NodeId, NodeProperty, NodeType, Null, Patch,
    Prompt,
};

mod config;
mod sync_directory;
mod sync_dom;
mod sync_file;
mod sync_format;
mod sync_object;
mod task_command;
mod task_update;

// Re-exports for convenience of consuming crates
pub use codecs::{self, DecodeOptions, EncodeOptions, Format, LossesResponse};
pub use schema;
pub use sync_dom::DomPatch;

#[derive(Default)]
pub struct Document_;

impl StaticType for Document_ {
    const TYPE: &'static str = "doc";
}

pub type DocumentId = TypeSafeId<Document_>;

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
    /// Save the document
    SaveDocument((SaveDocumentSource, SaveDocumentSidecar)),

    /// Export the document
    ExportDocument((PathBuf, EncodeOptions)),

    /// Compile the entire document
    CompileDocument,

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

/// Whether the document source file should be saved
#[derive(Default, Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum SaveDocumentSource {
    #[default]
    Yes,
    No,
}

/// Whether the document sidecar file should be saved
#[derive(Default, Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum SaveDocumentSidecar {
    Yes,
    #[default]
    IfExists,
    No,
}

/// The type of content in a
#[derive(Default, Debug, Display, EnumString, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ContentType {
    #[default]
    Block,
    Inline,
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

/// Whether or not to wait for a command
#[derive(Debug)]
pub enum CommandWait {
    Yes,
    No,
}

/// An update to the root node of the document
#[derive(Debug, Default)]
pub(crate) struct Update {
    /// The new value of the node (at present always the `root` of the document)
    pub node: Node,

    /// The source format that generated the update
    ///
    /// If `None` then the update is assumed to be programmatically generated
    /// internally, rather than from a source format.
    pub format: Option<Format>,

    /// The authors of the update
    pub authors: Option<Vec<AuthorRole>>,
}

impl Update {
    pub fn new(node: Node, format: Option<Format>, authors: Option<Vec<AuthorRole>>) -> Self {
        Self {
            node,
            format,
            authors,
        }
    }
}

type DocumentKernels = Arc<RwLock<Kernels>>;

type DocumentRoot = Arc<RwLock<Node>>;

type DocumentWatchSender = watch::Sender<Node>;
type DocumentWatchReceiver = watch::Receiver<Node>;

type DocumentUpdateSender = mpsc::Sender<Update>;
type DocumentUpdateReceiver = mpsc::Receiver<Update>;

type DocumentPatchSender = mpsc::UnboundedSender<Patch>;
type DocumentPatchReceiver = mpsc::UnboundedReceiver<Patch>;

type DocumentCommandCounter = AtomicU64;

type DocumentCommandSender = mpsc::Sender<(Command, u64)>;
type DocumentCommandReceiver = mpsc::Receiver<(Command, u64)>;

type DocumentCommandStatusSender = broadcast::Sender<(u64, CommandStatus)>;
type DocumentCommandStatusReceiver = broadcast::Receiver<(u64, CommandStatus)>;

/// A document
#[allow(unused)]
#[derive(Debug)]
pub struct Document {
    /// The document's id
    id: DocumentId,

    /// The document's home directory
    home: PathBuf,

    /// The path to the document's source file
    path: Option<PathBuf>,

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

    /// A counter of commands used for creating unique command ids
    command_counter: DocumentCommandCounter,

    /// A channel sender for sending commands to the document
    command_sender: DocumentCommandSender,

    /// A channel for receiving notifications of command status
    command_status_receiver: DocumentCommandStatusReceiver,
}

impl Document {
    /// Get the path to the sidecar file for a document
    ///
    /// Returns the first existing path matching a hard-coded, ordered list
    /// of possible sidecar formats, falling back to `<path>.json.zip` if no sidecar file exists.
    pub fn sidecar_path(path: &Path) -> PathBuf {
        static SIDECAR_FORMATS: [Format; 2] = [Format::JsonZip, Format::Json];

        // See if any existing paths have one of the formats.
        //
        // NOTE: Having extensions with two dots requires us to set an extension on different
        // instances of the original path inside the loop. Otherwise you can end up with `.json.json` etc
        for format in &SIDECAR_FORMATS {
            let mut candidate = path.to_path_buf();
            candidate.set_extension(format.extension());
            if candidate.exists() {
                return candidate;
            }
        }

        // Fallback to using the preferred format (the first in SIDECAR_FORMATS)
        let mut path = path.to_path_buf();
        path.set_extension(SIDECAR_FORMATS[0].extension());
        path
    }

    /// Initialize a new document
    ///
    /// Initializes the document's "watch", "update", "patch", and "command" channels, and
    /// starts the corresponding background tasks.
    #[tracing::instrument]
    pub fn init(home: PathBuf, path: Option<PathBuf>, node_type: Option<NodeType>) -> Result<Self> {
        let id = DocumentId::new();

        // Create the document's kernels with the same home directory
        let kernels = Arc::new(RwLock::new(Kernels::new(&home)));

        // Create the default root node type, if there is no sidecar file
        // The default node type itself defaults to none, because that is used in the
        // merge function to signal that root should be written over.
        let root_default = || match node_type.unwrap_or(NodeType::Null) {
            NodeType::Article => Node::Article(Article::default()),
            NodeType::Chat => Node::Chat(Chat::default()),
            NodeType::Prompt => Node::Prompt(Prompt::default()),
            _ => Node::Null(Null),
        };

        // Create the root node from the sidecar file or an empty article
        let root = match &path {
            Some(path) => {
                let sidecar = Self::sidecar_path(path);
                if sidecar.exists() {
                    match codec_json::from_path(&sidecar, None) {
                        Ok((node, ..)) => node,
                        Err(error) => {
                            tracing::warn!(
                                "Unable to read sidecar file {}: {error}",
                                sidecar.display()
                            );
                            root_default()
                        }
                    }
                } else {
                    root_default()
                }
            }
            None => root_default(),
        };
        let (watch_sender, watch_receiver) = watch::channel(root.clone());
        let root = Arc::new(RwLock::new(root));

        let (update_sender, update_receiver) = mpsc::channel(8);
        let (patch_sender, patch_receiver) = mpsc::unbounded_channel();
        let (command_sender, command_receiver) = mpsc::channel(256);
        let (command_status_sender, command_status_receiver) = broadcast::channel(256);

        // Start the update task
        {
            let root = root.clone();
            let command_sender = command_sender.clone();
            tokio::spawn(async move {
                Self::update_task(
                    update_receiver,
                    patch_receiver,
                    root,
                    watch_sender,
                    command_sender,
                )
                .await
            });
        }

        // Start command counter at one, so tasks that do not wait can use zero
        let command_counter = AtomicU64::new(1);

        // Start the command task
        {
            let home = home.clone();
            let path = path.clone();
            let root = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();
            tokio::spawn(async move {
                Self::command_task(
                    command_receiver,
                    command_status_sender,
                    home,
                    path,
                    root,
                    kernels,
                    patch_sender,
                )
                .await
            });
        }

        Ok(Self {
            id,
            home,
            path,
            root,
            kernels,
            watch_receiver,
            update_sender,
            patch_sender,
            command_counter,
            command_sender,
            command_status_receiver,
        })
    }

    /// Create a new in-memory document
    pub fn new(node_type: NodeType) -> Result<Self> {
        let home = std::env::current_dir()?;
        Self::init(home, None, Some(node_type))
    }

    /// Initialize a document at a path
    ///
    /// Note that this does not read the document from the path. Use `open`
    /// or `synced` for that.
    fn at(path: &Path) -> Result<Self> {
        let home = path
            .parent()
            .ok_or_else(|| eyre!("path has no parent; is it a file?"))?
            .to_path_buf();

        Self::init(home, Some(path.to_path_buf()), None)
    }

    /// Create a new document at a path
    #[tracing::instrument]
    pub async fn create(
        path: &Path,
        force: bool,
        node_type: NodeType,
        sidecar_format: Option<Format>,
    ) -> Result<Self> {
        // Check for existing file
        if path.exists() && !force {
            bail!("File already exists; remove the file or use the `--force` option")
        }

        // Check for existing sidecar (regardless of its format)
        let sidecar = Self::sidecar_path(path);
        if sidecar.exists() && !force {
            bail!(
                "Sidecar file already exists at `{}`; remove the file or use the `--force` option",
                sidecar.display()
            )
        }

        // If a sidecar format was specified then use that
        let sidecar = if let Some(format) = sidecar_format {
            let mut path = path.to_path_buf();
            path.set_extension(format.extension());
            path
        } else {
            sidecar
        };

        // Create the empty article and write to path
        let node = match node_type {
            NodeType::Article => Node::Article(Article::default()),
            NodeType::Prompt => Node::Prompt(Prompt::default()),
            _ => bail!("Unsupported document type: {node_type}"),
        };
        codecs::to_path(&node, path, None).await?;

        // Create the sidecar file
        codecs::to_path(&node, &sidecar, None).await?;

        Self::at(path)
    }

    /// Open an existing document
    ///
    /// If there is no sidecar file for the `path`, the file at `path` is
    /// loaded into memory.
    ///
    /// If there is a sidecar file for the `path`, that will be loaded into
    /// memory first and then the file at `path` will be merged into it, but
    /// only if it has a last modification time after the sidecar file.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        let doc = Self::at(path)?;

        let mut import = true;

        let sidecar = Self::sidecar_path(path);
        if sidecar.exists() {
            fn modification_time(path: &Path) -> io::Result<SystemTime> {
                let metadata = std::fs::File::open(path)?.metadata()?;
                metadata.modified()
            }

            if modification_time(path)? <= modification_time(&sidecar)? {
                import = false;
            }
        }

        if import {
            doc.import(path, None, None).await?;
        }

        Ok(doc)
    }

    /// Open an existing document with syncing
    #[tracing::instrument]
    pub async fn synced(path: &Path, sync: SyncDirection) -> Result<Self> {
        let doc = Self::open(path).await?;

        doc.sync_file(path, sync, None, None).await?;

        Ok(doc)
    }

    /// Get the id of the document
    pub fn id(&self) -> &DocumentId {
        &self.id
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
    /// a mutable reference to the root node.
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

    /// Assign a node to the root of a document
    ///
    /// The `authors` arguments will be used when recording authorship
    /// of the assigned node.
    #[tracing::instrument(skip(self, node))]
    pub async fn assign(&self, mut node: Node, authors: Option<Vec<AuthorRole>>) -> Result<()> {
        if let Some(authors) = authors {
            authorship(&mut node, authors)?;
        }

        let root = &mut *self.root.write().await;
        *root = node;

        Ok(())
    }

    /// Merge a node into the root of a document
    ///
    /// If the root node is `Null` it will be overwritten by the new
    /// node. Otherwise, the new node will be merged into the root.
    ///
    /// The `authors` arguments will be used when recording authorship
    /// of the assigned node or the merge.
    #[tracing::instrument(skip(self, node))]
    pub async fn merge(
        &self,
        mut node: Node,
        format: Option<Format>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        let root = &mut *self.root.write().await;
        if matches!(root, Node::Null(..)) {
            if let Some(authors) = authors {
                authorship(&mut node, authors)?;
            }
            *root = node;
        } else {
            schema::merge(root, &node, format, authors)?;
        }

        Ok(())
    }

    /// Load a string, in a given format, into a new, or existing, document
    ///
    /// The format must be specified in the `format` option.
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

        self.merge(node, Some(format), authors).await
    }

    /// Import a file into a new, or existing, document
    ///
    /// By default, the format of the `source` file is inferred from its extension but
    /// this can be overridden by providing the `format` option.
    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        source: &Path,
        options: Option<DecodeOptions>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        let format = options
            .as_ref()
            .and_then(|opts| opts.format.clone())
            .or_else(|| Some(Format::from_path(source)));
        let node = codecs::from_path(source, options).await?;

        self.merge(node, format, authors).await
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
    /// By default the format is inferred from the extension
    /// of the `dest` file but this can be overridden by providing the `format` option.
    #[tracing::instrument(skip(self))]
    pub async fn export(&self, dest: &Path, options: Option<EncodeOptions>) -> Result<()> {
        let root = self.root.read().await;

        codecs::to_path(&root, dest, options).await
    }

    /// Subscribe to updates to the document's root node
    pub fn watch(&self) -> watch::Receiver<Node> {
        self.watch_receiver.clone()
    }

    /// Update the root node of the document with a new node value
    pub async fn update(
        &self,
        node: Node,
        format: Option<Format>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        self.update_sender
            .send(Update::new(node, format, authors))
            .await?;
        Ok(())
    }

    /// Perform a command on the document and optionally wait for it to complete
    pub async fn command(&self, command: Command, wait: CommandWait) -> Result<()> {
        match wait {
            CommandWait::No => self.command_send(command).await,
            CommandWait::Yes => self.command_wait(command).await,
        }
    }

    /// Send a command to the document without waiting for it or subscribing to its status
    #[tracing::instrument(skip(self))]
    pub async fn command_send(&self, command: Command) -> Result<()> {
        tracing::trace!("Sending document command");

        Ok(self.command_sender.send((command, 0)).await?)
    }

    /// Send a command to the document and obtain a command id and
    /// a receiver to receive updates on its status
    #[tracing::instrument(skip(self))]
    pub async fn command_subscribe(
        &self,
        command: Command,
    ) -> Result<(u64, DocumentCommandStatusReceiver)> {
        tracing::trace!("Sending document command and returning status subscription");

        let command_id: u64 = self
            .command_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let status_receiver = self.command_status_receiver.resubscribe();

        self.command_sender.send((command, command_id)).await?;

        Ok((command_id, status_receiver))
    }

    /// Send a command to the document and wait for it to complete
    #[tracing::instrument(skip(self))]
    pub async fn command_wait(&self, command: Command) -> Result<()> {
        tracing::trace!("Sending document command and waiting for it to finish");

        // Set up things to be able to wait for completion
        let command_id: u64 = self
            .command_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut status_receiver = self.command_status_receiver.resubscribe();

        // Send command
        self.command_sender.send((command, command_id)).await?;

        // Wait for the command status to be finished
        tracing::trace!("Waiting for command to finish");
        while let Ok((id, status)) = status_receiver.recv().await {
            if id == command_id && status.finished() {
                tracing::trace!("Command finished");

                // Bail if command failed
                if let CommandStatus::Failed(error) = status {
                    bail!("Command failed: {error}")
                }

                break;
            }
        }

        // TODO: This is a hack to wait for any patches to be applied to the
        // store (e.g. updating execution status etc). Currently we have no
        // way to know when that is complete, so this this just sleeps for a bit
        // in the hope that this will be long enough.
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Save the document
    #[tracing::instrument(skip(self))]
    pub async fn save(&self, wait: CommandWait) -> Result<()> {
        tracing::trace!("Saving document");

        self.command(
            Command::SaveDocument((
                SaveDocumentSource::default(),
                SaveDocumentSidecar::default(),
            )),
            wait,
        )
        .await
    }

    /// Save the document with non-default options
    #[tracing::instrument(skip(self))]
    pub async fn save_with(
        &self,
        wait: CommandWait,
        source: SaveDocumentSource,
        sidecar: SaveDocumentSidecar,
    ) -> Result<()> {
        tracing::trace!("Saving document");

        self.command(Command::SaveDocument((source, sidecar)), wait)
            .await
    }

    /// Compile the document
    #[tracing::instrument(skip(self))]
    pub async fn compile(&self, wait: CommandWait) -> Result<()> {
        tracing::trace!("Compiling document");

        self.command(Command::CompileDocument, wait).await
    }

    /// Execute the document
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, options: ExecuteOptions, wait: CommandWait) -> Result<()> {
        tracing::trace!("Executing document");

        self.command(Command::ExecuteDocument(options), wait).await
    }
}
