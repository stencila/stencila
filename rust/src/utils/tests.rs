///! Helper functions and macros for tests
use std::{fs::read_to_string, path::PathBuf};

/// Get the path of the home directory of this repository
pub fn home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Get the path of the `fixtures` directory
pub fn fixtures() -> PathBuf {
    home().join("fixtures")
}

/// Generate snapshots from the string content of fixtures matching
/// a glob pattern.
///
/// # Arguments
///
/// - `pattern`: glob pattern _within_ the fixtures folder
/// - `func`: the test function to run on the string content of each
///           file matching the `pattern`.
///
/// `insta`'s `glob` macro seems to have difficulties with our Rust module
/// layout (workspaces and nested modules). This function deals with that
/// by making the pattern relative to the fixtures and adding some other
/// conveniences.
pub fn snapshot_content<F: FnMut(&str, &str)>(pattern: &str, mut func: F) {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::_macro_support::glob_exec(&fixtures(), pattern, |path| {
            let content = read_to_string(path).unwrap();
            let path = pathdiff::diff_paths(path, fixtures())
                .unwrap()
                .display()
                .to_string();
            func(&path, &content)
        });
    });
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

/// Assert that an expression is serialized to expected JSON
#[macro_export]
macro_rules! assert_json {
    ($expr:expr, $json:tt) => {
        pretty_assertions::assert_eq!(
            serde_json::to_value(&$expr).unwrap(),
            serde_json::json!($json)
        );
    };
}

/// Assert that the JSON serialization of two expression is equal
#[macro_export]
macro_rules! assert_json_eq {
    ($expr1:expr, $expr2:expr) => {
        pretty_assertions::assert_eq!(
            serde_json::to_value(&$expr1).unwrap(),
            serde_json::to_value(&$expr2).unwrap()
        );
    };
}
