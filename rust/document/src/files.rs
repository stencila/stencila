use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use eyre::Result;
use glob::{MatchOptions, glob, glob_with};
use itertools::Itertools;

use stencila_format::Format;

use crate::Document;

impl Document {
    /// Resolve a path to a file
    ///
    /// Given a, possibly incomplete, path resolves a file from it. Used when
    /// serving documents to determine which file to show in the browser.
    pub fn resolve_file(path: &Path) -> Result<Option<PathBuf>> {
        // If a file exists at the path then just resolve to it
        if path.exists() && path.is_file() {
            return Ok(Some(path.to_path_buf()));
        }

        // If any files have the same stem as the path (everything minus the extension)
        // then use the one with the format with highest precedence and latest modification date.
        // This checks that the file has a stem otherwise files like `.gitignore` match against it.
        let pattern = format!("{}.*", path.display());
        if let Some(path) = glob(&pattern)?
            .flatten()
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| !name.to_string_lossy().starts_with('.'))
                    && path.is_file()
            })
            .sorted_by(|a, b| {
                let a_format = Format::from_path(a);
                let b_format = Format::from_path(b);
                match a_format.rank().cmp(&b_format.rank()) {
                    Ordering::Equal => {
                        let a_modified = std::fs::metadata(a)
                            .and_then(|metadata| metadata.modified())
                            .unwrap_or(UNIX_EPOCH);
                        let b_modified = std::fs::metadata(b)
                            .and_then(|metadata| metadata.modified())
                            .unwrap_or(UNIX_EPOCH);
                        a_modified.cmp(&b_modified).reverse()
                    }
                    ordering => ordering,
                }
            })
            .next()
        {
            return Ok(Some(path));
        }

        // If the path correlates to a folder with an index, main, or readme file
        // then use the one with the highest precedence
        let pattern = format!("{}/*", path.display());
        if let Some(path) = glob_with(
            &pattern,
            MatchOptions {
                case_sensitive: false,
                ..Default::default()
            },
        )?
        .flatten()
        .find(|path| {
            // Select the first file matching these criteria
            // noting that `glob` returns entries sorted alphabetically
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        let name = name.to_lowercase();
                        name.starts_with("index.")
                            || name.starts_with("main.")
                            || name.starts_with("readme.")
                    })
                    .unwrap_or_default()
        }) {
            return Ok(Some(path));
        }

        Ok(None)
    }
}
