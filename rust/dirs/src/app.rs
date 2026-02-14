//! Application level (e.g. Stencila CLI or Stencila Desktop) config and directories

use std::{env, fs::create_dir_all, path::PathBuf};

use clap::ValueEnum;
use directories::ProjectDirs;
use eyre::{OptionExt, Result};
use strum::{Display, EnumString};

#[derive(Debug, Display, Clone, Copy, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DirType {
    // Application configuration
    Config,

    /// Config subdirectory for `.css` files defining Stencila themes
    /// which can be used across workspaces on the current machine
    Themes,

    Prompts,

    /// Application cache
    Cache,

    /// Cache subdirectory for `.csl` files downloaded from
    /// https://raw.githubusercontent.com/citation-style-language/styles
    Csl,

    /// Cache subdirectory for downloaded fonts
    Fonts,

    /// Cache subdirectory for microkernel scripts
    Kernels,

    /// Cache subdirectory for embedding, and other, models
    Models,

    /// Cache subdirectory for template files (e.g. for Pandoc)
    Templates,

    /// Runtime subdirectory for server info files
    Servers,

    /// Config subdirectory for user-level agent definitions
    Agents,
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

            DirType::Cache => dirs.cache_dir().to_path_buf(),
            DirType::Csl => dirs.cache_dir().join("csl"),
            DirType::Fonts => dirs.cache_dir().join("fonts"),
            DirType::Kernels => dirs.cache_dir().join("kernels"),
            DirType::Models => dirs.cache_dir().join("models"),
            DirType::Templates => dirs.cache_dir().join("templates"),

            DirType::Servers => dirs
                .runtime_dir()
                .unwrap_or_else(|| dirs.cache_dir())
                .join("servers"),

            DirType::Agents => dirs.config_dir().join("agents"),
        }
    };

    if ensure && !dir.exists() {
        create_dir_all(dir.clone())?;
    }

    Ok(dir)
}
