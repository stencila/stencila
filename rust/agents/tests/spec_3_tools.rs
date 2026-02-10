//! Tests for core tool implementations (spec 3.3, 3.6).
//!
//! Phase 5b+6a: schema fixture parity and tool executor behavior.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::registry::ToolRegistry;
use stencila_agents::tools;
use stencila_agents::types::{DirEntry, ExecResult, GrepOptions};
use stencila_models3::types::tool::ToolDefinition;

// ---------------------------------------------------------------------------
// MockExecutionEnvironment
// ---------------------------------------------------------------------------

/// Recorded write operation.
#[derive(Debug, Clone)]
struct WriteRecord {
    path: String,
    content: String,
}

/// Captured parameters from an `exec_command` call.
#[derive(Debug, Clone)]
struct ExecCall {
    command: String,
    timeout_ms: u64,
}

/// Captured parameters from a `grep` call.
#[derive(Debug, Clone)]
struct GrepCall {
    pattern: String,
    path: String,
    options: GrepOptions,
}

/// Captured parameters from a `glob_files` call.
#[derive(Debug, Clone)]
struct GlobCall {
    pattern: String,
    path: String,
}

/// Captured parameters from a `list_directory` call.
#[derive(Debug, Clone)]
struct ListDirCall {
    path: String,
    depth: usize,
}

/// Configurable mock for testing tool executors.
///
/// Uses interior mutability (`Arc<Mutex<..>>`) because
/// `ExecutionEnvironment` methods take `&self`. Captures forwarded
/// parameters so tests can verify correct parameter passing.
#[derive(Clone)]
struct MockExecutionEnvironment {
    working_dir: String,
    files: Arc<Mutex<HashMap<String, MockFileContent>>>,
    commands: HashMap<String, ExecResult>,
    grep_results: HashMap<String, String>,
    glob_results: HashMap<String, Vec<String>>,
    dir_entries: HashMap<String, Vec<DirEntry>>,
    writes: Arc<Mutex<Vec<WriteRecord>>>,
    exec_calls: Arc<Mutex<Vec<ExecCall>>>,
    grep_calls: Arc<Mutex<Vec<GrepCall>>>,
    glob_calls: Arc<Mutex<Vec<GlobCall>>>,
    list_dir_calls: Arc<Mutex<Vec<ListDirCall>>>,
}

#[derive(Clone, Debug)]
enum MockFileContent {
    Text(String),
    Image { data: Vec<u8>, media_type: String },
}

