use std::{fs, path::Path};

use eyre::{Result, eyre};
use figment::{
    Figment,
    providers::Serialized,
    value::{Map, Value},
};
use serial_test::serial;
use tempfile::TempDir;

use crate::config_update_remote_watch;

use super::{
    CONFIG_FILENAME, CONFIG_LOCAL_FILENAME, Config, find_config_file,
    remotes::{RemoteTarget, RemoteValue},
    site::RedirectStatus,
    utils::{ConfigTarget, collect_paths, normalize_path, set_value, unset_value},
};

/// Test-only helper that excludes user config for test isolation
fn config_isolated(path: &Path) -> Result<Config> {
    let start_path = normalize_path(path)?;
    // Exclude user config to make tests deterministic
    let config_paths = collect_paths(&start_path, false)?;

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

    let config: Config = figment.extract().map_err(|error| eyre!(error))?;

    // Validate workspace configuration
    if let Some(workspace) = &config.workspace {
        workspace.validate()?;
    }

    // Validate site configuration
    if let Some(site) = &config.site {
        site.validate()?;
    }

    // Validate site navigation items (must be internal routes)
    if let Some(site) = &config.site
        && let Some(nav) = &site.nav
    {
        for item in nav {
            item.validate()?;
        }
    }

    // Validate all route configurations
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
        for (path_key, target) in routes {
            target.validate(path_key)?;
        }
    }

    // Validate all remote configurations
    if let Some(remotes) = &config.remotes {
        for (path_key, value) in remotes {
            value.validate(path_key)?;
        }
    }

    // Validate all output configurations
    if let Some(outputs) = &config.outputs {
        for (path_key, target) in outputs {
            target.validate(path_key)?;
        }
    }

    // Validate models configuration
    if let Some(models) = &config.models {
        models.validate()?;
    }

    Ok(config)
}

#[test]
fn test_config_simple() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a simple config file
    let config_content = r#"
[workspace]
id = "ws1234567890"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // Test loading the config (isolated from user config)
    let cfg = config_isolated(temp_dir.path())?;

    assert_eq!(
        cfg.workspace.and_then(|ws| ws.id),
        Some("ws1234567890".to_string())
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
[workspace]
id = "wsparent9999"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), parent_config)?;

    // Child config (should override parent)
    let child_config = r#"
[workspace]
id = "wschild12345"
"#;
    fs::write(child_dir.join(CONFIG_FILENAME), child_config)?;

    // Test loading from child - should get child's value (isolated from user config)
    let cfg = config_isolated(&child_dir)?;

    assert_eq!(
        cfg.workspace.and_then(|ws| ws.id),
        Some("wschild12345".to_string()),
        "Child config should override parent"
    );

    Ok(())
}

#[test]
fn test_config_local_override() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Regular config
    let regular_config = r#"
[workspace]
id = "wsregular123"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), regular_config)?;

    // Local override
    let local_config = r#"
