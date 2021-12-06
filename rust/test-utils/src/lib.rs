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

/// Should test be skipped on CI?
pub fn skip_ci(reason: &str) -> bool {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping test on CI: {}", reason);
        true
    } else {
        false
    }
}

/// Should test be skipped on the current operating system?
///
/// See https://doc.rust-lang.org/std/env/consts/constant.OS.html for
/// possible values.
pub fn skip_os(os: &str, reason: &str) -> bool {
    if std::env::consts::OS == os {
        eprintln!("Skipping test on OS `{}`: {}", os, reason);
        true
    } else {
        false
    }
}

/// Should test be skipped on CI for an operating system?
pub fn skip_ci_os(os: &str, reason: &str) -> bool {
    if std::env::var("CI").is_ok() && std::env::consts::OS == os {
        eprintln!("Skipping test on CI for OS `{}`: {}", os, reason);
        true
    } else {
        false
    }
}

/// Should slow tests be skipped?
///
/// Use at the start of slow tests to return early except on CI or when
/// the env var `RUN_SLOW_TESTS` is set.
///
/// Inspired by https://github.com/rust-analyzer/rust-analyzer/pull/2491
pub fn skip_slow() -> bool {
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

/// Print log entries
///
/// Many of the sibling crates use `tracing` and seeing log entries can be
/// useful during testing.
/// This prints entries to stderr. Use `cargo test -- --nocapture`.
pub fn print_logs() {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .init()
}
