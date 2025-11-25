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
use toml_edit::{DocumentMut, InlineTable, Item, Table, value};

/// Format a TOML array to be multi-line for better readability
fn format_array_multiline(arr: &mut toml_edit::Array) {
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

/// Update the watch ID for a specific remote in the config
///
/// Finds the remote configuration matching the given file path and URL,
/// then updates its watch field. If watch_id is None, the watch field is removed.
/// Preserves formatting and comments in the TOML file.
///
/// This function:
/// 1. Finds the nearest stencila.toml file (or creates one if none exists)
/// 2. Locates the remote entry matching both path and URL
/// 3. Updates the watch field
/// 4. Writes the config back to the file
///
/// Returns the path to the modified config file.
pub fn config_update_remote_watch(
    file_path: &Path,
    remote_url: &str,
    watch_id: Option<String>,
) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Find the nearest stencila.toml starting from the file's directory
    // This ensures we find the config even when CWD is outside the workspace
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Get the remotes table (HashMap structure)
    let remotes_table = doc
        .get_mut("remotes")
        .ok_or_else(|| eyre!("No remotes configured in `{CONFIG_FILENAME}`"))?
        .as_table_mut()
        .ok_or_eyre("remotes field is not a table")?;

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Canonicalize and make file_path workspace-relative
    let file_canonical = match file_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            // If file doesn't exist, try to canonicalize its parent and rejoin filename
            let parent = file_path.parent().ok_or_eyre("File path has no parent")?;
            let filename = file_path
                .file_name()
                .ok_or_eyre("File path has no filename")?;
            parent.canonicalize()?.join(filename)
        }
    };

    let file_relative = file_canonical
        .strip_prefix(workspace_dir)
        .unwrap_or(&file_canonical);

    // Find matching remote by path (key) and URL (in value)
    let mut found = false;
    for (path_key, remote_value) in remotes_table.iter_mut() {
        // Resolve and canonicalize the remote path (from the HashMap key)
        let path_key_str = path_key.get();
        let remote_path = workspace_dir.join(path_key_str);
        let remote_canonical = remote_path.canonicalize().or_else(|_| {
            // If path doesn't exist (e.g., directory not created yet), use as-is
            Ok::<PathBuf, eyre::Error>(remote_path.clone())
        })?;

        let remote_relative = remote_canonical
            .strip_prefix(workspace_dir)
            .unwrap_or(&remote_canonical);

        // Match if either the paths are equal (file match) or file is under remote dir (directory match)
        let path_matches = file_relative == remote_relative
            || (remote_canonical.is_dir() && file_relative.starts_with(remote_relative));

        if !path_matches {
            continue;
        }

        // Helper function to update a single remote entry
        let update_entry = |entry: &mut Item| -> Result<bool> {
            // Check if this entry matches the URL
            let matches_url = if let Some(url_str) = entry.as_str() {
                // Simple string value
                url_str == remote_url
            } else if let Some(inline_table) = entry.as_inline_table() {
                // Inline table with url field
                inline_table.get("url").and_then(|v| v.as_str()) == Some(remote_url)
            } else if let Some(table) = entry.as_table() {
                // Regular table with url field
                table.get("url").and_then(|v| v.as_str()) == Some(remote_url)
            } else {
                false
            };

            if !matches_url {
                return Ok(false);
            }

            // Found a match! Update or remove watch
            if let Some(inline_table) = entry.as_inline_table_mut() {
                // It's an inline table
                if let Some(id) = &watch_id {
                    // Adding/updating watch ID
                    inline_table.insert("watch", id.as_str().into());
                } else {
                    // Removing watch ID - convert back to plain string
                    if let Some(url_value) = inline_table.get("url") {
                        if let Some(url_str) = url_value.as_str() {
                            *entry = value(url_str);
                        } else {
                            // If URL is not a string, just remove watch field
                            inline_table.remove("watch");
                        }
                    } else {
                        // No URL field, just remove watch
                        inline_table.remove("watch");
                    }
                }
            } else if let Some(table) = entry.as_table_mut() {
                // It's a regular table
                if let Some(id) = &watch_id {
                    // Adding/updating watch ID
                    table["watch"] = value(id.as_str());
                } else {
                    // Removing watch ID - convert back to plain string
                    if let Some(url_item) = table.get("url") {
                        if let Some(url_str) = url_item.as_str() {
                            *entry = value(url_str);
                        } else {
                            // If URL is not a string, just remove watch field
                            table.remove("watch");
                        }
                    } else {
                        // No URL field, just remove watch
                        table.remove("watch");
                    }
                }
            } else if let Some(url_str) = entry.as_str() {
                // It's a string, need to convert to inline table with watch
                if let Some(id) = &watch_id {
                    let mut new_table = InlineTable::new();
                    new_table.insert("url", url_str.into());
                    new_table.insert("watch", id.as_str().into());
                    *entry = value(new_table);
                }
                // If removing watch from a string entry, nothing to do (already a string)
            }
            Ok(true)
        };

        // The value can be:
        // 1. A simple string
        // 2. An inline table { url = "...", watch = "..." }
        // 3. An array of strings/tables
        if let Some(array) = remote_value.as_array_mut() {
            // Array of targets - array contains Values which we need to convert to Items
            for i in 0..array.len() {
                if let Some(item) = array.get_mut(i) {
                    // Convert Value to Item for the update
                    let mut item_wrapper = Item::Value(item.clone());
                    if update_entry(&mut item_wrapper)? {
                        // Update the array element with the modified value
                        if let Item::Value(updated_val) = item_wrapper {
                            *item = updated_val;
                        }
                        found = true;
                        break;
                    }
                }
            }
            // Format array for multi-line readability
            format_array_multiline(array);
        } else {
            // Single target
            if update_entry(remote_value)? {
                found = true;
            }
        }

        if found {
            break;
        }
    }

    if !found {
        return Err(eyre!(
            "No remote found matching path '{}' (workspace-relative: '{}') and URL '{}'",
            file_path.display(),
            file_relative.display(),
            remote_url
        ));
    }

    // Write back to file, preserving formatting
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Add a remote URL to the [remotes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Adds or updates the remote entry for the given path
/// 3. If the path already has remotes, converts to array and appends
/// 4. Avoids duplicates - does nothing if URL already exists
///
/// Returns the path to the modified config file.
pub fn config_add_remote(file_path: &Path, remote_url: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Canonicalize file_path first to get absolute path
    let file_path = file_path.canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
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

    // Ensure [remotes] table exists
    if doc.get("remotes").is_none() {
        doc["remotes"] = Item::Table(Table::new());
    }

    let remotes_table = doc
        .get_mut("remotes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("remotes field is not a table")?;

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Make file_path workspace-relative (file_path is already canonicalized)
    let file_relative = file_path.strip_prefix(workspace_dir).unwrap_or(&file_path);

    let path_key = file_relative.to_string_lossy().to_string();

    // Check if entry already exists for this path
    if let Some(existing) = remotes_table.get_mut(&path_key) {
        // Check if URL already exists - if so, do nothing
        let url_exists = if let Some(url_str) = existing.as_str() {
            url_str == remote_url
        } else if let Some(inline_table) = existing.as_inline_table() {
            inline_table.get("url").and_then(|v| v.as_str()) == Some(remote_url)
        } else if let Some(array) = existing.as_array() {
            array.iter().any(|item| {
                if let Some(url_str) = item.as_str() {
                    url_str == remote_url
                } else if let Some(inline_table) = item.as_inline_table() {
                    inline_table.get("url").and_then(|v| v.as_str()) == Some(remote_url)
                } else {
                    false
                }
            })
        } else {
            false
        };

        if url_exists {
            // URL already exists, nothing to do
            return Ok(config_path);
        }

        // URL doesn't exist - need to add it
        if let Some(array) = existing.as_array_mut() {
            // Already an array, append
            array.push(remote_url);
        } else if let Some(old_value) = existing.as_value().cloned() {
            // Single value - convert to array
            let mut new_array = toml_edit::Array::new();
            new_array.push(old_value);
            new_array.push(remote_url);
            *existing = Item::Value(toml_edit::Value::Array(new_array));
        }
    } else {
        // No entry for this path - add simple string
        remotes_table[&path_key] = value(remote_url);
    }

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Set a spread remote configuration in the [remotes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Replaces any existing entry for the path with the spread config
///
/// Returns the path to the modified config file.
pub fn config_set_remote_spread(file_path: &Path, spread: &crate::RemoteSpread) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Canonicalize file_path first to get absolute path
    let file_path = file_path.canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
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

    // Ensure [remotes] table exists
    if doc.get("remotes").is_none() {
        doc["remotes"] = Item::Table(Table::new());
    }

    let remotes_table = doc
        .get_mut("remotes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("remotes field is not a table")?;

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Make file_path workspace-relative (file_path is already canonicalized)
    let file_relative = file_path.strip_prefix(workspace_dir).unwrap_or(&file_path);

    let path_key = file_relative.to_string_lossy().to_string();

    // Build the spread config as an inline table
    let mut spread_table = InlineTable::new();
    spread_table.insert("service", spread.service.as_str().into());

    if let Some(ref title) = spread.title {
        spread_table.insert("title", title.as_str().into());
    }

    if let Some(ref spread_mode) = spread.spread {
        let spread_mode = match spread_mode {
            crate::SpreadMode::Grid => "grid",
            crate::SpreadMode::Zip => "zip",
        };
        spread_table.insert("spread", spread_mode.into());
    }

    // Build arguments as an inline table
    if !spread.arguments.is_empty() {
        let mut arguments_table = InlineTable::new();
        for (key, values) in &spread.arguments {
            let mut arr = toml_edit::Array::new();
            for v in values {
                arr.push(v.as_str());
            }
            arguments_table.insert(key.as_str(), toml_edit::Value::Array(arr));
        }
        spread_table.insert("arguments", toml_edit::Value::InlineTable(arguments_table));
    }

    // Check if there's an existing entry for this path
    // Create Value version for array operations, Item version for direct assignment
    let spread_value_for_array = toml_edit::Value::InlineTable(spread_table.clone());
    let spread_item = value(spread_table);

    if let Some(existing) = remotes_table
        .get(&path_key)
        .and_then(|item| item.as_value())
    {
        // Determine if existing is an array, URL string, or inline table
        if let Some(arr) = existing.as_array() {
            // Existing is already an array - find and replace any Spread, or append
            let mut new_arr = toml_edit::Array::new();
            let mut found_spread = false;

            for item in arr.iter() {
                // Check if this item is a spread config (inline table with "service" key)
                if let Some(tbl) = item.as_inline_table()
                    && tbl.contains_key("service")
                {
                    // Check if this spread is for the same service
                    let same_service = tbl
                        .get("service")
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s == spread.service);

                    if same_service {
                        // Replace existing spread for same service with new one
                        if !found_spread {
                            new_arr.push(spread_value_for_array.clone());
                            found_spread = true;
                        }
                        // Skip this spread (replaced or duplicate)
                        continue;
                    }
                    // Keep spreads for other services
                }
                // Keep non-spread items (URLs, watches) and spreads for other services
                new_arr.push(item.clone());
            }

            // If no existing spread was found, append the new one
            if !found_spread {
                new_arr.push(spread_value_for_array);
            }

            format_array_multiline(&mut new_arr);
            remotes_table[&path_key] = Item::Value(toml_edit::Value::Array(new_arr));
        } else {
            // Existing is a single value (URL string or inline table)
            // Check if it's already a spread config
            if let Some(tbl) = existing.as_inline_table()
                && tbl.contains_key("service")
            {
                // Existing is a spread - check if same service
                let same_service = tbl
                    .get("service")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s == spread.service);

                if same_service {
                    // Same service - just replace it
                    remotes_table[&path_key] = spread_item;
                } else {
                    // Different service - convert to array with both spreads
                    let mut arr = toml_edit::Array::new();
                    arr.push(existing.clone());
                    arr.push(spread_value_for_array);
                    format_array_multiline(&mut arr);
                    remotes_table[&path_key] = Item::Value(toml_edit::Value::Array(arr));
                }
            } else {
                // Existing is a URL string or watch config - convert to array
                let mut arr = toml_edit::Array::new();
                arr.push(existing.clone());
                arr.push(spread_value_for_array);
                format_array_multiline(&mut arr);
                remotes_table[&path_key] = Item::Value(toml_edit::Value::Array(arr));
            }
        }
    } else {
        // No existing entry - just set the spread config
        remotes_table[&path_key] = spread_item;
    }

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Add a route to the [routes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Adds the route entry mapping route path to file path
/// 3. Avoids duplicates - does nothing if route already exists
///
/// Returns the path to the modified config file.
pub fn config_add_route(file_path: &Path, route: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Canonicalize file_path first to get absolute path
    let file_path = file_path.canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
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

    // Ensure [routes] table exists
    if doc.get("routes").is_none() {
        doc["routes"] = Item::Table(Table::new());
    }

    let routes_table = doc
        .get_mut("routes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("routes field is not a table")?;

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Make file_path workspace-relative (file_path is already canonicalized)
    let file_relative = file_path.strip_prefix(workspace_dir).unwrap_or(&file_path);
    let file_relative_str = file_relative.to_string_lossy().to_string();

    // Check if route already exists
    if let Some(existing) = routes_table.get(route) {
        // Check if it's the same file path
        if let Some(existing_file) = existing.as_str()
            && existing_file == file_relative_str
        {
            // Route already exists with same file, nothing to do
            return Ok(config_path);
        }
        // Route exists but points to different file or is a different type
        // We'll update it to the new file
    }

    // Set the route to the file path
    routes_table[route] = value(&file_relative_str);

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Update the watch ID for a site in the [site] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (starting from the given path)
/// 2. Updates the site.watch field with the watch ID
/// 3. If watch_id is None, removes the watch field
///
/// Returns the path to the modified config file.
pub fn config_update_site_watch(path: &Path, watch_id: Option<String>) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Find the nearest stencila.toml starting from the path
    let search_dir = if path.is_file() {
        path.parent().ok_or_eyre("File has no parent directory")?
    } else {
        path
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .ok_or_else(|| eyre!("No `{CONFIG_FILENAME}` found"))?;

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    // Ensure [site] table exists
    if doc.get("site").is_none() {
        return Err(eyre!(
            "No [site] section in `{CONFIG_FILENAME}`. Create a site first with `stencila site create`."
        ));
    }

    let site_table = doc
        .get_mut("site")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site field is not a table")?;

    if let Some(id) = watch_id {
        site_table["watch"] = value(&id);
    } else {
        site_table.remove("watch");
    }

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Set a spread route configuration in the [routes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Adds or replaces the spread route entry
///
/// Returns the path to the modified config file.
pub fn config_set_route_spread(
    file_path: &Path,
    route_template: &str,
    spread: &crate::RouteSpread,
) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Canonicalize file_path first to get absolute path
    let file_path = file_path.canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
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

    // Ensure [routes] table exists
    if doc.get("routes").is_none() {
        doc["routes"] = Item::Table(Table::new());
    }

    let routes_table = doc
        .get_mut("routes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("routes field is not a table")?;

    // Build the spread config as an inline table
    let mut spread_table = InlineTable::new();
    spread_table.insert("file", spread.file.as_str().into());

    if let Some(ref spread_mode) = spread.spread {
        let spread_mode_str = match spread_mode {
            crate::SpreadMode::Grid => "grid",
            crate::SpreadMode::Zip => "zip",
        };
        spread_table.insert("spread", spread_mode_str.into());
    }

    // Build arguments as an inline table
    if !spread.arguments.is_empty() {
        let mut arguments_table = InlineTable::new();
        for (key, values) in &spread.arguments {
            let mut arr = toml_edit::Array::new();
            for v in values {
                arr.push(v.as_str());
            }
            arguments_table.insert(key.as_str(), toml_edit::Value::Array(arr));
        }
        spread_table.insert("arguments", toml_edit::Value::InlineTable(arguments_table));
    }

    // Set the route to the spread config
    routes_table[route_template] = value(spread_table);

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}
