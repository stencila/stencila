//! Application level (e.g. Stencila CLI or Stencila Desktop) config and directories

use std::{env, fs::create_dir_all, path::PathBuf};

use directories::ProjectDirs;

use common::{
    clap::{self, ValueEnum},
    eyre::{OptionExt, Result},
    strum::{Display, EnumString},
};

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "stencila", "stencila").ok_or_eyre("unable to build project dirs")
}

#[derive(Debug, Display, Default, Clone, Copy, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum DirType {
    #[default]
    Config,
    Cache,
    Assistants,
}

pub fn get_app_dir(dt: DirType, mut ensure: bool) -> Result<PathBuf> {
    let dirs = project_dirs()?;

    let dir = {
        match dt {
            DirType::Config => dirs.config_dir().to_path_buf(),
            DirType::Cache => dirs.cache_dir().to_path_buf(),
            DirType::Assistants => {
                if let Ok(dir) = env::var("STENCILA_ASSISTANTS_DIR") {
                    ensure = false;
                    PathBuf::from(dir)
                } else {
                    dirs.config_dir().join("assistants")
                }
            }
        }
    };
    if ensure && !dir.exists() {
        create_dir_all(dir.clone())?;
    }
    Ok(dir)
}
