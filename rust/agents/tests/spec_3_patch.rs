//! Tests for the `apply_patch` tool: v4a format parser and applicator (spec Appendix A).

#![allow(clippy::result_large_err)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::registry::ToolRegistry;
use stencila_agents::tools;
use stencila_agents::tools::apply_patch::{Hunk, HunkLine, Patch, PatchOperation, parse_patch};
use stencila_agents::types::{DirEntry, ExecResult, GrepOptions};
use stencila_models3::types::tool::ToolDefinition;

// ---------------------------------------------------------------------------
// MockExecutionEnvironment (duplicated — each integration test is a separate crate)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct WriteRecord {
    path: String,
    content: String,
}

#[derive(Clone, Debug)]
enum MockFileContent {
    Text(String),
}

#[derive(Clone)]
struct MockExecutionEnvironment {
    working_dir: String,
    files: Arc<Mutex<HashMap<String, MockFileContent>>>,
    writes: Arc<Mutex<Vec<WriteRecord>>>,
}

impl MockExecutionEnvironment {
    fn new() -> Self {
        Self {
            working_dir: "/mock/workspace".into(),
            files: Arc::new(Mutex::new(HashMap::new())),
            writes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_file(self, path: &str, content: &str) -> Self {
        {
            let mut files = self.files.lock().expect("lock poisoned");
            files.insert(path.into(), MockFileContent::Text(content.into()));
        }
        self
    }

    fn file_content(&self, path: &str) -> Option<String> {
        let files = self.files.lock().expect("lock poisoned");
        files.get(path).map(|c| match c {
            MockFileContent::Text(s) => s.clone(),
        })
    }

    fn has_file(&self, path: &str) -> bool {
        let files = self.files.lock().expect("lock poisoned");
        files.contains_key(path)
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
        }
    }

    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        {
            let mut writes = self.writes.lock().expect("lock poisoned");
            writes.push(WriteRecord {
                path: path.into(),
                content: content.into(),
            });
        }
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

    async fn list_directory(&self, path: &str, _depth: usize) -> AgentResult<Vec<DirEntry>> {
        Err(AgentError::FileNotFound { path: path.into() })
    }

    async fn exec_command(
        &self,
        _command: &str,
        _timeout_ms: u64,
        _working_dir: Option<&str>,
        _env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        Err(AgentError::Io {
            message: "not implemented in mock".into(),
        })
    }

    async fn grep(
        &self,
        _pattern: &str,
        _path: &str,
        _options: &GrepOptions,
    ) -> AgentResult<String> {
        Err(AgentError::Io {
            message: "not implemented in mock".into(),
        })
    }

    async fn glob_files(&self, _pattern: &str, _path: &str) -> AgentResult<Vec<String>> {
        Err(AgentError::Io {
            message: "not implemented in mock".into(),
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
// Schema Parity
// =========================================================================

#[test]
fn apply_patch_schema_matches_fixture() -> Result<(), String> {
    let fixture = load_fixture("apply_patch")?;
    let definition = tools::apply_patch::definition();
    assert_eq!(definition.name, fixture.name);
    assert_eq!(definition.description, fixture.description);
    assert_eq!(definition.parameters, fixture.parameters);
    assert_eq!(definition.strict, fixture.strict);
    Ok(())
}

// =========================================================================
// Parser Tests
// =========================================================================

#[test]
fn parse_add_file() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Add File: src/new.rs
+fn main() {
+    println!(\"hello\");
+}
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    assert_eq!(
        patch.operations[0],
        PatchOperation::AddFile {
            path: "src/new.rs".into(),
            lines: vec![
                "fn main() {".into(),
                "    println!(\"hello\");".into(),
                "}".into(),
            ],
        }
    );
    Ok(())
}

#[test]
fn parse_delete_file() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Delete File: old/junk.rs
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    assert_eq!(
        patch.operations[0],
        PatchOperation::DeleteFile {
            path: "old/junk.rs".into(),
        }
    );
    Ok(())
}

#[test]
fn parse_update_single_hunk() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: src/main.rs
@@ fn main() @@
 fn main() {
-    println!(\"old\");
+    println!(\"new\");
 }
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    match &patch.operations[0] {
        PatchOperation::UpdateFile {
            path,
            move_to,
            hunks,
        } => {
            assert_eq!(path, "src/main.rs");
            assert!(move_to.is_none());
            assert_eq!(hunks.len(), 1);
            assert_eq!(hunks[0].context_hint, "fn main()");
            assert_eq!(hunks[0].lines.len(), 4);
            assert_eq!(hunks[0].lines[0], HunkLine::Context("fn main() {".into()));
            assert_eq!(
                hunks[0].lines[1],
                HunkLine::Delete("    println!(\"old\");".into())
            );
            assert_eq!(
                hunks[0].lines[2],
                HunkLine::Add("    println!(\"new\");".into())
            );
            assert_eq!(hunks[0].lines[3], HunkLine::Context("}".into()));
        }
        other => panic!("expected UpdateFile, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn parse_update_with_move() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: old/path.rs
*** Move to: new/path.rs
@@ context @@
 line1
-old
+new
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    match &patch.operations[0] {
        PatchOperation::UpdateFile {
            path,
            move_to,
            hunks,
        } => {
            assert_eq!(path, "old/path.rs");
            assert_eq!(move_to.as_deref(), Some("new/path.rs"));
            assert_eq!(hunks.len(), 1);
        }
        other => panic!("expected UpdateFile, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn parse_multi_hunk_update() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: src/lib.rs
@@ first hunk @@
 fn a() {
-    old_a();
+    new_a();
 }
@@ second hunk @@
 fn b() {
-    old_b();
+    new_b();
 }
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    match &patch.operations[0] {
        PatchOperation::UpdateFile { hunks, .. } => {
            assert_eq!(hunks.len(), 2);
            assert_eq!(hunks[0].context_hint, "first hunk");
            assert_eq!(hunks[1].context_hint, "second hunk");
        }
        other => panic!("expected UpdateFile, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn parse_eof_marker_before_end_patch() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: src/main.rs
@@ fn main() @@
 fn main() {
-    old();
+    new();
 }
*** End of File
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    match &patch.operations[0] {
        PatchOperation::UpdateFile { hunks, .. } => {
            assert_eq!(hunks.len(), 1);
        }
        other => panic!("expected UpdateFile, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn parse_eof_marker_between_operations() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: a.rs
@@ hint @@
-old
+new
*** End of File
*** Add File: b.rs
+hello
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 2);
    assert!(matches!(
        &patch.operations[0],
        PatchOperation::UpdateFile { .. }
    ));
    assert!(matches!(
        &patch.operations[1],
        PatchOperation::AddFile { .. }
    ));
    Ok(())
}

#[test]
fn parse_bare_at_at_hunk_header() -> AgentResult<()> {
    let input = "\
*** Begin Patch
*** Update File: src/main.rs
@@
 fn main() {
-    println!(\"old\");
+    println!(\"new\");
 }
*** End Patch";

    let patch = parse_patch(input)?;
    assert_eq!(patch.operations.len(), 1);
    match &patch.operations[0] {
        PatchOperation::UpdateFile { hunks, .. } => {
            assert_eq!(hunks.len(), 1);
            assert_eq!(hunks[0].context_hint, "");
            assert_eq!(hunks[0].lines.len(), 4);
        }
        other => panic!("expected UpdateFile, got: {other:?}"),
    }
    Ok(())
}

// =========================================================================
// Parse Error Tests
// =========================================================================

#[test]
fn parse_error_missing_begin() {
    let input = "*** Add File: foo.rs\n+content\n*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("Begin Patch")),
        "expected error about missing Begin Patch, got: {result:?}"
    );
}

#[test]
fn parse_error_missing_end() {
    let input = "*** Begin Patch\n*** Add File: foo.rs\n+content";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("End Patch")),
        "expected error about missing End Patch, got: {result:?}"
    );
}

#[test]
fn parse_error_update_without_hunks() {
    let input = "*** Begin Patch\n*** Update File: foo.rs\n*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("has no hunks")),
        "expected error about missing hunks, got: {result:?}"
    );
}

#[test]
fn parse_error_trailing_content_after_end_patch() {
    let input = "\
*** Begin Patch
*** End Patch
*** Add File: foo.rs
+content
*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("unexpected content after '*** End Patch'")),
        "expected error about trailing content, got: {result:?}"
    );
}

#[test]
fn parse_error_empty_hunk_lines() {
    // @@ header immediately followed by *** End of File — zero hunk lines
    let input = "\
*** Begin Patch
*** Update File: foo.rs
@@ hint @@
*** End of File
*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("hunk has no lines")),
        "expected error about empty hunk, got: {result:?}"
    );
}

#[test]
fn parse_error_empty_hunk_before_next_hunk() {
    // @@ header immediately followed by another @@ header — zero hunk lines
    let input = "\
*** Begin Patch
*** Update File: foo.rs
@@ empty @@
@@ real @@
 context
-old
+new
*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("hunk has no lines")),
        "expected error about empty hunk, got: {result:?}"
    );
}

#[test]
fn parse_error_empty_hunk_before_operation() {
    // @@ header immediately followed by another operation — zero hunk lines
    let input = "\
*** Begin Patch
*** Update File: foo.rs
@@ empty @@
*** Add File: bar.rs
+content
*** End Patch";
    let result = parse_patch(input);
    assert!(
        matches!(&result, Err(AgentError::ValidationError { reason }) if reason.contains("hunk has no lines")),
        "expected error about empty hunk, got: {result:?}"
    );
}

// =========================================================================
// Applicator Tests
// =========================================================================

#[tokio::test]
async fn apply_add_file_creates_file() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new();
    let patch = Patch {
        operations: vec![PatchOperation::AddFile {
            path: "new.txt".into(),
            lines: vec!["hello".into(), "world".into()],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].contains("Created"));

    let content = env.file_content("new.txt");
    assert_eq!(content.as_deref(), Some("hello\nworld\n"));
    Ok(())
}

#[tokio::test]
async fn apply_delete_file_removes_file() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file("doomed.txt", "bye");
    let patch = Patch {
        operations: vec![PatchOperation::DeleteFile {
            path: "doomed.txt".into(),
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].contains("Deleted"));
    assert!(!env.has_file("doomed.txt"));
    Ok(())
}

#[tokio::test]
async fn apply_update_file_single_hunk() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new()
        .with_file("src/main.rs", "fn main() {\n    println!(\"old\");\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "fn main()".into(),
                lines: vec![
                    HunkLine::Context("fn main() {".into()),
                    HunkLine::Delete("    println!(\"old\");".into()),
                    HunkLine::Add("    println!(\"new\");".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/main.rs");
    assert_eq!(
        content.as_deref(),
        Some("fn main() {\n    println!(\"new\");\n}\n")
    );
    Ok(())
}

#[tokio::test]
async fn apply_update_file_multi_hunk() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn a() {\n    old_a();\n}\n\nfn b() {\n    old_b();\n}",
    );

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![
                Hunk {
                    context_hint: "first".into(),
                    lines: vec![
                        HunkLine::Context("fn a() {".into()),
                        HunkLine::Delete("    old_a();".into()),
                        HunkLine::Add("    new_a();".into()),
                        HunkLine::Context("}".into()),
                    ],
                },
                Hunk {
                    context_hint: "second".into(),
                    lines: vec![
                        HunkLine::Context("fn b() {".into()),
                        HunkLine::Delete("    old_b();".into()),
                        HunkLine::Add("    new_b();".into()),
                        HunkLine::Context("}".into()),
                    ],
                },
            ],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("2 hunks"));

    let content = env.file_content("src/lib.rs");
    let c = content.as_deref().unwrap_or("");
    assert!(c.contains("new_a()"), "got: {c}");
    assert!(c.contains("new_b()"), "got: {c}");
    assert!(!c.contains("old_a()"), "got: {c}");
    assert!(!c.contains("old_b()"), "got: {c}");
    Ok(())
}

#[tokio::test]
async fn apply_update_with_move_renames() -> AgentResult<()> {
    let env = MockExecutionEnvironment::new().with_file("old/file.rs", "fn foo() {\n    old();\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "old/file.rs".into(),
            move_to: Some("new/file.rs".into()),
            hunks: vec![Hunk {
                context_hint: "foo".into(),
                lines: vec![
                    HunkLine::Context("fn foo() {".into()),
                    HunkLine::Delete("    old();".into()),
                    HunkLine::Add("    new();".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("moved"));
    assert!(env.has_file("new/file.rs"));
    assert!(!env.has_file("old/file.rs"));

    let content = env.file_content("new/file.rs");
    let c = content.as_deref().unwrap_or("");
    assert!(c.contains("new()"), "got: {c}");
    Ok(())
}

#[tokio::test]
async fn apply_update_move_to_same_path() -> AgentResult<()> {
    // move_to == path should NOT delete the file
    let env = MockExecutionEnvironment::new().with_file("same.rs", "fn f() {\n    old();\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "same.rs".into(),
            move_to: Some("same.rs".into()),
            hunks: vec![Hunk {
                context_hint: "f".into(),
                lines: vec![
                    HunkLine::Context("fn f() {".into()),
                    HunkLine::Delete("    old();".into()),
                    HunkLine::Add("    new();".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("moved"));
    assert!(env.has_file("same.rs"), "file should still exist");

    let content = env.file_content("same.rs");
    let c = content.as_deref().unwrap_or("");
    assert!(c.contains("new()"), "got: {c}");
    Ok(())
}

#[tokio::test]
async fn apply_update_file_not_found() {
    let env = MockExecutionEnvironment::new();
    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "nonexistent.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "x".into(),
                lines: vec![HunkLine::Context("x".into())],
            }],
        }],
    };

    let result = tools::apply_patch::apply_patch_ops(&patch, &env).await;
    assert!(matches!(result, Err(AgentError::FileNotFound { .. })));
}

#[tokio::test]
async fn apply_hunk_mismatch_returns_edit_conflict() {
    let env = MockExecutionEnvironment::new()
        .with_file("src/main.rs", "fn main() {\n    something_else();\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "missing context".into(),
                lines: vec![
                    HunkLine::Context("this line does not exist".into()),
                    HunkLine::Delete("neither does this".into()),
                    HunkLine::Add("replacement".into()),
                ],
            }],
        }],
    };

    let result = tools::apply_patch::apply_patch_ops(&patch, &env).await;
    assert!(
        matches!(&result, Err(AgentError::EditConflict { reason }) if reason.contains("could not locate hunk")),
        "expected EditConflict, got: {result:?}"
    );
}

// =========================================================================
// Fuzzy Match Test
// =========================================================================

#[tokio::test]
async fn apply_update_fuzzy_whitespace_match() -> AgentResult<()> {
    // File has different whitespace than the hunk expects
    let env = MockExecutionEnvironment::new().with_file("ws.txt", "fn   main()  {\n    old();\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "ws.txt".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "main".into(),
                lines: vec![
                    HunkLine::Context("fn main() {".into()),
                    HunkLine::Delete("    old();".into()),
                    HunkLine::Add("    new();".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("ws.txt");
    let c = content.as_deref().unwrap_or("");
    assert!(c.contains("new()"), "got: {c}");
    Ok(())
}

// =========================================================================
// Large File Test — verifies >2000 line files are fully read
// =========================================================================

#[tokio::test]
async fn apply_update_file_beyond_2000_lines() -> AgentResult<()> {
    // Build a file with 3000 lines; the hunk targets line ~2500
    let mut lines: Vec<String> = (1..=3000).map(|i| format!("line {i}")).collect();
    let content = lines.join("\n");
    let env = MockExecutionEnvironment::new().with_file("big.txt", &content);

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "big.txt".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "line 2500".into(),
                lines: vec![
                    HunkLine::Context("line 2499".into()),
                    HunkLine::Delete("line 2500".into()),
                    HunkLine::Add("REPLACED 2500".into()),
                    HunkLine::Context("line 2501".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let result = env.file_content("big.txt").unwrap_or_default();
    assert!(
        result.contains("REPLACED 2500"),
        "hunk beyond line 2000 should apply"
    );
    assert!(
        !result.contains("\nline 2500\n"),
        "original line 2500 should be removed"
    );
    // Verify the file still has ~3000 lines
    lines[2499] = "REPLACED 2500".into();
    let line_count = result.lines().count();
    assert_eq!(line_count, 3000, "file should still have 3000 lines");
    Ok(())
}

// =========================================================================
// Executor End-to-End Test
// =========================================================================

#[tokio::test]
async fn apply_patch_executor_end_to_end() -> AgentResult<()> {
    let env =
        MockExecutionEnvironment::new().with_file("src/lib.rs", "fn hello() {\n    old();\n}");

    let patch_str = "\
*** Begin Patch
*** Update File: src/lib.rs
@@ hello @@
 fn hello() {
-    old();
+    new();
 }
*** End Patch";

    let exec = tools::apply_patch::executor();
    let result = exec(json!({"patch": patch_str}), &env).await?;
    assert!(result.as_text().contains("Updated"));

    let content = env.file_content("src/lib.rs");
    let c = content.as_deref().unwrap_or("");
    assert!(c.contains("new()"), "got: {c}");
    assert!(!c.contains("old()"), "got: {c}");
    Ok(())
}

// =========================================================================
// Context-hint proximity disambiguation (spec App A line 1368)
// =========================================================================

#[tokio::test]
async fn context_hint_disambiguates_repeated_pattern() -> AgentResult<()> {
    // File with two identical blocks — only the function names differ.
    // The context_hint should steer the hunk to the correct block.
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn alpha() {\n    do_work();\n}\n\nfn beta() {\n    do_work();\n}\n",
    );

    // Hunk targets the "do_work()" line (matches in both blocks).
    // context_hint says "beta", so the second block should be patched.
    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "fn beta()".into(),
                lines: vec![
                    HunkLine::Context("    do_work();".into()),
                    HunkLine::Add("    do_extra_work();".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // alpha block should be untouched
    assert!(
        content.contains("fn alpha() {\n    do_work();\n}"),
        "alpha block should be unchanged: {content}"
    );
    // beta block should have the addition
    assert!(
        content.contains("fn beta() {\n    do_work();\n    do_extra_work();\n}"),
        "beta block should have the new line: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn context_hint_empty_falls_back_to_first_match() -> AgentResult<()> {
    // When context_hint is empty, first match wins (backward-compatible).
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn alpha() {\n    do_work();\n}\n\nfn beta() {\n    do_work();\n}\n",
    );

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    HunkLine::Context("    do_work();".into()),
                    HunkLine::Add("    first_match();".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // First match (alpha block) should be patched
    assert!(
        content.contains("fn alpha() {\n    do_work();\n    first_match();\n}"),
        "first block should be patched when hint is empty: {content}"
    );
    // Second block should remain unchanged
    assert!(
        content.contains("fn beta() {\n    do_work();\n}"),
        "second block should be untouched: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn context_hint_not_found_falls_back_to_first_match() -> AgentResult<()> {
    // When context_hint text isn't found anywhere in the file, first match wins.
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn alpha() {\n    do_work();\n}\n\nfn beta() {\n    do_work();\n}\n",
    );

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "fn nonexistent_function".into(),
                lines: vec![
                    HunkLine::Context("    do_work();".into()),
                    HunkLine::Add("    fallback();".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // First match (alpha block) should be patched as fallback
    assert!(
        content.contains("fn alpha() {\n    do_work();\n    fallback();\n}"),
        "first block should be patched when hint not found: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn context_hint_fuzzy_whitespace_with_disambiguation() -> AgentResult<()> {
    // File with repeated patterns; the hunk lines differ only in whitespace
    // from the file. context_hint should still disambiguate.
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn first() {\n  val = 1;\n}\n\nfn second() {\n  val = 1;\n}\n",
    );

    // Hunk context uses different whitespace ("    val = 1;" vs "  val = 1;")
    // so exact match fails, fuzzy matches both. Hint selects second.
    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "fn second()".into(),
                lines: vec![
                    HunkLine::Delete("    val = 1;".into()),
                    HunkLine::Add("    val = 2;".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // first() should still have val = 1
    assert!(
        content.contains("fn first() {\n  val = 1;\n}"),
        "first block should be unchanged: {content}"
    );
    // second() should have val = 2
    assert!(
        content.contains("fn second() {\n    val = 2;\n}"),
        "second block should be updated: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn context_hint_ignores_earlier_comment_substring() -> AgentResult<()> {
    // The hint text "fn beta()" appears as a substring in a comment on line 0,
    // AND as an exact full-line match on line 5. The disambiguation should
    // prefer the exact full-line match, not the comment substring.
    let file = "\
# This module provides fn alpha() and fn beta() helpers
fn alpha() {
    do_work();
}

fn beta() {
    do_work();
}
";
    let env = MockExecutionEnvironment::new().with_file("src/lib.rs", file);

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "fn beta()".into(),
                lines: vec![
                    HunkLine::Context("    do_work();".into()),
                    HunkLine::Add("    do_extra();".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // alpha block should be untouched (the comment mentioning "fn beta" must
    // not attract the match toward alpha's do_work() line).
    assert!(
        content.contains("fn alpha() {\n    do_work();\n}"),
        "alpha block should be unchanged: {content}"
    );
    // beta block should have the addition
    assert!(
        content.contains("fn beta() {\n    do_work();\n    do_extra();\n}"),
        "beta block should have the new line: {content}"
    );
    Ok(())
}

// =========================================================================
// Unicode Punctuation Normalization (spec App A line 1370)
// =========================================================================

#[tokio::test]
async fn fuzzy_match_smart_quotes() -> AgentResult<()> {
    // File uses straight quotes, but the patch has smart (curly) quotes.
    let env = MockExecutionEnvironment::new()
        .with_file("src/main.rs", "fn greet() {\n    println!(\"hello\");\n}");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "greet".into(),
                lines: vec![
                    HunkLine::Context("fn greet() {".into()),
                    // U+201C / U+201D smart double quotes
                    HunkLine::Delete("    println!(\u{201C}hello\u{201D});".into()),
                    HunkLine::Add("    println!(\"world\");".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/main.rs").unwrap_or_default();
    assert!(
        content.contains("println!(\"world\")"),
        "smart quotes should fuzzy-match straight quotes: {content}"
    );
    assert!(
        !content.contains("hello"),
        "old line should be deleted: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn fuzzy_match_single_smart_quotes() -> AgentResult<()> {
    // File uses straight single quotes (apostrophes), patch uses curly.
    let env =
        MockExecutionEnvironment::new().with_file("src/main.rs", "let s = 'x';\nlet t = 'y';");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // U+2018 / U+2019 smart single quotes
                    HunkLine::Delete("let s = \u{2018}x\u{2019};".into()),
                    HunkLine::Add("let s = 'z';".into()),
                    HunkLine::Context("let t = 'y';".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/main.rs").unwrap_or_default();
    assert!(
        content.contains("let s = 'z';"),
        "smart single quotes should fuzzy-match: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn fuzzy_match_em_dash() -> AgentResult<()> {
    // File uses a hyphen-minus, patch uses an em-dash.
    let env = MockExecutionEnvironment::new().with_file("README.md", "title - subtitle\nbody text");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "README.md".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // U+2014 em-dash
                    HunkLine::Delete("title \u{2014} subtitle".into()),
                    HunkLine::Add("title - new subtitle".into()),
                    HunkLine::Context("body text".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("README.md").unwrap_or_default();
    assert!(
        content.contains("title - new subtitle"),
        "em-dash should fuzzy-match hyphen: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn fuzzy_match_en_dash_and_minus_sign() -> AgentResult<()> {
    // File uses hyphen-minus; patch uses en-dash and minus sign.
    let env = MockExecutionEnvironment::new().with_file("math.txt", "x = a - b\ny = c - d");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "math.txt".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // U+2013 en-dash
                    HunkLine::Delete("x = a \u{2013} b".into()),
                    HunkLine::Add("x = a + b".into()),
                    // U+2212 minus sign
                    HunkLine::Context("y = c \u{2212} d".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("math.txt").unwrap_or_default();
    assert!(
        content.contains("x = a + b"),
        "en-dash/minus should fuzzy-match hyphen: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn fuzzy_match_ellipsis() -> AgentResult<()> {
    // File uses three dots, patch uses horizontal ellipsis (U+2026).
    let env = MockExecutionEnvironment::new().with_file(
        "src/main.rs",
        "fn todo() {\n    unimplemented!(\"...\");\n}",
    );

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: "todo".into(),
                lines: vec![
                    HunkLine::Context("fn todo() {".into()),
                    // U+2026 horizontal ellipsis
                    HunkLine::Delete("    unimplemented!(\"\u{2026}\");".into()),
                    HunkLine::Add("    todo!();".into()),
                    HunkLine::Context("}".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/main.rs").unwrap_or_default();
    assert!(
        content.contains("todo!()"),
        "ellipsis should fuzzy-match three dots: {content}"
    );
    assert!(
        !content.contains("unimplemented"),
        "old line should be deleted: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn fuzzy_match_combined_unicode_and_whitespace() -> AgentResult<()> {
    // Both whitespace differences AND Unicode punctuation in the same hunk.
    let env =
        MockExecutionEnvironment::new().with_file("doc.txt", "He said  \"yes\" - then left...");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "doc.txt".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // smart quotes + em-dash + ellipsis + different spacing
                    HunkLine::Delete(
                        "He said \u{201C}yes\u{201D} \u{2014} then left\u{2026}".into(),
                    ),
                    HunkLine::Add("He said \"no\" - then stayed.".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("doc.txt").unwrap_or_default();
    assert!(
        content.contains("He said \"no\" - then stayed."),
        "combined normalization should work: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn context_hint_with_unicode_punctuation() -> AgentResult<()> {
    // The context_hint contains Unicode punctuation (smart quotes); hint
    // matching should find the correct line via normalized comparison (tier 3).
    // File has straight quotes — the hint must go through normalization to match.
    let env = MockExecutionEnvironment::new().with_file(
        "src/lib.rs",
        "fn alpha() {\n    let tag = \"alpha\";\n    do_work();\n}\n\nfn beta() {\n    let tag = \"beta\";\n    do_work();\n}\n",
    );

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/lib.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                // Smart double quotes in hint: U+201C / U+201D.
                // File has: let tag = "beta";  (straight quotes)
                // Normalized hint becomes: let tag = "beta";
                // This can only match via tier 3 (normalized substring).
                context_hint: "let tag = \u{201C}beta\u{201D};".into(),
                lines: vec![
                    HunkLine::Context("    do_work();".into()),
                    HunkLine::Add("    extra();".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/lib.rs").unwrap_or_default();
    // alpha block should be untouched — the hint should NOT match "alpha"
    assert!(
        content.contains("fn alpha() {\n    let tag = \"alpha\";\n    do_work();\n}"),
        "alpha should be unchanged: {content}"
    );
    // beta block should have the addition
    assert!(
        content.contains("fn beta() {\n    let tag = \"beta\";\n    do_work();\n    extra();\n}"),
        "beta should have addition: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn exact_match_preferred_over_unicode_fuzzy() -> AgentResult<()> {
    // When exact match succeeds, Unicode normalization should NOT be needed.
    // This test verifies the exact-match-first priority is preserved.
    let env = MockExecutionEnvironment::new()
        .with_file("src/main.rs", "let x = \"hello\";\nlet y = \"world\";");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // Exact match — straight quotes in both file and patch
                    HunkLine::Delete("let x = \"hello\";".into()),
                    HunkLine::Add("let x = \"greetings\";".into()),
                    HunkLine::Context("let y = \"world\";".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await?;
    assert!(summaries[0].contains("Updated"));

    let content = env.file_content("src/main.rs").unwrap_or_default();
    assert!(
        content.contains("let x = \"greetings\";"),
        "exact match should work: {content}"
    );
    Ok(())
}

#[tokio::test]
async fn non_equivalent_unicode_still_fails() {
    // Unicode characters NOT in the normalization table must not match ASCII.
    // Greek mu (U+03BC) should not fuzzy-match ASCII 'u', so the hunk should
    // fail with EditConflict.
    let env =
        MockExecutionEnvironment::new().with_file("src/main.rs", "let unit = 42;\nlet other = 0;");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    // Greek mu (μ) replacing ASCII 'u' — not equivalent
                    HunkLine::Delete("let \u{03BC}nit = 42;".into()),
                    HunkLine::Add("let unit = 99;".into()),
                    HunkLine::Context("let other = 0;".into()),
                ],
            }],
        }],
    };

    let result = tools::apply_patch::apply_patch_ops(&patch, &env).await;
    assert!(
        matches!(&result, Err(AgentError::EditConflict { reason }) if reason.contains("could not locate hunk")),
        "non-equivalent Unicode should not fuzzy-match: {result:?}"
    );
}

#[tokio::test]
async fn normalization_does_not_over_match_different_quote_styles() {
    // File has two lines that differ only in quote style. The patch targets
    // the backtick version. Since backticks are NOT normalized to straight
    // quotes, the exact match should select the correct line without fuzzy
    // matching conflating the two.
    let env = MockExecutionEnvironment::new()
        .with_file("src/main.rs", "let a = `hello`;\nlet b = \"hello\";");

    let patch = Patch {
        operations: vec![PatchOperation::UpdateFile {
            path: "src/main.rs".into(),
            move_to: None,
            hunks: vec![Hunk {
                context_hint: String::new(),
                lines: vec![
                    HunkLine::Context("let a = `hello`;".into()),
                    HunkLine::Delete("let b = \"hello\";".into()),
                    HunkLine::Add("let b = \"world\";".into()),
                ],
            }],
        }],
    };

    let summaries = tools::apply_patch::apply_patch_ops(&patch, &env).await;
    assert!(summaries.is_ok(), "should match exactly: {summaries:?}");

    let content = env.file_content("src/main.rs").unwrap_or_default();
    assert!(
        content.contains("let a = `hello`;"),
        "backtick line should be unchanged: {content}"
    );
    assert!(
        content.contains("let b = \"world\";"),
        "double-quote line should be updated: {content}"
    );
}

// =========================================================================
// Registration Test
// =========================================================================

/// NOTE: This test asserts the current OpenAI tool set (core + apply_patch).
/// When Phase 7a profiles are implemented, the OpenAI profile may replace
/// `edit_file` with `apply_patch` (spec §3.3 line 586), changing the count.
#[test]
fn register_openai_tools_adds_one() -> AgentResult<()> {
    let mut registry = ToolRegistry::new();
    tools::register_core_tools(&mut registry)?;
    let before = registry.len();
    tools::register_openai_tools(&mut registry)?;

    assert_eq!(registry.len(), before + 1);
    assert!(registry.names().contains(&"apply_patch"));
    Ok(())
}
