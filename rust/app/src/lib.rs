//! Application level (e.g. Stencila CLI or Stencila Desktop) config and directories

use std::{fs::create_dir_all, path::PathBuf};

use directories::ProjectDirs;

use common::eyre::{OptionExt, Result};

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "stencila", "stencila").ok_or_eyre("unable to build project dirs")
}

/// Get the cache directory for the app
pub fn cache_dir(ensure: bool) -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let dir = dirs.cache_dir();
    if ensure && !dir.exists() {
        create_dir_all(dir)?;
    }
    Ok(dir.to_path_buf())
}