[workspace]
id = "wslocal99999"
"#;
    fs::write(temp_dir.path().join(CONFIG_LOCAL_FILENAME), local_config)?;

    // Test loading - local should override regular (isolated from user config)
    let cfg = config_isolated(temp_dir.path())?;

    assert_eq!(
        cfg.workspace.and_then(|ws| ws.id),
        Some("wslocal99999".to_string()),
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
[workspace]
id = "wsparent9999"
this is not valid toml: [ { missing closing brackets
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), malformed_config)?;

    // Child config - VALID (should still load despite parent being malformed)
    let valid_config = r#"
[workspace]
id = "wschild12345"
"#;
    fs::write(child_dir.join(CONFIG_FILENAME), valid_config)?;

    // Test loading from child - should succeed with child's config (isolated from user config)
    let cfg = config_isolated(&child_dir)?;

    assert_eq!(
        cfg.workspace.and_then(|ws| ws.id),
        Some("wschild12345".to_string()),
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
fn test_models_providers_config_valid() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_content = r#"
[models]
providers = ["anthropic", "openai"]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;
    let providers = cfg
        .models
        .and_then(|models| models.providers)
        .ok_or_else(|| eyre!("missing providers"))?;
    assert_eq!(providers, vec!["anthropic", "openai"]);

    Ok(())
}

#[test]
fn test_models_providers_config_invalid_provider() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_content = r#"
[models]
providers = ["anthropic", "invalid"]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let result = config_isolated(temp_dir.path());
    assert!(result.is_err());
    let msg = format!("{}", result.expect_err("expected validation error"));
    assert!(msg.contains("Unknown model provider"));
    assert!(msg.contains("invalid"));

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
        "[workspace]\nid = \"wsparent1234\"",
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
    let config_path = set_value("workspace.id", "wsnewsite123", ConfigTarget::Nearest)?;
    assert_eq!(config_path, temp_path.join(CONFIG_FILENAME));

    // Verify the file was created and contains the value
    let contents = fs::read_to_string(&config_path)?;
    assert!(contents.contains("[workspace]"));
    assert!(contents.contains("id = \"wsnewsite123\""));

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
    set_value("site.settings.theme", "dark", ConfigTarget::Nearest)?;

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
    set_value("bool_value", "true", ConfigTarget::Nearest)?;
    set_value("int_value", "42", ConfigTarget::Nearest)?;
    set_value("float_value", "3.14", ConfigTarget::Nearest)?;
    set_value("string_value", "hello", ConfigTarget::Nearest)?;

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
    let config_path = set_value("workspace.id", "wslocal12345", ConfigTarget::Local)?;
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
    set_value("workspace.id", "wstest123456", ConfigTarget::Nearest)?;

    // Verify it exists
    let contents_before = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;
    assert!(contents_before.contains("id = \"wstest123456\""));

    // Now unset it
    unset_value("workspace.id", ConfigTarget::Nearest)?;

    // Verify it was removed
    let contents_after = fs::read_to_string(temp_path.join(CONFIG_FILENAME))?;
    assert!(!contents_after.contains("id = \"wstest123456\""));

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
    set_value("workspace.id", "wstest123456", ConfigTarget::Nearest)?;

    // Try to unset a non-existent key
    let result = unset_value("nonexistent.key", ConfigTarget::Nearest);
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
    let result = unset_value("workspace.id", ConfigTarget::Nearest);
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
    let paths = collect_paths(&current_dir, false)?;

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
fn test_user_config_includes_toml() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    // Include user config to test that the user file is present
    let paths = collect_paths(&current_dir, true)?;

    // Get user config directory
    if let Ok(user_config_dir) = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false) {
        let user_yaml = user_config_dir.join(CONFIG_FILENAME);

        // Both user config files should be in the paths
        let yaml_pos = paths.iter().position(|p| p == &user_yaml);
        assert!(yaml_pos.is_some(), "User stencila.toml should be included");
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
[site.routes]
"/about/" = "README.md"
"/" = "index.md"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg
        .site
        .as_ref()
        .and_then(|s| s.routes.as_ref())
        .expect("Expected routes to be present");
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
[site.routes]
"/old/" = { redirect = "/new/", status = 301 }
"/external/" = { redirect = "https://example.com" }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg
        .site
        .as_ref()
        .and_then(|s| s.routes.as_ref())
        .expect("Expected routes to be present");
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
[site.routes]
"/" = "index.md"
"/about/" = "README.md"
"/old-page/" = { redirect = "/new-page/", status = 301 }
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let routes = cfg
        .site
        .as_ref()
        .and_then(|s| s.routes.as_ref())
        .expect("Expected routes to be present");
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
[site.routes]
"/test/" = "file1.md"
"/test/" = "file2.md"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    // The TOML parser should reject this at parse time
    // If it doesn't, the second value will override the first
    let result = config_isolated(temp_dir.path());

    // Either it errors, or the last value wins (both are acceptable TOML behavior)
    if let Ok(cfg) = result
        && let Some(site) = &cfg.site
        && let Some(routes) = &site.routes
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
"docs/report.md" = { url = "https://docs.google.com/...", watch = "wa1234567890" }
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
        assert_eq!(info.watch, Some("wa1234567890".to_string()));
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
  { url = "https://docs.google.com/...", watch = "wa4567890abc" },
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
            assert_eq!(info.watch, Some("wa4567890abc".to_string()));
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

#[test]
fn test_extract_placeholders() {
    use crate::extract_placeholders;

    // Single placeholder
    assert_eq!(extract_placeholders("{region}/report.pdf"), vec!["region"]);

    // Multiple placeholders
    assert_eq!(
        extract_placeholders("{region}/{species}/report.pdf"),
        vec!["region", "species"]
    );

    // No placeholders
    assert!(extract_placeholders("report.pdf").is_empty());

    // Reserved placeholders
    assert_eq!(extract_placeholders("{tag}/report.pdf"), vec!["tag"]);
    assert_eq!(extract_placeholders("{branch}/report.pdf"), vec!["branch"]);

    // Mixed reserved and regular
    assert_eq!(
        extract_placeholders("{tag}/{region}/report.pdf"),
        vec!["tag", "region"]
    );

    // Empty braces (should be ignored)
    assert!(extract_placeholders("{}/report.pdf").is_empty());
}

#[test]
fn test_validate_placeholders() -> Result<()> {
    use crate::validate_placeholders;
    use std::collections::HashMap;

    // Valid: all placeholders have arguments
    let mut args = HashMap::new();
    args.insert("region".to_string(), vec!["north".to_string()]);
    validate_placeholders("{region}/report.pdf", Some(&args), "Output")?;

    // Valid: reserved placeholder without argument
    validate_placeholders("{tag}/report.pdf", None, "Output")?;
    validate_placeholders("{branch}/report.pdf", None, "Output")?;

    // Valid: mix of reserved and regular placeholders
    let mut args = HashMap::new();
    args.insert("region".to_string(), vec!["north".to_string()]);
    validate_placeholders("{tag}/{region}/report.pdf", Some(&args), "Output")?;

    // Invalid: placeholder without matching argument
    let result = validate_placeholders("{region}/report.pdf", None, "Output");
    assert!(
        result
            .expect_err("expected error for missing placeholder arg")
            .to_string()
            .contains("{region}")
    );

    // Invalid: one placeholder missing from arguments
    let mut args = HashMap::new();
    args.insert("region".to_string(), vec!["north".to_string()]);
    let result = validate_placeholders("{region}/{species}/report.pdf", Some(&args), "Output");
    assert!(
        result
            .expect_err("expected error for missing placeholder arg")
            .to_string()
            .contains("{species}")
    );

    Ok(())
}

/// Test that schemars regex patterns match the constant patterns
///
/// This ensures that if someone updates a pattern in one place,
/// they must also update it in the other, or this test will fail.
#[test]
fn test_pattern_constants_match_schemars() {
    // Generate the JSON schema
    let schema = schemars::schema_for!(crate::Config);
    let schema_json = serde_json::to_value(&schema).expect("Failed to serialize schema");

    // Helper to extract pattern from schema
    fn find_pattern(schema: &serde_json::Value, path: &[&str]) -> Option<String> {
        let mut current = schema.get("$defs")?;
        for &segment in path.iter().take(path.len() - 1) {
            current = current.get(segment)?;
        }
        current
            .get("properties")?
            .get(path.last()?)?
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    // Check workspace.id pattern
    let ws_pattern = find_pattern(&schema_json, &["WorkspaceConfig", "id"]);
    assert_eq!(
        ws_pattern.as_deref(),
        Some(crate::WORKSPACE_ID_PATTERN),
        "WorkspaceConfig.id schemars pattern doesn't match WORKSPACE_ID_PATTERN constant"
    );

    // Check site.domain pattern
    let domain_pattern = find_pattern(&schema_json, &["SiteConfig", "domain"]);
    assert_eq!(
        domain_pattern.as_deref(),
        Some(crate::DOMAIN_PATTERN),
        "SiteConfig.domain schemars pattern doesn't match DOMAIN_PATTERN constant"
    );

    // Check remotes watch pattern
    let watch_pattern = find_pattern(&schema_json, &["RemoteWatch", "watch"]);
    assert_eq!(
        watch_pattern.as_deref(),
        Some(crate::WATCH_ID_PATTERN),
        "RemoteWatch.watch schemars pattern doesn't match WATCH_ID_PATTERN constant"
    );
}

#[test]
fn test_nav_item_route_shorthand() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with route shorthand nav items
    let config_content = r#"
[site]
nav = ["/", "/docs/getting-started/", "/about/"]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 3);

    // Check that all items are Route variants
    use crate::NavItem;
    assert!(matches!(&nav[0], NavItem::Route(r) if r == "/"));
    assert!(matches!(&nav[1], NavItem::Route(r) if r == "/docs/getting-started/"));
    assert!(matches!(&nav[2], NavItem::Route(r) if r == "/about/"));

    Ok(())
}

