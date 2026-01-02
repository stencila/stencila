use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result, eyre};
use figment::{
    Figment,
    providers::{Format, Toml},
    value::Value,
};
use toml_edit::{DocumentMut, Item, Table, value};

/// Format a TOML array to be multi-line for better readability
pub(crate) fn format_array_multiline(arr: &mut toml_edit::Array) {
    // Set trailing newline on the array itself
    arr.set_trailing("\n");
    arr.set_trailing_comma(true);

    // Add newline prefix to each item for multi-line formatting
    for item in arr.iter_mut() {
        let decor = item.decor_mut();
        decor.set_prefix("\n  ");
    }
}

/// Normalize a path, handling both files and directories
///
/// If the path is a file, returns its parent directory.
/// Attempts to canonicalize the path, falling back to parent directories
/// if the path doesn't exist yet.
pub(crate) fn normalize_path(path: &Path) -> Result<PathBuf> {
    let mut current = path.to_path_buf();

    // Try to canonicalize, falling back to parent if path doesn't exist
    loop {
        match current.canonicalize() {
            Ok(canonical) => {
                // If it's a file, use its parent directory
                if canonical.is_file() {
                    return canonical
                        .parent()
                        .ok_or_eyre("File has no parent directory")
                        .map(|p| p.to_path_buf());
                }
                return Ok(canonical);
            }
            Err(_) => {
                // Path doesn't exist, try parent
                match current.parent() {
                    Some(parent) => current = parent.to_path_buf(),
                    None => {
                        // Reached root without finding existing path
                        return std::env::current_dir()
                            .map_err(|e| eyre!("Failed to get current directory: {}", e));
                    }
                }
            }
        }
    }
}

/// Collect all config file paths in merge order (lowest to highest precedence)
///
/// Returns paths in the order they should be merged:
/// 1. User config (~/.config/stencila/stencila.toml and stencila.local.toml) - if included
/// 2. Ancestor configs (from root down to current directory)
/// 3. Each level: stencila.toml then stencila.local.toml
pub(crate) fn collect_config_paths(
    start_dir: &Path,
    include_user_config: bool,
) -> Result<Vec<PathBuf>> {
    use crate::{CONFIG_FILENAME, CONFIG_LOCAL_FILENAME};

    let mut paths = Vec::new();

    // 1. Add user config (lowest precedence) - only if requested
    if include_user_config
        && let Ok(user_config_dir) =
            stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false)
    {
        paths.push(user_config_dir.join(CONFIG_FILENAME));
        paths.push(user_config_dir.join(CONFIG_LOCAL_FILENAME));
    }

    // 2. Collect all ancestor directories (from start up to root)
    let mut ancestors = Vec::new();
    let mut current = start_dir.to_path_buf();

    loop {
        ancestors.push(current.clone());

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }

    // 3. Reverse to get root-to-current order, then add config files
    ancestors.reverse();

    for ancestor in ancestors {
        // Add regular config first, then local override
        paths.push(ancestor.join(CONFIG_FILENAME));
        paths.push(ancestor.join(CONFIG_LOCAL_FILENAME));
    }

    Ok(paths)
}

