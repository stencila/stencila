//! Utility functions for testing

use std::path::PathBuf;

// Expose `pretty_assertions` for use by other internal crates
pub use pretty_assertions;

/// Get the path of the home directory of this repository
pub fn home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("Unable to get repository home directory")
}

/// Get the path of the `fixtures` directory
pub fn fixtures() -> PathBuf {
    home().join("fixtures")
}

/// Assert that two nodes are equal based on their `Debug` display
///
/// Indented debug display is used as it more easily allows differences to be
/// seen. It has the advantage over JSOn of not requiring another dependency
pub fn assert_debug_eq<T: std::fmt::Debug>(a: &T, b: &T) {
    pretty_assertions::assert_eq!(format!("{:#?}", a), format!("{:#?}", b))
}
