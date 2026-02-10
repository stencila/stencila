//! Tests for provider profiles (spec 3.1-3.7).
//!
//! Phase 7a: profile construction, tool set composition, capability flags,
//! shell timeout defaults, custom tool registration/override.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};
use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::profile::ProviderProfile;
use stencila_agents::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn};
use stencila_agents::types::{DirEntry, ExecResult, GrepOptions};
use stencila_models3::types::tool::ToolDefinition;

// ---------------------------------------------------------------------------
// Helper: create a no-op executor for custom tool tests
// ---------------------------------------------------------------------------

fn noop_executor() -> ToolExecutorFn {
    Box::new(
        |_args: Value,
         _env: &dyn ExecutionEnvironment|
         -> Pin<Box<dyn Future<Output = AgentResult<String>> + Send + '_>> {
            Box::pin(async { Ok("ok".into()) })
        },
    )
}

fn custom_tool(name: &str) -> RegisteredTool {
    RegisteredTool::new(
        ToolDefinition {
            name: name.into(),
            description: format!("Custom tool: {name}"),
            parameters: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            }),
            strict: false,
        },
        noop_executor(),
    )
}

// =========================================================================
// OpenAI Profile (spec 3.4)
// =========================================================================

#[test]
fn openai_profile_id() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    assert_eq!(profile.id(), "openai");
    Ok(())
}

#[test]
fn openai_profile_model() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    assert_eq!(profile.model(), "gpt-5.2-codex");
    Ok(())
}

#[test]
fn openai_profile_tool_count() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    // Spec 3.4: read_file, apply_patch, write_file, shell, grep, glob
    assert_eq!(profile.tool_registry().len(), 6);
    Ok(())
}

#[test]
fn openai_profile_tool_names() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let names = profile.tool_registry().names();
    assert_eq!(
        names,
        vec![
            "read_file",
            "apply_patch",
            "write_file",
            "shell",
            "grep",
            "glob"
        ]
    );
    Ok(())
}

#[test]
fn openai_profile_has_apply_patch_not_edit_file() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    assert!(profile.tool_registry().get("apply_patch").is_some());
    assert!(profile.tool_registry().get("edit_file").is_none());
    Ok(())
}

#[test]
fn openai_profile_tools_returns_definitions() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let tools = profile.tools();
    assert_eq!(tools.len(), 6);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        tool_names,
        vec![
            "read_file",
            "apply_patch",
            "write_file",
            "shell",
            "grep",
            "glob"
        ]
    );
    Ok(())
}

// =========================================================================
// Anthropic Profile (spec 3.5)
// =========================================================================

#[test]
fn anthropic_profile_id() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    assert_eq!(profile.id(), "anthropic");
    Ok(())
}

#[test]
fn anthropic_profile_model() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    assert_eq!(profile.model(), "claude-opus-4-6");
    Ok(())
}

#[test]
fn anthropic_profile_tool_count() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    // Spec 3.5: read_file, write_file, edit_file, shell, grep, glob
    assert_eq!(profile.tool_registry().len(), 6);
    Ok(())
}

#[test]
fn anthropic_profile_tool_names() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let names = profile.tool_registry().names();
    assert_eq!(
        names,
        vec![
            "read_file",
            "write_file",
            "edit_file",
            "shell",
            "grep",
            "glob"
        ]
    );
    Ok(())
}

#[test]
fn anthropic_profile_has_edit_file_not_apply_patch() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    assert!(profile.tool_registry().get("edit_file").is_some());
    assert!(profile.tool_registry().get("apply_patch").is_none());
    Ok(())
}

// =========================================================================
// Gemini Profile (spec 3.6)
// =========================================================================

#[test]
fn gemini_profile_id() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    assert_eq!(profile.id(), "gemini");
    Ok(())
}

#[test]
fn gemini_profile_model() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    assert_eq!(profile.model(), "gemini-3-flash");
    Ok(())
}

#[test]
fn gemini_profile_tool_count() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    // Spec 3.6: read_file, read_many_files, write_file, edit_file, shell, grep, glob, list_dir
    assert_eq!(profile.tool_registry().len(), 8);
    Ok(())
}

