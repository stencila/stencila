use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result, bail};
use smart_default::SmartDefault;
use tokio::fs::{create_dir_all, write};

pub const STENCILA_DIR: &str = ".stencila";
pub const DOCS_FILE: &str = "docs.json";
pub const CACHE_DIR: &str = "cache";
pub const ARTIFACTS_DIR: &str = "artifacts";
pub const SKILLS_DIR: &str = "skills";
pub const AGENTS_DIR: &str = "agents";
pub const DB_FILE: &str = "db.kuzu";

#[derive(SmartDefault)]
pub struct CreateStencilaDirOptions {
    #[default = true]
    pub docs_file: bool,

    #[default = true]
    pub gitignore_file: bool,

    #[default = true]
    pub cache_dir: bool,
}

/// Create a `.stencila` directory initialized with expected file and directory structure
///
/// Does not create the DB directory as that is done by Kuzu on database creation.
pub async fn stencila_dir_create(path: &Path, options: CreateStencilaDirOptions) -> Result<()> {
    if !path.exists() {
        create_dir_all(path).await?;
    }

    if options.docs_file {
        stencila_docs_file(path, true).await?;
    }

    if options.gitignore_file {
        write(path.join(".gitignore"), "*\n").await?;
    }

    if options.cache_dir {
        stencila_cache_dir(path, true).await?;
    }

    Ok(())
}

/// Get the path of the `.stencila/docs.json` file and optionally ensure it exists
pub async fn stencila_docs_file(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let tracking_file = stencila_dir.join(DOCS_FILE);

    if ensure && !tracking_file.exists() {
        write(&tracking_file, "{}\n").await?;
    }

    Ok(tracking_file)
}

/// Get the path of the `.stencila/cache` directory and optionally ensure it exists
pub async fn stencila_cache_dir(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let cache_dir = stencila_dir.join(CACHE_DIR);

    if ensure && !cache_dir.exists() {
        create_dir_all(&cache_dir).await?;
    }

    Ok(cache_dir)
}

/// Get the path of the `.stencila/artifacts` directory and optionally ensure it exists
pub async fn stencila_artifacts_dir(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let artifacts_dir = stencila_dir.join(ARTIFACTS_DIR);

    if ensure && !artifacts_dir.exists() {
        create_dir_all(&artifacts_dir).await?;
    }

    Ok(artifacts_dir)
}

/// Get the path of the `.stencila/skills` directory and optionally ensure it exists
pub async fn stencila_skills_dir(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let skills_dir = stencila_dir.join(SKILLS_DIR);

    if ensure && !skills_dir.exists() {
        create_dir_all(&skills_dir).await?;
    }

    Ok(skills_dir)
}

/// Get the path of the `.stencila/agents` directory and optionally ensure it exists
pub async fn stencila_agents_dir(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let agents_dir = stencila_dir.join(AGENTS_DIR);

    if ensure && !agents_dir.exists() {
        create_dir_all(&agents_dir).await?;
    }

    Ok(agents_dir)
}

/// Get the path of the `.stencila/db.kuzu` file and optionally ensure its parent exists
pub async fn stencila_db_file(stencila_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let db_file = stencila_dir.join(DB_FILE);

    if let Some(parent) = db_file.parent()
        && ensure
        && !parent.exists()
    {
        create_dir_all(&parent).await?;
    }

    Ok(db_file)
}

