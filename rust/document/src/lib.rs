#![recursion_limit = "256"]

use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use codecs::{DecodeOptions, EncodeOptions};
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
use format::Format;
use kernels::Kernels;
use node_execute::ExecuteOptions;
use schema::{Article, AuthorRole, Node, NodeId, NodeType, Patch};

mod sync_directory;
mod sync_dom;
mod sync_file;
mod sync_format;
mod sync_object;
mod task_command;
mod task_update;

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
pub enum Command {
    /// Save the document
    SaveDocument,

    /// Export the document
    ExportDocument((PathBuf, EncodeOptions)),

    /// Compile the entire document
    CompileDocument,

    /// Execute the entire document
    ExecuteDocument(ExecuteOptions),

    /// Execute specific nodes within the document
    ExecuteNodes(CommandNodes),

    /// Interrupt the entire document
    InterruptDocument,

    /// Interrupt specific nodes within the document
    InterruptNodes(CommandNodes),

    /// Patch a node in the document
    PatchNode(Patch),
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
    Failed,
    Interrupted,
}

impl CommandStatus {
    pub fn is_finished(&self) -> bool {
        use CommandStatus::*;
        matches!(self, Ignored | Succeeded | Failed | Interrupted)
    }
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
///
/// Each document has:
///
/// - A unique `id` which is used for things like establishing a WebSocket connection to it
/// - An Automerge `store` that has a [`Node`] at its root.
/// - An optional `path` that the `store` can be read from, and written to.
/// - A `watch_receiver` which can be cloned to watch for changes to the root [`Node`].
/// - An `update_sender` which can be cloned to send updates to the root [`Node`].
/// - A `command_sender` which can be cloned to send commands to the document
#[allow(unused)]
#[derive(Debug)]
pub struct Document {
    /// The document's id
    id: DocumentId,

    /// The document's home directory
    home: PathBuf,

    /// The filesystem path to the document's Automerge store
    path: Option<PathBuf>,

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
    /// Initialize a new document
    ///
    /// This initializes the document's "watch", "update", and "command" channels, and
    /// starts its background tasks.
    #[tracing::instrument]
    pub fn init(home: PathBuf, path: Option<PathBuf>) -> Result<Self> {
        let id = DocumentId::new();

        // Create the document's kernels with the same home directory
        let kernels = Arc::new(RwLock::new(Kernels::new(&home)));

        // Load the node from the store to initialize the watch channel
        let root = Node::Article(Article::default());
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
            let root = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();
            tokio::spawn(async move {
                Self::command_task(
                    command_receiver,
                    command_status_sender,
                    home,
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
    pub fn new() -> Result<Self> {
        let home = std::env::current_dir()?;
        Self::init(home, None)
    }

    /// Create a new document
    ///
    /// Creates a new Automerge store with a document of `type` at the `path`,
    /// optionally overwriting any existing file at the path by using the `overwrite` option.
    ///
    /// The document can be initialized from a `source` file, in which case `format` may
    /// be used to specify the format of that file, or `codec` the name of the codec to
    /// decode it with.
    #[tracing::instrument]
    pub async fn create(
        path: &Path,
        overwrite: bool,
        source: Option<&Path>,
        format: Option<Format>,
        codec: Option<String>,
    ) -> Result<Self> {
        if path.exists() && !overwrite {
            bail!("Path already exists; remove the file or use the `--overwrite` option")
        }

        let home = path
            .parent()
            .ok_or_else(|| eyre!("path has no parent; is it a file?"))?
            .to_path_buf();

        Self::init(home, None)
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

        Self::init(home, None)
    }

    /// Open an existing document
    ///
    /// If the path is a store the loads from that store, otherwise
    /// uses `codec` to import from the path and dump into a new store.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        let doc = Self::at(path)?;
        doc.import(path, None).await?;
        Ok(doc)
    }

    /// Open an existing document with syncing
    #[tracing::instrument]
    pub async fn synced(path: &Path, sync: SyncDirection) -> Result<Self> {
        let doc = Self::at(path)?;
        doc.sync_file(path, sync, None, None).await?;
        Ok(doc)
    }

    /// Get the id of the document
    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    /// Get the [`NodeType`] of the root node
    pub async fn root_type(&self) -> NodeType {
        self.root.read().await.node_type()
    }

    /// Import a file into a new, or existing, document
    ///
    /// By default the format of the `source` file is inferred from its extension but
    /// this can be overridden by providing the `format` option.
    #[tracing::instrument(skip(self))]
    pub async fn import(&self, source: &Path, options: Option<DecodeOptions>) -> Result<()> {
        let node = codecs::from_path(source, options).await?;

        let mut root = self.root.write().await;
        *root = node;

        Ok(())
    }

    /// Export a document to a file in another format
    ///
    /// This loads the root [`Node`] from the document's store and encodes it to
    /// the destination format. By default the format is inferred from the extension
    /// of the `dest` file but this can be overridden by providing the `format` option.
    ///
    /// If `dest` is `None`, then the root node is encoded as a string and returned. This
    /// is usually only desireable for text-based formats (e.g. JSON, Markdown).
    #[tracing::instrument(skip(self))]
    pub async fn export(
        &self,
        dest: Option<&Path>,
        options: Option<EncodeOptions>,
    ) -> Result<String> {
        let root = self.root.read().await;

        if let Some(dest) = dest {
            codecs::to_path(&root, dest, options).await?;
            Ok(String::new())
        } else {
            codecs::to_string(&root, options).await
        }
    }

    /// Subscribe to updates to the document's root node
    pub fn watch(&self) -> watch::Receiver<Node> {
        self.watch_receiver.clone()
    }

    /// Update the root node of the document
    pub async fn update(
        &self,
        node: Node,
        format: Option<Format>,
        authors: Option<Vec<AuthorRole>>,
    ) -> Result<()> {
        Ok(self
            .update_sender
            .send(Update::new(node, format, authors))
            .await?)
    }

    /// Perform a command on the document
    #[tracing::instrument(skip(self))]
    pub async fn command(&self, command: Command) -> Result<()> {
        tracing::trace!("Performing document command");
        Ok(self.command_sender.send((command, 0)).await?)
    }

    /// Perform a command on the document and obtain a command id and
    /// a receiver to receive updates on its status
    #[tracing::instrument(skip(self))]
    pub async fn command_subscribe(
        &self,
        command: Command,
    ) -> Result<(u64, DocumentCommandStatusReceiver)> {
        tracing::trace!("Performing document command and returning status subscription");

        let command_id: u64 = self
            .command_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let status_receiver = self.command_status_receiver.resubscribe();

        self.command_sender.send((command, command_id)).await?;

        Ok((command_id, status_receiver))
    }

    /// Perform a command on the document and optionally wait for it to complete
    #[tracing::instrument(skip(self))]
    pub async fn command_wait(&self, command: Command) -> Result<()> {
        tracing::trace!("Performing document command and waiting for it to finish");

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
            if id == command_id && status.is_finished() {
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

    /// Compile the document
    #[tracing::instrument(skip(self))]
    pub async fn compile(&self, wait: bool) -> Result<()> {
        tracing::trace!("Compiling document");
        let command = Command::CompileDocument;
        match wait {
            false => self.command(command).await,
            true => self.command_wait(command).await,
        }
    }

    /// Execute the document
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, options: ExecuteOptions, wait: bool) -> Result<()> {
        tracing::trace!("Executing document");
        let command = Command::ExecuteDocument(options);
        match wait {
            false => self.command(command).await,
            true => self.command_wait(command).await,
        }
    }
}
