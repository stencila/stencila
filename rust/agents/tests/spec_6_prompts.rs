//! Tests for system prompts and project doc discovery (spec 6.1-6.5).
//!
//! Phase 7b: layered prompt construction, environment context, git context,
//! provider-specific base instructions, project doc discovery with filtering.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::profile::ProviderProfile;
use stencila_agents::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use stencila_agents::project_docs::discover_project_docs;
use stencila_agents::prompts::{
    GitContext, build_environment_context, format_git_summary, gather_git_context,
};
use stencila_agents::types::{DirEntry, ExecResult, GrepOptions};

// ---------------------------------------------------------------------------
// Mock ExecutionEnvironment for prompt/doc tests
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct MockEnv {
    working_dir: String,
    platform: String,
    os_version: String,
    files: Arc<Mutex<HashMap<String, String>>>,
    commands: Arc<Mutex<HashMap<String, ExecResult>>>,
}

impl MockEnv {
    fn new(working_dir: &str) -> Self {
        Self {
            working_dir: working_dir.into(),
            platform: "linux".into(),
            os_version: "linux x86_64".into(),
            files: Arc::new(Mutex::new(HashMap::new())),
            commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn with_file(self, path: &str, content: &str) -> Self {
        {
            let mut files = self.files.lock().expect("lock poisoned");
            files.insert(path.into(), content.into());
        }
        self
    }

    fn with_command(self, cmd: &str, result: ExecResult) -> Self {
        {
            let mut commands = self.commands.lock().expect("lock poisoned");
            commands.insert(cmd.into(), result);
        }
        self
    }

    fn with_git_repo(self, branch: &str) -> Self {
        self.with_command(
            "git rev-parse --abbrev-ref HEAD",
            ExecResult {
                stdout: format!("{branch}\n"),
                stderr: String::new(),
                exit_code: 0,
                timed_out: false,
                duration_ms: 5,
            },
        )
        .with_command(
            "git status --short",
            ExecResult {
                stdout: " M src/main.rs\n?? new_file.txt\n?? other.tmp\n".into(),
                stderr: String::new(),
                exit_code: 0,
                timed_out: false,
                duration_ms: 5,
            },
        )
        .with_command(
            "git log --oneline -10",
            ExecResult {
                stdout: "abc1234 feat: add feature\ndef5678 fix: bug fix\n".into(),
                stderr: String::new(),
                exit_code: 0,
                timed_out: false,
                duration_ms: 5,
            },
        )
        .with_command(
            "git rev-parse --show-toplevel",
            ExecResult {
                stdout: "/project\n".into(),
                stderr: String::new(),
                exit_code: 0,
                timed_out: false,
                duration_ms: 5,
            },
        )
    }
}

impl std::fmt::Debug for MockEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockEnv")
            .field("working_dir", &self.working_dir)
            .finish()
    }
}

#[async_trait]
impl ExecutionEnvironment for MockEnv {
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

        let lines: Vec<&str> = content.lines().collect();
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

    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        let mut files = self.files.lock().expect("lock poisoned");
        files.insert(path.into(), content.into());
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> bool {
        let files = self.files.lock().expect("lock poisoned");
        files.contains_key(path)
    }

    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        let mut files = self.files.lock().expect("lock poisoned");
        files
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| AgentError::FileNotFound { path: path.into() })
    }

    async fn list_directory(&self, path: &str, _depth: usize) -> AgentResult<Vec<DirEntry>> {
        Err(AgentError::FileNotFound { path: path.into() })
    }

    async fn exec_command(
        &self,
        command: &str,
        _timeout_ms: u64,
        _working_dir: Option<&str>,
        _env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        let commands = self.commands.lock().expect("lock poisoned");
        commands
            .get(command)
            .cloned()
            .ok_or_else(|| AgentError::Io {
                message: format!("no mock command: {command}"),
            })
    }

    async fn grep(
        &self,
        _pattern: &str,
        _path: &str,
        _options: &GrepOptions,
    ) -> AgentResult<String> {
        Ok(String::new())
    }

    async fn glob_files(&self, _pattern: &str, _path: &str) -> AgentResult<Vec<String>> {
        Ok(Vec::new())
    }

    fn working_directory(&self) -> &str {
        &self.working_dir
    }

    fn platform(&self) -> &str {
        &self.platform
    }

    fn os_version(&self) -> String {
        self.os_version.clone()
    }
}

