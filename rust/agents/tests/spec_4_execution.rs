//! Spec 4.1-4.2: Execution environment abstraction and local implementation.
//!
//! Tests use `tempfile::tempdir()` for filesystem isolation and
//! `#[tokio::test]` for async execution.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;

use stencila_agents::error::AgentError;
use stencila_agents::execution::{
    EnvVarPolicy, ExecutionEnvironment, FileContent, LocalExecutionEnvironment,
    ScopedExecutionEnvironment, filter_env_vars,
};
use stencila_agents::types::GrepOptions;

/// Helper: create a `LocalExecutionEnvironment` rooted in a temp dir.
fn local_env(dir: &std::path::Path) -> LocalExecutionEnvironment {
    LocalExecutionEnvironment::new(dir)
}

/// Helper: create a temp directory, mapping the io error to `AgentError::Io`.
fn tmp() -> Result<tempfile::TempDir, AgentError> {
    tempfile::tempdir().map_err(|e| AgentError::Io {
        message: e.to_string(),
    })
}

/// Helper: write a file, using `from_io` for path-aware errors.
fn write_tmp(path: &std::path::Path, content: &str) -> Result<(), AgentError> {
    std::fs::write(path, content).map_err(|e| AgentError::from_io(e, path))
}

/// Helper: create a directory, using `from_io` for path-aware errors.
fn mkdir_tmp(path: &std::path::Path) -> Result<(), AgentError> {
    std::fs::create_dir_all(path).map_err(|e| AgentError::from_io(e, path))
}

/// Helper: read a file as string, using `from_io` for path-aware errors.
fn read_tmp(path: &std::path::Path) -> Result<String, AgentError> {
    std::fs::read_to_string(path).map_err(|e| AgentError::from_io(e, path))
}

// =========================================================================
// File operations
// =========================================================================

#[tokio::test]
async fn read_file_text_with_line_numbers() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("test.txt"), "alpha\nbeta\ngamma\n")?;

    let env = local_env(tmp.path());
    let content = env.read_file("test.txt", None, None).await?;

    match content {
        FileContent::Text(text) => {
            assert!(text.contains("1 | alpha"), "got: {text}");
            assert!(text.contains("2 | beta"), "got: {text}");
            assert!(text.contains("3 | gamma"), "got: {text}");
        }
        FileContent::Image { .. } => panic!("expected text, got image"),
    }
    Ok(())
}

#[tokio::test]
async fn read_file_with_offset_and_limit() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let lines: Vec<String> = (1..=10).map(|i| format!("line{i}")).collect();
    write_tmp(&tmp.path().join("multi.txt"), &lines.join("\n"))?;

    let env = local_env(tmp.path());

    // offset=3 (1-based), limit=2 → lines 3 and 4
    let content = env.read_file("multi.txt", Some(3), Some(2)).await?;
    match content {
        FileContent::Text(text) => {
            assert!(text.contains("3 | line3"), "got: {text}");
            assert!(text.contains("4 | line4"), "got: {text}");
            assert!(!text.contains("line5"), "should not include line5");
            assert!(!text.contains("line2"), "should not include line2");
        }
        FileContent::Image { .. } => panic!("expected text"),
    }
    Ok(())
}

#[tokio::test]
async fn read_file_not_found() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.read_file("nonexistent.txt", None, None).await;
    assert!(result.is_err());
    match result {
        Err(AgentError::FileNotFound { path }) => {
            assert!(path.contains("nonexistent.txt"), "got path: {path}");
        }
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn read_file_image_returns_image_content() -> Result<(), AgentError> {
    let tmp = tmp()?;
    // Write some bytes with a .png extension
    let fake_png = vec![0x89, 0x50, 0x4E, 0x47]; // PNG signature prefix
    let png_path = tmp.path().join("icon.png");
    std::fs::write(&png_path, &fake_png).map_err(|e| AgentError::from_io(e, &png_path))?;

    let env = local_env(tmp.path());
    let content = env.read_file("icon.png", None, None).await?;
    match content {
        FileContent::Image { data, media_type } => {
            assert_eq!(data, fake_png);
            assert_eq!(media_type, "image/png");
        }
        FileContent::Text(_) => panic!("expected Image, got Text"),
    }
    Ok(())
}

#[tokio::test]
async fn read_file_jpeg_detected() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("photo.jpg"), "fake jpeg")?;

    let env = local_env(tmp.path());
    let content = env.read_file("photo.jpg", None, None).await?;
    match content {
        FileContent::Image { media_type, .. } => {
            assert_eq!(media_type, "image/jpeg");
        }
        FileContent::Text(_) => panic!("expected Image"),
    }
    Ok(())
}

