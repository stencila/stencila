//! `create_workflow` tool: synthesize a workflow definition for immediate launch.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;
use stencila_schema::Workflow;

use crate::error::{AgentError, AgentResult};
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

const EPHEMERAL_SENTINEL: &str = ".gitignore";
const EPHEMERAL_SENTINEL_CONTENT: &str = "*\n";

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_workflow".into(),
        description: "Create a Stencila Workflow from structured fields, validate it immediately, and write it to the workspace. By default the workflow is marked ephemeral using a `.gitignore` sentinel file so it can be launched immediately and cleaned up later unless the user chooses to save it.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Workflow name in lowercase kebab-case."
                },
                "description": {
                    "type": "string",
                    "description": "Brief summary of what the workflow does."
                },
                "goal": {
                    "type": "string",
                    "description": "Optional default goal for the workflow."
                },
                "pipeline": {
                    "type": "string",
                    "description": "Optional Graphviz DOT digraph defining the workflow pipeline. Must use the form `digraph <name> { ... }` with a named graph (e.g. `digraph my_flow { start -> work -> end }`)."
                },
                "persist": {
                    "type": "boolean",
                    "description": "Whether to persist the workflow as a normal saved workflow. Defaults to false, which creates it as an ephemeral on-disk workflow marked by a `.gitignore` sentinel file."
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Whether to overwrite an existing persisted workflow when persist is true. Defaults to false."
                }
            },
            "required": ["name", "description"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move { execute(args, Path::new(env.working_directory())).await })
        },
    )
}

async fn execute(args: Value, cwd: &Path) -> AgentResult<ToolOutput> {
    let name = required_str(&args, "name")?;
    let description = required_str(&args, "description")?;
    let goal = optional_str(&args, "goal");
    let pipeline = optional_str(&args, "pipeline");
    let persist = optional_bool(&args, "persist").unwrap_or(false);
    let overwrite = optional_bool(&args, "overwrite").unwrap_or(false);

    let workflow = build_workflow(name, description, goal, pipeline).await?;

    let content = stencila_codecs::to_string(
        &stencila_schema::Node::Workflow(workflow.clone()),
        Some(stencila_codecs::EncodeOptions {
            format: Some(stencila_codecs::Format::Markdown),
            ..Default::default()
        }),
    )
    .await
    .map_err(|error| AgentError::ValidationError {
        reason: error.to_string(),
    })?;

    let path = write_workflow(cwd, &workflow.name, &content, overwrite, !persist).await?;

    let result = json!({
        "created": true,
        "persisted": persist,
        "ephemeral": !persist,
        "name": workflow.name,
        "description": workflow.description,
        "goal": workflow.goal,
        "path": path.display().to_string(),
    });

    Ok(ToolOutput::Text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| r#"{"created":true}"#.to_string()),
    ))
}

fn optional_str(args: &Value, name: &str) -> Option<String> {
    args.get(name).and_then(Value::as_str).map(String::from)
}

fn optional_bool(args: &Value, name: &str) -> Option<bool> {
    args.get(name).and_then(Value::as_bool)
}

async fn build_workflow(
    name: &str,
    description: &str,
    goal: Option<String>,
    pipeline: Option<String>,
) -> AgentResult<Workflow> {
    let name_errors = validate_name(name);
    if !name_errors.is_empty() {
        return Err(AgentError::ValidationError {
            reason: format!(
                "Invalid workflow name `{name}`: {}",
                name_errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        });
    }

    let mut workflow = Workflow::new(description.to_string(), name.to_string());
    workflow.goal = goal;
    workflow.pipeline = pipeline;

    let validation_errors = validate_workflow(&workflow, None);
    if !validation_errors.is_empty() {
        return Err(AgentError::ValidationError {
            reason: format!(
                "Invalid workflow definition: {}",
                validation_errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        });
    }

    // Build the YAML frontmatter string.
    // The Markdown codec's `to_markdown` only emits frontmatter when
    // `self.frontmatter` is `Some`, so we must populate it here.
    let mut yaml_parts = vec![
        format!("type: Workflow"),
        format!("name: {name}"),
        format!("description: {description}"),
    ];
    if let Some(ref goal) = workflow.goal {
        yaml_parts.push(format!("goal: {goal}"));
    }
    if let Some(ref pipeline) = workflow.pipeline {
        // Pipeline DOT strings can be multi-line, so use a YAML literal block
        yaml_parts.push(format!("pipeline: |\n  {}", pipeline.replace('\n', "\n  ")));
    }
    workflow.frontmatter = Some(yaml_parts.join("\n"));

    Ok(workflow)
}

async fn write_workflow(
    cwd: &Path,
    name: &str,
    content: &str,
    overwrite: bool,
    ephemeral: bool,
) -> AgentResult<PathBuf> {
    let workflows_dir = closest_workflows_dir(cwd, true).await?;
    let workflow_dir = workflows_dir.join(name);
    let workflow_path = workflow_dir.join("WORKFLOW.md");
    let sentinel_path = workflow_dir.join(EPHEMERAL_SENTINEL);

    if workflow_path.exists() && !overwrite {
        return Err(AgentError::ValidationError {
            reason: format!(
                "Workflow `{name}` already exists at `{}`",
                workflow_dir.display()
            ),
        });
    }

    tokio::fs::create_dir_all(&workflow_dir)
        .await
        .map_err(|error| AgentError::ValidationError {
            reason: error.to_string(),
        })?;
    tokio::fs::write(&workflow_path, content)
        .await
        .map_err(|error| AgentError::ValidationError {
            reason: error.to_string(),
        })?;

    if ephemeral {
        tokio::fs::write(&sentinel_path, EPHEMERAL_SENTINEL_CONTENT)
            .await
            .map_err(|error| AgentError::ValidationError {
                reason: error.to_string(),
            })?;
    } else if sentinel_path.exists() {
        tokio::fs::remove_file(&sentinel_path)
            .await
            .map_err(|error| AgentError::ValidationError {
                reason: error.to_string(),
            })?;
    }

    Ok(workflow_path)
}

static VALID_NAME_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-z0-9\-]+$").expect("workflow name regex should always compile")
});