#[test]
fn gemini_profile_tool_names() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    let names = profile.tool_registry().names();
    assert_eq!(
        names,
        vec![
            "read_file",
            "read_many_files",
            "write_file",
            "edit_file",
            "shell",
            "grep",
            "glob",
            "list_dir"
        ]
    );
    Ok(())
}

#[test]
fn gemini_profile_has_gemini_specific_tools() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    assert!(profile.tool_registry().get("read_many_files").is_some());
    assert!(profile.tool_registry().get("list_dir").is_some());
    // Gemini uses edit_file, not apply_patch
    assert!(profile.tool_registry().get("edit_file").is_some());
    assert!(profile.tool_registry().get("apply_patch").is_none());
    Ok(())
}

// =========================================================================
// No subagent tools yet (Phase 9)
// =========================================================================

#[test]
fn profiles_have_no_subagent_tools() -> AgentResult<()> {
    let subagent_tools = ["spawn_agent", "send_input", "wait", "close_agent"];

    let openai = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let anthropic = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let gemini = GeminiProfile::new("gemini-3-flash", 600_000)?;

    for name in &subagent_tools {
        assert!(
            openai.tool_registry().get(name).is_none(),
            "OpenAI should not have {name}"
        );
        assert!(
            anthropic.tool_registry().get(name).is_none(),
            "Anthropic should not have {name}"
        );
        assert!(
            gemini.tool_registry().get(name).is_none(),
            "Gemini should not have {name}"
        );
    }
    Ok(())
}

// =========================================================================
// Capability flags (spec 3.2)
// =========================================================================

#[test]
fn openai_capability_flags() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    assert!(profile.supports_reasoning());
    assert!(profile.supports_streaming());
    assert!(profile.supports_parallel_tool_calls());
    assert_eq!(profile.context_window_size(), 200_000);
    Ok(())
}

#[test]
fn anthropic_capability_flags() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    assert!(profile.supports_reasoning());
    assert!(profile.supports_streaming());
    assert!(profile.supports_parallel_tool_calls());
    assert_eq!(profile.context_window_size(), 200_000);
    Ok(())
}

#[test]
fn gemini_capability_flags() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    assert!(profile.supports_reasoning());
    assert!(profile.supports_streaming());
    assert!(profile.supports_parallel_tool_calls());
    assert_eq!(profile.context_window_size(), 1_000_000);
    Ok(())
}

// =========================================================================
// Provider options (spec 3.2)
// =========================================================================

#[test]
fn provider_options_are_some_empty() -> AgentResult<()> {
    let openai = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let anthropic = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let gemini = GeminiProfile::new("gemini-3-flash", 600_000)?;

    // All profiles return Some(empty map) — proves plumbing works
    // without committing to specific provider-version-dependent values.
    assert!(openai.provider_options().is_some_and(|m| m.is_empty()));
    assert!(anthropic.provider_options().is_some_and(|m| m.is_empty()));
    assert!(gemini.provider_options().is_some_and(|m| m.is_empty()));
    Ok(())
}

// =========================================================================
// System prompt (spec 3.2 + 6.1)
// =========================================================================

#[test]
fn build_system_prompt_contains_base_instructions() -> AgentResult<()> {
    let openai = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let anthropic = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let gemini = GeminiProfile::new("gemini-3-flash", 600_000)?;

    assert!(
        openai
            .build_system_prompt("", "")
            .contains("coding assistant")
    );
    assert!(
        anthropic
            .build_system_prompt("", "")
            .contains("coding assistant")
    );
    assert!(
        gemini
            .build_system_prompt("", "")
            .contains("coding assistant")
    );
    Ok(())
}

// =========================================================================
// Custom tool registration / override (spec 3.7)
// =========================================================================

#[test]
fn custom_tool_registration() -> AgentResult<()> {
    let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    assert_eq!(profile.tool_registry().len(), 6);

    profile
        .tool_registry_mut()
        .register(custom_tool("run_tests"))?;
    assert_eq!(profile.tool_registry().len(), 7);
    assert!(profile.tool_registry().get("run_tests").is_some());
    Ok(())
}

