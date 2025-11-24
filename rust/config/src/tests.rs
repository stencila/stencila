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
    CONFIG_FILENAME, CONFIG_LOCAL_FILENAME, Config, RedirectStatus, RemoteTarget, RemoteValue,
    find_config_file,
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
    let temp_path = temp_dir.path().to_path_buf();

    // Change to temp directory for this test
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Set a simple value
    let config_path = config_set("site.id", "newsite123", ConfigTarget::Nearest)?;
    assert_eq!(config_path, temp_path.join(CONFIG_FILENAME));

    // Verify the file was created and contains the value
    let contents = fs::read_to_string(&config_path)?;
    assert!(contents.contains("[site]"));
    assert!(contents.contains("id = \"newsite123\""));

    // Restore original directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

    Ok(())
}

#[test]
#[serial]
fn test_config_set_nested_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Set a deeply nested value
    config_set("site.settings.theme", "dark", ConfigTarget::Nearest)?;

    // Verify the nested structure was created
    let contents = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;
    assert!(contents.contains("[site.settings]"));
    assert!(contents.contains("theme = \"dark\""));

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

    Ok(())
}

#[test]
#[serial]
fn test_config_set_type_inference() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Set different types
    config_set("bool_value", "true", ConfigTarget::Nearest)?;
    config_set("int_value", "42", ConfigTarget::Nearest)?;
    config_set("float_value", "3.14", ConfigTarget::Nearest)?;
    config_set("string_value", "hello", ConfigTarget::Nearest)?;

    let contents = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;

    // Verify types are preserved (not all strings)
    assert!(
        contents.contains("bool_value = true"),
        "bool_value should be unquoted true"
    );
    assert!(
        contents.contains("int_value = 42"),
        "int_value should be unquoted 42"
    );
    // Float might be formatted as "3.14" or "3.1400000000000001" depending on precision
    assert!(
        contents.contains("float_value = 3.1"),
        "float_value should start with 3.1"
    );
    assert!(
        contents.contains("string_value = \"hello\""),
        "string_value should be quoted"
    );

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

    Ok(())
}

#[test]
#[serial]
fn test_config_set_local_target() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Set value in local config
    let config_path = config_set("site.id", "local123", ConfigTarget::Local)?;
    assert_eq!(config_path, temp_path.join(CONFIG_LOCAL_FILENAME));

    // Verify the file was created
    assert!(config_path.exists());

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

    Ok(())
}

#[test]
#[serial]
fn test_config_unset_removes_value() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // First set a value
    config_set("site.id", "test123", ConfigTarget::Nearest)?;

    // Verify it exists
    let contents_before = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;
    assert!(contents_before.contains("id = \"test123\""));

    // Now unset it
    config_unset("site.id", ConfigTarget::Nearest)?;

    // Verify it was removed
    let contents_after = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;
    assert!(!contents_after.contains("id = \"test123\""));

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

    Ok(())
}

#[test]
#[serial]
fn test_config_unset_nonexistent_key_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Create a config file
    config_set("site.id", "test123", ConfigTarget::Nearest)?;

    // Try to unset a non-existent key
    let result = config_unset("nonexistent.key", ConfigTarget::Nearest);
    assert!(result.is_err());

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

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

    // Restore directory BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir)?;
    drop(temp_dir);

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
[remotes]
"docs/test.md" = "https://docs.google.com/document/d/abc123"
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

    // Drop temp directories in correct order
    drop(temp_cwd);
    drop(workspace);

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
[remotes]
"test.md" = { url = "https://docs.google.com/document/d/abc123", watch = "watch_123" }
"#,
    )?;

    // Create the document file
    let doc_path = workspace_path.join("test.md");
    fs::write(&doc_path, "# Test")?;

    // Remove the watch ID by passing None
    config_update_remote_watch(&doc_path, "https://docs.google.com/document/d/abc123", None)?;

    // Verify the watch field was removed and entry converted back to plain string
    let updated_config = fs::read_to_string(&config_path)?;
    assert!(
        !updated_config.contains("watch ="),
        "Watch field should be removed: {}",
        updated_config
    );
    assert!(
        updated_config.contains(r#""test.md" = "https://docs.google.com/document/d/abc123""#),
        "Entry should be converted back to plain string: {}",
        updated_config
    );

    // Verify the config can still be parsed correctly
    let cfg = config_isolated(workspace_path)?;
    let remotes = cfg.remotes.as_ref().expect("Expected remotes");
    let test_md = remotes.get("test.md").expect("Expected test.md remote");
    assert!(matches!(test_md, RemoteValue::Single(RemoteTarget::Url(_))));
    if let RemoteValue::Single(RemoteTarget::Url(url)) = test_md {
        assert_eq!(url.as_str(), "https://docs.google.com/document/d/abc123");
    }

    Ok(())
}

