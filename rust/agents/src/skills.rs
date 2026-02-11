//! Skill discovery and tool integration.
//!
//! Provides progressive disclosure of workspace skills: compact metadata is
//! included in the system prompt at startup, and full skill content is loaded
//! on demand via the `use_skill` tool.

use std::path::Path;

use serde_json::json;
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};

/// Tool name for loading full skill content.
pub const TOOL_USE_SKILL: &str = "use_skill";

/// Discover workspace skills and register the `use_skill` tool if any are found.
///
/// Uses multi-source discovery: skills from `.stencila/skills/` are loaded first,
/// then provider-specific directories (e.g. `.claude/skills/` for Anthropic).
/// On name conflicts, the provider-specific source wins (last wins).
///
/// Returns a formatted metadata section for the system prompt, or an empty
/// string if no skills are found.
///
/// When skills are found, the `use_skill` tool is registered in the profile's
/// tool registry so the model can load full skill content on demand.
pub async fn discover_and_register_skills(
    profile: &mut dyn ProviderProfile,
    working_dir: &str,
) -> String {
    let working_path = Path::new(working_dir);
    let sources = stencila_skills::SkillSource::for_provider(profile.id());

    let skills = stencila_skills::discover(working_path, &sources).await;
    if skills.is_empty() {
        return String::new();
    }

    // Register the use_skill tool with the same sources used for discovery
    if let Err(e) = profile
        .tool_registry_mut()
        .register(RegisteredTool::new(definition(), executor(sources)))
    {
        tracing::warn!("failed to register use_skill tool: {e}");
    }

    let metadata_xml = stencila_skills::metadata_to_xml(&skills);
    format!(
        "# Workspace Skills\n\n\
         The following skills are available in this workspace. \
         Use the `use_skill` tool with a skill's name to load its full instructions.\n\n\
         {metadata_xml}"
    )
}

/// Tool definition for `use_skill`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_USE_SKILL.into(),
        description: "Load the full instructions for a workspace skill by name. \
                       Use this when you want to follow a skill's workflow."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the skill to load."
                }
            },
            "required": ["name"]
        }),
        strict: false,
    }
}

/// Async executor for the `use_skill` tool.
///
/// The `sources` parameter should match the sources used during discovery,
/// ensuring the executor can find exactly the skills the model saw in the
/// system prompt metadata.
pub fn executor(sources: Vec<stencila_skills::SkillSource>) -> ToolExecutorFn {
    Box::new(move |args, env| {
        let sources = sources.clone();
        Box::pin(async move {
            let name = crate::tools::required_str(&args, "name")?;
            let working_path = Path::new(env.working_directory());

            let skill = stencila_skills::get_from(working_path, name, &sources)
                .await
                .map_err(|_| AgentError::ValidationError {
                    reason: format!("skill not found: {name}"),
                })?;

            Ok(ToolOutput::Text(stencila_skills::to_xml(&skill)))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::LocalExecutionEnvironment;
    use crate::profiles::OpenAiProfile;

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, TOOL_USE_SKILL);
        assert!(!def.description.is_empty());

        // Verify "name" is a required parameter
        let required = def.parameters.get("required").and_then(|v| v.as_array());
        assert!(required.is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some("name"))));
    }

    /// Create a temp directory with a `<dot_dir>/skills/<name>/SKILL.md` file.
    fn setup_skill_dir_in(tmp: &tempfile::TempDir, dot_dir: &str, name: &str, description: &str) {
        let skill_dir = tmp.path().join(dot_dir).join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");
        let content =
            format!("---\nname: {name}\ndescription: {description}\n---\n\nSome instructions.\n");
        std::fs::write(skill_dir.join("SKILL.md"), content).expect("write SKILL.md");
    }

    /// Create a temp directory with a `.stencila/skills/<name>/SKILL.md` file.
    fn setup_skill_dir(name: &str, description: &str) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".stencila", name, description);
        tmp
    }

    // -- discover_and_register_skills tests --

    #[tokio::test]
    async fn discover_empty_when_no_skills_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).expect("profile");

        let result =
            discover_and_register_skills(&mut profile, tmp.path().to_str().expect("path")).await;

        assert!(
            result.is_empty(),
            "should return empty string with no skills dir"
        );
        assert!(
            profile.tool_registry().get(TOOL_USE_SKILL).is_none(),
            "use_skill should not be registered when no skills found"
        );
    }

    #[tokio::test]
    async fn discover_registers_tool_when_skills_exist() {
        let tmp = setup_skill_dir("test-skill", "A test skill for testing.");
        let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).expect("profile");

        let result =
            discover_and_register_skills(&mut profile, tmp.path().to_str().expect("path")).await;

        // Should contain skills metadata
        assert!(
            result.contains("<skills>"),
            "result should contain <skills> XML"
        );
        assert!(
            result.contains("test-skill"),
            "result should contain skill name"
        );
        assert!(
            result.contains("Workspace Skills"),
            "result should contain header text"
        );

        // Tool should be registered
        assert!(
            profile.tool_registry().get(TOOL_USE_SKILL).is_some(),
            "use_skill tool should be registered"
        );
    }

    // -- executor tests --

    #[tokio::test]
    async fn executor_returns_xml_for_valid_skill() {
        let tmp = setup_skill_dir("my-skill", "Does something useful.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all());

        let args = json!({"name": "my-skill"});
        let result = exec(args, &env).await;

        assert!(result.is_ok(), "executor should succeed: {result:?}");
        let output = result.expect("ok");
        let text = output.as_text();
        assert!(
            text.contains("<skill name=\"my-skill\">"),
            "output should contain skill XML: {text}"
        );
        assert!(
            text.contains("Some instructions."),
            "output should contain skill instructions"
        );
    }

    #[tokio::test]
    async fn executor_finds_skill_in_claude_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".claude", "claude-skill", "A Claude-specific skill.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::for_provider("anthropic"));

        let args = json!({"name": "claude-skill"});
        let result = exec(args, &env).await;

        assert!(
            result.is_ok(),
            "executor should find skill in .claude: {result:?}"
        );
        let output = result.expect("ok");
        let text = output.as_text();
        assert!(
            text.contains("<skill name=\"claude-skill\">"),
            "output should contain skill XML: {text}"
        );
    }

    #[tokio::test]
    async fn executor_skill_not_found_is_validation_error() {
        let tmp = setup_skill_dir("existing-skill", "Exists in the workspace.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all());

        let args = json!({"name": "nonexistent"});
        let result = exec(args, &env).await;

        assert!(
            result.is_err(),
            "executor should fail for nonexistent skill"
        );
        let err = result.expect_err("err");
        assert!(
            matches!(err, AgentError::ValidationError { .. }),
            "error should be ValidationError, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn executor_no_local_skills_dir_returns_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all());

        let args = json!({"name": "nonexistent-skill-xyz"});
        let result = exec(args, &env).await;

        assert!(
            result.is_err(),
            "executor should fail for nonexistent skill"
        );
        let err = result.expect_err("err");
        assert!(
            matches!(err, AgentError::ValidationError { .. }),
            "error should be ValidationError, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn executor_missing_name_param_is_validation_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all());

        let args = json!({});
        let result = exec(args, &env).await;

        assert!(result.is_err());
        let err = result.expect_err("err");
        assert!(
            matches!(err, AgentError::ValidationError { .. }),
            "missing name should be ValidationError, got: {err:?}"
        );
    }
}
