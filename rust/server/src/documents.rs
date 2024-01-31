use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    eyre::{eyre, Result},
    tokio::sync::RwLock,
    uuid::Uuid,
};
use document::{Document, DocumentId, SyncDirection};

/// A store of documents
#[derive(Debug, Default)]
pub(crate) struct Documents {
    /// A mapping between a file system path and the id of the trunk
    /// [`Document`] instance for that path
    paths: RwLock<HashMap<PathBuf, Uuid>>,

    /// A mapping of document ids to [`Document`]s
    docs: RwLock<HashMap<Uuid, Arc<Document>>>,
}

impl Documents {
    /// Get a document by path
    ///
    /// At present this always returns the trunk document for the path.
    /// In the future, based on arguments and/or the user's permissions on the
    /// document, will return a branch or a twig document.
    pub async fn by_path(&self, path: &Path, sync: Option<SyncDirection>) -> Result<Arc<Document>> {
        {
            // In block to ensure lock is dropped when no longer needed
            let paths = self.paths.read().await;
            if let Some(uuid) = paths.get(path) {
                return self.by_uuid(uuid).await;
            }
        }

        let doc = Document::open(path).await?;

        if let Some(direction) = sync {
            doc.sync_file(path, direction, None, None).await?;
        }

        let uuid = doc.id().uuid();

        self.paths.write().await.insert(path.to_path_buf(), uuid);
        self.docs.write().await.insert(uuid, Arc::new(doc));

        self.by_uuid(&uuid).await
    }

    /// Get a document by [`DocumentId`]
    pub async fn by_id(&self, id: &DocumentId) -> Result<Arc<Document>> {
        self.by_uuid(&id.uuid()).await
    }

    /// Get a document by [`Uuid`]
    pub async fn by_uuid(&self, uuid: &Uuid) -> Result<Arc<Document>> {
        let doc = self
            .docs
            .read()
            .await
            .get(uuid)
            .ok_or_else(|| eyre!("No doc with UUID `{uuid}`"))?
            .clone();

        Ok(doc)
    }
}