#[test]
fn custom_tool_override_replaces_existing() -> AgentResult<()> {
    let mut profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;

    // Override edit_file with a custom version.
    let original_desc = profile
        .tool_registry()
        .get("edit_file")
        .map(|t| t.definition().description.clone());

    profile
        .tool_registry_mut()
        .register(custom_tool("edit_file"))?;

    let new_desc = profile
        .tool_registry()
        .get("edit_file")
        .map(|t| t.definition().description.clone());

    // Description changed (latest-wins override).
    assert_ne!(original_desc, new_desc);
    assert_eq!(new_desc.as_deref(), Some("Custom tool: edit_file"));

    // Tool count unchanged (replacement, not addition).
    assert_eq!(profile.tool_registry().len(), 6);
    Ok(())
}

#[test]
fn custom_tool_override_preserves_position() -> AgentResult<()> {
    let mut profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let original_names: Vec<String> = profile
        .tool_registry()
        .names()
        .into_iter()
        .map(String::from)
        .collect();

    // Override edit_file (3rd tool in Anthropic profile).
    profile
        .tool_registry_mut()
        .register(custom_tool("edit_file"))?;

    let new_names: Vec<String> = profile
        .tool_registry()
        .names()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        original_names, new_names,
        "Override must preserve insertion position"
    );
    Ok(())
}

// =========================================================================
// Shell timeout defaults (spec 3.4: 10s, spec 3.5: 120s, spec 3.6: 10s)
// =========================================================================

// ---------------------------------------------------------------------------
// Minimal mock for behavioral shell timeout verification
// ---------------------------------------------------------------------------

/// Captured parameters from an `exec_command` call.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExecCall {
    command: String,
    timeout_ms: u64,
}

/// Minimal mock that captures `exec_command` calls so we can verify
/// the default timeout wired by each profile's shell executor.
#[derive(Clone)]
struct ShellMockEnv {
    exec_calls: Arc<Mutex<Vec<ExecCall>>>,
}

impl ShellMockEnv {
    fn new() -> Self {
        Self {
            exec_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn recorded_exec_calls(&self) -> Vec<ExecCall> {
        self.exec_calls.lock().expect("lock poisoned").clone()
    }
}

impl std::fmt::Debug for ShellMockEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellMockEnv").finish()
    }
}

#[async_trait]
impl ExecutionEnvironment for ShellMockEnv {
    async fn read_file(
        &self,
        path: &str,
        _offset: Option<usize>,
        _limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        Err(AgentError::FileNotFound { path: path.into() })
    }

    async fn write_file(&self, _path: &str, _content: &str) -> AgentResult<()> {
        Ok(())
    }

    async fn file_exists(&self, _path: &str) -> bool {
        false
    }

    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        Err(AgentError::FileNotFound { path: path.into() })
    }

    async fn list_directory(&self, path: &str, _depth: usize) -> AgentResult<Vec<DirEntry>> {
        Err(AgentError::FileNotFound { path: path.into() })
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
        Ok(ExecResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 10,
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
        "/mock"
    }

    fn platform(&self) -> &str {
        "mock"
    }

    fn os_version(&self) -> String {
        "mock-os 1.0".into()
    }
}

// ---------------------------------------------------------------------------
// Behavioral timeout tests
// ---------------------------------------------------------------------------

/// Execute the shell tool from a profile's registry with no `timeout_ms`
/// argument and return the default timeout captured by the mock.
async fn execute_shell_default_timeout(profile: &dyn ProviderProfile) -> AgentResult<u64> {
    let env = ShellMockEnv::new();
    let shell = profile
        .tool_registry()
        .get("shell")
        .ok_or_else(|| AgentError::UnknownTool {
            name: "shell".into(),
        })?;
    // Call with no timeout_ms — forces use of the profile's default.
    let args = json!({ "command": "echo hello" });
    let _output = shell.execute(args, &env).await?;
    let calls = env.recorded_exec_calls();
    let call = calls.first().ok_or_else(|| AgentError::Io {
        message: "no exec_command call recorded".into(),
    })?;
    Ok(call.timeout_ms)
}

