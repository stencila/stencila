//! Runtime validation tests for Stencila config
//!
//! These tests verify that Stencila's runtime validation correctly rejects
//! invalid configurations that pass JSON Schema validation.

use std::{fs, path::Path};

use tempfile::TempDir;

/// Test that invalid examples fail Stencila's runtime validation
///
/// These are configurations that pass JSON Schema validation but are
/// rejected by Stencila's runtime checks (e.g., route paths must start with `/`).
#[test]
fn invalid_examples_fail_runtime_validation() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/examples/invalid");

    let entries: Vec<_> = fs::read_dir(&examples_dir)
        .expect("should be able to read examples directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .collect();

    assert!(!entries.is_empty(), "should have invalid test examples");

    for entry in entries {
        let source_path = entry.path();

        // Create a temporary directory with the config file
        let temp_dir = TempDir::new().expect("should create temp dir");
        let config_path = temp_dir.path().join("stencila.toml");

        // Copy the test file to the temp directory
        fs::copy(&source_path, &config_path).expect("should copy file");

        // Try to load the config - it should fail runtime validation
        let result = stencila_config::config(temp_dir.path());

        assert!(
            result.is_err(),
            "Invalid example should fail runtime validation: {}",
            source_path.display()
        );
    }
}