impl MockExecutionEnvironment {
    fn new() -> Self {
        Self {
            working_dir: "/mock/workspace".into(),
            files: Arc::new(Mutex::new(HashMap::new())),
            commands: HashMap::new(),
            grep_results: HashMap::new(),
            glob_results: HashMap::new(),
            dir_entries: HashMap::new(),
            writes: Arc::new(Mutex::new(Vec::new())),
            exec_calls: Arc::new(Mutex::new(Vec::new())),
            grep_calls: Arc::new(Mutex::new(Vec::new())),
            glob_calls: Arc::new(Mutex::new(Vec::new())),
            list_dir_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_file(self, path: &str, content: &str) -> Self {
        {
            let mut files = self.files.lock().expect("lock poisoned");
            files.insert(path.into(), MockFileContent::Text(content.into()));
        }
        self
    }

    fn with_image(self, path: &str, media_type: &str) -> Self {
        {
            let mut files = self.files.lock().expect("lock poisoned");
            files.insert(
                path.into(),
                MockFileContent::Image {
                    data: vec![0x89, 0x50, 0x4E, 0x47],
                    media_type: media_type.into(),
                },
            );
        }
        self
    }

    fn with_command(mut self, command: &str, result: ExecResult) -> Self {
        self.commands.insert(command.into(), result);
        self
    }

    fn with_grep(mut self, pattern: &str, output: &str) -> Self {
        self.grep_results.insert(pattern.into(), output.into());
        self
    }

    fn with_glob(mut self, pattern: &str, files: Vec<&str>) -> Self {
        self.glob_results.insert(
            pattern.into(),
            files.into_iter().map(String::from).collect(),
        );
        self
    }

    fn with_dir(mut self, path: &str, entries: Vec<DirEntry>) -> Self {
        self.dir_entries.insert(path.into(), entries);
        self
    }

    fn recorded_writes(&self) -> Vec<WriteRecord> {
        self.writes.lock().expect("lock poisoned").clone()
    }

    fn recorded_exec_calls(&self) -> Vec<ExecCall> {
        self.exec_calls.lock().expect("lock poisoned").clone()
    }

    fn recorded_grep_calls(&self) -> Vec<GrepCall> {
        self.grep_calls.lock().expect("lock poisoned").clone()
    }

    fn recorded_glob_calls(&self) -> Vec<GlobCall> {
        self.glob_calls.lock().expect("lock poisoned").clone()
    }

    fn recorded_list_dir_calls(&self) -> Vec<ListDirCall> {
        self.list_dir_calls.lock().expect("lock poisoned").clone()
    }
}

#[async_trait]
impl ExecutionEnvironment for MockExecutionEnvironment {
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        let files = self.files.lock().expect("lock poisoned");
        let content = files
            .get(path)
            .ok_or_else(|| AgentError::FileNotFound { path: path.into() })?;

        match content {
            MockFileContent::Text(text) => {
                let lines: Vec<&str> = text.lines().collect();
                let start = offset.unwrap_or(1).saturating_sub(1);
                let count = limit.unwrap_or(2000);
                let selected: Vec<&str> = lines.iter().skip(start).take(count).copied().collect();

                let numbered = selected
                    .iter()
                    .enumerate()
                    .map(|(i, line)| format!("{:>6} | {line}", start + i + 1))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(FileContent::Text(numbered))
            }
            MockFileContent::Image { data, media_type } => Ok(FileContent::Image {
                data: data.clone(),
                media_type: media_type.clone(),
            }),
        }
    }

    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        // Record the write
        {
            let mut writes = self.writes.lock().expect("lock poisoned");
            writes.push(WriteRecord {
                path: path.into(),
                content: content.into(),
            });
        }
        // Update the file map so subsequent reads see the new content
        {
            let mut files = self.files.lock().expect("lock poisoned");
            files.insert(path.into(), MockFileContent::Text(content.into()));
        }
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> bool {
        let files = self.files.lock().expect("lock poisoned");
        files.contains_key(path)
    }

    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        let mut files = self.files.lock().expect("lock poisoned");
        if files.remove(path).is_some() {
            Ok(())
        } else {
            Err(AgentError::FileNotFound { path: path.into() })
        }
    }

    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>> {
        {
            let mut calls = self.list_dir_calls.lock().expect("lock poisoned");
            calls.push(ListDirCall {
                path: path.into(),
                depth,
            });
        }
        self.dir_entries
            .get(path)
            .cloned()
            .ok_or_else(|| AgentError::FileNotFound { path: path.into() })
    }

    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        _working_dir: Option<&str>,
        _env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        {
            let mut calls = self.exec_calls.lock().expect("lock poisoned");
            calls.push(ExecCall {
                command: command.into(),
                timeout_ms,
            });
        }
        self.commands
            .get(command)
            .cloned()
            .ok_or_else(|| AgentError::Io {
                message: format!("no mock command: {command}"),
            })
    }

    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String> {
        {
            let mut calls = self.grep_calls.lock().expect("lock poisoned");
            calls.push(GrepCall {
                pattern: pattern.into(),
                path: path.into(),
                options: options.clone(),
            });
        }
        self.grep_results
            .get(pattern)
            .cloned()
            .ok_or_else(|| AgentError::Io {
                message: format!("no mock grep: {pattern}"),
            })
    }

    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>> {
        {
            let mut calls = self.glob_calls.lock().expect("lock poisoned");
            calls.push(GlobCall {
                pattern: pattern.into(),
                path: path.into(),
            });
        }
        self.glob_results
            .get(pattern)
            .cloned()
            .ok_or_else(|| AgentError::Io {
                message: format!("no mock glob: {pattern}"),
            })
    }

    fn working_directory(&self) -> &str {
        &self.working_dir
    }

    fn platform(&self) -> &str {
        "mock"
    }

    fn os_version(&self) -> String {
        "mock-os 1.0".into()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load a fixture and deserialize as `ToolDefinition`.
fn load_fixture(name: &str) -> Result<ToolDefinition, String> {
    let path = format!(
        "{}/tests/fixtures/tool_schemas/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read fixture {path}: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("failed to parse fixture {path}: {e}"))
}

// =========================================================================
// Schema Parity Tests (8)
// =========================================================================

/// Assert that a tool module's `definition()` matches its JSON fixture.
macro_rules! schema_parity_test {
    ($test_name:ident, $module:ident, $fixture:expr) => {
        #[test]
        fn $test_name() -> Result<(), String> {
            let fixture = load_fixture($fixture)?;
            let definition = tools::$module::definition();
            assert_eq!(definition.name, fixture.name);
            assert_eq!(definition.description, fixture.description);
            assert_eq!(definition.parameters, fixture.parameters);
            assert_eq!(definition.strict, fixture.strict);
            Ok(())
        }
    };
}

schema_parity_test!(read_file_schema_matches_fixture, read_file, "read_file");
schema_parity_test!(write_file_schema_matches_fixture, write_file, "write_file");
schema_parity_test!(edit_file_schema_matches_fixture, edit_file, "edit_file");
schema_parity_test!(shell_schema_matches_fixture, shell, "shell");
schema_parity_test!(grep_schema_matches_fixture, grep, "grep");
schema_parity_test!(glob_schema_matches_fixture, glob, "glob");
schema_parity_test!(
    read_many_files_schema_matches_fixture,
    read_many_files,
    "read_many_files"
);
schema_parity_test!(list_dir_schema_matches_fixture, list_dir, "list_dir");

// =========================================================================
// read_file tests (3)
// =========================================================================

#[tokio::test]
async fn read_file_text_content() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new()
        .with_file("/test/hello.rs", "fn main() {\n    println!(\"hello\");\n}");
    let exec = tools::read_file::executor();
    let result = exec(json!({"file_path": "/test/hello.rs"}), &env).await?;

    assert!(result.contains("fn main()"));
    assert!(result.contains("println!"));
    Ok(())
}

#[tokio::test]
async fn read_file_with_offset_and_limit() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new()
        .with_file("/test/lines.txt", "line1\nline2\nline3\nline4\nline5");
    let exec = tools::read_file::executor();
    let result = exec(
        json!({"file_path": "/test/lines.txt", "offset": 2, "limit": 2}),
        &env,
    )
    .await?;

    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
    assert!(!result.contains("line1"));
    assert!(!result.contains("line4"));
    Ok(())
}