#[tokio::test]
async fn openai_shell_default_timeout_is_10s() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let timeout = execute_shell_default_timeout(&profile).await?;
    assert_eq!(timeout, 10_000, "OpenAI shell default should be 10s");
    Ok(())
}

#[tokio::test]
async fn anthropic_shell_default_timeout_is_120s() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let timeout = execute_shell_default_timeout(&profile).await?;
    assert_eq!(timeout, 120_000, "Anthropic shell default should be 120s");
    Ok(())
}

#[tokio::test]
async fn gemini_shell_default_timeout_is_10s() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    let timeout = execute_shell_default_timeout(&profile).await?;
    assert_eq!(timeout, 10_000, "Gemini shell default should be 10s");
    Ok(())
}

#[test]
fn all_profiles_have_shell_tool() -> AgentResult<()> {
    let openai = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let anthropic = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let gemini = GeminiProfile::new("gemini-3-flash", 600_000)?;

    assert!(openai.tool_registry().get("shell").is_some());
    assert!(anthropic.tool_registry().get("shell").is_some());
    assert!(gemini.tool_registry().get("shell").is_some());
    Ok(())
}

// =========================================================================
// Vendored schema parity for assembled profiles (spec 3.4-3.6)
// =========================================================================

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

/// Assert that every tool in a profile matches its vendored fixture.
fn assert_profile_schema_parity(profile: &dyn ProviderProfile) -> Result<(), String> {
    for def in profile.tools() {
        let fixture = load_fixture(&def.name)?;
        if def.name != fixture.name {
            return Err(format!(
                "profile {} tool name mismatch: got {:?}, fixture {:?}",
                profile.id(),
                def.name,
                fixture.name
            ));
        }
        if def.description != fixture.description {
            return Err(format!(
                "profile {} tool {:?} description mismatch",
                profile.id(),
                def.name,
            ));
        }
        if def.parameters != fixture.parameters {
            return Err(format!(
                "profile {} tool {:?} parameters mismatch",
                profile.id(),
                def.name,
            ));
        }
        if def.strict != fixture.strict {
            return Err(format!(
                "profile {} tool {:?} strict mismatch: got {:?}, fixture {:?}",
                profile.id(),
                def.name,
                def.strict,
                fixture.strict,
            ));
        }
    }
    Ok(())
}

#[test]
fn openai_profile_schema_parity() -> Result<(), String> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).map_err(|e| e.to_string())?;
    assert_profile_schema_parity(&profile)
}

#[test]
fn anthropic_profile_schema_parity() -> Result<(), String> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000).map_err(|e| e.to_string())?;
    assert_profile_schema_parity(&profile)
}

#[test]
fn gemini_profile_schema_parity() -> Result<(), String> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000).map_err(|e| e.to_string())?;
    assert_profile_schema_parity(&profile)
}

// =========================================================================
// Trait object usage (profiles should be usable as dyn ProviderProfile)
// =========================================================================

#[test]
fn profiles_usable_as_trait_objects() -> AgentResult<()> {
    let profiles: Vec<Box<dyn ProviderProfile>> = vec![
        Box::new(OpenAiProfile::new("gpt-5.2-codex", 600_000)?),
        Box::new(AnthropicProfile::new("claude-opus-4-6", 600_000)?),
        Box::new(GeminiProfile::new("gemini-3-flash", 600_000)?),
    ];

    let ids: Vec<&str> = profiles.iter().map(|p| p.id()).collect();
    assert_eq!(ids, vec!["openai", "anthropic", "gemini"]);
    Ok(())
}

#[test]
fn profile_debug_output() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let debug = format!("{profile:?}");
    assert!(debug.contains("OpenAiProfile"));
    assert!(debug.contains("gpt-5.2-codex"));
    Ok(())
}

// =========================================================================
// Tool definitions from tools() match registry
// =========================================================================

#[test]
fn tools_method_matches_registry_definitions() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-3-flash", 600_000)?;
    let from_method = profile.tools();
    let from_registry = profile.tool_registry().definitions();
    assert_eq!(from_method.len(), from_registry.len());
    for (a, b) in from_method.iter().zip(from_registry.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.description, b.description);
    }
    Ok(())
}
