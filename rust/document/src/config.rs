use common::eyre::Result;
use schema::{Config, Node};

use crate::Document;

impl Document {
    /// Get a resolved [`Config`] for the document
    pub async fn config(&self) -> Result<Config> {
        // TODO: walk up from the document to find any ancestor
        // configuration files
        // See https://github.com/stencila/stencila/issues/2387
        let root = &*self.root.read().await;
        match root {
            Node::Article(article) => Ok(article.config.clone().unwrap_or_default()),
            _ => Ok(Config::default()),
        }
    }
}
