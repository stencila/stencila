use std::path::{Path, PathBuf};

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    strum::{Display, EnumString},
    tokio::sync::RwLock,
    tracing,
};
use format::Format;
use node_store::{inspect_store, load_store, Read, Write, WriteStore};
use schema::{Article, Node};

/// A document type
///
/// Defines which `CreativeWork` variants can be the root node of a document
/// and the default file extension etc for each variant.
#[derive(Debug, Display, Clone, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum Type {
    Article,
}

impl Type {
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

    /// Get the default main file name for the document type
    fn main(&self) -> PathBuf {
        PathBuf::from(format!("main.{}", self.extension()))
    }

    /// Get an empty root [`Node`] for the document type
    fn empty(&self) -> Node {
        match self {
            Type::Article => Node::Article(Article::default()),
        }
    }
}

/// A document
pub struct Document {
    /// The path to the document Automerge file
    path: PathBuf,

    /// The document Automerge store
    store: RwLock<WriteStore>,

    /// The root node of the document
    root: RwLock<Node>,
}

impl Document {
    /// Initialize a new document
    ///
    /// Creates a new Automerge store of `type` at the `path`, optionally overwriting any
    /// existing file at the path.
    #[tracing::instrument]
    pub async fn init(r#type: Type, path: Option<&Path>, overwrite: bool) -> Result<Self> {
        let path = path.map_or_else(
            || PathBuf::from(format!("main.{}", r#type.extension())),
            PathBuf::from,
        );

        if path.exists() && !overwrite {
            bail!("Path already exists; remove the file or use the `--overwrite` option")
        }

        let mut store = WriteStore::new();
        let root = r#type.empty();
        root.write(
            &mut store,
            &path,
            &format!("Initial commit of empty {}", r#type),
        )
        .await?;

        let store = RwLock::new(store);
        let root = RwLock::new(root);

        Ok(Self { path, store, root })
    }

    /// Open an existing document
    ///
    /// Opens the document from the Automerge store at `path` erroring if the path does not exist
    /// or is a directory.
    #[tracing::instrument]
    pub async fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }
        if path.is_dir() {
            bail!("Path `{}` is a directory; expected a file", path.display());
        }
        let path = path.canonicalize()?;

        let (store, root) = Node::read(&path).await?;

        let store = RwLock::new(store);
        let root = RwLock::new(root);

        Ok(Self { path, store, root })
    }

    /// Inspect a document
    ///
    /// Loads the Automerge store at the `path` (without attempting to load as a `Node`)
    /// and returns a JSON representation of the contents of the store.
    #[tracing::instrument]
    pub async fn inspect(path: &Path) -> Result<String> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }
        if path.is_dir() {
            bail!("Path `{}` is a directory; expected a file", path.display());
        }

        let store = load_store(path).await?;
        inspect_store(&store)
    }

    /// Import a file into a new, or existing, document
    ///
    /// By default the format of the source file is inferred from its extension but
    /// this can be overridden by providing the `format` option.
    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        source: &Path,
        format: Option<Format>,
        r#type: Option<Type>,
    ) -> Result<()> {
        let root = codecs::from_path(source, format, None).await?;

        // TODO assert type

        let mut store = self.store.write().await;

        let filename = source
            .file_name()
            .map_or_else(|| "unnamed", |name| name.to_str().unwrap_or_default());

        root.write(
            &mut store,
            &self.path,
            &format!("Import from `{}`", filename),
        )
        .await?;

        Ok(())
    }

    /// Export a document to a file in another format
    #[tracing::instrument(skip(self))]
    pub async fn export(&self, dest: Option<&Path>, format: Option<Format>) -> Result<String> {
        let root = self.root.read().await;

        if let Some(dest) = dest {
            codecs::to_path(&root, dest, format, None).await?;
            Ok(String::new())
        } else {
            let format = format.unwrap_or(Format::Json);
            codecs::to_string(&root, format, None).await
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
