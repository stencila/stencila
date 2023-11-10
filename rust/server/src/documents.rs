use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{eyre::Result, tokio::sync::RwLock};
use document::{Document, SyncDirection};

/// A cache of documents
#[derive(Default)]
pub(crate) struct Documents {
    map: RwLock<HashMap<PathBuf, Arc<Document>>>,
}

impl Documents {
    /// Get a document from the cache
    pub async fn get(&self, path: &Path, sync: Option<SyncDirection>) -> Result<Arc<Document>> {
        let map = self.map.read().await;
        if let Some(doc) = map.get(path) {
            return Ok(doc.clone());
        }
        drop(map);

        let doc = Document::open(&path).await?;

        if let Some(direction) = sync {
            doc.sync_file(path, direction, None, None).await?;
        }

        let mut map = self.map.write().await;
        map.insert(path.to_path_buf(), Arc::new(doc));

        let doc = map
            .get(path)
            .expect("Should be present because just inserted")
            .clone();

        Ok(doc)
    }
}
