use path_clean::PathClean;
use path_slash::PathExt;
use std::path::{Path, PathBuf};

/// Merge paths
///
/// Differs from `path.join()` in that it joins the *parent* of `path1`, not `path1` itself.
/// Does not canonicalize the path, or check for its existence.
pub fn merge<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> PathBuf {
    let path1 = PathBuf::from(path1.as_ref().to_slash_lossy());
    let path2 = PathBuf::from(path2.as_ref().to_slash_lossy());
    path1.join("..").join(path2).clean()
}