#[tokio::test]
async fn read_file_image_returns_placeholder() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_image("/test/photo.png", "image/png");
    let exec = tools::read_file::executor();
    let result = exec(json!({"file_path": "/test/photo.png"}), &env).await?;

    assert_eq!(result, "[Image file: /test/photo.png (image/png)]");
    Ok(())
}

#[tokio::test]
async fn read_file_not_found() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::read_file::executor();
    let result = exec(json!({"file_path": "/nonexistent"}), &env).await;

    assert!(matches!(result, Err(AgentError::FileNotFound { .. })));
}

// =========================================================================
// write_file tests (2)
// =========================================================================

#[tokio::test]
async fn write_file_success_with_byte_count() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new();
    let exec = tools::write_file::executor();
    let result = exec(
        json!({"file_path": "/test/out.txt", "content": "hello world"}),
        &env,
    )
    .await?;

    assert!(result.contains("11 bytes"));
    assert!(result.contains("/test/out.txt"));
    Ok(())
}

#[tokio::test]
async fn write_file_records_write() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new();
    let exec = tools::write_file::executor();
    exec(
        json!({"file_path": "/test/out.txt", "content": "hello"}),
        &env,
    )
    .await?;

    let writes = env.recorded_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].path, "/test/out.txt");
    assert_eq!(writes[0].content, "hello");
    Ok(())
}

#[tokio::test]
async fn write_file_missing_content() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::write_file::executor();
    let result = exec(json!({"file_path": "/test/out.txt"}), &env).await;

    assert!(matches!(result, Err(AgentError::ValidationError { .. })));
}

// =========================================================================
// edit_file tests (6)
// =========================================================================

#[tokio::test]
async fn edit_file_single_replace() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file("/test/code.rs", "let x = 1;\nlet y = 2;");
    let exec = tools::edit_file::executor();
    let result = exec(
        json!({
            "file_path": "/test/code.rs",
            "old_string": "let x = 1;",
            "new_string": "let x = 42;"
        }),
        &env,
    )
    .await?;

    assert!(result.contains("1 occurrence"));

    // Verify the written content
    let writes = env.recorded_writes();
    assert_eq!(writes.len(), 1);
    assert!(writes[0].content.contains("let x = 42;"));
    assert!(writes[0].content.contains("let y = 2;"));
    Ok(())
}