/// Find the closest ancestor directory containing a subdirectory named `dir_name`.
///
/// Walks from `path` upward. Returns `None` if not found.
/// Read-only — never creates directories.
pub fn closest_dot_dir(path: &Path, dir_name: &str) -> Option<PathBuf> {
    let starting_path = path.canonicalize().ok()?;
    let starting_dir = if starting_path.is_file() {
        starting_path.parent()?.to_path_buf()
    } else {
        starting_path
    };
    let mut current = starting_dir;
    loop {
        let candidate = current.join(dir_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
        current = current.parent()?.to_path_buf();
    }
}

/// Get the closest `.stencila` directory to a path
///
/// If the `path` is a file then starts with the parent directory of that file.
/// Walks up the directory tree until a `.stencila` or `.git` directory is found.
/// If none is found, and `ensure` is true, then creates one, next to the `.git`
/// directory if any, or in the starting directory.
pub async fn closest_stencila_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    // Get a canonicalized starting path
    // This allows for accepting files that do not exist by finding the
    // closest ancestor dir that does exist. This is necessary when a
    // user wants to untrack a deleted file, possibly in a subdir of the current dir
    let mut starting_path = path.to_path_buf();
    loop {
        match starting_path.canonicalize() {
            Ok(path) => {
                starting_path = path;
                break;
            }
            Err(..) => {
                starting_path = match starting_path.parent() {
                    Some(path) => path.to_path_buf(),
                    None => current_dir()?,
                }
            }
        }
    }

    let starting_dir = if starting_path.is_file() {
        starting_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
            .to_path_buf()
    } else {
        starting_path
    };

    // Walk up dir tree
    let mut current_dir = starting_dir.clone();
    loop {
        let stencila_dir = current_dir.join(STENCILA_DIR);
        if stencila_dir.exists() {
            return Ok(stencila_dir);
        }

        if ensure {
            // If this is a Git repository then create a `.stencila` dir here
            let git_dir = current_dir.join(".git");
            if git_dir.exists() {
                stencila_dir_create(&stencila_dir, CreateStencilaDirOptions::default()).await?;
                return Ok(stencila_dir);
            }
        }

        let Some(parent_dir) = current_dir.parent() else {
            break;
        };
        current_dir = parent_dir.to_path_buf();
    }

    // Not found so create one in the starting dir
    let stencila_dir = starting_dir.join(STENCILA_DIR);
    if ensure {
        stencila_dir_create(&stencila_dir, CreateStencilaDirOptions::default()).await?;
    }

    Ok(stencila_dir)
}

/// Get or create a new directory within the closest `.stencila/artifacts` directory
/// with specified unique key
///
/// Used for caching artifacts such as downloaded files or costly API responses.
/// It is up to the caller to generate unique keys.
pub async fn closest_artifacts_for(path: &Path, key: &str) -> Result<PathBuf> {
    let stencila_dir = closest_stencila_dir(path, true).await?;
    let artifacts_dir = stencila_artifacts_dir(&stencila_dir, true).await?;

    let artifact_dir = artifacts_dir.join(key);
    create_dir_all(&artifact_dir).await?;

    Ok(artifact_dir)
}

/// Get the path of the closest `.stencila/docs.json` file to a path
///
/// Unless `ensure` is true, the returned path may not exist
#[allow(unused)]
pub async fn closest_docs_file(path: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = closest_stencila_dir(path, ensure).await?;
    stencila_docs_file(&stencila_dir, ensure).await
}

/// Get the path of the workspace directory for a given Stencila directory
pub fn workspace_dir(stencila_dir: &Path) -> Result<PathBuf> {
    match stencila_dir.parent() {
        Some(working_dir) => Ok(working_dir.to_path_buf()),
        None => bail!(
            "The `{STENCILA_DIR}` directory `{}` has no parent",
            stencila_dir.display()
        ),
    }
}

/// Get the path of closest working dir to a path
pub async fn closest_workspace_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    workspace_dir(&closest_stencila_dir(path, ensure).await?)
}

/// Make a path relative to the workspace directory of a `.stencila` directory
pub fn workspace_relative_path(
    stencila_dir: &Path,
    doc_path: &Path,
    must_exist: bool,
) -> Result<PathBuf> {
    let workspace_dir = workspace_dir(stencila_dir)?.canonicalize()?;

    let relative_path = match doc_path.canonicalize() {
        // The document exists so make relative to the working directory
        Ok(doc_path) => match doc_path.strip_prefix(workspace_dir) {
            Ok(path) => path.to_path_buf(),
            Err(..) => bail!(
                "Path is not in the workspace being tracked: {}",
                doc_path.display()
            ),
        },
        // The document does not exist
        Err(..) => {
            if must_exist {
                bail!("File does not exist: {}", doc_path.display())
            }
            doc_path.to_path_buf()
        }
    };

    Ok(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_dot_dir_finds_in_direct_parent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dot_claude = tmp.path().join(".claude");
        std::fs::create_dir_all(&dot_claude).expect("create .claude");

        let result = closest_dot_dir(tmp.path(), ".claude");
        assert_eq!(
            result.as_deref(),
            Some(dot_claude.canonicalize().expect("canon").as_path())
        );
    }

    #[test]
    fn closest_dot_dir_walks_up_to_ancestor() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dot_codex = tmp.path().join(".codex");
        std::fs::create_dir_all(&dot_codex).expect("create .codex");

        let child = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&child).expect("create child");

        let result = closest_dot_dir(&child, ".codex");
        assert_eq!(
            result.as_deref(),
            Some(dot_codex.canonicalize().expect("canon").as_path())
        );
    }

    #[test]
    fn closest_dot_dir_returns_none_when_not_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = closest_dot_dir(tmp.path(), ".nonexistent-dir-xyz");
        assert!(result.is_none());
    }
}
