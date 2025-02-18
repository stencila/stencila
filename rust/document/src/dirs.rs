use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use common::{
    eyre::{bail, OptionExt, Result},
    tokio::fs::create_dir_all,
};

pub(super) const STENCILA_DIR: &str = ".stencila";

/// Get the path of the closest `.stencila` directory to a path
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
            let git_dir = current_dir.join(".git");
            if git_dir.exists() {
                create_dir_all(&stencila_dir).await?;
                return Ok(stencila_dir);
            }
        }

        let Some(parent_dir) = current_dir.parent() else {
            break;
        };
        current_dir = parent_dir.to_path_buf();
    }

    // Not found so create one in starting dir
    let stencila_dir = starting_dir.join(STENCILA_DIR);
    if ensure {
        create_dir_all(&stencila_dir).await?;
    }

    Ok(stencila_dir)
}

/// Get the path of closest working dir to a path
pub async fn closest_workspace_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = closest_stencila_dir(path, ensure).await?;
    match stencila_dir.parent() {
        Some(working_dir) => Ok(working_dir.to_path_buf()),
        None => bail!(
            "The `{STENCILA_DIR}` directory `{}` has no parent",
            stencila_dir.display()
        ),
    }
}