#[tokio::test]
async fn edit_file_replace_all() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file("/test/code.rs", "foo bar foo baz foo");
    let exec = tools::edit_file::executor();
    let result = exec(
        json!({
            "file_path": "/test/code.rs",
            "old_string": "foo",
            "new_string": "qux",
            "replace_all": true
        }),
        &env,
    )
    .await?;

    assert!(result.contains("3 occurrence"));
    let writes = env.recorded_writes();
    assert_eq!(writes[0].content, "qux bar qux baz qux");
    Ok(())
}

#[tokio::test]
async fn edit_file_not_found() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::edit_file::executor();
    let result = exec(
        json!({
            "file_path": "/nonexistent",
            "old_string": "x",
            "new_string": "y"
        }),
        &env,
    )
    .await;

    assert!(matches!(result, Err(AgentError::FileNotFound { .. })));
}

#[tokio::test]
async fn edit_file_old_string_missing() {
    let env = MockExecutionEnvironment::new().with_file("/test/code.rs", "let x = 1;");
    let exec = tools::edit_file::executor();
    let result = exec(
        json!({
            "file_path": "/test/code.rs",
            "old_string": "let y = 2;",
            "new_string": "let y = 3;"
        }),
        &env,
    )
    .await;

    assert!(
        matches!(&result, Err(AgentError::EditConflict { reason }) if reason.contains("not found")),
        "expected EditConflict containing 'not found', got: {result:?}"
    );
}

#[tokio::test]
async fn edit_file_not_unique() {
    let env = MockExecutionEnvironment::new().with_file("/test/code.rs", "foo bar foo");
    let exec = tools::edit_file::executor();
    let result = exec(
        json!({
            "file_path": "/test/code.rs",
            "old_string": "foo",
            "new_string": "baz"
        }),
        &env,
    )
    .await;

    assert!(
        matches!(&result, Err(AgentError::EditConflict { reason }) if reason.contains("2 times")),
        "expected EditConflict containing '2 times', got: {result:?}"
    );
}

#[tokio::test]
async fn edit_file_correct_writeback() -> AgentResult<()> {
    // Verify that line numbers are stripped before matching and the full
    // file content is written back (not just the matched portion).
    let env = MockExecutionEnvironment::new().with_file("/test/multi.txt", "alpha\nbeta\ngamma");
    let exec = tools::edit_file::executor();
    exec(
        json!({
            "file_path": "/test/multi.txt",
            "old_string": "beta",
            "new_string": "BETA"
        }),
        &env,
    )
    .await?;

    let writes = env.recorded_writes();
    assert_eq!(writes[0].content, "alpha\nBETA\ngamma");
    Ok(())
}

// =========================================================================
// shell tests (3)
// =========================================================================

#[tokio::test]
async fn shell_success_format() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_command(
        "echo hello",
        ExecResult {
            stdout: "hello\n".into(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 42,
        },
    );
    let exec = tools::shell::executor();
    let result = exec(json!({"command": "echo hello"}), &env).await?;

    assert!(result.contains("Exit code: 0"));
    assert!(result.contains("Duration: 42ms"));
    assert!(result.contains("STDOUT:"));
    assert!(result.contains("hello"));
    assert!(!result.contains("STDERR:"));
    Ok(())
}

#[tokio::test]
async fn shell_exit_code() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_command(
        "false",
        ExecResult {
            stdout: String::new(),
            stderr: "error\n".into(),
            exit_code: 1,
            timed_out: false,
            duration_ms: 5,
        },
    );
    let exec = tools::shell::executor();
    let result = exec(json!({"command": "false"}), &env).await?;

    assert!(result.contains("Exit code: 1"));
    assert!(result.contains("STDERR:"));
    assert!(result.contains("error"));
    Ok(())
}

#[tokio::test]
async fn shell_custom_timeout() -> AgentResult<()> {
    // Test that executor_with_timeout forwards the custom default timeout
    let env = MockExecutionEnvironment::new().with_command(
        "slow",
        ExecResult {
            stdout: "done".into(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 100,
        },
    );
    let exec = tools::shell::executor_with_timeout(120_000, 600_000);
    exec(json!({"command": "slow"}), &env).await?;

    let calls = env.recorded_exec_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].command, "slow");
    assert_eq!(calls[0].timeout_ms, 120_000);
    Ok(())
}

