//! Phase 5 tests: Tool Registry
//!
//! Covers spec section 3.8 — ToolRegistry, RegisteredTool, argument validation.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use serde_json::json;
use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::ExecutionEnvironment;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput, ToolRegistry};
use stencila_models3::types::tool::ToolDefinition;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a minimal valid `ToolDefinition` with the given name.
fn mock_definition(name: &str) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: format!("Mock tool: {name}"),
        parameters: json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        }),
        strict: false,
    }
}

/// Create a `ToolExecutorFn` that always returns the given output string.
fn mock_executor(output: &str) -> ToolExecutorFn {
    let output = Arc::new(output.to_string());
    Box::new(move |_args, _env| {
        let output = Arc::clone(&output);
        Box::pin(async move { Ok(ToolOutput::Text((*output).clone())) })
    })
}

/// Create a `ToolExecutorFn` that always returns an error.
fn error_executor(message: &str) -> ToolExecutorFn {
    let message = Arc::new(message.to_string());
    Box::new(move |_args, _env| {
        let message = Arc::clone(&message);
        Box::pin(async move {
            Err(AgentError::Io {
                message: (*message).clone(),
            })
        })
    })
}

/// Register a mock tool with the given name and output, propagating errors.
fn register_mock(reg: &mut ToolRegistry, name: &str, output: &str) -> AgentResult<()> {
    reg.register(RegisteredTool::new(
        mock_definition(name),
        mock_executor(output),
    ))
}

/// A minimal `ExecutionEnvironment` for testing (no real I/O).
struct MockEnv;

#[async_trait::async_trait]
impl ExecutionEnvironment for MockEnv {
    async fn read_file(
        &self,
        _path: &str,
        _offset: Option<usize>,
        _limit: Option<usize>,
    ) -> AgentResult<stencila_agents::execution::FileContent> {
        Ok(stencila_agents::execution::FileContent::Text(String::new()))
    }

    async fn write_file(&self, _path: &str, _content: &str) -> AgentResult<()> {
        Ok(())
    }

    async fn file_exists(&self, _path: &str) -> bool {
        false
    }

    async fn delete_file(&self, _path: &str) -> AgentResult<()> {
        Ok(())
    }

    async fn list_directory(
        &self,
        _path: &str,
        _depth: usize,
    ) -> AgentResult<Vec<stencila_agents::types::DirEntry>> {
        Ok(vec![])
    }

    async fn exec_command(
        &self,
        _command: &str,
        _timeout_ms: u64,
        _working_dir: Option<&str>,
        _env_vars: Option<&std::collections::HashMap<String, String>>,
    ) -> AgentResult<stencila_agents::types::ExecResult> {
        Ok(stencila_agents::types::ExecResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 0,
        })
    }

    async fn grep(
        &self,
        _pattern: &str,
        _path: &str,
        _options: &stencila_agents::types::GrepOptions,
    ) -> AgentResult<String> {
        Ok(String::new())
    }

    async fn glob_files(&self, _pattern: &str, _path: &str) -> AgentResult<Vec<String>> {
        Ok(vec![])
    }

    fn working_directory(&self) -> &str {
        "/tmp"
    }

    fn platform(&self) -> &str {
        "linux"
    }

    fn os_version(&self) -> String {
        "mock 1.0".to_string()
    }
}

// ===========================================================================
// ToolRegistry — construction and size
// ===========================================================================

#[test]
fn new_registry_is_empty() {
    let reg = ToolRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

// ===========================================================================
// ToolRegistry — register / get
// ===========================================================================

#[test]
fn register_and_get() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    reg.register(RegisteredTool::new(
        mock_definition("read_file"),
        mock_executor("file content"),
    ))?;

    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());

    let got = reg.get("read_file");
    assert!(got.is_some());
    assert_eq!(
        got.map(|t| &t.definition().name),
        Some(&"read_file".to_string())
    );
    Ok(())
}

