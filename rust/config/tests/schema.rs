//! Schema validation tests for Stencila config
//!
//! These tests validate TOML example files against the JSON Schema
//! to ensure the schema correctly accepts valid configs and rejects invalid ones.

use std::{fs, path::Path};

use jsonschema::Validator;
use serde_json::Value;

/// Load and compile the JSON Schema
fn load_schema() -> Validator {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("should have parent")
        .parent()
        .expect("should have grandparent")
        .join("json/stencila-config.schema.json");

    let schema_str = fs::read_to_string(&schema_path).expect("should be able to read schema file");
    let schema: Value = serde_json::from_str(&schema_str).expect("should be valid JSON");

    Validator::new(&schema).expect("should compile schema")
}

/// Parse a TOML file to JSON Value for schema validation
fn toml_to_json(path: &Path) -> Value {
    let toml_str = fs::read_to_string(path).expect("should be able to read TOML file");
    let toml_value: toml::Value = toml::from_str(&toml_str).expect("should be valid TOML");

    // Convert TOML to JSON via serde
    serde_json::to_value(toml_value).expect("should convert to JSON")
}

#[test]
fn positive_examples_validate() {
    let schema = load_schema();
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/examples/positive");

    let entries: Vec<_> = fs::read_dir(&examples_dir)
        .expect("should be able to read examples directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .collect();

    assert!(!entries.is_empty(), "should have positive test examples");

    for entry in entries {
        let path = entry.path();
        let json_value = toml_to_json(&path);

        let result = schema.validate(&json_value);
        assert!(
            result.is_ok(),
            "Positive example should validate: {}\nErrors: {:?}",
            path.display(),
            result.err()
        );
    }
}

#[test]
fn negative_examples_fail_validation() {
    let schema = load_schema();
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/examples/negative");

    let entries: Vec<_> = fs::read_dir(&examples_dir)
        .expect("should be able to read examples directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .collect();

    assert!(!entries.is_empty(), "should have negative test examples");

    for entry in entries {
        let path = entry.path();
        let json_value = toml_to_json(&path);

        let result = schema.validate(&json_value);
        assert!(
            result.is_err(),
            "Negative example should fail validation: {}",
            path.display()
        );
    }
}