#[tokio::test]
async fn write_file_creates_file() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    env.write_file("output.txt", "hello world").await?;

    let written = read_tmp(&tmp.path().join("output.txt"))?;
    assert_eq!(written, "hello world");
    Ok(())
}

#[tokio::test]
async fn write_file_creates_parent_directories() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    env.write_file("deep/nested/dir/file.txt", "content")
        .await?;

    assert!(tmp.path().join("deep/nested/dir/file.txt").exists());
    Ok(())
}

#[tokio::test]
async fn write_file_overwrites_existing() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    env.write_file("file.txt", "original").await?;
    env.write_file("file.txt", "updated").await?;

    let content = read_tmp(&tmp.path().join("file.txt"))?;
    assert_eq!(content, "updated");
    Ok(())
}

#[tokio::test]
async fn file_exists_true_and_false() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    env.write_file("exists.txt", "yes").await?;

    assert!(env.file_exists("exists.txt").await);
    assert!(!env.file_exists("nope.txt").await);
    Ok(())
}

#[tokio::test]
async fn list_directory_basic() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("a.txt"), "a")?;
    mkdir_tmp(&tmp.path().join("subdir"))?;

    let env = local_env(tmp.path());
    let entries = env.list_directory(".", 1).await?;

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.contains(&"a.txt"),
        "should list a.txt, got: {names:?}"
    );
    assert!(
        names.contains(&"subdir"),
        "should list subdir, got: {names:?}"
    );

    // Check is_dir and size
    let file_entry = entries
        .iter()
        .find(|e| e.name == "a.txt")
        .ok_or_else(|| AgentError::Io {
            message: "a.txt not found in entries".to_string(),
        })?;
    assert!(!file_entry.is_dir);
    assert_eq!(file_entry.size, Some(1));

    let dir_entry = entries
        .iter()
        .find(|e| e.name == "subdir")
        .ok_or_else(|| AgentError::Io {
            message: "subdir not found in entries".to_string(),
        })?;
    assert!(dir_entry.is_dir);
    assert_eq!(dir_entry.size, None);

    Ok(())
}

#[tokio::test]
async fn list_directory_with_depth() -> Result<(), AgentError> {
    let tmp = tmp()?;
    mkdir_tmp(&tmp.path().join("a/b"))?;
    write_tmp(&tmp.path().join("a/b/deep.txt"), "deep")?;

    let env = local_env(tmp.path());

    // Depth 1 should NOT see deep.txt
    let shallow = env.list_directory(".", 1).await?;
    let names: Vec<&str> = shallow.iter().map(|e| e.name.as_str()).collect();
    assert!(
        !names.iter().any(|n| n.contains("deep.txt")),
        "depth 1 should not recurse: {names:?}"
    );

    // Depth 3 should see deep.txt
    let deep = env.list_directory(".", 3).await?;
    let names: Vec<&str> = deep.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.iter().any(|n| n.contains("deep.txt")),
        "depth 3 should recurse: {names:?}"
    );

    Ok(())
}

// =========================================================================
// Delete file
// =========================================================================

#[tokio::test]
async fn delete_file_removes_file() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("doomed.txt"), "bye")?;

    let env = local_env(tmp.path());
    assert!(env.file_exists("doomed.txt").await);
    env.delete_file("doomed.txt").await?;
    assert!(!env.file_exists("doomed.txt").await);
    Ok(())
}

#[tokio::test]
async fn delete_file_not_found() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.delete_file("nonexistent.txt").await;
    assert!(result.is_err());
    match result {
        Err(AgentError::FileNotFound { path }) => {
            assert!(path.contains("nonexistent.txt"), "got path: {path}");
        }
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
    Ok(())
}

// =========================================================================
// Command execution
// =========================================================================

#[tokio::test]
async fn exec_command_success() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.exec_command("echo hello", 10_000, None, None).await?;

    assert_eq!(result.stdout.trim(), "hello");
    assert_eq!(result.exit_code, 0);
    assert!(!result.timed_out);
    Ok(())
}

