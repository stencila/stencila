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

    /// A config subdirectory for `.css` files defining Stencila themes
    /// which can be used across workspaces on the current machine
    Themes,

    Prompts,
    
    Kernels,

    Cache,
    Templates,
    Models,

    /// A cache subdirectory for `.csl` files downloaded from
    /// https://raw.githubusercontent.com/citation-style-language/styles
    Csl,
}

/// Get an application directory
pub fn get_app_dir(dir_type: DirType, mut ensure: bool) -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "stencila", "stencila")
        .ok_or_eyre("unable to build project dirs")?;

    let dir = {
        match dir_type {
            DirType::Config => dirs.config_dir().to_path_buf(),
            DirType::Themes => dirs.config_dir().join("themes"),

            DirType::Prompts => {
                if let Ok(dir) = env::var("STENCILA_PROMPTS_DIR") {
                    ensure = false;
                    PathBuf::from(dir)
                } else {
                    dirs.config_dir().join("prompts")
                }
            }

            DirType::Kernels => dirs.config_dir().join("kernels"),

            DirType::Cache => dirs.cache_dir().to_path_buf(),
            DirType::Templates => dirs.cache_dir().join("templates"),
            DirType::Models => dirs.cache_dir().join("models"),
            DirType::Csl => dirs.cache_dir().join("csl"),
        }
    };

    if ensure && !dir.exists() {
        create_dir_all(dir.clone())?;
    }

    Ok(dir)
}