#[tokio::test]
async fn shell_per_call_timeout_overrides_default() -> AgentResult<()> {
    // Test that timeout_ms arg overrides the default
    let env = MockExecutionEnvironment::new().with_command(
        "quick",
        ExecResult {
            stdout: "done".into(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 10,
        },
    );
    let exec = tools::shell::executor_with_timeout(120_000, 600_000);
    exec(json!({"command": "quick", "timeout_ms": 5000}), &env).await?;

    let calls = env.recorded_exec_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].command, "quick");
    assert_eq!(calls[0].timeout_ms, 5000);
    Ok(())
}

// =========================================================================
// grep tests (3)
// =========================================================================

#[tokio::test]
async fn grep_basic() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_grep("TODO", "src/main.rs:10:// TODO: fix this");
    let exec = tools::grep::executor();
    let result = exec(json!({"pattern": "TODO", "path": "/some/dir"}), &env).await?;

    assert!(result.contains("TODO: fix this"));

    let calls = env.recorded_grep_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].pattern, "TODO");
    assert_eq!(calls[0].path, "/some/dir");
    Ok(())
}

#[tokio::test]
async fn grep_with_options() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_grep("error", "lib.rs:5:handle_error()");
    let exec = tools::grep::executor();
    let result = exec(
        json!({
            "pattern": "error",
            "path": "/src",
            "glob_filter": "*.rs",
            "case_insensitive": true,
            "max_results": 50
        }),
        &env,
    )
    .await?;

    assert!(result.contains("handle_error"));

    // Verify parameters were forwarded correctly
    let calls = env.recorded_grep_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/src");
    assert_eq!(calls[0].options.glob_filter.as_deref(), Some("*.rs"));
    assert!(calls[0].options.case_insensitive);
    assert_eq!(calls[0].options.max_results, 50);
    Ok(())
}

#[tokio::test]
async fn grep_default_path() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_grep("fn main", "main.rs:1:fn main() {");
    let exec = tools::grep::executor();
    // Omit "path" — should default to working_directory
    exec(json!({"pattern": "fn main"}), &env).await?;

    let calls = env.recorded_grep_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/mock/workspace");
    Ok(())
}

#[tokio::test]
async fn grep_missing_pattern() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::grep::executor();
    let result = exec(json!({"path": "/some/dir"}), &env).await;

    assert!(matches!(result, Err(AgentError::ValidationError { .. })));
}

// =========================================================================
// glob tests (3)
// =========================================================================

#[tokio::test]
async fn glob_basic() -> AgentResult<()> {
    let env =
        MockExecutionEnvironment::new().with_glob("**/*.rs", vec!["src/main.rs", "src/lib.rs"]);
    let exec = tools::glob::executor();
    let result = exec(json!({"pattern": "**/*.rs", "path": "/project"}), &env).await?;

    assert!(result.contains("src/main.rs"));
    assert!(result.contains("src/lib.rs"));

    let calls = env.recorded_glob_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].pattern, "**/*.rs");
    assert_eq!(calls[0].path, "/project");
    Ok(())
}

#[tokio::test]
async fn glob_empty_results() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_glob("*.xyz", vec![]);
    let exec = tools::glob::executor();
    let result = exec(json!({"pattern": "*.xyz"}), &env).await?;

    assert_eq!(result, "No files found.");
    Ok(())
}

#[tokio::test]
async fn glob_default_path() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_glob("Cargo.toml", vec!["Cargo.toml"]);
    let exec = tools::glob::executor();
    // Omit "path" — should default to working_directory
    exec(json!({"pattern": "Cargo.toml"}), &env).await?;

    let calls = env.recorded_glob_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/mock/workspace");
    Ok(())
}

#[tokio::test]
async fn glob_missing_pattern() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::glob::executor();
    let result = exec(json!({"path": "/some/dir"}), &env).await;

    assert!(matches!(result, Err(AgentError::ValidationError { .. })));
}

// =========================================================================
// read_many_files tests (3)
// =========================================================================

#[tokio::test]
async fn read_many_files_batch() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new()
        .with_file("/a.txt", "content a")
        .with_file("/b.txt", "content b");
    let exec = tools::read_many_files::executor();
    let result = exec(json!({"paths": ["/a.txt", "/b.txt"]}), &env).await?;

    assert!(result.contains("=== /a.txt ==="));
    assert!(result.contains("=== /b.txt ==="));
    assert!(result.contains("content a"));
    assert!(result.contains("content b"));
    Ok(())
}

#[tokio::test]
async fn read_many_files_partial_failure() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file("/good.txt", "ok");
    let exec = tools::read_many_files::executor();
    let result = exec(json!({"paths": ["/good.txt", "/bad.txt"]}), &env).await?;

    assert!(result.contains("=== /good.txt ==="));
    assert!(result.contains("=== /bad.txt ==="));
    assert!(result.contains("[Error:"));
    Ok(())
}

