//! Generate JSON Schema for Stencila project configuration

use std::path::PathBuf;

use eyre::Result;
use schemars::generate::SchemaSettings;
use serde_json::json;
use stencila_config::Config;

fn main() -> Result<()> {
    // Generate the schema from the Config type using draft-07
    // (matching the main Stencila schemas for consistency)
    let settings = SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<Config>();

    // Convert to JSON value so we can add metadata
    let mut schema_value = serde_json::to_value(schema)?;

    // Add metadata fields
    if let Some(obj) = schema_value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            json!("https://stencila.org/stencila-config.schema.json"),
        );
        obj.insert("title".to_string(), json!("Stencila Config"));
        obj.insert(
            "description".to_string(),
            json!("Configuration for Stencila workspaces"),
        );
    }

    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../json");
    std::fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join("stencila-config.schema.json");
    let json = serde_json::to_string_pretty(&schema_value)?;
    std::fs::write(&output_path, json)?;

    eprintln!("Generated JSON Schema at {}", output_path.display());

    Ok(())
}