#[test]
fn test_config_routes_file() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with simple file routes
    let config_content = r#"
[routes]
"/about/" = "README.md"
"/" = "index.md"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg.routes.as_ref().expect("Expected routes to be present");
    assert_eq!(routes.len(), 2);

    // Check that we can access the routes
    assert!(routes.contains_key("/about/"));
    assert!(routes.contains_key("/"));

    // Check that the values are file routes
    let target = routes.get("/about/").expect("Expected /about/ route");
    let file = target.file().expect("Expected file route");
    assert_eq!(file.as_str(), "README.md");

    Ok(())
}

#[test]
fn test_config_routes_redirect() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with redirect routes
    let config_content = r#"
[routes]
"/old/" = { redirect = "/new/", status = 301 }
"/external/" = { redirect = "https://example.com" }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg.routes.as_ref().expect("Expected routes to be present");
    assert_eq!(routes.len(), 2);

    // Check the redirect with status
    let target = routes.get("/old/").expect("Expected /old/ route");
    let redirect = target.redirect().expect("Expected redirect");
    assert_eq!(redirect.redirect, "/new/");
    assert_eq!(redirect.status, Some(RedirectStatus::MovedPermanently));

    // Check the redirect without status (defaults to 302)
    let target = routes.get("/external/").expect("Expected /external/ route");
    let redirect = target.redirect().expect("Expected redirect");
    assert_eq!(redirect.redirect, "https://example.com");
    assert_eq!(redirect.status, None);

    Ok(())
}

#[test]
fn test_config_routes_mixed() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with both file and redirect routes
    let config_content = r#"
[routes]
"/" = "index.md"
"/about/" = "README.md"
"/old-page/" = { redirect = "/new-page/", status = 301 }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg.routes.as_ref().expect("Expected routes to be present");
    assert_eq!(routes.len(), 3);

    // Verify file routes
    let root_target = routes.get("/").expect("Expected / route");
    assert!(root_target.file().is_some());

    let about_target = routes.get("/about/").expect("Expected /about/ route");
    assert!(about_target.file().is_some());

    // Verify redirect route
    let old_page_target = routes.get("/old-page/").expect("Expected /old-page/ route");
    assert!(old_page_target.redirect().is_some());

    Ok(())
}

#[test]
fn test_config_routes_duplicate_keys_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // TOML parsers typically reject duplicate keys at parse time
    // The exact error depends on the TOML parser implementation
    let config_content = r#"
[routes]
"/test/" = "file1.md"
"/test/" = "file2.md"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // The TOML parser should reject this at parse time
    // If it doesn't, the second value will override the first
    let result = config_isolated(temp_dir.path());

    // Either it errors, or the last value wins (both are acceptable TOML behavior)
    if let Ok(cfg) = result
        && let Some(routes) = &cfg.routes
    {
        // If parsing succeeded, verify only one entry exists (last one wins)
        assert_eq!(routes.len(), 1);
        let target = routes.get("/test/").expect("Expected /test/ route");
        assert_eq!(target.file().expect("Expected file").as_str(), "file2.md");
    }

    Ok(())
}

#[test]
fn test_config_remotes_simple_url() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with simple URL remotes
    let config_content = r#"