#[tokio::test]
async fn read_many_files_empty_paths() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new();
    let exec = tools::read_many_files::executor();
    let result = exec(json!({"paths": []}), &env).await?;

    assert!(result.is_empty());
    Ok(())
}

// =========================================================================
// list_dir tests (2)
// =========================================================================

#[tokio::test]
async fn list_dir_basic() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_dir(
        "/project",
        vec![
            DirEntry {
                name: "src".into(),
                is_dir: true,
                size: None,
            },
            DirEntry {
                name: "Cargo.toml".into(),
                is_dir: false,
                size: Some(256),
            },
        ],
    );
    let exec = tools::list_dir::executor();
    let result = exec(json!({"path": "/project"}), &env).await?;

    assert!(result.contains("src/"));
    assert!(result.contains("Cargo.toml (256 bytes)"));
    Ok(())
}

#[tokio::test]
async fn list_dir_with_depth() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_dir(
        "/project",
        vec![DirEntry {
            name: "src/main.rs".into(),
            is_dir: false,
            size: Some(100),
        }],
    );
    let exec = tools::list_dir::executor();
    exec(json!({"path": "/project", "depth": 2}), &env).await?;

    // Verify depth parameter was forwarded
    let calls = env.recorded_list_dir_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/project");
    assert_eq!(calls[0].depth, 2);
    Ok(())
}

#[tokio::test]
async fn list_dir_not_found() {
    let env = MockExecutionEnvironment::new();
    let exec = tools::list_dir::executor();
    let result = exec(json!({"path": "/nonexistent"}), &env).await;

    assert!(matches!(result, Err(AgentError::FileNotFound { .. })));
}

// =========================================================================
// Registration tests (2)
// =========================================================================

#[test]
fn register_core_tools_adds_six() -> AgentResult<()> {
    let mut registry = ToolRegistry::new();
    tools::register_core_tools(&mut registry)?;

    assert_eq!(registry.len(), 6);
    let names = registry.names();
    assert!(names.contains(&"read_file"));
    assert!(names.contains(&"write_file"));
    assert!(names.contains(&"edit_file"));
    assert!(names.contains(&"shell"));
    assert!(names.contains(&"grep"));
    assert!(names.contains(&"glob"));
    Ok(())
}

#[test]
fn register_gemini_tools_adds_two_more() -> AgentResult<()> {
    let mut registry = ToolRegistry::new();
    tools::register_core_tools(&mut registry)?;
    tools::register_gemini_tools(&mut registry)?;

    assert_eq!(registry.len(), 8);
    assert!(registry.names().contains(&"read_many_files"));
    assert!(registry.names().contains(&"list_dir"));
    Ok(())
}

// =========================================================================
// strip_line_numbers tests (2)
// =========================================================================

#[test]
fn strip_line_numbers_basic() {
    let input = "     1 | fn main() {\n     2 |     println!(\"hi\");\n     3 | }";
    let result = tools::strip_line_numbers(input);
    assert_eq!(result, "fn main() {\n    println!(\"hi\");\n}");
}

#[test]
fn strip_line_numbers_preserves_trailing_newline() {
    let input = "     1 | hello\n     2 | world\n";
    let result = tools::strip_line_numbers(input);
    assert_eq!(result, "hello\nworld\n");
}

#[test]
fn strip_line_numbers_no_trailing_newline() {
    let input = "     1 | hello\n     2 | world";
    let result = tools::strip_line_numbers(input);
    assert_eq!(result, "hello\nworld");
}

#[test]
fn strip_line_numbers_passthrough() {
    // Lines without " | " pass through unchanged
    let input = "no line numbers here";
    let result = tools::strip_line_numbers(input);
    assert_eq!(result, "no line numbers here");
}

// =========================================================================
// required_str tests (2)
// =========================================================================

#[test]
fn required_str_extracts_value() -> AgentResult<()> {
    let args = json!({"name": "hello"});
    let val = tools::required_str(&args, "name")?;
    assert_eq!(val, "hello");
    Ok(())
}

#[test]
fn required_str_missing_returns_error() {
    let args = json!({});
    let result = tools::required_str(&args, "name");
    assert!(result.is_err());
    assert!(matches!(result, Err(AgentError::ValidationError { .. })));
}
