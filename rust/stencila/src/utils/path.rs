use lexiclean::Lexiclean;
use std::path::{Path, PathBuf};

/// Merge paths
///
/// Differs from `path.join()` in that it joins the *parent* of `path1`, not `path1` itself.
/// Does not canonicalize the path, or check for its existence.
pub fn merge<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> PathBuf {
    path1.as_ref().join("..").join(path2).lexiclean()
}

#[cfg(test)]
mod test {
    use super::*;
    use path_slash::PathBufExt;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_merge() {
        assert_eq!(merge("a", "c").to_slash_lossy(), "c");
        assert_eq!(merge("a/b", "c").to_slash_lossy(), "a/c");
        assert_eq!(merge("a/b/../d/e", "c").to_slash_lossy(), "a/d/c")
    }
}