#[test]
fn get_unknown_returns_none() {
    let reg = ToolRegistry::new();
    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn register_rejects_invalid_definition() {
    let mut reg = ToolRegistry::new();

    // Empty name
    let bad = ToolDefinition {
        name: String::new(),
        description: "has no name".to_string(),
        parameters: json!({"type": "object"}),
        strict: false,
    };
    let result = reg.register(RegisteredTool::new(bad, mock_executor("x")));
    assert!(matches!(result, Err(AgentError::Sdk(_))));
    assert!(reg.is_empty());
}

#[test]
fn register_rejects_non_object_schema() {
    let mut reg = ToolRegistry::new();

    // parameters root is "array" instead of "object"
    let bad = ToolDefinition {
        name: "bad_params".to_string(),
        description: "has array params".to_string(),
        parameters: json!({"type": "array"}),
        strict: false,
    };
    let result = reg.register(RegisteredTool::new(bad, mock_executor("x")));
    assert!(matches!(result, Err(AgentError::Sdk(_))));
    assert!(reg.is_empty());
}

// ===========================================================================
// ToolRegistry — definitions / names ordering
// ===========================================================================

#[test]
fn register_returns_definitions_in_order() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "alpha", "a")?;
    register_mock(&mut reg, "beta", "b")?;
    register_mock(&mut reg, "gamma", "c")?;

    let defs = reg.definitions();
    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    Ok(())
}

#[test]
fn register_returns_names_in_order() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "alpha", "a")?;
    register_mock(&mut reg, "beta", "b")?;
    register_mock(&mut reg, "gamma", "c")?;

    assert_eq!(reg.names(), vec!["alpha", "beta", "gamma"]);
    Ok(())
}

// ===========================================================================
// ToolRegistry — unregister
// ===========================================================================

#[test]
fn unregister_existing() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "tool_a", "a")?;
    assert_eq!(reg.len(), 1);

    let removed = reg.unregister("tool_a");
    assert!(removed);
    assert!(reg.is_empty());
    assert!(reg.get("tool_a").is_none());
    Ok(())
}

#[test]
fn unregister_nonexistent() {
    let mut reg = ToolRegistry::new();
    let removed = reg.unregister("no_such_tool");
    assert!(!removed);
}

// ===========================================================================
// ToolRegistry — override behavior
// ===========================================================================

#[test]
fn register_override_latest_wins() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();

    let mut def_v1 = mock_definition("my_tool");
    def_v1.description = "version 1".to_string();
    reg.register(RegisteredTool::new(def_v1, mock_executor("v1")))?;

    let mut def_v2 = mock_definition("my_tool");
    def_v2.description = "version 2".to_string();
    reg.register(RegisteredTool::new(def_v2, mock_executor("v2")))?;

    assert_eq!(reg.len(), 1);
    let tool = reg.get("my_tool");
    assert_eq!(
        tool.map(|t| t.definition().description.as_str()),
        Some("version 2")
    );
    Ok(())
}

#[test]
fn register_override_preserves_position() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "first", "1")?;
    register_mock(&mut reg, "second", "2")?;
    register_mock(&mut reg, "third", "3")?;

    // Override "second" — should keep its position (index 1)
    let mut new_second = mock_definition("second");
    new_second.description = "updated".to_string();
    reg.register(RegisteredTool::new(new_second, mock_executor("2b")))?;

    assert_eq!(reg.names(), vec!["first", "second", "third"]);
    Ok(())
}

// ===========================================================================
// RegisteredTool — execute
// ===========================================================================

#[tokio::test]
async fn execute_tool_success() -> AgentResult<()> {
    let tool = RegisteredTool::new(mock_definition("echo"), mock_executor("hello world"));
    let env = MockEnv;
    let result = tool.execute(json!({"input": "test"}), &env).await?;
    assert_eq!(result.as_text(), "hello world");
    Ok(())
}

#[tokio::test]
async fn execute_tool_error_propagates() {
    let tool = RegisteredTool::new(mock_definition("failing"), error_executor("disk full"));
    let env = MockEnv;
    let result = tool.execute(json!({}), &env).await;
    assert!(matches!(
        result,
        Err(AgentError::Io { ref message }) if message == "disk full"
    ));
}