#[tokio::test]
async fn exec_command_exit_code() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.exec_command("exit 42", 10_000, None, None).await?;

    assert_eq!(result.exit_code, 42);
    assert!(!result.timed_out);
    Ok(())
}

#[tokio::test]
async fn exec_command_stderr_captured() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env
        .exec_command("echo error >&2", 10_000, None, None)
        .await?;

    assert_eq!(result.stderr.trim(), "error");
    Ok(())
}

#[tokio::test]
async fn exec_command_timeout() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    // Use a very short timeout with a long sleep
    let result = env.exec_command("sleep 30", 200, None, None).await?;

    assert!(result.timed_out, "should report timed out");
    assert!(result.duration_ms >= 200, "should wait at least timeout_ms");
    Ok(())
}

#[tokio::test]
async fn exec_command_timeout_message_in_stderr() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.exec_command("sleep 30", 200, None, None).await?;

    assert!(result.timed_out);
    assert!(
        result
            .stderr
            .contains("[ERROR: Command timed out after 200ms"),
        "timeout message should be appended to stderr, got: {}",
        result.stderr
    );
    assert!(
        result.stderr.contains("timeout_ms parameter"),
        "should mention retry hint, got: {}",
        result.stderr
    );
    Ok(())
}

#[tokio::test]
async fn exec_command_partial_output_on_timeout() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    // Print something before sleeping
    let result = env
        .exec_command("echo partial && sleep 30", 500, None, None)
        .await?;

    assert!(result.timed_out);
    assert!(
        result.stdout.contains("partial"),
        "should capture partial output, got: {}",
        result.stdout
    );
    Ok(())
}

#[tokio::test]
async fn exec_command_with_working_dir() -> Result<(), AgentError> {
    let tmp = tmp()?;
    mkdir_tmp(&tmp.path().join("sub"))?;

    let env = local_env(tmp.path());
    let result = env.exec_command("pwd", 10_000, Some("sub"), None).await?;

    assert!(
        result.stdout.trim().ends_with("/sub"),
        "should run in subdir, got: {}",
        result.stdout.trim()
    );
    Ok(())
}

#[tokio::test]
async fn exec_command_with_custom_env_vars() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let mut extra = HashMap::new();
    extra.insert("MY_CUSTOM_VAR".to_string(), "test_value".to_string());

    let result = env
        .exec_command("echo $MY_CUSTOM_VAR", 10_000, None, Some(&extra))
        .await?;

    assert_eq!(result.stdout.trim(), "test_value");
    Ok(())
}

// =========================================================================
// Environment variable filtering
// =========================================================================

#[test]
fn filter_env_vars_filtered_excludes_api_keys() {
    let vars = vec![
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("OPENAI_API_KEY".to_string(), "sk-secret".to_string()),
        ("SOME_VAR".to_string(), "value".to_string()),
    ];
    let result = filter_env_vars(vars.into_iter(), &EnvVarPolicy::InheritFiltered);

    assert!(result.contains_key("PATH"));
    assert!(result.contains_key("SOME_VAR"));
    assert!(
        !result.contains_key("OPENAI_API_KEY"),
        "API key should be excluded"
    );
}

#[test]
fn filter_env_vars_filtered_case_insensitive_deny() {
    let vars = vec![
        ("my_api_key".to_string(), "lower".to_string()),
        ("MY_API_KEY".to_string(), "upper".to_string()),
        ("Some_Secret".to_string(), "mixed".to_string()),
        ("db_password".to_string(), "pw".to_string()),
        ("auth_token".to_string(), "tok".to_string()),
        ("gcp_credential".to_string(), "cred".to_string()),
    ];
    let result = filter_env_vars(vars.into_iter(), &EnvVarPolicy::InheritFiltered);

    assert!(result.is_empty(), "all should be denied, got: {result:?}");
}

#[test]
fn filter_env_vars_filtered_allowlist_always_present() {
    let vars = vec![
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("HOME".to_string(), "/home/user".to_string()),
        ("CARGO_HOME".to_string(), "/home/.cargo".to_string()),
        ("NVM_DIR".to_string(), "/home/.nvm".to_string()),
    ];
    let result = filter_env_vars(vars.into_iter(), &EnvVarPolicy::InheritFiltered);

    assert!(result.contains_key("PATH"));
    assert!(result.contains_key("HOME"));
    assert!(result.contains_key("CARGO_HOME"));
    assert!(result.contains_key("NVM_DIR"));
}

