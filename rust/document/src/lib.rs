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
    itertools::Itertools,
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
use node_patch::NodePatchSender;
use node_store::{inspect_store, load_store, ReadNode, WriteNode, WriteStore};
use schema::{Article, Node, NodeId};

mod sync_directory;
mod sync_file;
mod sync_format;
mod sync_object;
mod task_command;
mod task_update;

/// The document type
///
/// Defines which `CreativeWork` variants can be the root node of a document
/// and the default file extension etc for each variant.
#[derive(Debug, Display, Clone, PartialEq, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum DocumentType {
    Article,
}

impl DocumentType {
    /// Determine the document type from the type of a [`Node`]
    fn from_node(node: &Node) -> Result<Self> {
        match node {
            Node::Article(..) => Ok(Self::Article),
            _ => bail!(
                "Node of type `{}` is not associated with a document type",
                node
            ),
        }
    }

    /// Determine the document type from the [`Format`]
    ///
    /// Returns `None` if the format can be associated with more than one
    /// document type (e.g. [`Format::Json`]).
    #[allow(unused)]
    fn from_format(format: &Format) -> Option<Self> {
        match format {
            Format::Jats | Format::Markdown => Some(Self::Article),
            _ => None,
        }
    }

    /// Get the file extension for the document type
    fn extension(&self) -> &str {
        match self {
            Self::Article => "sta",
        }
    }

    /// Get the default 'main' file name for the document type
    ///
    /// This filename is the default used when creating a document
    /// of this type.
    fn main(&self) -> PathBuf {
        PathBuf::from(format!("main.{}", self.extension()))
    }

    /// Get an empty root [`Node`] for the document type
    fn empty(&self) -> Node {
        match self {
            DocumentType::Article => Node::Article(Article::default()),
        }
    }
}

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
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(tag = "command", rename_all = "kebab-case", crate = "common::serde")]
pub enum Command {
    /// Save the document
    SaveDocument,

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
}

/// The node ids for commands that require them
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde")]
pub struct CommandNodes {
    /// The list of nodes involved in a command
    #[serde(alias = "nodeIds")]
    node_ids: Vec<NodeId>,

    /// The scope for the command
    scope: Option<CommandScope>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
pub enum CommandScope {
    /// Only listed nodes
    #[default]
    Only,

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
    fn is_finished(&self) -> bool {
        use CommandStatus::*;
        matches!(self, Ignored | Succeeded | Failed | Interrupted)
    }
}

type DocumentKernels = Arc<RwLock<Kernels>>;

type DocumentStore = Arc<RwLock<WriteStore>>;

type DocumentWatchSender = watch::Sender<Node>;
type DocumentWatchReceiver = watch::Receiver<Node>;

type DocumentUpdateSender = mpsc::Sender<Node>;
type DocumentUpdateReceiver = mpsc::Receiver<Node>;

type DocumentPatchSender = NodePatchSender;

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

    /// The document's execution kernels
    kernels: DocumentKernels,

    /// The document's Automerge store with a [`Node`] at its root
    store: DocumentStore,

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
    #[tracing::instrument(skip(store))]
    fn init(store: WriteStore, home: PathBuf, path: Option<PathBuf>) -> Result<Self> {
        let id = DocumentId::new();

        // Create the document's kernels with the same home directory
        let kernels = Arc::new(RwLock::new(Kernels::new(&home)));

        // Load the node from the store to initialize the watch channel
        let node = Node::load(&store)?;
        let (watch_sender, watch_receiver) = watch::channel(node);

        // Create an `Arc` for the store so it can be cloned for the document's
        // background tasks
        let store = Arc::new(RwLock::new(store));

        // Start the update task
        let (update_sender, update_receiver) = mpsc::channel(8);
        let (patch_sender, patch_receiver) = mpsc::unbounded_channel();
        let store_clone = store.clone();
        tokio::spawn(async move {
            Self::update_task(update_receiver, patch_receiver, store_clone, watch_sender).await
        });

        // Start counter at one, so tasks that do not wait can use zero
        let command_counter = AtomicU64::new(1);

        // Start the command task
        let (command_sender, command_receiver) = mpsc::channel(256);
        let (command_status_sender, command_status_receiver) = broadcast::channel(256);
        let home_clone = home.clone();
        let store_clone = store.clone();
        let kernels_clone = kernels.clone();
        let patch_sender_clone = patch_sender.clone();
        tokio::spawn(async move {
            Self::command_task(
                command_receiver,
                command_status_sender,
                home_clone,
                store_clone,
                kernels_clone,
                patch_sender_clone,
            )
            .await
        });

        Ok(Self {
            id,
            home,
            path,
            kernels,
            store,
            watch_receiver,
            update_sender,
            patch_sender,
            command_counter,
            command_sender,
            command_status_receiver,
        })
    }

