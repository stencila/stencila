//! Skill discovery and tool integration.
//!
//! Provides progressive disclosure of workspace skills: compact metadata is
//! included in the system prompt at startup, and full skill content is loaded
//! on demand via the `use_skill` tool. As an extension, when an agent allows
//! exactly one skill, that skill is also preloaded into the system prompt.

use std::path::Path;

use serde_json::json;
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};

/// Tool name for loading full skill content.
pub const TOOL_USE_SKILL: &str = "use_skill";

/// Result of skill discovery and prompt preparation.
#[derive(Debug, Default)]
pub struct SkillPromptContext {
    /// Metadata describing the discovered skills available to the agent.
    pub metadata: String,

    /// Full XML for a single allowed skill preloaded into the system prompt.
    pub preloaded_skill: Option<String>,
}

/// Discover workspace skills and register the `use_skill` tool if any remain
/// after filtering.
///
/// Uses multi-source discovery: skills from `.stencila/skills/` are loaded first,
/// then provider-specific directories (e.g. `.claude/skills/` for Anthropic).
/// On name conflicts, the provider-specific source wins (last wins).
///
/// Returns a [`SkillPromptContext`] containing the metadata section for the
/// system prompt and, when exactly one skill is allowed, the preloaded skill
/// content. Returns a default (empty) context if no skills survive filtering.
///
/// When filtered skills are found, the `use_skill` tool is registered in the
/// profile's tool registry so the model can load full skill content on demand.
/// The executor enforces the `allowed_skills` list at call time.
pub async fn discover_and_register_skills(
    profile: &mut dyn ProviderProfile,
    working_dir: &str,
    allowed_skills: Option<&[String]>,
) -> SkillPromptContext {
    let working_path = Path::new(working_dir);
    let sources = stencila_skills::SkillSource::for_provider(profile.id());

    let skills = stencila_skills::discover(working_path, &sources).await;
    if skills.is_empty() {
        return SkillPromptContext::default();
    }

    let filtered_skills = filter_allowed_skills(skills, allowed_skills);
    if filtered_skills.is_empty() {
        return SkillPromptContext::default();
    }

    // Register the use_skill tool only when filtered skills remain.
    // The executor enforces the allowed_skills list at call time.
    let allowed_names = allowed_skills.map(|s| s.to_vec());
    if let Err(e) = profile.tool_registry_mut().register(RegisteredTool::new(
        definition(),
        executor(sources, allowed_names),
    )) {
        tracing::warn!("failed to register use_skill tool: {e}");
    }

    let metadata_xml = stencila_skills::metadata_to_xml(&filtered_skills);
    let metadata = format!(
        "# Workspace Skills\n\n\
         The following skills are available in this workspace. \
         Use the `use_skill` tool with a skill's name to load its full instructions.\n\n\
         {metadata_xml}"
    );

    let preloaded_skill = match allowed_skills {
        Some([name]) => filtered_skills
            .iter()
            .find(|skill| skill.name == *name)
            .map(|skill| {
                format!(
                    "# Preloaded Skill\n\n\
                     This agent has exactly one allowed skill, so its full content is \
                     preloaded below in addition to being available via `use_skill`.\n\n\
                     {}",
                    stencila_skills::to_xml(skill)
                )
            }),
        _ => None,
    };

    SkillPromptContext {
        metadata,
        preloaded_skill,
    }
}