#[test]
fn test_nav_item_link_with_label() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with link nav items (explicit labels)
    let config_content = r#"
[site]
nav = [
    { label = "Home", route = "/" },
    { label = "Getting Started", route = "/docs/getting-started/" },
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 2);

    use crate::NavItem;
    if let NavItem::Link { label, route, .. } = &nav[0] {
        assert_eq!(label, "Home");
        assert_eq!(route, "/");
    } else {
        panic!("Expected NavItem::Link for first item");
    }

    if let NavItem::Link { label, route, .. } = &nav[1] {
        assert_eq!(label, "Getting Started");
        assert_eq!(route, "/docs/getting-started/");
    } else {
        panic!("Expected NavItem::Link for second item");
    }

    Ok(())
}

#[test]
fn test_nav_item_group_without_route() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with group nav item (no header route)
    let config_content = r#"
[site]
nav = [
    { label = "Docs", children = [
        "/docs/getting-started/",
        "/docs/configuration/",
    ]},
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 1);

    use crate::NavItem;
    if let NavItem::Group {
        label,
        route,
        children,
        ..
    } = &nav[0]
    {
        assert_eq!(label, "Docs");
        assert!(route.is_none());
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], NavItem::Route(r) if r == "/docs/getting-started/"));
        assert!(matches!(&children[1], NavItem::Route(r) if r == "/docs/configuration/"));
    } else {
        panic!("Expected NavItem::Group");
    }

    Ok(())
}

