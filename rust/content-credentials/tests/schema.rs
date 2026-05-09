//! Tests for generated content credentials schema and documentation artifacts.

use std::{error::Error, fs, io, path::PathBuf};

use serde_json::Value;
use stencila_content_credentials::PROVENANCE_SCHEMA;

#[test]
fn provenance_assertion_schema_has_current_id() -> Result<(), Box<dyn Error>> {
    let schema_path = repo_dir()?.join("json/stencila-provenance-assertion-v1.schema.json");

    let schema_str = fs::read_to_string(&schema_path)?;
    let schema: Value = serde_json::from_str(&schema_str)?;

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(PROVENANCE_SCHEMA)
    );

    Ok(())
}

#[test]
fn provenance_assertion_schema_links_to_generated_docs() -> Result<(), Box<dyn Error>> {
    let repo_dir = repo_dir()?;
    let schema_path = repo_dir.join("json/stencila-provenance-assertion-v1.schema.json");
    let docs_path = repo_dir.join("site/docs/content-credentials/index.md");

    let schema_str = fs::read_to_string(&schema_path)?;
    let schema: Value = serde_json::from_str(&schema_str)?;
    let docs = fs::read_to_string(&docs_path)?;

    let ai_disclosure_description = schema
        .pointer("/properties/aiDisclosure/description")
        .and_then(Value::as_str);

    assert!(matches!(
        ai_disclosure_description,
        Some(description)
            if description
                .contains("https://stencila.io/docs/content-credentials#ai-disclosure")
    ));
    assert!(docs.contains("| [`aiDisclosure`](#ai-disclosure) |"));
    assert!(!docs.contains("<a id=\""));

    Ok(())
}

#[test]
fn provenance_assertion_docs_nav_uses_schema_first_use_order() -> Result<(), Box<dyn Error>> {
    let nav_path = repo_dir()?.join("site/docs/content-credentials/_nav.yaml");
    let nav = fs::read_to_string(&nav_path)?;

    assert_order(
        &nav,
        &[
            "index",
            "producer-record",
            "asset-record",
            "document-record",
            "execution-digest-record",
            "activity-record",
            "attribution-record",
            "agent-record",
            "identifier-record",
        ],
    )?;
    assert_order(
        &nav,
        &["workflow-record", "definition-record", "environment-record"],
    )?;

    Ok(())
}

fn assert_order(text: &str, slugs: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut cursor = 0;

    for slug in slugs {
        let pattern = format!("  - \"{slug}\"");
        let index = text[cursor..]
            .find(&pattern)
            .ok_or_else(|| io::Error::other(format!("missing nav item in order: {slug}")))?;
        cursor += index + pattern.len();
    }

    Ok(())
}

fn repo_dir() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("crate manifest should have parent"))?
        .parent()
        .ok_or_else(|| io::Error::other("crate manifest should have grandparent"))?
        .to_path_buf())
}
