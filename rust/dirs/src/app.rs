//! Application level (e.g. Stencila CLI or Stencila Desktop) config and directories

use std::{env, fs::create_dir_all, path::PathBuf};

use clap::ValueEnum;
use directories::ProjectDirs;
use eyre::{OptionExt, Result};
use strum::{Display, EnumString};

#[derive(Debug, Display, Default, Clone, Copy, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DirType {
    #[default]
    Config,
    Cache,
    Prompts,
    Kernels,
    Templates,
    Models,
}

/// Get an application directory
pub fn get_app_dir(dir_type: DirType, mut ensure: bool) -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "stencila", "stencila")
        .ok_or_eyre("unable to build project dirs")?;

    let dir = {
        match dir_type {
            DirType::Config => dirs.config_dir().to_path_buf(),
            DirType::Cache => dirs.cache_dir().to_path_buf(),
            DirType::Prompts => {
                if let Ok(dir) = env::var("STENCILA_PROMPTS_DIR") {
                    ensure = false;
                    PathBuf::from(dir)
                } else {
                    dirs.config_dir().join("prompts")
                }
            }
            DirType::Kernels => dirs.config_dir().join("kernels"),
            DirType::Templates => dirs.cache_dir().join("templates"),
            DirType::Models => dirs.cache_dir().join("models"),
        }
    };

    if ensure && !dir.exists() {
        create_dir_all(dir.clone())?;
    }

    Ok(dir)
}