fn filter_allowed_skills(
    skills: Vec<stencila_skills::SkillInstance>,
    allowed_skills: Option<&[String]>,
) -> Vec<stencila_skills::SkillInstance> {
    match allowed_skills {
        Some(names) => skills
            .into_iter()
            .filter(|skill| names.iter().any(|name| name == &skill.name))
            .collect(),
        None => skills,
    }
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
/// system prompt metadata. When `allowed_skills` is `Some`, the executor
/// rejects requests for skill names not in the list.
pub fn executor(
    sources: Vec<stencila_skills::SkillSource>,
    allowed_skills: Option<Vec<String>>,
) -> ToolExecutorFn {
    Box::new(move |args, env| {
        let sources = sources.clone();
        let allowed = allowed_skills.clone();
        Box::pin(async move {
            let name = crate::tools::required_str(&args, "name")?;

            if let Some(ref names) = allowed
                && !names.iter().any(|n| n == name)
            {
                return Err(AgentError::ValidationError {
                    reason: format!("skill not allowed: {name}"),
                });
            }

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
            discover_and_register_skills(&mut profile, tmp.path().to_str().expect("path"), None)
                .await;

        assert!(
            result.metadata.is_empty(),
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
            discover_and_register_skills(&mut profile, tmp.path().to_str().expect("path"), None)
                .await;

        // Should contain skills metadata
        assert!(
            result.metadata.contains("<skills>"),
            "result should contain <skills> XML"
        );
        assert!(
            result.metadata.contains("test-skill"),
            "result should contain skill name"
        );
        assert!(
            result.metadata.contains("Workspace Skills"),
            "result should contain header text"
        );

        // Tool should be registered
        assert!(
            profile.tool_registry().get(TOOL_USE_SKILL).is_some(),
            "use_skill tool should be registered"
        );
    }

    #[tokio::test]
    async fn discover_filters_to_allowed_skills_and_preloads_single_skill() {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".stencila", "alpha", "First skill.");
        setup_skill_dir_in(&tmp, ".stencila", "beta", "Second skill.");
        let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).expect("profile");

        let result = discover_and_register_skills(
            &mut profile,
            tmp.path().to_str().expect("path"),
            Some(&["beta".to_string()]),
        )
        .await;

        assert!(result.metadata.contains("beta"));
        assert!(!result.metadata.contains("alpha"));

        let preloaded = result.preloaded_skill.expect("preloaded skill");
        assert!(preloaded.contains("Preloaded Skill"));
        assert!(preloaded.contains("<skill name=\"beta\">"));
    }

    #[tokio::test]
    async fn discover_no_allowed_skills_matched() {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".stencila", "alpha", "First skill.");
        setup_skill_dir_in(&tmp, ".stencila", "beta", "Second skill.");
        let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).expect("profile");

        let result = discover_and_register_skills(
            &mut profile,
            tmp.path().to_str().expect("path"),
            Some(&["nonexistent".to_string()]),
        )
        .await;

        assert!(
            result.metadata.is_empty(),
            "metadata should be empty when no allowed skills match"
        );
        assert!(
            result.preloaded_skill.is_none(),
            "no skill should be preloaded"
        );
        assert!(
            profile.tool_registry().get(TOOL_USE_SKILL).is_none(),
            "use_skill should not be registered when no skills survive filtering"
        );
    }

    #[tokio::test]
    async fn discover_multiple_allowed_skills_no_preload() {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".stencila", "alpha", "First skill.");
        setup_skill_dir_in(&tmp, ".stencila", "beta", "Second skill.");
        setup_skill_dir_in(&tmp, ".stencila", "gamma", "Third skill.");
        let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000).expect("profile");

        let result = discover_and_register_skills(
            &mut profile,
            tmp.path().to_str().expect("path"),
            Some(&["alpha".to_string(), "gamma".to_string()]),
        )
        .await;

        assert!(result.metadata.contains("alpha"));
        assert!(!result.metadata.contains("beta"));
        assert!(result.metadata.contains("gamma"));
        assert!(
            result.preloaded_skill.is_none(),
            "no skill should be preloaded when multiple are allowed"
        );
    }

    // -- executor tests --

    #[tokio::test]
    async fn executor_returns_xml_for_valid_skill() {
        let tmp = setup_skill_dir("my-skill", "Does something useful.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all(), None);

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
        let exec = executor(
            stencila_skills::SkillSource::for_provider("anthropic"),
            None,
        );

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
    async fn executor_rejects_disallowed_skill() {
        let tmp = tempfile::tempdir().expect("tempdir");
        setup_skill_dir_in(&tmp, ".stencila", "allowed-skill", "Allowed.");
        setup_skill_dir_in(&tmp, ".stencila", "secret-skill", "Secret.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(
            stencila_skills::SkillSource::all(),
            Some(vec!["allowed-skill".to_string()]),
        );

        let args = json!({"name": "secret-skill"});
        let result = exec(args, &env).await;

        assert!(result.is_err(), "executor should reject disallowed skill");
        let err = result.expect_err("err");
        assert!(
            matches!(err, AgentError::ValidationError { .. }),
            "error should be ValidationError, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn executor_allows_permitted_skill() {
        let tmp = setup_skill_dir("my-skill", "Permitted skill.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(
            stencila_skills::SkillSource::all(),
            Some(vec!["my-skill".to_string()]),
        );

        let args = json!({"name": "my-skill"});
        let result = exec(args, &env).await;

        assert!(
            result.is_ok(),
            "executor should allow permitted skill: {result:?}"
        );
        let output = result.expect("ok");
        let text = output.as_text();
        assert!(
            text.contains("<skill name=\"my-skill\">"),
            "output should contain skill XML: {text}"
        );
    }

    #[tokio::test]
    async fn executor_skill_not_found_is_validation_error() {
        let tmp = setup_skill_dir("existing-skill", "Exists in the workspace.");
        let env = LocalExecutionEnvironment::new(tmp.path());
        let exec = executor(stencila_skills::SkillSource::all(), None);

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
        let exec = executor(stencila_skills::SkillSource::all(), None);

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
        let exec = executor(stencila_skills::SkillSource::all(), None);

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
