use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, Result, bail, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use toml_edit::{DocumentMut, InlineTable, Item, Table, value};
use url::Url;

use crate::{SpreadMode, WATCH_ID_REGEX, find_config_file, utils::format_array_multiline};

/// Value for a remote configuration entry - can be single or multiple targets
///
/// Supports both simple cases (one URL) and complex cases (multiple URLs per path).
/// Each target can be a simple URL string or an object with a watch ID.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RemoteValue {
    /// Multiple remote targets for the same path
    ///
    /// Example in TOML:
    /// ```toml
    /// [remotes]
    /// "article.md" = [
    ///   { url = "https://docs.google.com/...", watch = "w456" },
    ///   "https://sharepoint.com/..."
    /// ]
    /// ```
    Multiple(Vec<RemoteTarget>), // Keep first for correct deserialization of multiple remotes

    /// Single remote target
    ///
    /// Example in TOML:
    /// ```toml
    /// [remotes]
    /// "site" = "https://example.stencila.site/"
    /// "file.md" = { url = "https://...", watch = "w123" }
    /// ```
    Single(RemoteTarget),
}

impl RemoteValue {
    /// Convert to a vector of targets, flattening single or multiple variants
    pub fn to_vec(&self) -> Vec<&RemoteTarget> {
        match self {
            RemoteValue::Single(target) => vec![target],
            RemoteValue::Multiple(targets) => targets.iter().collect(),
        }
    }

    /// Find the watch ID for a specific URL, if it exists
    pub fn find_watch(&self, url: &str) -> Option<&str> {
        for target in self.to_vec() {
            if target.url() == Some(url) {
                return target.watch();
            }
        }
        None
    }

    /// Validate the remote value configuration
    ///
    /// Ensures that:
    /// - Each target is valid
    /// - Multiple targets array is not empty
    pub fn validate(&self, path: &str) -> Result<()> {
        match self {
            RemoteValue::Single(target) => target.validate(path)?,
            RemoteValue::Multiple(targets) => {
                if targets.is_empty() {
                    bail!("Remote for path '{}' has an empty array of targets", path);
                }
                for target in targets {
                    target.validate(path)?;
                }
            }
        }
        Ok(())
    }
}

/// A remote synchronization target
///
/// Can be:
/// - A simple URL string (for remotes without watch IDs)
/// - An object with URL and watch ID
/// - A spread configuration for multi-variant pushes
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RemoteTarget {
    /// Simple URL string (no watch)
    ///
    /// Example: `"https://example.stencila.site/"`
    Url(Url),

    /// URL with watch information
    ///
    /// Example: `{ url = "https://...", watch = "w123" }`
    Watch(RemoteWatch),

    /// Spread configuration for multi-variant pushes
    ///
    /// Example: `{ service = "gdoc", title = "Report {region}", arguments = { region = ["north", "south"] } }`
    Spread(RemoteSpread),
}

impl RemoteTarget {
    /// Get the URL from this target as a string slice (if it has one)
    ///
    /// Returns None for Spread targets which have a service instead of a URL.
    pub fn url(&self) -> Option<&str> {
        match self {
            RemoteTarget::Url(url) => Some(url.as_str()),
            RemoteTarget::Watch(watch) => Some(watch.url.as_str()),
            RemoteTarget::Spread(_) => None,
        }
    }

    /// Get the URL from this target as an owned Url (if it has one)
    ///
    /// Returns None for Spread targets which have a service instead of a URL.
    pub fn url_owned(&self) -> Option<Url> {
        match self {
            RemoteTarget::Url(url) => Some(url.clone()),
            RemoteTarget::Watch(watch) => Some(watch.url.clone()),
            RemoteTarget::Spread(_) => None,
        }
    }

    /// Get the watch ID if this target has one
    pub fn watch(&self) -> Option<&str> {
        match self {
            RemoteTarget::Url(_) | RemoteTarget::Spread(_) => None,
            RemoteTarget::Watch(watch) => watch.watch.as_deref(),
        }
    }

    /// Get the spread configuration if this is a spread target
    pub fn spread(&self) -> Option<&RemoteSpread> {
        match self {
            RemoteTarget::Spread(spread) => Some(spread),
            _ => None,
        }
    }

    /// Check if this is a spread target
    pub fn is_spread(&self) -> bool {
        matches!(self, RemoteTarget::Spread(_))
    }

    /// Validate the remote value configuration
    ///
    /// Ensures that:
    /// - URL targets have non-empty URLs
    /// - Watch IDs match the required pattern
    /// - Spread targets have a non-empty service
    /// - Multiple targets array is not empty
    pub fn validate(&self, path: &str) -> Result<()> {
        match self {
            RemoteTarget::Url(url) => {
                if url.as_str().is_empty() {
                    bail!("Remote for path `{path}` has an empty URL");
                }
            }
            RemoteTarget::Watch(watch) => {
                if watch.url.as_str().is_empty() {
                    bail!("Remote for path `{path}` has an empty URL");
                }
                if let Some(watch_id) = &watch.watch
                    && !WATCH_ID_REGEX.is_match(watch_id)
                {
                    bail!(
                        "Invalid watch ID `{watch_id}` for remote `{path}`: must match pattern 'wa' followed by 10 lowercase alphanumeric characters (e.g., 'wa3x9k2m7fab')"
                    );
                }
            }
            RemoteTarget::Spread(spread) => {
                if spread.service.is_empty() {
                    bail!("Spread remote for path `{path}` has an empty service");
                }
                if spread.arguments.is_empty() {
                    bail!("Spread remote for path `{path}` has no `params`");
                }
            }
        }

        Ok(())
    }
}

/// Remote synchronization information with watch ID
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemoteWatch {
    /// Remote URL
    ///
    /// The service type is inferred from the URL host:
    /// - Google Docs: https://docs.google.com/document/d/...
    /// - Microsoft 365: https://*.sharepoint.com/...
    /// - Stencila Sites: https://*.stencila.site/...
    pub url: Url,

    /// Watch ID from Stencila Cloud
    ///
    /// If this remote is being watched for automatic synchronization, this
    /// field contains the watch ID. Watch configuration (direction, PR mode,
    /// debounce) is stored in Stencila Cloud and queried via the API.
    #[schemars(regex(pattern = r"^wa[a-z0-9]{10}$"))]
    pub watch: Option<String>,
}

/// Spread configuration for multi-variant pushes
///
/// Used in `[remotes]` to configure spread pushing of a document to multiple
/// remote variants with different parameter values.
///
/// Example:
/// ```toml
/// [remotes]
/// "report.smd" = { service = "gdoc", title = "Report {region}", arguments = { region = ["north", "south"] } }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemoteSpread {
    /// Target service
    ///
    /// One of: "gdoc", "m365"
    pub service: String,

    /// Title template with placeholders
    ///
    /// Placeholders like `{param}` are replaced with arguments.
    /// Example: "Report - {region}"
    pub title: Option<String>,

    /// Spread mode
    ///
    /// - `grid`: Cartesian product of all arguments (default)
    /// - `zip`: Positional pairing of values (all params must have same length)
    pub spread: Option<SpreadMode>,

    /// Arguments for spread variants
    ///
    /// Keys are parameter names, values are arrays of possible values.
    /// Example: `{ region = ["north", "south"], species = ["A", "B"] }`
    pub arguments: HashMap<String, Vec<String>>,
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
pub fn config_set_remote_spread(
    file_path: &Path,
    spread: &super::remotes::RemoteSpread,
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
