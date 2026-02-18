//! Agent validation.
//!
//! Validates agent definitions against naming rules (same as skills) and
//! property constraints.

use regex::Regex;
use thiserror::Error;

use stencila_schema::Agent;

/// Validation errors for an agent definition
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("name must not be empty")]
    NameEmpty,

    #[error("name must be at most 64 characters, got {0}")]
    NameTooLong(usize),

    #[error("name may only contain lowercase alphanumeric characters and hyphens")]
    NameInvalidChars,

    #[error("name must not start with a hyphen")]
    NameLeadingHyphen,

    #[error("name must not end with a hyphen")]
    NameTrailingHyphen,

    #[error("name must not contain consecutive hyphens")]
    NameConsecutiveHyphens,

    #[error("name `{name}` does not match directory name `{dir_name}`")]
    NameDirMismatch { name: String, dir_name: String },

    #[error("description must not be empty")]
    DescriptionEmpty,

    #[error("description appears to be a placeholder")]
    DescriptionPlaceholder,

    #[error("description must be at most 1024 characters, got {0}")]
    DescriptionTooLong(usize),

    #[error("compatibility must not be empty if provided")]
    CompatibilityEmpty,

    #[error("compatibility must be at most 500 characters, got {0}")]
    CompatibilityTooLong(usize),

    #[error("toolTimeout must be greater than 0, got {0}")]
    ToolTimeoutInvalid(i64),

    #[error("maxTurns must not be negative, got {0}")]
    MaxTurnsNegative(i64),

    #[error("maxToolRounds must be greater than 0, got {0}")]
    MaxToolRoundsInvalid(i64),

    #[error("maxSubagentDepth must not be negative, got {0}")]
    MaxSubagentDepthNegative(i64),
}

/// Validate an agent name.
///
/// Names must be lowercase kebab-case:
/// - 1-64 characters
/// - Only lowercase alphanumeric characters and hyphens
/// - Must not start or end with a hyphen
/// - Must not contain consecutive hyphens
///
/// By convention, names follow a `thing-role` pattern describing
/// the agent's domain and function (e.g. `code-engineer`,
/// `code-reviewer`, `data-analyst`, `site-designer`).
pub fn validate_name(name: &str) -> Vec<ValidationError> {
    let valid_chars =
        Regex::new(r"^[a-z0-9\-]+$").expect("validate_name: invalid regex should not happen");

    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push(ValidationError::NameEmpty);
        return errors;
    }

    if name.len() > 64 {
        errors.push(ValidationError::NameTooLong(name.len()));
    }

    if !valid_chars.is_match(name) {
        errors.push(ValidationError::NameInvalidChars);
    }

    if name.starts_with('-') {
        errors.push(ValidationError::NameLeadingHyphen);
    }

    if name.ends_with('-') {
        errors.push(ValidationError::NameTrailingHyphen);
    }

    if name.contains("--") {
        errors.push(ValidationError::NameConsecutiveHyphens);
    }

    errors
}