// ===========================================================================
// ToolRegistry — validate_arguments
// ===========================================================================

#[test]
fn validate_arguments_valid() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "tool_a", "ok")?;

    reg.validate_arguments("tool_a", &json!({"input": "hello"}))?;
    Ok(())
}

#[test]
fn validate_arguments_invalid() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "tool_a", "ok")?;

    // Missing required "input" field
    let result = reg.validate_arguments("tool_a", &json!({}));
    assert!(matches!(
        result,
        Err(AgentError::ValidationError { ref reason }) if reason.contains("input")
    ));
    Ok(())
}

#[test]
fn validate_arguments_unknown_tool() {
    let reg = ToolRegistry::new();
    let result = reg.validate_arguments("no_such_tool", &json!({}));
    assert!(matches!(
        result,
        Err(AgentError::UnknownTool { ref name }) if name == "no_such_tool"
    ));
}

#[test]
fn validate_arguments_uncompilable_schema_skips() -> AgentResult<()> {
    // Schema is object-rooted (passes ToolDefinition::validate()) but
    // contains a $ref to a missing $defs entry, which causes
    // jsonschema::validator_for() to fail at compile time.
    let def = ToolDefinition {
        name: "bad_schema".to_string(),
        description: "Tool with uncompilable schema".to_string(),
        parameters: json!({
            "type": "object",
            "$ref": "#/$defs/missing"
        }),
        strict: false,
    };
    let mut reg = ToolRegistry::new();
    reg.register(RegisteredTool::new(def, mock_executor("ok")))?;

    // Should gracefully skip (return Ok) rather than error, because
    // the schema can't be compiled by jsonschema.
    reg.validate_arguments("bad_schema", &json!({"anything": true}))?;
    Ok(())
}

// ===========================================================================
// ToolRegistry — integrated lookup → validate → execute
// ===========================================================================

#[tokio::test]
async fn lookup_validate_execute_integrated() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "greet", "hello!")?;

    let args = json!({"input": "world"});

    // 1. Lookup
    let tool = reg.get("greet").ok_or(AgentError::UnknownTool {
        name: "greet".to_string(),
    })?;

    // 2. Validate
    reg.validate_arguments("greet", &args)?;

    // 3. Execute
    let env = MockEnv;
    let output = tool.execute(args, &env).await?;
    assert_eq!(output.as_text(), "hello!");
    Ok(())
}

#[tokio::test]
async fn lookup_validate_execute_rejects_invalid_args() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "greet", "hello!")?;

    // Valid lookup, but invalid args — should fail at validation step
    let args = json!({"wrong_field": 42});
    let result = reg.validate_arguments("greet", &args);
    assert!(matches!(result, Err(AgentError::ValidationError { .. })));
    Ok(())
}

// ===========================================================================
// ToolRegistry — misc
// ===========================================================================

#[test]
fn definitions_clones_not_references() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "tool_a", "a")?;

    let defs1 = reg.definitions();
    let defs2 = reg.definitions();

    // Both should be equal but independent allocations
    assert_eq!(defs1.len(), defs2.len());
    assert_eq!(defs1[0].name, defs2[0].name);
    Ok(())
}

#[test]
fn multiple_tools_independent() -> AgentResult<()> {
    let mut reg = ToolRegistry::new();
    register_mock(&mut reg, "tool_a", "a")?;
    register_mock(&mut reg, "tool_b", "b")?;
    register_mock(&mut reg, "tool_c", "c")?;

    assert_eq!(reg.len(), 3);
    assert!(reg.get("tool_a").is_some());
    assert!(reg.get("tool_b").is_some());
    assert!(reg.get("tool_c").is_some());

    // Validate each independently
    reg.validate_arguments("tool_a", &json!({"input": "x"}))?;
    reg.validate_arguments("tool_b", &json!({"input": "y"}))?;
    reg.validate_arguments("tool_c", &json!({"input": "z"}))?;

    Ok(())
}

#[test]
fn empty_registry_definitions_and_names() {
    let reg = ToolRegistry::new();
    assert!(reg.definitions().is_empty());
    assert!(reg.names().is_empty());
}