// =========================================================================
// Environment context (spec 6.3)
// =========================================================================

#[test]
fn environment_context_format() {
    let git = GitContext {
        is_repo: true,
        branch: "main".into(),
        modified_count: 2,
        untracked_count: 1,
        recent_commits: vec!["abc1234 feat: stuff".into()],
    };

    let ctx = build_environment_context(
        "/project",
        "linux",
        "linux x86_64",
        "gpt-5.2",
        Some("2025-04"),
        &git,
    );

    assert!(ctx.starts_with("<environment>"));
    assert!(ctx.ends_with("</environment>"));
    assert!(ctx.contains("Working directory: /project"));
    assert!(ctx.contains("Is git repository: true"));
    assert!(ctx.contains("Git branch: main"));
    assert!(ctx.contains("Platform: linux"));
    assert!(ctx.contains("OS version: linux x86_64"));
    assert!(ctx.contains("Today's date:"));
    assert!(ctx.contains("Model: gpt-5.2"));
    assert!(ctx.contains("Knowledge cutoff: 2025-04"));
    // Git status counts included (spec 6.4)
    assert!(ctx.contains("2 modified/staged"));
    assert!(ctx.contains("1 untracked"));
    // Recent commits included (spec 6.4)
    assert!(ctx.contains("Recent commits:"));
    assert!(ctx.contains("abc1234 feat: stuff"));
}

#[test]
fn environment_context_no_git() {
    let git = GitContext::default();

    let ctx = build_environment_context(
        "/project",
        "darwin",
        "macos arm64",
        "claude-opus-4-6",
        None,
        &git,
    );

    assert!(ctx.contains("Is git repository: false"));
    // Git branch line should not appear when not a repo
    assert!(!ctx.contains("Git branch:"));
    assert!(ctx.contains("Platform: darwin"));
    assert!(ctx.contains("Model: claude-opus-4-6"));
    // No knowledge cutoff when None
    assert!(!ctx.contains("Knowledge cutoff:"));
    // No git status or commits when not a repo
    assert!(!ctx.contains("Git status:"));
    assert!(!ctx.contains("Recent commits:"));
}

#[test]
fn environment_context_date_is_present() {
    let git = GitContext::default();
    let ctx = build_environment_context("/w", "linux", "linux x86_64", "m", None, &git);

    // The date line should match YYYY-MM-DD format
    let date_line = ctx
        .lines()
        .find(|l| l.starts_with("Today's date:"))
        .expect("date line missing");
    let date_part = date_line.split(": ").nth(1).expect("no date value");
    assert_eq!(date_part.len(), 10); // YYYY-MM-DD
    assert_eq!(&date_part[4..5], "-");
    assert_eq!(&date_part[7..8], "-");
}

// =========================================================================
// Git context (spec 6.4)
// =========================================================================

#[tokio::test]
async fn gather_git_context_in_repo() -> AgentResult<()> {
    let env = MockEnv::new("/project").with_git_repo("main");

    let ctx = gather_git_context(&env).await;

    assert!(ctx.is_repo);
    assert_eq!(ctx.branch, "main");
    assert_eq!(ctx.modified_count, 1); // " M src/main.rs"
    assert_eq!(ctx.untracked_count, 2); // "?? new_file.txt", "?? other.tmp"
    assert_eq!(ctx.recent_commits.len(), 2);
    assert!(ctx.recent_commits[0].contains("feat: add feature"));
    Ok(())
}

#[tokio::test]
async fn gather_git_context_not_a_repo() -> AgentResult<()> {
    // No git commands configured → all will fail
    let env = MockEnv::new("/no-repo");

    let ctx = gather_git_context(&env).await;

    assert!(!ctx.is_repo);
    assert!(ctx.branch.is_empty());
    assert_eq!(ctx.modified_count, 0);
    assert_eq!(ctx.untracked_count, 0);
    assert!(ctx.recent_commits.is_empty());
    Ok(())
}

#[test]
fn format_git_summary_with_data() {
    let git = GitContext {
        is_repo: true,
        branch: "feature/x".into(),
        modified_count: 3,
        untracked_count: 1,
        recent_commits: vec!["abc fix stuff".into(), "def add thing".into()],
    };

    let summary = format_git_summary(&git);

    assert!(summary.contains("Git branch: feature/x"));
    assert!(summary.contains("3 modified/staged files"));
    assert!(summary.contains("1 untracked file"));
    assert!(summary.contains("Recent commits:"));
    assert!(summary.contains("abc fix stuff"));
}

