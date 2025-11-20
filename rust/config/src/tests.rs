use std::{fs, path::Path};

use eyre::{Result, eyre};
use figment::{
    Figment,
    providers::Serialized,
    value::{Map, Value},
};
use serial_test::serial;
use tempfile::TempDir;

use crate::{
    CONFIG_FILENAME, CONFIG_LOCAL_FILENAME, Config, find_config_file,
    utils::{
        ConfigTarget, collect_config_paths, config_set, config_unset, config_update_remote_watch,
        config_value, normalize_path,
    },
};

/// Test-only helper that excludes user config for test isolation
fn config_isolated(path: &Path) -> Result<Config> {
    let start_path = normalize_path(path)?;
    // Exclude user config to make tests deterministic
    let config_paths = collect_config_paths(&start_path, false)?;

    let mut figment = Figment::new();

    for config_path in &config_paths {
        if !config_path.exists() {
            continue;
        }

        match fs::read_to_string(config_path) {
            Ok(contents) => match toml::from_str::<Map<String, Value>>(&contents) {
                Ok(data) => {
                    figment = figment.merge(Serialized::defaults(data));
                }
                Err(error) => {
                    tracing::warn!(
                        "Skipping malformed config file {}: {}",
                        config_path.display(),
                        error
                    );
                }
            },
            Err(error) => {
                tracing::warn!(
                    "Failed to read config file {}: {}",
                    config_path.display(),
                    error
                );
            }
        }
    }

    figment.extract().map_err(|error| eyre!(error))
}

#[test]
fn test_config_simple() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a simple config file
    let config_content = r#"
[site]
id = "test1234"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // Test loading the config (isolated from user config)
    let cfg = config_isolated(temp_dir.path())?;

    assert_eq!(
        cfg.site.and_then(|site| site.id),
        Some("test1234".to_string())
    );

    Ok(())
}

#[test]
fn test_config_hierarchical_merge() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let child_dir = temp_dir.path().join("child");
    fs::create_dir_all(&child_dir)?;

    // Parent config
    let parent_config = r#"
[site]
id = "parent99"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), parent_config)?;

    // Child config (should override parent)
    let child_config = r#"
[site]
id = "child123"
"#;
    fs::write(child_dir.join(CONFIG_FILENAME), child_config)?;

    // Test loading from child - should get child's value (isolated from user config)
    let cfg = config_isolated(&child_dir)?;

    assert_eq!(
        cfg.site.and_then(|site| site.id),
        Some("child123".to_string()),
        "Child config should override parent"
    );

    Ok(())
}

#[test]
fn test_config_local_override() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Regular config
    let regular_config = r#"
[site]
id = "regular1"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), regular_config)?;

    // Local override
    let local_config = r#"
[site]
id = "local999"
"#;
    fs::write(temp_dir.path().join(CONFIG_LOCAL_FILENAME), local_config)?;

    // Test loading - local should override regular (isolated from user config)
    let cfg = config_isolated(temp_dir.path())?;

    assert_eq!(
        cfg.site.and_then(|site| site.id),
        Some("local999".to_string()),
        "Local config should override regular"
    );

    Ok(())
}

#[test]
fn test_config_missing_files_ok() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Try to load config - should succeed with empty config
    // Use isolated version to exclude user config
    let cfg = config_isolated(temp_dir.path())?;

    // Should succeed with an empty config (all fields None)
    assert!(
        cfg.site.is_none(),
        "Config should be empty when no config files exist"
    );

    Ok(())
}

#[test]
fn test_config_malformed_file_skipped() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let child_dir = temp_dir.path().join("child");
    fs::create_dir_all(&child_dir)?;

    // Parent config - MALFORMED (invalid TOML syntax)
    let malformed_config = r#"
[site]
id = "parent99"
this is not valid toml: [ { missing closing brackets
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), malformed_config)?;

    // Child config - VALID (should still load despite parent being malformed)
    let valid_config = r#"
[site]
id = "child123"
"#;
    fs::write(child_dir.join(CONFIG_FILENAME), valid_config)?;

    // Test loading from child - should succeed with child's config (isolated from user config)
    let cfg = config_isolated(&child_dir)?;

    assert_eq!(
        cfg.site.and_then(|site| site.id),
        Some("child123".to_string()),
        "Valid child config should load despite malformed parent config"
    );

    Ok(())
}

#[test]
fn test_config_all_malformed_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create only malformed config files
    let malformed_config = r#"
this is = { not valid toml [
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), malformed_config)?;
    fs::write(
        temp_dir.path().join(CONFIG_LOCAL_FILENAME),
        malformed_config,
    )?;

    // Try to load config - should succeed with empty config
    // Use isolated version to exclude user config
    let cfg = config_isolated(temp_dir.path())?;

    // Should succeed with empty config when all config files are malformed
    assert!(
        cfg.site.is_none(),
        "Config should be empty when all config files are malformed"
    );
    Ok(())
}

