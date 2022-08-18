use std::{collections::HashMap, path::Path, sync::Arc};

use common::{
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    tokio::sync::Mutex,
};
use path_utils::pathdiff;

use crate::document::{Document, DocumentHandler};

/// An in-memory store of documents
#[derive(Debug, Default)]
pub struct Documents {
    /// A mapping of file paths to open documents
    registry: Mutex<HashMap<String, DocumentHandler>>,
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
        for document in self.registry.lock().await.values() {
            let path = &document.document.lock().await.path;
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
    pub async fn create<P: AsRef<Path>>(
        &self,
        path: Option<P>,
        content: Option<String>,
        format: Option<String>,
    ) -> Result<String> {
        let document = Document::create(path, content, format).await?;
        let document_id = document.id.clone();
        let handler = DocumentHandler::new(document, false);
        self.registry
            .lock()
            .await
            .insert(document_id.clone(), handler);

        Ok(document_id)
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
    pub async fn open<P: AsRef<Path>>(&self, path: P, format: Option<String>) -> Result<String> {
        let path = Path::new(path.as_ref()).canonicalize()?;

        for handler in self.registry.lock().await.values() {
            let document = handler.document.lock().await;
            if document.path == path {
                return Ok(document.id.clone());
            }
        }

        let document = Document::open(path, format).await?;
        let document_id = document.id.clone();
        let handler = DocumentHandler::new(document, true);
        self.registry
            .lock()
            .await
            .insert(document_id.clone(), handler);

        Ok(document_id)
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

        if self.registry.lock().await.contains_key(&id_or_path_string) {
            id_to_remove = id_or_path_string
        } else {
            let path = id_or_path_path.canonicalize()?;
            for handler in self.registry.lock().await.values() {
                let document = handler.document.lock().await;
                if document.path == path {
                    id_to_remove = document.id.clone();
                    break;
                }
            }
        };
        self.registry.lock().await.remove(&id_to_remove);

        Ok(id_to_remove)
    }

    /// Subscribe a client to a topic for a document
    pub async fn subscribe(&self, id: &str, topic: &str, client: &str) -> Result<String> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        let topic = document_guard.subscribe(topic, client);
        Ok(topic)
    }

    /// Unsubscribe a client from a topic for a document
    pub async fn unsubscribe(&self, id: &str, topic: &str, client: &str) -> Result<String> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        let topic = document_guard.unsubscribe(topic, client);
        Ok(topic)
    }

    /// Get a document that has previously been opened
    pub async fn get(&self, id: &str) -> Result<Arc<Mutex<Document>>> {
        if let Some(handler) = self.registry.lock().await.get(id) {
            Ok(handler.document.clone())
        } else {
            bail!("No document with id {}", id)
        }
    }
}

/// The global documents store
pub static DOCUMENTS: Lazy<Documents> = Lazy::new(Documents::new);
