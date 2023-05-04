use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    strum::{Display, EnumString},
    tokio::{
        self,
        sync::{mpsc, watch, RwLock},
    },
    tracing,
};
use format::Format;
use node_store::{inspect_store, load_store, Read, Write, WriteStore};
use schema::{Article, Node};

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
    /// Returns `None` is the format can be associated with more than one document
    /// type (e.g. [`Format::Json`]).
    fn from_format(format: &Format) -> Option<Self> {
        match format {
            Format::Jats | Format::Md => Some(Self::Article),
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

/// The synchronization mode between documents and external resources
///
/// Examples of external resources which may be synchonized with a document include
/// a file on the local file system or an editor in a web browser. This enum determines
/// whether changes in the document chould be reflected in the resource and vice versa.
#[derive(Debug, Display, Default, Clone, Copy, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum SyncDirection {
    In,
    Out,
    #[default]
    InOut,
}

type DocumentStore = Arc<RwLock<WriteStore>>;

type DocumentWatchSender = watch::Sender<Node>;
type DocumentWatchReceiver = watch::Receiver<Node>;

type DocumentUpdateSender = mpsc::Sender<Node>;
type DocumentUpdateReceiver = mpsc::Receiver<Node>;

/// A document
///
/// Each document has:
///
/// - An Automerge `store` that has a [`Node`] at its root.
/// The `store` is read from, and written to, the document's `path`.
///
/// - A `watch_receiver` which can be cloned to watch
/// for changes to the root [`Node`].
///
/// - An `update_sender` which can be cloned to send updates to the
/// root [`Node`].
pub struct Document {
    /// The document's Automerge store with a [`Node`] to this root
    store: DocumentStore,

    /// The filesystem path to the document's Automerge store
    path: PathBuf,

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
    fn init(path: &Path, store: WriteStore) -> Result<Self> {
        let path = path.canonicalize()?;

        let node = Node::load(&store)?;

        let (watch_sender, watch_receiver) = watch::channel(node);
        let (update_sender, update_receiver) = mpsc::channel(8);

        let store = Arc::new(RwLock::new(store));
        let store_clone = store.clone();
        tokio::spawn(
            async move { Self::update_task(store_clone, update_receiver, watch_sender).await },
        );

        Ok(Self {
            path,
            store,
            watch_receiver,
            update_sender,
        })
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
    pub async fn new(
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
                format!("Initial commit of imported {type} from `{filename}`"),
            )
        } else {
            (r#type.empty(), format!("Initial commit of empty {type}"))
        };

        let mut store = WriteStore::new();
        root.write(&mut store, &path, &message).await?;

        Self::init(&path, store)
    }

    /// Open an existing document
    ///
    /// Opens the document from the Automerge store at `path` erroring if the path does not exist
    /// or is a directory.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        let store = load_store(path).await?;
        Self::init(path, store)
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
            tracing::trace!("Document node updated, dumping to store");

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

        root.write(&mut store, &self.path, &format!("Import from `{filename}`"))
            .await?;

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
            codecs::to_path(&root, dest, options).await?;
            Ok(String::new())
        } else {
            codecs::to_string(&root, options).await
        }
    }

    /// Get the history of commits to the document
    #[tracing::instrument(skip(self))]
    pub async fn history(&self) -> Result<()> {
        let mut store = self.store.write().await;

        let changes = store.get_changes(&[])?;

        for change in changes {
            let hash = change.hash();
            let prev = change.deps();
            let timestamp = change.timestamp();
            let actor = change.actor_id();
            let message = change.message().cloned().unwrap_or_default();

            println!("{hash} {prev:?} {timestamp} {actor} {message}\n")
        }

        Ok(())
    }
}

mod sync_file;
