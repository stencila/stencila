use std::path::{Path, PathBuf};

use codec_json::{FromJson, ToJson};

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    strum::{Display, EnumString},
    tokio::{fs::read_to_string, sync::Mutex},
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
    /// Derive the document type from the type of a `Node`
    fn derive(node: &Node) -> Result<Self> {
        match node {
            Node::Article(..) => Ok(Self::Article),
            _ => bail!("Node of type `{}` can not be a document type", node),
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

    /// Get an empty root node for the document type
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
    store: Mutex<WriteStore>,

    /// The root node of the document
    root: Node,
}

impl Document {
    /// Initialize a new document
    #[tracing::instrument]
    pub async fn init(r#type: Type, path: Option<&Path>, overwrite: bool) -> Result<Self> {
        let root = r#type.empty();

        let path = path.map_or_else(
            || PathBuf::from(format!("main.{}", r#type.extension())),
            PathBuf::from,
        );

        if path.exists() && !overwrite {
            bail!("Path already exists; remove the file or use the `--overwrite` option")
        }

        let mut store = WriteStore::new();
        root.write(
            &mut store,
            &path,
            &format!("Initial commit of empty {}", r#type),
        )
        .await?;

        let store = Mutex::new(store);

        Ok(Self { path, store, root })
    }

    /// Open a document
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
        let store = Mutex::new(store);

        Ok(Self { path, store, root })
    }

    /// Inspect a document
    ///
    /// Loads the store at the `path` (without attempting to load as a `Node`)
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

    /// Import a file in another format into a new, or existing, document
    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        source: &Path,
        format: Option<Format>,
        r#type: Option<Type>,
    ) -> Result<()> {
        if !source.exists() {
            bail!("Path `{}` does not exist", source.display());
        }

        // TODO: Use format to select codec
        let json = read_to_string(source).await?;

        let root = match r#type {
            Some(r#type) => match r#type {
                Type::Article => Node::Article(Article::from_json(&json)?),
            },
            // TODO: Infer type from format / peek type in source
            None => Node::Article(Article::from_json(&json)?),
        };

        let mut store = self.store.lock().await;

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
    pub async fn export(&self, dest: Option<&Path>, format: Option<Format>) -> Result<()> {
        let format = match format {
            Some(format) => format,
            None => match dest {
                Some(path) => Format::from_path(path)?,
                None => Format::Json,
            },
        };

        let content = match format {
            Format::Json => self.root.to_json()?,
            _ => todo!(),
        };

        println!("{}", content);

        Ok(())
    }

    /// Get the history of commits to the document
    #[tracing::instrument(skip(self))]
    pub async fn history(&self) -> Result<()> {
        let mut store = self.store.lock().await;

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
