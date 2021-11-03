//! Utility functions for testing

use std::path::PathBuf;

// Expose dependency for use by other internal crates (e.g. so macros work)
pub use pretty_assertions;
pub use serde_json;

/// Get the path of the home directory of this repository
pub fn home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("Unable to get repository home directory")
}

/// Get the path of the `fixtures` directory
pub fn fixtures() -> PathBuf {
    home().join("fixtures")
}

/// Should slow tests be skipped?
///
/// Use at the start of slow tests to return early except on CI or when
/// the env var `RUN_SLOW_TESTS` is set.
///
/// Inspired by https://github.com/rust-analyzer/rust-analyzer/pull/2491
pub fn skip_slow_tests() -> bool {
    if std::env::var("CI").is_err() && std::env::var("RUN_SLOW_TESTS").is_err() {
        eprintln!("Skipping slow test");
        true
    } else {
        false
    }
}

/// Assert that two nodes are equal based on their JSON representation
///
/// This has the advantage over `pretty_assertions::assert_eq` of not requiring the
/// `==` operator to be defined for the types and hiding less usually irrelevant
/// details such as `Box` wrappers.
#[macro_export]
macro_rules! assert_json_eq {
    ($a:expr, $b:expr) => {
        test_utils::pretty_assertions::assert_eq!(
            test_utils::serde_json::to_value(&$a).unwrap(),
            test_utils::serde_json::to_value(&$b).unwrap()
        );
    };
}