#[test]
fn filter_env_vars_inherit_all_includes_everything() {
    let vars = vec![
        ("OPENAI_API_KEY".to_string(), "sk-secret".to_string()),
        ("NORMAL_VAR".to_string(), "value".to_string()),
    ];
    let result = filter_env_vars(vars.into_iter(), &EnvVarPolicy::InheritAll);

    assert!(
        result.contains_key("OPENAI_API_KEY"),
        "InheritAll should keep secrets"
    );
    assert!(result.contains_key("NORMAL_VAR"));
}

#[test]
fn filter_env_vars_inherit_none_only_allowlist() {
    let vars = vec![
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("OPENAI_API_KEY".to_string(), "sk-secret".to_string()),
        ("RANDOM_VAR".to_string(), "value".to_string()),
        ("CARGO_HOME".to_string(), "/home/.cargo".to_string()),
    ];
    let result = filter_env_vars(vars.into_iter(), &EnvVarPolicy::InheritNone);

    assert!(result.contains_key("PATH"), "allowlist var should be kept");
    assert!(
        result.contains_key("CARGO_HOME"),
        "allowlist var should be kept"
    );
    assert!(
        !result.contains_key("OPENAI_API_KEY"),
        "non-allowlist should be excluded"
    );
    assert!(
        !result.contains_key("RANDOM_VAR"),
        "non-allowlist should be excluded"
    );
}

#[test]
fn filter_env_vars_default_policy_is_filtered() {
    assert_eq!(EnvVarPolicy::default(), EnvVarPolicy::InheritFiltered);
}

// =========================================================================
// Search operations
// =========================================================================

#[tokio::test]
async fn grep_basic_match() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(
        &tmp.path().join("file.txt"),
        "hello world\nfoo bar\nhello again\n",
    )?;

    let env = local_env(tmp.path());
    let result = env.grep("hello", ".", &GrepOptions::default()).await?;

    assert!(result.contains("hello world"), "got: {result}");
    assert!(result.contains("hello again"), "got: {result}");
    assert!(!result.contains("foo bar"), "should not match foo bar");
    Ok(())
}

#[tokio::test]
async fn grep_case_insensitive() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("file.txt"), "Hello World\nhello world\n")?;

    let env = local_env(tmp.path());
    let opts = GrepOptions {
        case_insensitive: true,
        ..GrepOptions::default()
    };
    let result = env.grep("hello", ".", &opts).await?;

    // Both lines should match
    assert!(result.contains("Hello World"), "got: {result}");
    assert!(result.contains("hello world"), "got: {result}");
    Ok(())
}

#[tokio::test]
async fn grep_max_results() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let lines: Vec<String> = (0..50).map(|i| format!("match line {i}")).collect();
    write_tmp(&tmp.path().join("many.txt"), &lines.join("\n"))?;

    let env = local_env(tmp.path());
    let opts = GrepOptions {
        max_results: 5,
        ..GrepOptions::default()
    };
    let result = env.grep("match", ".", &opts).await?;

    let match_count = result.lines().count();
    assert!(
        match_count <= 5,
        "should respect max_results, got {match_count} lines"
    );
    Ok(())
}

#[tokio::test]
async fn grep_single_file() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("target.txt"), "alpha\nbeta\nalpha again\n")?;

    let env = local_env(tmp.path());
    // Pass a file path directly (not a directory)
    let result = env
        .grep("alpha", "target.txt", &GrepOptions::default())
        .await?;

    assert!(result.contains("alpha"), "got: {result}");
    assert!(!result.contains("beta"), "should not match beta");
    // Should find both matching lines
    let match_count = result.lines().count();
    assert_eq!(match_count, 2, "should find 2 matches, got: {match_count}");
    Ok(())
}