#[test]
fn test_nav_item_group_with_route() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with group nav item that has a clickable header
    let config_content = r#"
[site]
nav = [
    { label = "Guides", route = "/guides/", children = [
        "/guides/deployment/",
        "/guides/configuration/",
    ]},
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 1);

    use crate::NavItem;
    if let NavItem::Group {
        label,
        route,
        children,
        ..
    } = &nav[0]
    {
        assert_eq!(label, "Guides");
        assert_eq!(route.as_deref(), Some("/guides/"));
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected NavItem::Group");
    }

    Ok(())
}

#[test]
fn test_nav_item_mixed_types() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with mixed nav item types
    let config_content = r#"
[site]
nav = [
    "/",
    { label = "Docs", children = [
        "/docs/getting-started/",
        { label = "Advanced", route = "/docs/advanced/" },
    ]},
    { label = "About", route = "/about/" },
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 3);

    use crate::NavItem;

    // First item: Route shorthand
    assert!(matches!(&nav[0], NavItem::Route(r) if r == "/"));

    // Second item: Group with mixed children
    if let NavItem::Group { children, .. } = &nav[1] {
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], NavItem::Route(_)));
        assert!(matches!(&children[1], NavItem::Link { .. }));
    } else {
        panic!("Expected NavItem::Group for second item");
    }

    // Third item: Link with label
    assert!(
        matches!(&nav[2], NavItem::Link { label, route, .. } if label == "About" && route == "/about/")
    );

    Ok(())
}

#[test]
fn test_nav_item_nested_groups() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create config with nested groups
    let config_content = r#"
