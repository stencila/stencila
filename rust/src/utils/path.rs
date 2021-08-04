use path_clean::PathClean;
use std::path::{Path, PathBuf};

/// Merge paths
///
/// Differs from `path.join()` in that it joins the *parent* of `path1`, not `path1` itself.
/// Does not canonicalize the path, or check for its existence.
pub fn merge<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> PathBuf {
    path1.as_ref().join("..").join(path2).clean()
}
