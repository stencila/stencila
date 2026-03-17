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
    #[error("Name must not be empty")]
    NameEmpty,

    #[error("Name must be at most 64 characters, got {0}")]
    NameTooLong(usize),

    #[error("Name may only contain lowercase alphanumeric characters and hyphens")]
    NameInvalidChars,

    #[error("Name must not start with a hyphen")]
    NameLeadingHyphen,

    #[error("Name must not end with a hyphen")]
    NameTrailingHyphen,

    #[error("Name must not contain consecutive hyphens")]
    NameConsecutiveHyphens,

    #[error("Name `{name}` does not match directory name `{dir_name}`")]
    NameDirMismatch { name: String, dir_name: String },

    #[error("Description must not be empty")]
    DescriptionEmpty,

    #[error("Description appears to be a placeholder")]
    DescriptionPlaceholder,

    #[error("Description must be at most 1024 characters, got {0}")]
    DescriptionTooLong(usize),

    #[error("Pipeline DOT parse error: {0}")]
    PipelineParseError(String),

    #[error("Pipeline validation error: {0}")]
    PipelineValidationError(String),
}

/// Validation warnings for a workflow definition
///
/// Warnings are advisory and do not prevent the workflow from being used.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationWarning {
    #[error("Pipeline warning: {0}")]
    PipelineValidationWarning(String),

    #[error("{0}")]
    GoalNotPassedToChild(String),
}

fn validate_workflow_handler_node(
    workflow_name: &str,
    node: &stencila_attractor::graph::Node,
) -> Vec<ValidationError> {
    if node.handler_type() != "workflow" {
        return Vec::new();
    }

    let mut errors = Vec::new();

    match node.get_str_attr("workflow") {
        Some(name) if !name.trim().is_empty() => {
            if name == workflow_name {
                errors.push(ValidationError::PipelineValidationError(format!(
                    "workflow node `{}` references workflow `{name}` recursively",
                    node.id
                )));
            }
        }
        _ => errors.push(ValidationError::PipelineValidationError(format!(
            "workflow node `{}` must have a non-empty `workflow` attribute",
            node.id
        ))),
    }

    if node
        .get_str_attr("goal")
        .is_some_and(|goal| goal.trim().is_empty())
    {
        errors.push(ValidationError::PipelineValidationError(format!(
            "workflow node `{}` has an empty `goal` attribute",
            node.id
        )));
    }

    errors
}