[site]
nav = [
    { label = "Docs", children = [
        { label = "Getting Started", children = [
            "/docs/getting-started/installation/",
            "/docs/getting-started/quick-start/",
        ]},
        "/docs/configuration/",
    ]},
]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let cfg = config_isolated(temp_dir.path())?;

    let nav = cfg
        .site
        .as_ref()
        .and_then(|s| s.nav.as_ref())
        .expect("Expected nav to be present");

    assert_eq!(nav.len(), 1);

    use crate::NavItem;
    if let NavItem::Group { children, .. } = &nav[0] {
        assert_eq!(children.len(), 2);

        // First child is a nested group
        if let NavItem::Group {
            label,
            children: nested,
            ..
        } = &children[0]
        {
            assert_eq!(label, "Getting Started");
            assert_eq!(nested.len(), 2);
        } else {
            panic!("Expected nested NavItem::Group");
        }

        // Second child is a route
        assert!(matches!(&children[1], NavItem::Route(_)));
    } else {
        panic!("Expected NavItem::Group");
    }

    Ok(())
}

#[test]
fn test_nav_item_validation_rejects_external_route() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // External URL in Route variant should fail validation
    let config_content = r#"
[site]
nav = ["https://example.com"]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let result = config_isolated(temp_dir.path());
    assert!(result.is_err());
    let err = result.expect_err("expected validation error").to_string();
    assert!(err.contains("must be an internal route starting with '/'"));

    Ok(())
}

#[test]
fn test_nav_item_validation_rejects_external_link() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // External URL in Link variant should fail validation
    let config_content = r#"
[site]
nav = [{ label = "GitHub", route = "https://github.com/example" }]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let result = config_isolated(temp_dir.path());
    assert!(result.is_err());
    let err = result.expect_err("expected validation error").to_string();
    assert!(err.contains("must be an internal route starting with '/'"));

    Ok(())
}

#[test]
fn test_nav_item_validation_rejects_external_group_route() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // External URL in Group.route should fail validation
    let config_content = r#"
[site]
nav = [{ label = "External", route = "https://example.com", children = ["/docs/"] }]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let result = config_isolated(temp_dir.path());
    assert!(result.is_err());
    let err = result.expect_err("expected validation error").to_string();
    assert!(err.contains("must be an internal route starting with '/'"));

    Ok(())
}

#[test]
fn test_nav_item_validation_rejects_nested_external() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // External URL nested inside children should fail validation
    let config_content = r#"
[site]
nav = [{ label = "Docs", children = ["/docs/", "https://example.com"] }]
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let result = config_isolated(temp_dir.path());
    assert!(result.is_err());
    let err = result.expect_err("expected validation error").to_string();
    assert!(err.contains("must be an internal route starting with '/'"));

    Ok(())
}

/// Test that deny_unknown_fields works with both direct toml deserialization
/// and figment extraction
#[test]
fn test_deny_unknown_fields_behavior() -> Result<()> {
    use crate::workspace::WorkspaceConfig;

    // Direct toml deserialization SHOULD reject unknown fields
    let toml_str = r#"
id = "ws1234567890"
unknown_field = "test"
"#;
    let result: Result<WorkspaceConfig, _> = toml::from_str(toml_str);
    let err = result
        .expect_err("direct toml should reject unknown fields")
        .to_string();
    assert!(
        err.contains("unknown field"),
        "Error should mention unknown field: {}",
        err
    );

    // Figment extraction ALSO rejects unknown fields when deny_unknown_fields is set
    let temp_dir = TempDir::new()?;
    let config_content = r#"
[workspace]
id = "ws1234567890"
unknown_field = "test"
"#;
    fs::write(temp_dir.path().join(CONFIG_FILENAME), config_content)?;

    let err = config_isolated(temp_dir.path())
        .expect_err("figment should also reject unknown fields")
        .to_string();
    assert!(
        err.contains("unknown field") || err.contains("unknown_field"),
        "Error should mention unknown field: {}",
        err
    );

    Ok(())
}