fn validate_name(name: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push("name must not be empty".to_string());
        return errors;
    }
    if name.len() > 64 {
        errors.push(format!(
            "name must be at most 64 characters, got {}",
            name.len()
        ));
    }
    if !VALID_NAME_RE.is_match(name) {
        errors.push(
            "name may only contain lowercase alphanumeric characters and hyphens".to_string(),
        );
    }
    if name.starts_with('-') {
        errors.push("name must not start with a hyphen".to_string());
    }
    if name.ends_with('-') {
        errors.push("name must not end with a hyphen".to_string());
    }
    if name.contains("--") {
        errors.push("name must not contain consecutive hyphens".to_string());
    }

    errors
}

/// Regex for the expected DOT digraph envelope: `digraph <name> { … }`.
///
/// The graph name must be a bare DOT identifier (letter or underscore,
/// followed by alphanumerics/underscores). This mirrors the grammar
/// enforced by the Attractor parser (§2.2) so that obviously-invalid DOT
/// is rejected before the workflow is written to disk.
static DIGRAPH_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)^\s*digraph\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{.*\}\s*$")
        .expect("digraph regex should always compile")
});

fn validate_pipeline_dot(pipeline: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let trimmed = pipeline.trim();

    if trimmed.is_empty() {
        errors.push("pipeline must not be empty".to_string());
        return errors;
    }

    if trimmed.starts_with("strict") {
        errors.push("pipeline DOT parse error: strict modifier not supported".to_string());
    }

    if trimmed.starts_with("graph ") || trimmed.starts_with("graph\t") || trimmed == "graph" {
        errors.push(
            "pipeline DOT parse error: only directed graphs (digraph) are supported".to_string(),
        );
    }

    if !DIGRAPH_RE.is_match(trimmed) {
        errors.push(
            "pipeline DOT parse error: expected `digraph <name> { ... }` where <name> is a bare identifier (letters, digits, underscores)".to_string(),
        );
        return errors;
    }

    // Check for `--` (undirected edges) which are not supported
    if trimmed.contains(" -- ") {
        errors.push(
            "pipeline DOT parse error: undirected edges (--) are not supported, use directed edges (->)".to_string(),
        );
    }

    errors
}

fn validate_workflow(workflow: &Workflow, dir_name: Option<&str>) -> Vec<String> {
    let mut errors = validate_name(&workflow.name);

    if let Some(dir_name) = dir_name
        && workflow.name != dir_name
    {
        errors.push(format!(
            "name `{}` does not match directory name `{dir_name}`",
            workflow.name
        ));
    }

    if workflow.description.is_empty() {
        errors.push("description must not be empty".to_string());
    } else if workflow.description.trim().eq_ignore_ascii_case("todo") {
        errors.push("description appears to be a placeholder".to_string());
    } else if workflow.description.len() > 1024 {
        errors.push(format!(
            "description must be at most 1024 characters, got {}",
            workflow.description.len()
        ));
    }

    if let Some(ref pipeline) = workflow.pipeline {
        errors.extend(validate_pipeline_dot(pipeline));
    }

    errors
}

