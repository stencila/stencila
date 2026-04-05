//! `list_workflows` tool: discover available Stencila Workflows.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_workflows".into(),
        description: "List available Stencila Workflows from the workspace. Returns workflow \
            names, descriptions, goals, and ephemeral status to help decide which workflow to delegate to."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |_args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let cwd = Path::new(env.working_directory());
                let workflows = discover_workflows(cwd).await;

                let entries: Vec<Value> = workflows
                    .into_iter()
                    .map(|wf| {
                        let mut entry = json!({
                            "name": wf.name,
                            "description": wf.description,
                            "goal": wf.goal,
                            "path": wf.path,
                            "ephemeral": wf.ephemeral,
                        });

                        if let Some(ref keywords) = wf.keywords {
                            entry["keywords"] = json!(keywords);
                        }

                        if let Some(ref when_to_use) = wf.when_to_use {
                            entry["whenToUse"] = json!(when_to_use);
                        }
                        if let Some(ref when_not_to_use) = wf.when_not_to_use {
                            entry["whenNotToUse"] = json!(when_not_to_use);
                        }

                        entry
                    })
                    .collect();

                Ok(ToolOutput::Text(
                    serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string()),
                ))
            })
        },
    )
}

struct WorkflowSummary {
    name: String,
    description: String,
    goal: Option<String>,
    keywords: Option<Vec<String>>,
    when_to_use: Option<Vec<String>>,
    when_not_to_use: Option<Vec<String>>,
    path: String,
    ephemeral: bool,
}

async fn discover_workflows(cwd: &Path) -> Vec<WorkflowSummary> {
    let mut by_name: std::collections::HashMap<String, WorkflowSummary> =
        std::collections::HashMap::new();

    // Builtin workflows first (lowest precedence).
    // In debug mode, filter out test-* workflows (in release they are excluded
    // from the embedded assets by the #[exclude] attribute in the workflows crate).
    if let Some(builtin_dir) = builtin_workflows_dir() {
        for wf in list_workflow_dir(&builtin_dir).await {
            if cfg!(debug_assertions) && wf.name.starts_with("test-") {
                continue;
            }
            by_name.insert(wf.name.clone(), wf);
        }
    }

    // Workspace workflows (highest precedence, overwrites builtins)
    if let Some(stencila_dir) = stencila_dirs::closest_dot_dir(cwd, ".stencila") {
        let workflows_dir = stencila_dir.join("workflows");
        for wf in list_workflow_dir(&workflows_dir).await {
            by_name.insert(wf.name.clone(), wf);
        }
    }

    let mut summaries: Vec<WorkflowSummary> = by_name.into_values().collect();
    summaries.sort_by(|a, b| a.name.cmp(&b.name));
    summaries
}

/// Get the directory containing builtin workflows.
///
/// In debug mode, reads directly from the repo source tree.
/// In release mode, reads from the versioned cache directory that is
/// populated at startup by `stencila_workflows::initialize_builtin()`.
fn builtin_workflows_dir() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.stencila/workflows");
        return dir.exists().then_some(dir);
    }

    stencila_dirs::get_versioned_app_dir(
        stencila_dirs::DirType::BuiltinWorkflows,
        stencila_version::STENCILA_VERSION,
        false,
        false,
    )
    .ok()
}

/// List all workflow summaries in a directory.
async fn list_workflow_dir(workflows_dir: &Path) -> Vec<WorkflowSummary> {
    if !workflows_dir.exists() {
        return Vec::new();
    }

    let pattern = format!("{}/*/WORKFLOW.md", workflows_dir.display());
    let paths: Vec<_> = glob::glob(&pattern)
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    let mut summaries = Vec::new();
    for path in paths {
        if let Some(summary) = load_workflow_summary(&path).await {
            summaries.push(summary);
        }
    }
    summaries
}

async fn load_workflow_summary(path: &Path) -> Option<WorkflowSummary> {
    use stencila_codecs::{DecodeOptions, Format};
    use stencila_schema::{Node, NodeType};

    let content = tokio::fs::read_to_string(path).await.ok()?;

    let node = stencila_codecs::from_str(
        &content,
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            node_type: Some(NodeType::Workflow),
            ..Default::default()
        }),
    )
    .await
    .ok()?;

    if let Node::Workflow(wf) = node {
        let ephemeral = path
            .parent()
            .map(|dir| {
                let sentinel = dir.join(".gitignore");
                sentinel
                    .is_file()
                    .then(|| std::fs::read_to_string(&sentinel).ok())
                    .flatten()
                    .is_some_and(|content| content.trim() == "*")
            })
            .unwrap_or(false);
        Some(WorkflowSummary {
            name: wf.name.clone(),
            description: wf.description.clone(),
            goal: wf.goal.clone(),
            keywords: wf.options.keywords.clone(),
            when_to_use: wf.when_to_use.clone(),
            when_not_to_use: wf.when_not_to_use.clone(),
            path: path.display().to_string(),
            ephemeral,
        })
    } else {
        None
    }
}