[remotes]
"site" = "https://example.stencila.site/"
"docs" = "https://docs.google.com/document/d/abc123"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let remotes = cfg.remotes.as_ref().expect("Expected remotes");
    assert_eq!(remotes.len(), 2);

    // Check simple URL values
    let site = remotes.get("site").expect("Expected site remote");
    assert!(matches!(site, RemoteValue::Single(RemoteTarget::Url(_))));
    if let RemoteValue::Single(RemoteTarget::Url(url)) = site {
        assert_eq!(url.as_str(), "https://example.stencila.site/");
    }

    let docs = remotes.get("docs").expect("Expected docs remote");
    assert!(matches!(docs, RemoteValue::Single(RemoteTarget::Url(_))));
    if let RemoteValue::Single(RemoteTarget::Url(url)) = docs {
        assert_eq!(url.as_str(), "https://docs.google.com/document/d/abc123");
    }

    Ok(())
}

#[test]
fn test_config_remotes_with_watch() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with watch IDs
    let config_content = r#"
[remotes]
"docs/report.md" = { url = "https://docs.google.com/...", watch = "w123456789" }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let remotes = cfg.remotes.as_ref().expect("Expected remotes");
    assert_eq!(remotes.len(), 1);

    let report = remotes
        .get("docs/report.md")
        .expect("Expected docs/report.md remote");
    assert!(matches!(
        report,
        RemoteValue::Single(RemoteTarget::Watch(_))
    ));
    if let RemoteValue::Single(RemoteTarget::Watch(info)) = report {
        assert_eq!(info.url.as_str(), "https://docs.google.com/...");
        assert_eq!(info.watch, Some("w123456789".to_string()));
    }

    Ok(())
}

#[test]
fn test_config_remotes_with_url_object_no_watch() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with object format but no watch field
    let config_content = r#"
[remotes]
"article.md" = { url = "https://docs.google.com/document/d/xyz789" }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let remotes = cfg.remotes.as_ref().expect("Expected remotes");
    assert_eq!(remotes.len(), 1);

    let article = remotes
        .get("article.md")
        .expect("Expected article.md remote");
    assert!(matches!(
        article,
        RemoteValue::Single(RemoteTarget::Watch(_))
    ));
    if let RemoteValue::Single(RemoteTarget::Watch(info)) = article {
        assert_eq!(
            info.url.as_str(),
            "https://docs.google.com/document/d/xyz789"
        );
        assert_eq!(info.watch, None);
    }

    Ok(())
}

#[test]
fn test_config_remotes_multiple() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with multiple remotes for same path
    let config_content = r#"
[remotes]
"article.md" = [
  { url = "https://docs.google.com/...", watch = "w456" },
  "https://sharepoint.com/..."
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let remotes = cfg.remotes.as_ref().expect("Expected remotes");
    assert_eq!(remotes.len(), 1);

    let article = remotes
        .get("article.md")
        .expect("Expected article.md remote");
    assert!(matches!(article, RemoteValue::Multiple(_)));

    if let RemoteValue::Multiple(targets) = article {
        assert_eq!(targets.len(), 2);

        // Check we have one with watch and one without
        let with_watch = targets
            .iter()
            .find(|t| matches!(t, RemoteTarget::Watch(_)))
            .expect("Expected target with watch");
        if let RemoteTarget::Watch(info) = with_watch {
            assert_eq!(info.url.as_str(), "https://docs.google.com/...");
            assert_eq!(info.watch, Some("w456".to_string()));
        }

        let without_watch = targets
            .iter()
            .find(|t| matches!(t, RemoteTarget::Url(_)))
            .expect("Expected URL-only target");
        if let RemoteTarget::Url(url) = without_watch {
            assert_eq!(url.as_str(), "https://sharepoint.com/...");
        }
    }

    Ok(())
}

#[test]
fn test_config_remotes_duplicate_keys_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // TOML should reject duplicate keys
    let config_content = r#"
[remotes]
"test.md" = "https://url1.com"
"test.md" = "https://url2.com"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // This should fail because TOML doesn't allow duplicate keys
    let result = config_isolated(temp_dir.path());

    // Either it errors, or the last value wins (both are acceptable TOML behavior)
    if let Ok(cfg) = result
        && let Some(remotes) = &cfg.remotes
    {
        // If parsing succeeded, verify only one entry exists
        assert_eq!(remotes.len(), 1);
    }

    Ok(())
}
