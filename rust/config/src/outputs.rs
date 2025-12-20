use std::{fs, path::PathBuf};

use eyre::{Result, bail, eyre};
use toml_edit::{value, DocumentMut, InlineTable, Item, Table};

use crate::{find_config_file, OutputCommand};

/// Validate output configuration options before writing to config
///
/// Checks the same constraints as OutputConfig::validate to prevent
/// writing invalid configurations that would fail at load time.
fn validate_output_options(
    key: &str,
    source: Option<&str>,
    refs: Option<&[String]>,
    pattern: Option<&str>,
    exclude: Option<&[String]>,
) -> Result<()> {
    // Source and pattern are mutually exclusive
    if source.is_some() && pattern.is_some() {
        bail!("Cannot specify both --source and --pattern for output `{key}`",);
    }

    // If pattern is set, key must include *.ext suffix
    if pattern.is_some() && !key.contains("*.") {
        bail!("Output `{key}` with --pattern must include `*.ext` suffix (e.g., 'reports/*.pdf')",);
    }

    // Exclude only applies with pattern
    if exclude.is_some() && pattern.is_none() {
        bail!("Output `{key}` has --exclude but no --pattern");
    }

    // If refs is present, it must be non-empty
    if let Some(refs_list) = refs
        && refs_list.is_empty()
    {
        bail!("Output `{key}` has empty --refs");
    }

    Ok(())
}

/// Add an output to the [outputs] section of stencila.toml
///
/// This function:
/// 1. Validates the output configuration options
/// 2. Finds the nearest stencila.toml (or creates one if none exists)
/// 3. Adds or updates the output entry for the given key
/// 4. Supports both simple source paths and full config objects
///
/// Returns the path to the modified config file.
pub fn config_add_output(
    key: &str,
    source: Option<&str>,
    command: Option<OutputCommand>,
    refs: Option<&[String]>,
    pattern: Option<&str>,
    exclude: Option<&[String]>,
) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Validate options before writing config
    validate_output_options(key, source, refs, pattern, exclude)?;

    let cwd = std::env::current_dir()?;
    let config_path =
        find_config_file(&cwd, CONFIG_FILENAME).unwrap_or_else(|| cwd.join(CONFIG_FILENAME));

    // Canonicalize config_path if it exists
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        config_path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(CONFIG_FILENAME))
            .unwrap_or(config_path)
    };

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Ensure [outputs] table exists
    if doc.get("outputs").is_none() {
        doc["outputs"] = Item::Table(Table::new());
    }

    let outputs_table = doc
        .get_mut("outputs")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| eyre!("outputs field is not a table"))?;

    // Determine if we need a simple string or full config
    let needs_config = command.is_some()
        || refs.is_some()
        || pattern.is_some()
        || exclude.is_some()
        || source.is_none();

    if needs_config {
        // Build inline table for full config
        let mut config_table = InlineTable::new();

        if let Some(src) = source {
            config_table.insert("source", src.into());
        }

        if let Some(pat) = pattern {
            config_table.insert("pattern", pat.into());
        }

        if let Some(cmd) = command {
            config_table.insert("command", cmd.to_string().into());
        }

        if let Some(refs_list) = refs {
            let mut arr = toml_edit::Array::new();
            for r in refs_list {
                arr.push(r.as_str());
            }
            config_table.insert("refs", toml_edit::Value::Array(arr));
        }

        if let Some(exclude_list) = exclude {
            let mut arr = toml_edit::Array::new();
            for e in exclude_list {
                arr.push(e.as_str());
            }
            config_table.insert("exclude", toml_edit::Value::Array(arr));
        }

        outputs_table[key] = value(config_table);
    } else if let Some(src) = source {
        // Simple string value
        outputs_table[key] = value(src);
    }

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Remove an output from the [outputs] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml
/// 2. Removes the output entry for the given key
///
/// Returns the path to the modified config file.
pub fn config_remove_output(key: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    let cwd = std::env::current_dir()?;
    let config_path = find_config_file(&cwd, CONFIG_FILENAME)
        .ok_or_else(|| eyre!("No `{CONFIG_FILENAME}` found"))?;

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    // Get the outputs table
    let outputs_table = doc
        .get_mut("outputs")
        .ok_or_else(|| eyre!("No [outputs] section in `{CONFIG_FILENAME}`"))?
        .as_table_mut()
        .ok_or_else(|| eyre!("outputs field is not a table"))?;

    // Remove the key
    outputs_table
        .remove(key)
        .ok_or_else(|| eyre!("Output `{key}` not found"))?;

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}