/// Check whether a workflow-type node passes `$goal` to the child workflow.
///
/// Should be called on the **pre-transform** graph so that `$goal` placeholders
/// have not yet been expanded by the variable-expansion transform.
fn check_goal_pass_through(node: &stencila_attractor::graph::Node) -> Option<ValidationWarning> {
    // Before transforms, workflow nodes are identified by an explicit `type="workflow"`
    // attribute or the presence of a `workflow` attribute.
    if node.get_str_attr("type") != Some("workflow") && !node.attrs.contains_key("workflow") {
        return None;
    }

    let attrs_contain_goal_var = ["goal", "prompt", "label"]
        .iter()
        .any(|attr| node.get_str_attr(attr).is_some_and(|v| v.contains("$goal")));

    if attrs_contain_goal_var {
        return None;
    }

    let child_name = node.get_str_attr("workflow").unwrap_or("(unknown)");

    Some(ValidationWarning::GoalNotPassedToChild(format!(
        "Workflow node `{}` does not pass $goal to child workflow `{}`; \
         the child will receive the node's label as its goal instead of \
         the user's input — consider adding goal=\"$goal\"",
        node.id, child_name
    )))
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
///
/// Returns a tuple of `(errors, warnings)`. Errors indicate the workflow
/// is invalid; warnings are advisory but may indicate issues that will
/// surface at runtime.
pub fn validate_workflow(
    workflow: &Workflow,
    dir_name: Option<&str>,
) -> (Vec<ValidationError>, Vec<ValidationWarning>) {
    let mut errors = validate_name(&workflow.name);
    let mut warnings = Vec::new();

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

    let parent_has_user_goal = workflow.goal_hint.is_some() && workflow.goal.is_none();

    if let Some(ref pipeline) = workflow.pipeline {
        match stencila_attractor::parse_dot(pipeline) {
            Ok(mut graph) => {
                // Check for $goal pass-through on the raw graph *before*
                // transforms, because variable expansion replaces `$goal`
                // placeholders and would hide them from the check.
                if parent_has_user_goal {
                    for node in graph.nodes.values() {
                        warnings.extend(check_goal_pass_through(node));
                    }
                }

                // Apply transforms (sugar, variable expansion, stylesheet) so that
                // validation sees the canonical node attributes. Without this, nodes
                // using sugar attributes like `shell` still appear as codergen nodes
                // and trigger false-positive warnings.
                let transforms = stencila_attractor::TransformRegistry::with_defaults();
                if let Err(e) = transforms.apply_all(&mut graph) {
                    errors.push(ValidationError::PipelineValidationError(e.to_string()));
                    return (errors, warnings);
                }

                for node in graph.nodes.values() {
                    errors.extend(validate_workflow_handler_node(&workflow.name, node));
                }

                let diagnostics = stencila_attractor::validation::validate(&graph, &[]);
                for diag in &diagnostics {
                    match diag.severity {
                        stencila_attractor::validation::Severity::Error => {
                            errors.push(ValidationError::PipelineValidationError(
                                diag.message.clone(),
                            ));
                        }
                        stencila_attractor::validation::Severity::Warning => {
                            warnings.push(ValidationWarning::PipelineValidationWarning(
                                diag.message.clone(),
                            ));
                        }
                        stencila_attractor::validation::Severity::Info => {}
                    }
                }
            }
            Err(e) => {
                errors.push(ValidationError::PipelineParseError(e.to_string()));
            }
        }
    }

    (errors, warnings)
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

        let (errors, _) = validate_workflow(&workflow, None);
        assert!(errors.is_empty());

        let (errors, _) = validate_workflow(&workflow, Some("test-workflow"));
        assert!(errors.is_empty());

        let (errors, _) = validate_workflow(&workflow, Some("other-dir"));
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::NameDirMismatch { .. }))
        );

        let empty_desc = Workflow::new(String::new(), "test".into());
        let (errors, _) = validate_workflow(&empty_desc, None);
        assert!(errors.contains(&ValidationError::DescriptionEmpty));

        let long_desc = Workflow::new("x".repeat(1025), "test".into());
        let (errors, _) = validate_workflow(&long_desc, None);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::DescriptionTooLong(1025)))
        );

        let todo_desc = Workflow::new("TODO".into(), "test".into());
        let (errors, _) = validate_workflow(&todo_desc, None);
        assert!(errors.contains(&ValidationError::DescriptionPlaceholder));

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

        let (errors, _) = validate_workflow(&workflow, None);
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

        let (errors, _) = validate_workflow(&workflow, None);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::PipelineParseError(_)))
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_surfaces_pipeline_warnings() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "test".into());
        // The codergen node "work" has no prompt or label → prompt_on_llm_nodes warning
        workflow.pipeline = Some(
            r#"digraph test {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    work  [shape=box]
    start -> work -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::PipelineValidationWarning(msg) if msg.contains("no input or label"))),
            "should warn about missing prompt: {warnings:?}"
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_handler_requires_workflow_attribute() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "test".into());
        workflow.pipeline = Some(
            r#"digraph test {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    child [type="workflow"]
    start -> child -> exit
}"#
            .to_string(),
        );

        let (errors, _) = validate_workflow(&workflow, None);
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::PipelineValidationError(msg)
            if msg.contains("must have a non-empty `workflow` attribute")
        )));
        Ok(())
    }

    #[test]
    fn validate_workflow_handler_rejects_empty_goal_attribute() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "test".into());
        workflow.pipeline = Some(
            r#"digraph test {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    child [type="workflow", workflow="other-workflow", goal="   "]
    start -> child -> exit
}"#
            .to_string(),
        );

        let (errors, _) = validate_workflow(&workflow, None);
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::PipelineValidationError(msg)
            if msg.contains("empty `goal` attribute")
        )));
        Ok(())
    }

    #[test]
    fn validate_workflow_warns_goal_not_passed_to_child() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "parent".into());
        workflow.goal_hint = Some("What do you want to build?".into());
        workflow.goal = None;
        workflow.pipeline = Some(
            r#"digraph parent {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    Design [type="workflow", workflow="software-design-iterative"]
    start -> Design -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::GoalNotPassedToChild(msg) if msg.contains("$goal"))),
            "should warn about missing $goal pass-through: {warnings:?}"
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_no_goal_warning_when_goal_passed() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "parent".into());
        workflow.goal_hint = Some("What do you want to build?".into());
        workflow.goal = None;
        workflow.pipeline = Some(
            r#"digraph parent {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    Design [type="workflow", workflow="software-design-iterative", goal="$goal"]
    start -> Design -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            !warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::GoalNotPassedToChild(_))),
            "should not warn when $goal is passed: {warnings:?}"
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_no_goal_warning_when_parent_has_fixed_goal() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "parent".into());
        workflow.goal_hint = Some("What do you want to build?".into());
        workflow.goal = Some("Build a spaceship".into());
        workflow.pipeline = Some(
            r#"digraph parent {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    Design [type="workflow", workflow="software-design-iterative"]
    start -> Design -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            !warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::GoalNotPassedToChild(_))),
            "should not warn when parent has a fixed goal: {warnings:?}"
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_no_goal_warning_when_no_goal_hint() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "parent".into());
        workflow.goal_hint = None;
        workflow.goal = None;
        workflow.pipeline = Some(
            r#"digraph parent {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    Design [type="workflow", workflow="software-design-iterative"]
    start -> Design -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            !warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::GoalNotPassedToChild(_))),
            "should not warn when parent has no goal_hint: {warnings:?}"
        );
        Ok(())
    }

    #[test]
    fn validate_workflow_no_goal_warning_when_prompt_contains_goal() -> eyre::Result<()> {
        let mut workflow = Workflow::new("A test workflow".into(), "parent".into());
        workflow.goal_hint = Some("What do you want to build?".into());
        workflow.goal = None;
        workflow.pipeline = Some(
            r#"digraph parent {
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    Design [type="workflow", workflow="software-design-iterative", prompt="Do this: $goal"]
    start -> Design -> exit
}"#
            .to_string(),
        );

        let (errors, warnings) = validate_workflow(&workflow, None);
        assert!(errors.is_empty(), "should have no errors: {errors:?}");
        assert!(
            !warnings
                .iter()
                .any(|w| matches!(w, ValidationWarning::GoalNotPassedToChild(_))),
            "should not warn when $goal is in prompt: {warnings:?}"
        );
        Ok(())
    }
}
