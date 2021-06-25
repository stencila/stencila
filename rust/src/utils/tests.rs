///! Helper functions for tests
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
pub fn snapshot_content<F: FnMut(&str)>(pattern: &str, mut func: F) {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::_macro_support::glob_exec(&fixtures(), pattern, |path| {
            let content = read_to_string(path).unwrap();
            func(&content)
        });
    });
}
