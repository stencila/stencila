use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use app::{get_app_dir, DirType};
use common::{
    eyre::Result,
    serde_yaml,
    tokio::{
        fs::{read_to_string, write},
        sync::RwLock,
    },
};
use schema::{Article, Config, Node};

use crate::{
    dirs::{closest_config_file, CONFIG_FILE},
    Document,
};

/// Read in the closest config file
///
/// If no config file exists for the path will return `None`.
pub async fn closest_config(path: &Path) -> Result<Option<(Config, PathBuf)>> {
    let config_file = closest_config_file(path, false).await?;

    if !config_file.exists() {
        return Ok(None);
    }

    let yaml = read_to_string(&config_file).await?;
    let config = serde_yaml::from_str(&yaml)?;

    Ok(Some((config, config_file)))
}

/// Get the path of the user config file for a path
///
/// Unless `ensure` is true, the returned path may not exist
pub async fn user_config_file(ensure: bool) -> Result<PathBuf> {
    let config_dir = get_app_dir(DirType::Config, ensure)?;
    let config_file = config_dir.join(CONFIG_FILE);

    if ensure && !config_file.exists() {
        write(&config_file, "\n").await?;
    }

    Ok(config_file)
}

/// Read in the user config file
///
/// If no config file exists for the path will return `None`.
pub async fn user_config() -> Result<Option<(Config, PathBuf)>> {
    let config_file = user_config_file(false).await?;

    if !config_file.exists() {
        return Ok(None);
    }

    let yaml = read_to_string(&config_file).await?;
    let config = serde_yaml::from_str(&yaml)?;

    Ok(Some((config, config_file)))
}

impl Document {
    /// Get the resolved [`Config`] for the document (with sources) from its root and path
    pub async fn config_for(
        root: &Arc<RwLock<Node>>,
        path: &Option<PathBuf>,
    ) -> Result<(Config, Vec<PathBuf>)> {
        // Check for document level config
        let root = &*root.read().await;
        if let Node::Article(Article {
            config: Some(config),
            ..
        }) = root
        {
            let path = path
                .clone()
                .unwrap_or_else(|| PathBuf::from("Unsaved document"));
            return Ok((config.clone(), vec![path]));
        };

        // Check for a workspace config file
        if let Some(path) = path {
            if let Some((config, path)) = closest_config(path).await? {
                return Ok((config, vec![path]));
            }
        }

        // Check for a user config file
        if let Some((config, path)) = user_config().await? {
            return Ok((config, vec![path]));
        }

        Ok((Config::default(), Vec::new()))
    }

    /// Merge config from the root of a document into an existing config
    pub async fn config_merge_root(config: Config, root: &Arc<RwLock<Node>>) -> Config {
        // TODO: make this a proper merge. Currently, it just uses the config from the doc
        if let Node::Article(Article {
            config: Some(config),
            ..
        }) = &*root.read().await
        {
            config.clone()
        } else {
            config
        }
    }

    /// Get a resolved [`Config`] for the document with sources
    pub async fn config_with_sources(&self) -> Result<(Config, Vec<PathBuf>)> {
        Document::config_for(&self.root, &self.path).await
    }

    /// Get a resolved [`Config`] for the document
    pub async fn config(&self) -> Result<Config> {
        Ok(self.config_with_sources().await?.0)
    }
}