    /// Create a new in-memory document
    pub fn new(r#type: DocumentType) -> Result<Self> {
        let root = r#type.empty();

        let mut store = WriteStore::new();
        root.dump(&mut store)?;

        let home = std::env::current_dir()?;

        Self::init(store, home, None)
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
        r#type: DocumentType,
        path: Option<&Path>,
        overwrite: bool,
        source: Option<&Path>,
        format: Option<Format>,
        codec: Option<String>,
    ) -> Result<Self> {
        let path = path.map_or_else(|| r#type.main(), PathBuf::from);

        if path.exists() && !overwrite {
            bail!("Path already exists; remove the file or use the `--overwrite` option")
        }

        let (root, message) = if let Some(source) = source {
            let decode_options = Some(DecodeOptions {
                format,
                codec,
                ..Default::default()
            });
            let filename = source
                .file_name()
                .map_or_else(|| "unnamed", |name| name.to_str().unwrap_or_default());
            (
                codecs::from_path(source, decode_options).await?,
                format!("Initial commit of {type} imported from `{filename}`"),
            )
        } else {
            (r#type.empty(), format!("Initial commit of empty {type}"))
        };

        let mut store = WriteStore::new();
        root.write(&mut store, &path, &message).await?;

        let home = path
            .parent()
            .ok_or_else(|| eyre!("path has no parent; is it a file?"))?
            .to_path_buf();

        Self::init(store, home, Some(path))
    }

    /// Open an existing document
    ///
    /// If the path is a store the loads from that store, otherwise
    /// uses `codec` to import from the path and dump into a new store.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        let home = path
            .parent()
            .ok_or_else(|| eyre!("path has no parent; is it a file?"))?
            .to_path_buf();

        let format = Format::from_path(path)?;
        if format.is_store() {
            let store = load_store(path).await?;
            Self::init(store, home, Some(path.to_path_buf()))
        } else {
            let root = codecs::from_path(path, None).await?;
            let mut store = WriteStore::new();
            root.dump(&mut store)?;
            Self::init(store, home, None)
        }
    }

    /// Get the id of the document
    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    /// Load the root [`Node`] from the document's Automerge store
    async fn load(&self) -> Result<Node> {
        let store = self.store.read().await;
        Node::load(&*store)
    }

    /// Dump a [`Node`] to the root of the document's Automerge store
    async fn dump(&self, node: &Node) -> Result<()> {
        let mut store = self.store.write().await;
        node.dump(&mut store)
    }

    /// Inspect a document
    ///
    /// Loads the Automerge store at the `path` (without attempting to load as a `Node`)
    /// and returns a JSON representation of the contents of the store. This is mainly useful
    /// during development to debug issues with loading a node from a store because it allows
    /// us to inspect the "raw" structure in the store.
    #[tracing::instrument]
    pub async fn inspect(path: &Path) -> Result<String> {
        let store = load_store(path).await?;
        inspect_store(&store)
    }

    /// Import a file into a new, or existing, document
    ///
    /// By default the format of the `source` file is inferred from its extension but
    /// this can be overridden by providing the `format` option.
    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        source: &Path,
        options: Option<DecodeOptions>,
        r#type: Option<DocumentType>,
    ) -> Result<()> {
        let root = codecs::from_path(source, options).await?;

        if let Some(expected_type) = r#type {
            let actual_type = DocumentType::from_node(&root)?;
            if expected_type == actual_type {
                bail!(
                    "The imported document is of type `{actual_type}` but expected type `{expected_type}`"
                )
            }
        }

        let mut store = self.store.write().await;

        let filename = source
            .file_name()
            .map_or_else(|| "unnamed", |name| name.to_str().unwrap_or_default());

        if let Some(path) = &self.path {
            root.write(&mut store, path, &format!("Import from `{filename}`"))
                .await?;
        } else {
            root.dump(&mut store)?;
        }

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
        let root = self.load().await?;

        if let Some(dest) = dest {
            let mut store = self.store.write().await;

            if let Some(path) = &self.path {
                let _commit = root
                    .write(&mut store, path, &format!("Export to `{}`", dest.display()))
                    .await?;
            }

            codecs::to_path(&root, dest, options).await?;

            Ok(String::new())
        } else {
            codecs::to_string(&root, options).await
        }
    }

    /// Get the history of commits to the document
    #[tracing::instrument(skip(self))]
    pub async fn log(&self) -> Result<Vec<LogEntry>> {
        let mut store = self.store.write().await;

        let changes = store.get_changes(&[]);

        let entries = changes
            .iter()
            .map(|change| {
                let hash = change.hash().to_string();
                let parents = change
                    .deps()
                    .iter()
                    .map(|hash| hash.to_string())
                    .collect_vec();
                let timestamp = change.timestamp();
                let author = change.actor_id().to_hex_string();
                let message = change.message().cloned().unwrap_or_default();

                LogEntry {
                    hash,
                    parents,
                    timestamp,
                    author,
                    message,
                }
            })
            .collect_vec();

        Ok(entries)
    }

    /// Perform a command on the document and optionally wait for it to complete
    #[tracing::instrument(skip(self))]
    pub async fn command(&self, command: Command, wait: bool) -> Result<()> {
        tracing::trace!("Performing document command");

        // If not waiting then just send the command and return
        if !wait {
            self.command_sender.send((command, 0)).await?;

            return Ok(());
        }

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
        sleep(Duration::from_millis(50)).await;

        Ok(())
    }

    /// Compile the document
    #[tracing::instrument(skip(self))]
    pub async fn compile(&self, wait: bool) -> Result<()> {
        tracing::trace!("Compiling document");
        self.command(Command::CompileDocument, wait).await
    }

    /// Execute the document
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, options: ExecuteOptions, wait: bool) -> Result<()> {
        tracing::trace!("Executing document");
        self.command(Command::ExecuteDocument(options), wait).await
    }
}