/// Validate an agent against naming and property rules.
///
/// Optionally checks that the name matches the parent directory name.
pub fn validate_agent(agent: &Agent, dir_name: Option<&str>) -> Vec<ValidationError> {
    let mut errors = validate_name(&agent.name);

    if let Some(dir_name) = dir_name
        && agent.name != dir_name
    {
        errors.push(ValidationError::NameDirMismatch {
            name: agent.name.clone(),
            dir_name: dir_name.to_string(),
        });
    }

    if agent.description.is_empty() {
        errors.push(ValidationError::DescriptionEmpty);
    } else if agent.description.trim().eq_ignore_ascii_case("todo") {
        errors.push(ValidationError::DescriptionPlaceholder);
    } else if agent.description.len() > 1024 {
        errors.push(ValidationError::DescriptionTooLong(agent.description.len()));
    }

    if let Some(compat) = &agent.options.compatibility {
        if compat.is_empty() {
            errors.push(ValidationError::CompatibilityEmpty);
        } else if compat.len() > 500 {
            errors.push(ValidationError::CompatibilityTooLong(compat.len()));
        }
    }

    if let Some(timeout) = agent.options.tool_timeout
        && timeout <= 0
    {
        errors.push(ValidationError::ToolTimeoutInvalid(timeout));
    }

    if let Some(max_turns) = agent.options.max_turns
        && max_turns < 0
    {
        errors.push(ValidationError::MaxTurnsNegative(max_turns));
    }

    if let Some(max_rounds) = agent.options.max_tool_rounds
        && max_rounds < 0
    {
        errors.push(ValidationError::MaxToolRoundsInvalid(max_rounds));
    }

    if let Some(depth) = agent.options.max_subagent_depth
        && depth < 0
    {
        errors.push(ValidationError::MaxSubagentDepthNegative(depth));
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        // Conventional thing-role names
        assert!(validate_name("code-engineer").is_empty());
        assert!(validate_name("code-reviewer").is_empty());
        assert!(validate_name("data-analyst").is_empty());
        assert!(validate_name("site-designer").is_empty());
        assert!(validate_name("site-tester").is_empty());
        // Other valid kebab-case
        assert!(validate_name("a").is_empty());
        assert!(validate_name("abc123").is_empty());
        assert!(validate_name("my-agent").is_empty());
        Ok(())
    }

    #[test]
    fn invalid_names() -> eyre::Result<()> {
        assert!(validate_name("").contains(&ValidationError::NameEmpty));
        assert!(validate_name("Data-Analysis").contains(&ValidationError::NameInvalidChars));
        assert!(validate_name("-data").contains(&ValidationError::NameLeadingHyphen));
        assert!(validate_name("data-").contains(&ValidationError::NameTrailingHyphen));
        assert!(validate_name("data--analysis").contains(&ValidationError::NameConsecutiveHyphens));

        let long_name = "a".repeat(65);
        assert!(validate_name(&long_name).contains(&ValidationError::NameTooLong(65)));

        Ok(())
    }

    #[test]
    fn validate_agent_checks() -> eyre::Result<()> {
        let agent = Agent::new("A test agent".into(), "test-agent".into());

        assert!(validate_agent(&agent, None).is_empty());
        assert!(validate_agent(&agent, Some("test-agent")).is_empty());

        let errors = validate_agent(&agent, Some("other-dir"));
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::NameDirMismatch { .. }))
        );

        let empty_desc = Agent::new(String::new(), "test".into());
        assert!(validate_agent(&empty_desc, None).contains(&ValidationError::DescriptionEmpty));

        let long_desc = Agent::new("x".repeat(1025), "test".into());
        assert!(
            validate_agent(&long_desc, None)
                .iter()
                .any(|e| matches!(e, ValidationError::DescriptionTooLong(1025)))
        );

        let todo_desc = Agent::new("TODO".into(), "test".into());
        assert!(
            validate_agent(&todo_desc, None).contains(&ValidationError::DescriptionPlaceholder)
        );

        Ok(())
    }

    #[test]
    fn validate_agent_property_checks() -> eyre::Result<()> {
        let mut agent = Agent::new("A test agent".into(), "test".into());

        // Valid timeout
        agent.options.tool_timeout = Some(30);
        assert!(validate_agent(&agent, None).is_empty());

        // Invalid timeout
        agent.options.tool_timeout = Some(0);
        assert!(
            validate_agent(&agent, None)
                .iter()
                .any(|e| matches!(e, ValidationError::ToolTimeoutInvalid(0)))
        );
        agent.options.tool_timeout = None;

        // Negative max_turns
        agent.options.max_turns = Some(-1);
        assert!(
            validate_agent(&agent, None)
                .iter()
                .any(|e| matches!(e, ValidationError::MaxTurnsNegative(-1)))
        );
        agent.options.max_turns = None;

        // Zero max_tool_rounds is valid (0 = unlimited per spec §2.2)
        agent.options.max_tool_rounds = Some(0);
        assert!(
            !validate_agent(&agent, None)
                .iter()
                .any(|e| matches!(e, ValidationError::MaxToolRoundsInvalid(_)))
        );

        // Negative max_tool_rounds is invalid
        agent.options.max_tool_rounds = Some(-1);
        assert!(
            validate_agent(&agent, None)
                .iter()
                .any(|e| matches!(e, ValidationError::MaxToolRoundsInvalid(-1)))
        );
        agent.options.max_tool_rounds = None;

        // Negative max_subagent_depth
        agent.options.max_subagent_depth = Some(-1);
        assert!(
            validate_agent(&agent, None)
                .iter()
                .any(|e| matches!(e, ValidationError::MaxSubagentDepthNegative(-1)))
        );

        Ok(())
    }
}