#[test]
fn test_config_value_get_simple() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let config_content = r#"
[site]
id = "test1234"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // Test getting a nested value
    let value = config_value(temp_dir.path(), "site.id")?;
    assert!(value.is_some());

    // Deserialize the value to a string
    if let Some(val) = value {
        let as_string: String = val.deserialize()?;
        assert_eq!(as_string, "test1234");
    }

    Ok(())
}

#[test]
fn test_config_value_missing_key() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let config_content = r#"
[site]
id = "test1234"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // Test getting a non-existent value
    let value = config_value(temp_dir.path(), "nonexistent.key")?;
    assert!(value.is_none());

    Ok(())
}

#[test]
fn test_find_config_file_finds_nearest() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let child_dir = temp_dir.path().join("child");
    fs::create_dir_all(&child_dir)?;

    // Create config in parent
    fs::write(
        temp_dir.path().join(CONFIG_FILENAME),
        "[site]\nid = \"parent\"",
    )?;

    // Search from child should find parent's config
    let found = find_config_file(&child_dir, CONFIG_FILENAME);
    assert_eq!(found, Some(temp_dir.path().join(CONFIG_FILENAME)));

    Ok(())
}

#[test]
fn test_find_config_file_returns_none() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Search for non-existent config
    let found = find_config_file(temp_dir.path(), CONFIG_FILENAME);
    assert!(found.is_none());

    Ok(())
}

#[test]
#[serial]
fn test_config_set_and_get() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Change to temp directory for this test
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Set a simple value
    let config_path = config_set("site.id", "newsite123", ConfigTarget::Nearest)?;
    assert_eq!(config_path, temp_dir.path().join(CONFIG_FILENAME));

    // Verify the file was created and contains the value
    let contents = fs::read_to_string(&config_path)?;
    assert!(contents.contains("[site]"));
    assert!(contents.contains("id = \"newsite123\""));

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_set_nested_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Set a deeply nested value
    config_set("site.settings.theme", "dark", ConfigTarget::Nearest)?;

    // Verify the nested structure was created
    let contents = fs::read_to_string(temp_dir.path().join(CONFIG_FILENAME))?;
    assert!(contents.contains("[site.settings]"));
    assert!(contents.contains("theme = \"dark\""));

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_set_type_inference() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Set different types
    config_set("bool_value", "true", ConfigTarget::Nearest)?;
    config_set("int_value", "42", ConfigTarget::Nearest)?;
    config_set("float_value", "3.14", ConfigTarget::Nearest)?;
    config_set("string_value", "hello", ConfigTarget::Nearest)?;

    let contents = fs::read_to_string(temp_dir.path().join(CONFIG_FILENAME))?;

    // Verify types are preserved (not all strings)
    assert!(contents.contains("bool_value = true"));
    assert!(contents.contains("int_value = 42"));
    assert!(contents.contains("float_value = 3.14"));
    assert!(contents.contains("string_value = \"hello\""));

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_set_local_target() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Set value in local config
    let config_path = config_set("site.id", "local123", ConfigTarget::Local)?;
    assert_eq!(config_path, temp_dir.path().join(CONFIG_LOCAL_FILENAME));

    // Verify the file was created
    assert!(config_path.exists());

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_unset_removes_value() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // First set a value
    config_set("site.id", "test123", ConfigTarget::Nearest)?;

    // Verify it exists
    let contents_before = fs::read_to_string(temp_dir.path().join(CONFIG_FILENAME))?;
    assert!(contents_before.contains("id = \"test123\""));

    // Now unset it
    config_unset("site.id", ConfigTarget::Nearest)?;

    // Verify it was removed
    let contents_after = fs::read_to_string(temp_dir.path().join(CONFIG_FILENAME))?;
    assert!(!contents_after.contains("id = \"test123\""));

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_unset_nonexistent_key_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Create a config file
    config_set("site.id", "test123", ConfigTarget::Nearest)?;

    // Try to unset a non-existent key
    let result = config_unset("nonexistent.key", ConfigTarget::Nearest);
    assert!(result.is_err());

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_config_unset_no_config_file_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Try to unset when no config file exists
    let result = config_unset("site.id", ConfigTarget::Nearest);
    assert!(result.is_err());

    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
fn test_normalize_path_with_directory() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let normalized = normalize_path(&current_dir)?;

    assert!(normalized.is_absolute());
    assert!(normalized.is_dir());

    Ok(())
}

#[test]
fn test_normalize_path_with_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test")?;

    let normalized = normalize_path(&test_file)?;

    assert!(normalized.is_absolute());
    assert!(normalized.is_dir());
    // The normalized path should be the parent directory of the file
    assert_eq!(normalized, temp_dir.path());

    Ok(())
}

