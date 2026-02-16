//! Workflow validation.
//!
//! Validates workflow definitions against naming rules (same as agents) and
//! property constraints, including optional pipeline parsing and lint validation.

use regex::Regex;
use thiserror::Error;

use stencila_schema::Workflow;

/// Validation errors for a workflow definition
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

    #[error("pipeline DOT parse error: {0}")]
    PipelineParseError(String),

    #[error("pipeline validation error: {0}")]
    PipelineValidationError(String),
}

/// Validate a workflow name.
///
/// Names must be lowercase kebab-case:
/// - 1-64 characters
/// - Only lowercase alphanumeric characters and hyphens
/// - Must not start or end with a hyphen
/// - Must not contain consecutive hyphens
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

/// Validate a workflow against naming, property, and pipeline rules.
///
/// Optionally checks that the name matches the parent directory name.
/// If a pipeline is present, attempts to parse and validate it.
pub fn validate_workflow(workflow: &Workflow, dir_name: Option<&str>) -> Vec<ValidationError> {
    let mut errors = validate_name(&workflow.name);

    if let Some(dir_name) = dir_name
        && workflow.name != dir_name
    {
        errors.push(ValidationError::NameDirMismatch {
            name: workflow.name.clone(),
            dir_name: dir_name.to_string(),
        });
    }

    if workflow.description.is_empty() {
        errors.push(ValidationError::DescriptionEmpty);
    } else if workflow.description.trim().eq_ignore_ascii_case("todo") {
        errors.push(ValidationError::DescriptionPlaceholder);
    } else if workflow.description.len() > 1024 {
        errors.push(ValidationError::DescriptionTooLong(
            workflow.description.len(),
        ));
    }

    if let Some(ref pipeline) = workflow.pipeline {
        match stencila_attractor::parse_dot(pipeline) {
            Ok(graph) => {
                let diagnostics = stencila_attractor::validation::validate(&graph, &[]);
                let error_diagnostics: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| d.severity == stencila_attractor::validation::Severity::Error)
                    .collect();
                for diag in error_diagnostics {
                    errors.push(ValidationError::PipelineValidationError(
                        diag.message.clone(),
                    ));
                }
            }
            Err(e) => {
                errors.push(ValidationError::PipelineParseError(e.to_string()));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() -> eyre::Result<()> {
        assert!(validate_name("data-pipeline").is_empty());
        assert!(validate_name("code-review").is_empty());
        assert!(validate_name("build-and-test").is_empty());
        assert!(validate_name("a").is_empty());
        assert!(validate_name("abc123").is_empty());
        assert!(validate_name("my-workflow").is_empty());
        Ok(())
    }

    #[test]
    fn invalid_names() -> eyre::Result<()> {
        assert!(validate_name("").contains(&ValidationError::NameEmpty));
        assert!(validate_name("Data-Pipeline").contains(&ValidationError::NameInvalidChars));
        assert!(validate_name("-data").contains(&ValidationError::NameLeadingHyphen));
        assert!(validate_name("data-").contains(&ValidationError::NameTrailingHyphen));
        assert!(validate_name("data--pipeline").contains(&ValidationError::NameConsecutiveHyphens));

        let long_name = "a".repeat(65);
        assert!(validate_name(&long_name).contains(&ValidationError::NameTooLong(65)));

        Ok(())
    }

    #[test]
    fn validate_workflow_checks() -> eyre::Result<()> {
        let workflow = Workflow::new("A test workflow".into(), "test-workflow".into());

        assert!(validate_workflow(&workflow, None).is_empty());
        assert!(validate_workflow(&workflow, Some("test-workflow")).is_empty());

        let errors = validate_workflow(&workflow, Some("other-dir"));
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::NameDirMismatch { .. }))
        );

        let empty_desc = Workflow::new(String::new(), "test".into());
        assert!(validate_workflow(&empty_desc, None).contains(&ValidationError::DescriptionEmpty));

        let long_desc = Workflow::new("x".repeat(1025), "test".into());
        assert!(
            validate_workflow(&long_desc, None)
                .iter()
                .any(|e| matches!(e, ValidationError::DescriptionTooLong(1025)))
        );

        let todo_desc = Workflow::new("TODO".into(), "test".into());
        assert!(
            validate_workflow(&todo_desc, None)
                .contains(&ValidationError::DescriptionPlaceholder)
        );

        Ok(())
    }

    #[test]
    fn validate_workflow_with_valid_pipeline() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "test".into());
        workflow.pipeline = Some(
            r#"digraph test {
    node [shape=box]
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    work  [agent="helper", prompt="Do work"]
    start -> work -> exit
}"#
            .to_string(),
        );

        let errors = validate_workflow(&workflow, None);
        // Should have no pipeline parse or validation errors
        assert!(
            !errors
                .iter()
                .any(|e| matches!(e, ValidationError::PipelineParseError(_)))
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_with_invalid_pipeline() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "test".into());
        workflow.pipeline = Some("this is not valid DOT".to_string());

        let errors = validate_workflow(&workflow, None);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::PipelineParseError(_)))
        );
        Ok(())
    }
}