/// Find the nearest config file by walking up the directory tree
///
/// Starts from `start_dir` and walks up until it finds a file with the given name,
/// or reaches the root.
pub fn find_config_file(start_dir: &Path, filename: &str) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join(filename);
        if config_path.exists() {
            return Some(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Build a figment by merging config files from various sources
///
/// This is the core config loading logic, extracted to allow reuse
/// for both structured config extraction and value queries.
pub(crate) fn build_figment(path: &Path, include_user_config: bool) -> Result<Figment> {
    // Normalize the starting path
    let start_path = normalize_path(path)?;

    // Collect all config file paths in merge order (lowest to highest precedence)
    let config_paths = collect_config_paths(&start_path, include_user_config)?;

    // Build figment by merging configs in order, with individual file error handling
    let mut figment = Figment::new();

    for config_path in &config_paths {
        // Skip if file doesn't exist
        if !config_path.exists() {
            continue;
        }

        tracing::debug!("Loading config from: {}", config_path.display());

        // Use Toml::file() provider to load config with metadata tracking
        // This enables RelativePathBuf to resolve paths relative to the config file
        // Note: With Toml::file(), parse errors are deferred until extraction time
        figment = figment.merge(Toml::file(config_path));
    }

    Ok(figment)
}

/// Get a specific config value by dot-notation key
///
/// Uses Figment's built-in `find_value()` which understands dot notation
/// including array access (e.g., `packages[0].name`).
///
/// Returns `None` if the key doesn't exist, or an error if config loading fails.
pub fn config_value(path: &Path, key: &str) -> Result<Option<Value>> {
    let figment = build_figment(path, true)?;

    match figment.find_value(key) {
        Ok(value) => Ok(Some(value)),
        Err(err) if matches!(err.kind, figment::error::Kind::MissingField(_)) => Ok(None),
        Err(err) => Err(eyre!(err)),
    }
}

/// Target location for config operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigTarget {
    /// Nearest stencila.toml (or create in CWD if none exists)
    Nearest,
    /// User-level stencila.toml (~/.config/stencila/stencila.toml)
    User,
    /// Nearest stencila.local.toml (or create in CWD if none exists)
    Local,
}

/// Set a configuration value
///
/// Finds or creates the appropriate config file based on the target,
/// updates the value at the specified key path, and writes the file back.
/// Preserves formatting and comments in the TOML file.
///
/// Returns the path to the modified config file.
pub fn config_set(key: &str, value_str: &str, target: ConfigTarget) -> Result<std::path::PathBuf> {
    use crate::{CONFIG_FILENAME, CONFIG_LOCAL_FILENAME};

    // Determine the target file path
    let config_path = match target {
        ConfigTarget::Nearest => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, CONFIG_FILENAME).unwrap_or_else(|| cwd.join(CONFIG_FILENAME))
        }
        ConfigTarget::User => {
            let user_config_dir = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, true)?;
            user_config_dir.join(CONFIG_FILENAME)
        }
        ConfigTarget::Local => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, CONFIG_LOCAL_FILENAME)
                .unwrap_or_else(|| cwd.join(CONFIG_LOCAL_FILENAME))
        }
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

    // Navigate/create the nested path and set the value
    set_nested_value_toml(&mut doc, key, value_str)?;

    // Write back to file, preserving formatting
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Remove a configuration value
///
/// Finds the appropriate config file based on the target and removes
/// the value at the specified key path. Preserves formatting and comments.
///
/// Returns the path to the modified config file.
pub fn config_unset(key: &str, target: ConfigTarget) -> Result<std::path::PathBuf> {
    use crate::{CONFIG_FILENAME, CONFIG_LOCAL_FILENAME};

    // Determine the target file path
    let config_path = match target {
        ConfigTarget::Nearest => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, CONFIG_FILENAME)
                .ok_or_else(|| eyre!("No `{CONFIG_FILENAME}` found"))?
        }
        ConfigTarget::User => {
            let user_config_dir =
                stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false)?;
            let path = user_config_dir.join(CONFIG_FILENAME);
            if !path.exists() {
                return Err(eyre!("User config file does not exist"));
            }
            path
        }
        ConfigTarget::Local => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, CONFIG_LOCAL_FILENAME)
                .ok_or_else(|| eyre!("No `{CONFIG_LOCAL_FILENAME}` found"))?
        }
    };

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    // Remove the nested value
    unset_nested_value_toml(&mut doc, key)?;

    // Write back to file, preserving formatting
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Helper to set a nested value in a TOML document using dot notation
///
/// Preserves all formatting and comments while updating the value.
pub(crate) fn set_nested_value_toml(
    doc: &mut DocumentMut,
    key: &str,
    value_str: &str,
) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(eyre!("Empty key path"));
    }

    // Navigate to the parent table, creating nested tables as needed
    let mut current = doc.as_table_mut();

    for part in &parts[..parts.len() - 1] {
        current = current
            .entry(part)
            .or_insert(Item::Table(Table::new()))
            .as_table_mut()
            .ok_or_else(|| eyre!("Expected table at key '{}'", part))?;
    }

    // Set the final value with type inference
    let final_key = parts[parts.len() - 1];
    let parsed_value = infer_toml_value(value_str);

    current[final_key] = parsed_value;

    Ok(())
}

/// Helper to infer TOML value type from string
fn infer_toml_value(value_str: &str) -> Item {
    // Try to parse as bool
    if let Ok(b) = value_str.parse::<bool>() {
        return value(b);
    }

    // Try to parse as i64
    if let Ok(n) = value_str.parse::<i64>() {
        return value(n);
    }

    // Try to parse as f64
    if let Ok(f) = value_str.parse::<f64>() {
        return value(f);
    }

    // Default to string
    value(value_str)
}

/// Helper to remove a nested value from a TOML document using dot notation
///
/// Preserves all formatting and comments while removing the value.
pub(crate) fn unset_nested_value_toml(doc: &mut DocumentMut, key: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(eyre!("Empty key path"));
    }

    // Navigate to the parent table
    let mut current = doc.as_table_mut();

    for part in &parts[..parts.len() - 1] {
        current = current
            .get_mut(part)
            .ok_or_else(|| eyre!("Key path not found: {key}"))?
            .as_table_mut()
            .ok_or_else(|| eyre!("Expected table at key '{part}'"))?;
    }

    // Remove the final key
    let final_key = parts[parts.len() - 1];
    current
        .remove(final_key)
        .ok_or_else(|| eyre!("Key not found: {key}"))?;

    Ok(())
}
