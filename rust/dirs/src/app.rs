//! Application level (e.g. Stencila CLI or Stencila Desktop) config and directories

use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
};

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

    /// Cache subdirectory for builtin agent definitions
    BuiltinAgents,

    /// Cache subdirectory for builtin skill definitions
    BuiltinSkills,

    /// Cache subdirectory for builtin workflow definitions
    BuiltinWorkflows,

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
            DirType::BuiltinAgents => dirs.cache_dir().join("agents"),
            DirType::BuiltinSkills => dirs.cache_dir().join("skills"),
            DirType::BuiltinWorkflows => dirs.cache_dir().join("workflows"),
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

/// Get a version-specific application subdirectory.
///
/// In debug builds, returns a `dev` subdirectory so local changes are picked up.
/// In release builds, returns a version-specific subdirectory so cached content
/// can be reused without unnecessary rewrites across process invocations.
pub fn get_versioned_app_dir(
    dir_type: DirType,
    version: &str,
    debug_assertions: bool,
    ensure: bool,
) -> Result<PathBuf> {
    let base = get_app_dir(dir_type, ensure)?;
    let dir = if debug_assertions {
        base.join("dev")
    } else {
        base.join(version)
    };

    if ensure && !dir.exists() {
        create_dir_all(&dir)?;
    }

    Ok(dir)
}

/// Ensure a directory contains embedded files extracted to disk exactly once.
///
/// Uses a `.initialized` sentinel file to avoid repeated writes. If the sentinel
/// is absent, the provided files are written to disk and the sentinel is created.
/// If the sentinel exists, the directory is assumed to already contain the same
/// embedded content for the current version.
pub fn ensure_embedded_dir<I, P>(dir: &Path, files: I) -> Result<()>
where
    I: IntoIterator<Item = (P, Vec<u8>)>,
    P: AsRef<Path>,
{
    let files: Vec<(PathBuf, Vec<u8>)> = files
        .into_iter()
        .map(|(path, content)| (path.as_ref().to_path_buf(), content))
        .collect();

    let sentinel = dir.join(".initialized");
    if sentinel.exists()
        && files
            .iter()
            .all(|(relative_path, _)| dir.join(relative_path).exists())
    {
        return Ok(());
    }

    if !dir.exists() {
        create_dir_all(dir)?;
    }

    for (relative_path, content) in files {
        let path = dir.join(&relative_path);
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
    }

    std::fs::write(sentinel, [])?;

    Ok(())
}
