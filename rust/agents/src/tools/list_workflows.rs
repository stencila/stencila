//! `list_workflows` tool: discover available Stencila Workflows.

use std::path::Path;

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
                        json!({
                            "name": wf.name,
                            "description": wf.description,
                            "goal": wf.goal,
                            "path": wf.path,
                            "ephemeral": wf.ephemeral,
                        })
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
    path: String,
    ephemeral: bool,
}

async fn discover_workflows(cwd: &Path) -> Vec<WorkflowSummary> {
    let Some(stencila_dir) = stencila_dirs::closest_dot_dir(cwd, ".stencila") else {
        return Vec::new();
    };

    let workflows_dir = stencila_dir.join("workflows");
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
    summaries.sort_by(|a, b| a.name.cmp(&b.name));
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
            path: path.display().to_string(),
            ephemeral,
        })
    } else {
        None
    }
}