async fn closest_workflows_dir(cwd: &Path, ensure: bool) -> AgentResult<PathBuf> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, ensure)
        .await
        .map_err(|error| AgentError::ValidationError {
            reason: error.to_string(),
        })?;
    stencila_dirs::stencila_workflows_dir(&stencila_dir, ensure)
        .await
        .map_err(|error| AgentError::ValidationError {
            reason: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_ephemeral_workflow() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        // Create a `.stencila` dir so `closest_stencila_dir` finds it here
        // rather than walking up the directory tree.
        std::fs::create_dir(tempdir.path().join(".stencila")).expect(".stencila dir");
        let args = json!({
            "name": "repo-audit",
            "description": "Audit a repository and summarize findings",
            "goal": "Audit the current repository and propose improvements",
            "pipeline": "digraph repo_audit { start -> work -> end work [agent=\"coder-g\", prompt=\"Audit: $goal\"] }"
        });

        let output = execute(args, tempdir.path()).await.expect("output");
        let ToolOutput::Text(text) = output else {
            panic!("expected text output");
        };
        let value: Value = serde_json::from_str(&text).expect("json");

        assert_eq!(value["created"], true);
        assert_eq!(value["persisted"], false);
        assert_eq!(value["name"], "repo-audit");

        let path_str = value["path"].as_str().expect("path should be a string");
        let expected_suffix = Path::new(".stencila")
            .join("workflows")
            .join("repo-audit")
            .join("WORKFLOW.md");
        assert!(
            path_str.ends_with(&expected_suffix.to_string_lossy().as_ref()),
            "path `{path_str}` should end with `{}`",
            expected_suffix.display()
        );

        let workflow_path = Path::new(path_str);
        assert!(workflow_path.exists(), "WORKFLOW.md should exist on disk");

        let sentinel_path = workflow_path
            .parent()
            .expect("parent")
            .join(EPHEMERAL_SENTINEL);
        assert!(
            sentinel_path.exists(),
            ".gitignore sentinel should exist on disk"
        );
        let sentinel_content = std::fs::read_to_string(&sentinel_path).expect("read sentinel");
        assert_eq!(
            sentinel_content, EPHEMERAL_SENTINEL_CONTENT,
            ".gitignore should contain *"
        );

        // Verify the file content is non-empty and contains required frontmatter
        let content = std::fs::read_to_string(workflow_path).expect("read WORKFLOW.md");
        assert!(
            content.contains("type: Workflow"),
            "WORKFLOW.md should contain `type: Workflow` frontmatter, got: {content}"
        );
        assert!(
            content.contains("name: repo-audit"),
            "WORKFLOW.md should contain `name: repo-audit`, got: {content}"
        );
        assert!(
            content.contains("description: Audit a repository"),
            "WORKFLOW.md should contain the description, got: {content}"
        );
    }

    #[test]
    fn rejects_pipeline_without_graph_name() {
        let errors = validate_pipeline_dot("digraph { start -> end }");
        assert!(!errors.is_empty(), "should reject digraph without a name");
        assert!(
            errors[0].contains("expected `digraph <name>"),
            "error should mention the expected format, got: {}",
            errors[0]
        );
    }

    #[test]
    fn rejects_undirected_graph() {
        let errors = validate_pipeline_dot("graph G { A -- B }");
        assert!(
            errors.iter().any(|e| e.contains("only directed graphs")),
            "should reject undirected graph"
        );
    }

    #[test]
    fn rejects_strict_modifier() {
        let errors = validate_pipeline_dot("strict digraph G { A -> B }");
        assert!(
            errors.iter().any(|e| e.contains("strict modifier")),
            "should reject strict modifier"
        );
    }

    #[test]
    fn rejects_undirected_edges() {
        let errors = validate_pipeline_dot("digraph G { A -- B }");
        assert!(
            errors.iter().any(|e| e.contains("undirected edges")),
            "should reject undirected edges"
        );
    }

    #[test]
    fn accepts_valid_pipeline() {
        let errors = validate_pipeline_dot(
            "digraph my_workflow { start -> work -> end work [agent=\"coder\"] }",
        );
        assert!(
            errors.is_empty(),
            "should accept valid DOT, got: {errors:?}"
        );
    }

    #[test]
    fn accepts_multiline_pipeline() {
        let errors = validate_pipeline_dot(
            r#"digraph review_flow {
    start -> generate -> review -> end
    generate [agent="coder", prompt="Write code"]
    review [agent="reviewer", prompt="Review code"]
}"#,
        );
        assert!(
            errors.is_empty(),
            "should accept multiline DOT, got: {errors:?}"
        );
    }

    #[tokio::test]
    async fn rejects_invalid_pipeline_at_creation() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(tempdir.path().join(".stencila")).expect(".stencila dir");
        let args = json!({
            "name": "bad-pipeline",
            "description": "A workflow with an invalid pipeline",
            "pipeline": "digraph { start -> end }"
        });

        let result = execute(args, tempdir.path()).await;
        let err = result
            .expect_err("should return error for invalid pipeline")
            .to_string();
        assert!(
            err.contains("expected `digraph <name>"),
            "error should mention the expected format, got: {err}"
        );
    }
}