#[tokio::test]
async fn grep_path_not_found() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env
        .grep("pattern", "nonexistent_dir", &GrepOptions::default())
        .await;

    assert!(result.is_err(), "should error on missing path");
    match result {
        Err(AgentError::FileNotFound { path }) => {
            assert!(
                path.contains("nonexistent_dir"),
                "error should mention the path, got: {path}"
            );
        }
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn glob_basic() -> Result<(), AgentError> {
    let tmp = tmp()?;
    write_tmp(&tmp.path().join("a.rs"), "")?;
    write_tmp(&tmp.path().join("b.rs"), "")?;
    write_tmp(&tmp.path().join("c.txt"), "")?;

    let env = local_env(tmp.path());
    let result = env.glob_files("*.rs", ".").await?;

    assert_eq!(result.len(), 2, "should find 2 .rs files, got: {result:?}");
    assert!(
        result.iter().all(|p| p.ends_with(".rs")),
        "all should be .rs: {result:?}"
    );
    Ok(())
}

#[tokio::test]
async fn glob_path_not_found() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let env = local_env(tmp.path());
    let result = env.glob_files("*.rs", "nonexistent_dir").await;

    assert!(result.is_err(), "should error on missing path");
    match result {
        Err(AgentError::FileNotFound { path }) => {
            assert!(
                path.contains("nonexistent_dir"),
                "error should mention the path, got: {path}"
            );
        }
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn glob_sorted_by_mtime_newest_first() -> Result<(), AgentError> {
    let tmp = tmp()?;

    // Create files with staggered modification times
    write_tmp(&tmp.path().join("old.rs"), "old")?;
    // Sleep briefly so mtimes differ
    std::thread::sleep(std::time::Duration::from_millis(50));
    write_tmp(&tmp.path().join("mid.rs"), "mid")?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    write_tmp(&tmp.path().join("new.rs"), "new")?;

    let env = local_env(tmp.path());
    let result = env.glob_files("*.rs", ".").await?;

    assert_eq!(result.len(), 3, "should find 3 .rs files, got: {result:?}");

    // Newest file should come first
    assert!(
        result[0].ends_with("new.rs"),
        "first should be newest (new.rs), got: {result:?}"
    );
    assert!(
        result[2].ends_with("old.rs"),
        "last should be oldest (old.rs), got: {result:?}"
    );
    Ok(())
}

// =========================================================================
// Metadata
// =========================================================================

#[test]
fn working_directory_returns_configured_path() {
    let env = LocalExecutionEnvironment::new("/tmp/test");
    assert_eq!(env.working_directory(), "/tmp/test");
}

#[test]
fn platform_returns_spec_value() {
    let env = LocalExecutionEnvironment::new("/tmp");
    let platform = env.platform();
    // Spec 4.1 line 743: "darwin", "linux", "windows", "wasm"
    assert!(
        ["linux", "darwin", "windows", "wasm"].contains(&platform),
        "got: {platform}"
    );
}

#[test]
fn os_version_returns_nonempty() {
    let env = LocalExecutionEnvironment::new("/tmp");
    let version = env.os_version();
    assert!(!version.is_empty());
}

// =========================================================================
// ScopedExecutionEnvironment
// =========================================================================

use std::sync::Arc;

/// Helper: create a `ScopedExecutionEnvironment` wrapping a local env.
fn scoped_env(
    inner: Arc<dyn ExecutionEnvironment>,
    scope_dir: &str,
) -> Result<ScopedExecutionEnvironment, AgentError> {
    ScopedExecutionEnvironment::new(inner, scope_dir)
}

#[tokio::test]
async fn scoped_relative_path_within_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;
    write_tmp(&scope.join("file.txt"), "hello scoped")?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Relative path within scope should work
    let content = env.read_file("file.txt", None, None).await?;
    match content {
        FileContent::Text(text) => {
            assert!(text.contains("hello scoped"), "got: {text}");
        }
        FileContent::Image { .. } => panic!("expected text, got image"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_absolute_path_within_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;
    write_tmp(&scope.join("file.txt"), "absolute ok")?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Absolute path within scope should work
    let abs_path = scope.join("file.txt");
    let content = env
        .read_file(abs_path.to_str().unwrap_or("."), None, None)
        .await?;
    match content {
        FileContent::Text(text) => {
            assert!(text.contains("absolute ok"), "got: {text}");
        }
        FileContent::Image { .. } => panic!("expected text, got image"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_absolute_path_outside_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Absolute path outside scope should be rejected
    let result = env.read_file("/etc/passwd", None, None).await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "/etc/passwd");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_dotdot_traversal_rejected() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Traversal via `..` should be rejected
    let result = env.read_file("../../etc/passwd", None, None).await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "../../etc/passwd");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_dotdot_within_scope_accepted() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    let sub = scope.join("sub");
    mkdir_tmp(&sub)?;
    write_tmp(&scope.join("file.txt"), "normalized ok")?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // `sub/../file.txt` normalizes to `file.txt` within scope
    let content = env.read_file("sub/../file.txt", None, None).await?;
    match content {
        FileContent::Text(text) => {
            assert!(text.contains("normalized ok"), "got: {text}");
        }
        FileContent::Image { .. } => panic!("expected text, got image"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_working_directory_returns_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    let canonical_scope = scope
        .canonicalize()
        .map_err(|e| AgentError::from_io(e, &scope))?;
    assert_eq!(
        env.working_directory(),
        canonical_scope.to_str().unwrap_or(".")
    );
    Ok(())
}

#[tokio::test]
async fn scoped_file_exists_outside_returns_false() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;
    // Create a file outside the scope
    write_tmp(&tmp.path().join("outside.txt"), "outside")?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Out-of-scope path should return false (not error)
    assert!(!env.file_exists("/etc/passwd").await);
    assert!(!env.file_exists("../outside.txt").await);
    Ok(())
}

#[tokio::test]
async fn scoped_exec_command_default_cwd() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    let result = env.exec_command("pwd", 10_000, None, None).await?;
    let canonical_scope = scope
        .canonicalize()
        .map_err(|e| AgentError::from_io(e, &scope))?;
    assert_eq!(
        result.stdout.trim(),
        canonical_scope.to_str().unwrap_or("."),
        "default cwd should be scope dir"
    );
    Ok(())
}

#[tokio::test]
async fn scoped_exec_command_escape_rejected() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Working dir override pointing outside scope should be rejected
    let result = env.exec_command("pwd", 10_000, Some("/tmp"), None).await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "/tmp");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_write_file_within_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    env.write_file("new_file.txt", "scoped write").await?;

    let written = read_tmp(&scope.join("new_file.txt"))?;
    assert_eq!(written, "scoped write");
    Ok(())
}

#[tokio::test]
async fn scoped_list_directory_outside_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    let result = env.list_directory("/etc", 1).await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "/etc");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_grep_outside_scope() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    mkdir_tmp(&scope)?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    let result = env.grep("pattern", "/etc", &GrepOptions::default()).await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "/etc");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn scoped_new_rejects_scope_outside_parent_working_dir() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));

    // Absolute path outside parent working dir should be rejected
    let result = ScopedExecutionEnvironment::new(Arc::clone(&inner), "/etc");
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "/etc");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }

    // Relative path that escapes via `..` should be rejected
    let result = ScopedExecutionEnvironment::new(Arc::clone(&inner), "../../etc");
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "../../etc");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn scoped_symlink_write_outside_scope_rejected() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    mkdir_tmp(&scope)?;
    mkdir_tmp(&outside)?;

    // Create a symlink inside scope pointing outside
    let link = scope.join("escape_link");
    std::os::unix::fs::symlink(&outside, &link).map_err(|e| AgentError::from_io(e, &link))?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    // Writing through the symlink should be rejected — the symlink dir
    // canonicalizes to `outside/` which is outside scope.
    let result = env.write_file("escape_link/evil.txt", "pwned").await;
    match result {
        Err(AgentError::PermissionDenied { path }) => {
            assert_eq!(path, "escape_link/evil.txt");
        }
        other => panic!("expected PermissionDenied, got: {other:?}"),
    }

    // Verify the file was NOT created outside scope
    assert!(!outside.join("evil.txt").exists());
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn scoped_list_directory_filters_symlinked_entries() -> Result<(), AgentError> {
    let tmp = tmp()?;
    let scope = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    mkdir_tmp(&scope)?;
    mkdir_tmp(&outside)?;
    write_tmp(&scope.join("in_scope.txt"), "ok")?;

    // Create a symlink inside scope pointing outside
    let link = scope.join("escape_link");
    std::os::unix::fs::symlink(&outside, &link).map_err(|e| AgentError::from_io(e, &link))?;

    let inner: Arc<dyn ExecutionEnvironment> = Arc::new(local_env(tmp.path()));
    let env = scoped_env(inner, scope.to_str().unwrap_or("."))?;

    let entries = env.list_directory(".", 1).await?;
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

    // in_scope.txt should be listed
    assert!(
        names.contains(&"in_scope.txt"),
        "should list in_scope.txt, got: {names:?}"
    );
    // escape_link should be filtered out (resolves outside scope)
    assert!(
        !names.contains(&"escape_link"),
        "symlink to outside should be filtered, got: {names:?}"
    );
    Ok(())
}