#[test]
fn format_git_summary_not_a_repo() {
    let git = GitContext::default();
    assert!(format_git_summary(&git).is_empty());
}

// =========================================================================
// Project doc discovery (spec 6.5)
// =========================================================================

#[tokio::test]
async fn discover_agents_md_always_loaded() -> AgentResult<()> {
    let env = MockEnv::new("/project").with_file("/project/AGENTS.md", "Universal instructions");

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project").await?;
    assert!(docs.contains("Universal instructions"));

    let docs = discover_project_docs(&env, "openai", "/project", "/project").await?;
    assert!(docs.contains("Universal instructions"));

    let docs = discover_project_docs(&env, "gemini", "/project", "/project").await?;
    assert!(docs.contains("Universal instructions"));
    Ok(())
}

#[tokio::test]
async fn discover_anthropic_loads_claude_md_not_gemini() -> AgentResult<()> {
    let env = MockEnv::new("/project")
        .with_file("/project/AGENTS.md", "Universal")
        .with_file("/project/CLAUDE.md", "Anthropic-specific")
        .with_file("/project/GEMINI.md", "Gemini-specific")
        .with_file("/project/.codex/instructions.md", "OpenAI-specific");

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project").await?;
    assert!(docs.contains("Universal"));
    assert!(docs.contains("Anthropic-specific"));
    assert!(!docs.contains("Gemini-specific"));
    assert!(!docs.contains("OpenAI-specific"));
    Ok(())
}

#[tokio::test]
async fn discover_openai_loads_codex_not_claude() -> AgentResult<()> {
    let env = MockEnv::new("/project")
        .with_file("/project/AGENTS.md", "Universal")
        .with_file("/project/CLAUDE.md", "Anthropic-specific")
        .with_file("/project/GEMINI.md", "Gemini-specific")
        .with_file("/project/.codex/instructions.md", "OpenAI-specific");

    let docs = discover_project_docs(&env, "openai", "/project", "/project").await?;
    assert!(docs.contains("Universal"));
    assert!(docs.contains("OpenAI-specific"));
    assert!(!docs.contains("Anthropic-specific"));
    assert!(!docs.contains("Gemini-specific"));
    Ok(())
}

#[tokio::test]
async fn discover_gemini_loads_gemini_md_not_claude() -> AgentResult<()> {
    let env = MockEnv::new("/project")
        .with_file("/project/AGENTS.md", "Universal")
        .with_file("/project/CLAUDE.md", "Anthropic-specific")
        .with_file("/project/GEMINI.md", "Gemini-specific")
        .with_file("/project/.codex/instructions.md", "OpenAI-specific");

    let docs = discover_project_docs(&env, "gemini", "/project", "/project").await?;
    assert!(docs.contains("Universal"));
    assert!(docs.contains("Gemini-specific"));
    assert!(!docs.contains("Anthropic-specific"));
    assert!(!docs.contains("OpenAI-specific"));
    Ok(())
}

#[tokio::test]
async fn discover_root_first_subdirectory_appended() -> AgentResult<()> {
    let env = MockEnv::new("/project/sub")
        .with_file("/project/AGENTS.md", "Root instructions")
        .with_file("/project/sub/AGENTS.md", "Sub instructions");

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project/sub").await?;

    // Root comes first, sub is appended
    let root_pos = docs.find("Root instructions").expect("root doc missing");
    let sub_pos = docs.find("Sub instructions").expect("sub doc missing");
    assert!(
        root_pos < sub_pos,
        "Root instructions should come before sub instructions"
    );
    Ok(())
}

#[tokio::test]
async fn discover_32kb_budget_enforcement() -> AgentResult<()> {
    // Create a file larger than 32KB
    let big_content = "x".repeat(33 * 1024);
    let env = MockEnv::new("/project").with_file("/project/AGENTS.md", &big_content);

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project").await?;
    assert!(docs.contains("[Project instructions truncated at 32KB]"));
    Ok(())
}