#[test]
fn test_collect_config_paths_order() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    // Exclude user config to make test deterministic
    let paths = collect_config_paths(&current_dir, false)?;

    // Should have at least the current directory's configs
    assert!(!paths.is_empty());

    // Check that paths come in correct order:
    // - Configs from root directories should come before deeper ones
    // - At each level, .toml should come before .local.toml

    let current_yaml = current_dir.join(CONFIG_FILENAME);
    let current_local = current_dir.join(CONFIG_LOCAL_FILENAME);

    let yaml_pos = paths.iter().position(|p| p == &current_yaml);
    let local_pos = paths.iter().position(|p| p == &current_local);

    // Both should be in the list
    assert!(yaml_pos.is_some());
    assert!(local_pos.is_some());

    // .toml should come before .local.toml
    if let (Some(yaml), Some(local)) = (yaml_pos, local_pos) {
        assert!(yaml < local);
    }

    Ok(())
}

#[test]
fn test_user_config_includes_local() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    // Include user config to test that both files are present
    let paths = collect_config_paths(&current_dir, true)?;

    // Get user config directory
    if let Ok(user_config_dir) = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false) {
        let user_yaml = user_config_dir.join(CONFIG_FILENAME);
        let user_local = user_config_dir.join(CONFIG_LOCAL_FILENAME);

        // Both user config files should be in the paths
        let yaml_pos = paths.iter().position(|p| p == &user_yaml);
        let local_pos = paths.iter().position(|p| p == &user_local);

        assert!(yaml_pos.is_some(), "User stencila.toml should be included");
        assert!(
            local_pos.is_some(),
            "User stencila.local.toml should be included"
        );

        // .toml should come before .local.toml
        if let (Some(yaml), Some(local)) = (yaml_pos, local_pos) {
            assert!(
                yaml < local,
                "User stencila.toml should come before stencila.local.toml"
            );
        }
    }

    Ok(())
}

#[test]
fn test_config_update_remote_watch_finds_config_from_file_path() -> Result<()> {
    // Create a temporary workspace with a config file and a document
    let workspace = TempDir::new()?;
    let workspace_path = workspace.path();

    // Create a nested directory structure
    let docs_dir = workspace_path.join("docs");
    fs::create_dir_all(&docs_dir)?;

    // Create a stencila.toml in the workspace root
    let config_path = workspace_path.join(CONFIG_FILENAME);
    fs::write(
        &config_path,
        r#"
[[remotes]]
path = "docs/test.md"
url = "https://docs.google.com/document/d/abc123"
"#,
    )?;

    // Create the document file
    let doc_path = docs_dir.join("test.md");
    fs::write(&doc_path, "# Test")?;

    // Change to a completely different directory (simulating running command from outside workspace)
    let original_dir = std::env::current_dir()?;
    let temp_cwd = TempDir::new()?;
    std::env::set_current_dir(temp_cwd.path())?;

    // Try to update watch ID - this should find the config based on file path, not CWD
    let result = config_update_remote_watch(
        &doc_path,
        "https://docs.google.com/document/d/abc123",
        Some("watch_123".to_string()),
    );

    // Restore original directory first before temp_cwd is dropped
    std::env::set_current_dir(&original_dir)?;

    // Keep temp_cwd alive until after we restore directory
    drop(temp_cwd);

    // Verify the update succeeded
    assert!(
        result.is_ok(),
        "Should find config based on file path, not CWD: {:?}",
        result.err()
    );

    // Verify the config was actually updated
    let updated_config = fs::read_to_string(&config_path)?;
    assert!(
        updated_config.contains("watch_123"),
        "Config should contain the watch ID"
    );
    assert!(
        updated_config.contains("watch = \"watch_123\""),
        "Watch field should be set correctly"
    );

    Ok(())
}

#[test]
fn test_config_update_remote_watch_removes_watch_id() -> Result<()> {
    // Create a temporary workspace
    let workspace = TempDir::new()?;
    let workspace_path = workspace.path();

    // Create stencila.toml with an existing watch ID
    let config_path = workspace_path.join(CONFIG_FILENAME);
    fs::write(
        &config_path,
        r#"
[[remotes]]
path = "test.md"
url = "https://docs.google.com/document/d/abc123"
watch = "watch_123"
"#,
    )?;

    // Create the document file
    let doc_path = workspace_path.join("test.md");
    fs::write(&doc_path, "# Test")?;

    // Remove the watch ID by passing None
    config_update_remote_watch(&doc_path, "https://docs.google.com/document/d/abc123", None)?;

    // Verify the watch field was removed
    let updated_config = fs::read_to_string(&config_path)?;
    assert!(
        !updated_config.contains("watch ="),
        "Watch field should be removed: {}",
        updated_config
    );

    Ok(())
}
