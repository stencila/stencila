use std::path::{Path, PathBuf};

use common::{
    eyre::Result,
    serde_yaml,
    tokio::fs::{read_to_string, write},
};
use schema::{Article, Config, Node};

use crate::{
    dirs::{closest_stencila_dir, STENCILA_DIR},
    Document,
};

const CONFIG_FILE: &str = "config.yaml";

/// Get the path of the Stencila config file for a workspace directory
pub async fn config_file(workspace_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let config_file = workspace_dir.join(STENCILA_DIR).join(CONFIG_FILE);

    if ensure && !config_file.exists() {
        write(&config_file, "\n").await?;
    }

    Ok(config_file)
}

/// Get the path of the closest Stencila config file for a path
///
/// Unless `ensure` is true, the returned path may not exist
pub async fn closest_config_file(path: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = closest_stencila_dir(path, ensure).await?;
    let config_file = stencila_dir.join(CONFIG_FILE);

    if ensure && !config_file.exists() {
        write(&config_file, "\n").await?;
    }

    Ok(config_file)
}

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

impl Document {
    /// Get a resolved [`Config`] for the document
    pub async fn config_with_sources(&self) -> Result<(Config, Vec<PathBuf>)> {
        // Check for document level config
        let root = &*self.root.read().await;
        if let Node::Article(Article {
            config: Some(config),
            ..
        }) = root
        {
            let path = self
                .path
                .clone()
                .unwrap_or_else(|| PathBuf::from("Unsaved document"));
            return Ok((config.clone(), vec![path]));
        };

        // Check for a workspace config file
        if let Some(path) = self.path() {
            if let Some((config, path)) = closest_config(path).await? {
                return Ok((config, vec![path]));
            }
        }

        Ok((Config::default(), Vec::new()))
    }

    /// Get a resolved [`Config`] for the document
    pub async fn config(&self) -> Result<Config> {
        Ok(self.config_with_sources().await?.0)
    }
}
