use std::{collections::HashMap, path::Path, sync::Arc};

use common::{
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    tokio::sync::RwLock,
};
use path_utils::pathdiff;

use crate::document::Document;

/// The global documents store
pub static DOCUMENTS: Lazy<Documents> = Lazy::new(Documents::new);

/// An in-memory store of documents
#[derive(Debug, Default)]
pub struct Documents {
    /// A mapping of file paths to open documents
    registry: RwLock<HashMap<String, Arc<Document>>>,
}

impl Documents {
    /// Create a new documents store
    pub fn new() -> Self {
        Self::default()
    }

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub async fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for doc in self.registry.read().await.values() {
            let path = &doc.path;
            let path = match pathdiff::diff_paths(path, &cwd) {
                Some(path) => path,
                None => path.clone(),
            };
            let path = path.display().to_string();
            paths.push(path);
        }
        Ok(paths)
    }

    /// Create a new document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the new document
    /// - `content`: Content for the new document
    /// - `format`: The format of the content
    pub async fn create<P: AsRef<Path>>(
        &self,
        path: Option<P>,
        content: Option<String>,
        format: Option<String>,
    ) -> Result<Arc<Document>> {
        let doc = Document::create(path, content, format).await?;
        let doc_id = doc.id.clone();
        let doc = Arc::new(doc);

        self.registry
            .write()
            .await
            .insert(doc_id.clone(), doc.clone());

        Ok(doc)
    }

    /// Open a document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the document to open
    /// - `format`: The format to open the document as (inferred from filename extension if not supplied)
    ///
    /// If the document has already been opened, it will not be re-opened, but rather the existing
    /// in-memory instance will be returned.
    pub async fn open<P: AsRef<Path>>(
        &self,
        path: P,
        format: Option<String>,
    ) -> Result<Arc<Document>> {
        let path = Path::new(path.as_ref()).canonicalize()?;

        for doc in self.registry.read().await.values() {
            if doc.path == path {
                return Ok(doc.clone());
            }
        }

        let doc = Document::open(path, format).await?;
        let doc_id = doc.id.clone();
        let doc = Arc::new(doc);

        self.registry
            .write()
            .await
            .insert(doc_id.clone(), doc.clone());

        Ok(doc)
    }

    /// Close a document
    ///
    /// # Arguments
    ///
    /// - `id_or_path`: The id or path of the document to close
    ///
    /// If `id_or_path` matches an existing document `id` then that document will
    /// be closed. Otherwise a search will be done and the first document with a matching
    /// path will be closed.
    pub async fn close<P: AsRef<Path>>(&self, id_or_path: P) -> Result<String> {
        let id_or_path_path = id_or_path.as_ref();
        let id_or_path_string = id_or_path_path.to_string_lossy().to_string();
        let mut id_to_remove = String::new();

        let mut registry = self.registry.write().await;
        if registry.contains_key(&id_or_path_string) {
            id_to_remove = id_or_path_string
        } else {
            let path = id_or_path_path.canonicalize()?;
            for doc in registry.values() {
                if doc.path == path {
                    id_to_remove = doc.id.clone();
                    break;
                }
            }
        };
        registry.remove(&id_to_remove);

        Ok(id_to_remove)
    }

    /// Get a document that has previously been opened by its id
    pub async fn get(&self, id: &str) -> Result<Arc<Document>> {
        if let Some(doc) = self.registry.read().await.get(id) {
            Ok(doc.clone())
        } else {
            bail!("No document with id {}", id)
        }
    }
}