#[tokio::test]
async fn discover_budget_stops_at_second_file() -> AgentResult<()> {
    // First file fills most of the budget
    let big = "a".repeat(30 * 1024);
    // Second file pushes over
    let medium = "b".repeat(5 * 1024);
    let env = MockEnv::new("/project")
        .with_file("/project/AGENTS.md", &big)
        .with_file("/project/CLAUDE.md", &medium);

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project").await?;
    assert!(docs.contains("[Project instructions truncated at 32KB]"));
    // First file content should be present
    assert!(docs.contains('a'));
    Ok(())
}

#[tokio::test]
async fn discover_no_docs_returns_empty() -> AgentResult<()> {
    let env = MockEnv::new("/project");
    let docs = discover_project_docs(&env, "anthropic", "/project", "/project").await?;
    assert!(docs.is_empty());
    Ok(())
}

#[tokio::test]
async fn discover_nested_path() -> AgentResult<()> {
    let env = MockEnv::new("/project/src/lib")
        .with_file("/project/AGENTS.md", "Root")
        .with_file("/project/src/AGENTS.md", "Src level")
        .with_file("/project/src/lib/AGENTS.md", "Lib level");

    let docs = discover_project_docs(&env, "anthropic", "/project", "/project/src/lib").await?;

    let root_pos = docs.find("Root").expect("root missing");
    let src_pos = docs.find("Src level").expect("src missing");
    let lib_pos = docs.find("Lib level").expect("lib missing");
    assert!(root_pos < src_pos);
    assert!(src_pos < lib_pos);
    Ok(())
}

// =========================================================================
// System prompt assembly (spec 6.1)
// =========================================================================

#[test]
fn system_prompt_contains_base_instructions() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let prompt = profile.build_system_prompt("", "");
    assert!(prompt.contains("apply_patch"));
    assert!(prompt.contains("coding assistant"));
    Ok(())
}

#[test]
fn system_prompt_layer_ordering() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let env_ctx = "<environment>\nWorking directory: /project\n</environment>";
    let project_docs = "# Project Instructions\nDo things.";

    let prompt = profile.build_system_prompt(env_ctx, project_docs);

    // Layer 1 (base instructions) comes first
    let base_pos = prompt
        .find("coding assistant")
        .expect("base instructions missing");
    // Layer 2 (environment) comes after base
    let env_pos = prompt.find("<environment>").expect("env context missing");
    // Layer 4 (project docs) comes after environment
    let docs_pos = prompt
        .find("Project Instructions")
        .expect("project docs missing");

    assert!(base_pos < env_pos, "base should come before env");
    assert!(env_pos < docs_pos, "env should come before project docs");
    Ok(())
}

#[test]
fn system_prompt_empty_layers_omitted() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2", 600_000)?;
    let prompt = profile.build_system_prompt("", "");

    // Should not have double blank lines from empty layers
    assert!(!prompt.contains("\n\n\n\n"));
    Ok(())
}

#[test]
fn system_prompt_with_all_layers() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    let env_ctx = "<environment>\nPlatform: linux\n</environment>";
    let project_docs = "Follow GEMINI.md conventions.";

    let prompt = profile.build_system_prompt(env_ctx, project_docs);

    assert!(prompt.contains("coding assistant"));
    assert!(prompt.contains("<environment>"));
    assert!(prompt.contains("Follow GEMINI.md conventions."));
    Ok(())
}

// =========================================================================
// Provider-specific required topics (spec 6.2)
// =========================================================================

#[test]
fn openai_base_instructions_topics() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let base = profile.base_instructions();

    assert!(
        base.contains("apply_patch"),
        "OpenAI should mention apply_patch"
    );
    assert!(base.contains("shell"), "OpenAI should mention shell");
    assert!(base.contains("grep"), "OpenAI should mention grep");
    Ok(())
}

#[test]
fn anthropic_base_instructions_topics() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let base = profile.base_instructions();

    assert!(
        base.contains("edit_file"),
        "Anthropic should mention edit_file"
    );
    assert!(
        base.contains("old_string"),
        "Anthropic should mention old_string"
    );
    assert!(
        base.contains("read") && base.contains("before"),
        "Anthropic should mention read-before-edit"
    );
    Ok(())
}

#[test]
fn gemini_base_instructions_topics() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    let base = profile.base_instructions();

    assert!(
        base.contains("read_many_files"),
        "Gemini should mention read_many_files"
    );
    assert!(base.contains("list_dir"), "Gemini should mention list_dir");
    assert!(
        base.contains("GEMINI.md"),
        "Gemini should mention GEMINI.md"
    );
    Ok(())
}
