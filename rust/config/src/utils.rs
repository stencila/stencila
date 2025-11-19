use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use figment::{
    Figment,
    providers::{Format, Yaml},
    value::Value,
};

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
                        .ok_or_else(|| eyre!("File has no parent directory"))
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
/// 1. User config (~/.config/stencila/stencila.yaml and stencila.local.yaml) - if included
/// 2. Ancestor configs (from root down to current directory)
/// 3. Each level: stencila.yaml then stencila.local.yaml
pub(crate) fn collect_config_paths(
    start_dir: &Path,
    include_user_config: bool,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    // 1. Add user config (lowest precedence) - only if requested
    if include_user_config
        && let Ok(user_config_dir) =
            stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false)
    {
        paths.push(user_config_dir.join("stencila.yaml"));
        paths.push(user_config_dir.join("stencila.local.yaml"));
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
        paths.push(ancestor.join("stencila.yaml"));
        paths.push(ancestor.join("stencila.local.yaml"));
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

        // Use Yaml::file() provider to load config with metadata tracking
        // This enables RelativePathBuf to resolve paths relative to the config file
        // Note: With Yaml::file(), parse errors are deferred until extraction time
        figment = figment.merge(Yaml::file(config_path));
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
    /// Nearest stencila.yaml (or create in CWD if none exists)
    Nearest,
    /// User-level stencila.yaml (~/.config/stencila/stencila.yaml)
    User,
    /// Nearest stencila.local.yaml (or create in CWD if none exists)
    Local,
}

/// Set a configuration value
///
/// Finds or creates the appropriate config file based on the target,
/// updates the value at the specified key path, and writes the file back.
///
/// Returns the path to the modified config file.
pub fn config_set(key: &str, value: &str, target: ConfigTarget) -> Result<std::path::PathBuf> {
    // Determine the target file path
    let config_path = match target {
        ConfigTarget::Nearest => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, "stencila.yaml").unwrap_or_else(|| cwd.join("stencila.yaml"))
        }
        ConfigTarget::User => {
            let user_config_dir = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, true)?;
            user_config_dir.join("stencila.yaml")
        }
        ConfigTarget::Local => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, "stencila.local.yaml")
                .unwrap_or_else(|| cwd.join("stencila.local.yaml"))
        }
    };

    // Load existing config or create empty
    let mut config_value: serde_yaml::Value = if config_path.exists() {
        let contents = fs::read_to_string(&config_path)?;
        serde_yaml::from_str(&contents)?
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };

    // Navigate/create the nested path and set the value
    set_nested_value(&mut config_value, key, value)?;

    // Write back to file
    let yaml_string = serde_yaml::to_string(&config_value)?;
    fs::write(&config_path, yaml_string)?;

    Ok(config_path)
}

/// Remove a configuration value
///
/// Finds the appropriate config file based on the target and removes
/// the value at the specified key path.
///
/// Returns the path to the modified config file.
pub fn config_unset(key: &str, target: ConfigTarget) -> Result<std::path::PathBuf> {
    // Determine the target file path
    let config_path = match target {
        ConfigTarget::Nearest => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, "stencila.yaml")
                .ok_or_else(|| eyre!("No stencila.yaml found"))?
        }
        ConfigTarget::User => {
            let user_config_dir =
                stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false)?;
            let path = user_config_dir.join("stencila.yaml");
            if !path.exists() {
                return Err(eyre!("User config file does not exist"));
            }
            path
        }
        ConfigTarget::Local => {
            let cwd = std::env::current_dir()?;
            find_config_file(&cwd, "stencila.local.yaml")
                .ok_or_else(|| eyre!("No stencila.local.yaml found"))?
        }
    };

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut config_value: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    // Remove the nested value
    unset_nested_value(&mut config_value, key)?;

    // Write back to file
    let yaml_string = serde_yaml::to_string(&config_value)?;
    fs::write(&config_path, yaml_string)?;

    Ok(config_path)
}

/// Helper to set a nested value in a YAML structure using dot notation
pub(crate) fn set_nested_value(root: &mut serde_yaml::Value, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(eyre!("Empty key path"));
    }

    // Navigate to the parent, creating nested maps as needed
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if !current.is_mapping() {
            *current = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        }

        let map = current
            .as_mapping_mut()
            .ok_or_else(|| eyre!("Expected mapping"))?;
        let key = serde_yaml::Value::String(part.to_string());

        current = map
            .entry(key.clone())
            .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    }

    // Set the final value
    if !current.is_mapping() {
        *current = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
    }

    let map = current
        .as_mapping_mut()
        .ok_or_else(|| eyre!("Expected mapping"))?;
    let final_key = serde_yaml::Value::String(parts[parts.len() - 1].to_string());

    // Try to parse value as different types
    let parsed_value = if let Ok(b) = value.parse::<bool>() {
        serde_yaml::Value::Bool(b)
    } else if let Ok(n) = value.parse::<i64>() {
        serde_yaml::Value::Number(n.into())
    } else if let Ok(f) = value.parse::<f64>() {
        serde_yaml::Value::Number(serde_yaml::Number::from(f))
    } else {
        serde_yaml::Value::String(value.to_string())
    };

    map.insert(final_key, parsed_value);

    Ok(())
}

/// Helper to remove a nested value from a YAML structure using dot notation
pub(crate) fn unset_nested_value(root: &mut serde_yaml::Value, key: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return Err(eyre!("Empty key path"));
    }

    // Navigate to the parent
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if !current.is_mapping() {
            return Err(eyre!("Key path not found: {}", key));
        }

        let map = current
            .as_mapping_mut()
            .ok_or_else(|| eyre!("Expected mapping"))?;
        let key_value = serde_yaml::Value::String(part.to_string());

        current = map
            .get_mut(&key_value)
            .ok_or_else(|| eyre!("Key path not found: {}", key))?;
    }

    // Remove the final key
    if !current.is_mapping() {
        return Err(eyre!("Key path not found: {}", key));
    }

    let map = current
        .as_mapping_mut()
        .ok_or_else(|| eyre!("Expected mapping"))?;
    let final_key = serde_yaml::Value::String(parts[parts.len() - 1].to_string());

    map.remove(&final_key)
        .ok_or_else(|| eyre!("Key not found: {}", key))?;

    Ok(())
}
