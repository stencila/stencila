///! Utility functions for snapshot testing
use std::path::Path;
use test_utils::fixtures;

// Expose `insta` for use by other internal crates
pub use insta;

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
pub fn snapshot_fixtures<F: FnMut(&Path)>(pattern: &str, func: F) {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::_macro_support::glob_exec(&fixtures(), pattern, func);
    });
}
