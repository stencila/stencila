use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    itertools::Itertools,
    serde::Serialize,
    strum::{Display, EnumString},
    tokio::{
        self,
        sync::{mpsc, watch, RwLock},
    },
    tracing,
    type_safe_id::{StaticType, TypeSafeId},
};
use format::Format;
use kernels::Kernels;
use node_store::{inspect_store, load_store, ReadNode, WriteNode, WriteStore};
use schema::{Article, Node, NodeId};

mod sync_directory;
mod sync_file;
mod sync_format;
mod sync_nodes;
mod sync_object;

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

type DocumentKernels = Arc<RwLock<Kernels>>;

type DocumentStore = Arc<RwLock<WriteStore>>;

type DocumentWatchSender = watch::Sender<Node>;
type DocumentWatchReceiver = watch::Receiver<Node>;

type DocumentUpdateSender = mpsc::Sender<Node>;
type DocumentUpdateReceiver = mpsc::Receiver<Node>;

/// A document
///
/// Each document has:
///
/// - A unique `id` which is used for things like establishing a
///   WebSocket connection to it
///
/// - An Automerge `store` that has a [`Node`] at its root.
///
/// - An optional `path` that the `store` can be read from, and written to.
///
/// - A `watch_receiver` which can be cloned to watch for changes
///   to the root [`Node`].
///
/// - An `update_sender` which can be cloned to send updates to the
///   root [`Node`].
#[derive(Debug)]
pub struct Document {
    /// The document's id
    id: DocumentId,

    /// The document's execution kernels
    kernels: DocumentKernels,

    /// The document's Automerge store with a [`Node`] at its root
    store: DocumentStore,

    /// The filesystem path to the document's Automerge store
    path: Option<PathBuf>,

    /// A channel receiver for watching for changes to the root [`Node`]
    watch_receiver: DocumentWatchReceiver,

    /// A channel sender for sending updates to the root [`Node`]
    update_sender: DocumentUpdateSender,
}

impl Document {
    /// Initialize a new document
    ///
    /// This initializes the document's "watch" and "update" channels and starts the
    /// `update_task` to respond to incoming updates to the root node of the document.
    #[tracing::instrument(skip(store))]
    fn init(store: WriteStore, path: Option<PathBuf>) -> Result<Self> {
        let id = DocumentId::new();

        let kernels = DocumentKernels::default();

        let node = Node::load(&store)?;

        let (watch_sender, watch_receiver) = watch::channel(node);
        let (update_sender, update_receiver) = mpsc::channel(8);

        let store = Arc::new(RwLock::new(store));
        let store_clone = store.clone();
        tokio::spawn(
            async move { Self::update_task(store_clone, update_receiver, watch_sender).await },
        );

        Ok(Self {
            id,
            kernels,
            store,
            path,
            watch_receiver,
            update_sender,
        })
    }

    /// Crete a new in-memory document
    pub fn new(r#type: DocumentType) -> Result<Self> {
        let root = r#type.empty();

        let mut store = WriteStore::new();
        root.dump(&mut store)?;

        Self::init(store, None)
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

        Self::init(store, Some(path))
    }

    /// Open an existing document
    ///
    /// If the path is a store the loads from that store, otherwise
    /// uses `codec` to import from the path and dump into a new store.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        let format = Format::from_path(path)?;
        if format.is_store() {
            let store = load_store(path).await?;
            Self::init(store, Some(path.to_path_buf()))
        } else {
            let root = codecs::from_path(path, None).await?;
            let mut store = WriteStore::new();
            root.dump(&mut store)?;
            Self::init(store, None)
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

    /// Async task to update the document's store and notify watchers of the update
    async fn update_task(
        store: DocumentStore,
        mut update_receiver: DocumentUpdateReceiver,
        watch_sender: DocumentWatchSender,
    ) {
        tracing::debug!("Document update task started");

        while let Some(node) = update_receiver.recv().await {
            tracing::trace!("Document node updated");

            // Dump the node to the store
            let mut store = store.write().await;
            if let Err(error) = node.dump(&mut store) {
                tracing::error!("While dumping node to store: {error}");
            }

            // Load the node from the store. This is necessary, rather than just
            // sending watchers the incoming node, because the incoming node
            // may be partial (e.g. may be missing `id` or other fields) but watchers
            // need complete nodes (e.g `id` for HTML)
            let node = match Node::load(&*store) {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While loading node from store: {error}");
                    continue;
                }
            };

            // Send the node to watchers
            if let Err(error) = watch_sender.send(node) {
                tracing::error!("While notifying watchers: {error}");
            }
        }

        tracing::debug!("Document update task stopped");
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

    /// Execute the document
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, node_id: Option<NodeId>) -> Result<()> {
        tracing::trace!("Executing document");

        let mut root = self.load().await?;
        let mut kernels = self.kernels.write().await;

        node_execute::execute(&mut root, &mut kernels, node_id).await?;

        self.dump(&root).await?;

        Ok(())
    }
}
