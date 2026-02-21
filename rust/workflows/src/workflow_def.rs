//! Workflow definition loading and discovery.
//!
//! Parses `WORKFLOW.md` files (YAML frontmatter + optional Markdown body with
//! DOT pipeline) from workspace `.stencila/workflows/<name>/WORKFLOW.md`
//! and provides a public API mirroring the agents crate pattern.

use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Node, NodeType, Workflow};

/// Where a workflow was discovered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum WorkflowSource {
    /// `.stencila/workflows/` in the workspace
    Workspace,
}

impl std::fmt::Display for WorkflowSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace => f.write_str("workspace"),
        }
    }
}

/// An instance of a workflow loaded from disk.
///
/// Wraps a [`Workflow`] with its file path, home directory, and source metadata.
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub struct WorkflowInstance {
    #[serde(flatten)]
    pub inner: Workflow,

    /// Path to the WORKFLOW.md file
    path: PathBuf,

    /// Home directory of the workflow (parent of WORKFLOW.md)
    #[serde(skip)]
    home: PathBuf,

    /// Which source this workflow was loaded from
    #[serde(skip)]
    source: Option<WorkflowSource>,
}

impl std::ops::Deref for WorkflowInstance {
    type Target = Workflow;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for WorkflowInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Custom serialization for display purposes
impl Serialize for WorkflowInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WorkflowInstance", 6)?;

        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("goal", &self.inner.goal)?;
        state.serialize_field("source", &self.source.map(|s| s.to_string()))?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("pipeline", &self.inner.pipeline)?;

        state.end()
    }
}

impl WorkflowInstance {
    fn new(inner: Workflow, path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;

        let home = path
            .parent()
            .ok_or_else(|| eyre::eyre!("WORKFLOW.md not in a directory"))?
            .to_path_buf();

        Ok(Self {
            inner,
            path,
            home,
            source: None,
        })
    }

    /// Return a copy with the source set.
    fn with_source(mut self, source: WorkflowSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Get the path to the WORKFLOW.md file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the home directory of the workflow
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Which source this workflow was loaded from
    pub fn source(&self) -> Option<WorkflowSource> {
        self.source
    }

    /// Parse the pipeline DOT source into an attractor Graph.
    pub fn graph(&self) -> eyre::Result<stencila_attractor::Graph> {
        let pipeline = self
            .pipeline
            .as_deref()
            .ok_or_else(|| eyre::eyre!("Workflow `{}` has no pipeline defined", self.name))?;
        stencila_attractor::parse_dot(pipeline)
            .map_err(|e| eyre::eyre!("Failed to parse pipeline for workflow `{}`: {e}", self.name))
    }

    /// Return the agent names referenced in the pipeline DOT.
    pub fn agent_references(&self) -> Vec<String> {
        let Ok(graph) = self.graph() else {
            return Vec::new();
        };
        let mut agents = Vec::new();
        for node in graph.nodes.values() {
            if let Some(stencila_attractor::AttrValue::String(agent)) = node.attrs.get("agent")
                && !agents.contains(agent)
            {
                agents.push(agent.clone());
            }
        }
        agents
    }
}

/// Get the closest `.stencila/workflows` directory, optionally creating it
pub(crate) async fn closest_workflows_dir(cwd: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, ensure).await?;
    stencila_dirs::stencila_workflows_dir(&stencila_dir, ensure).await
}

/// Discover workflows from the workspace.
///
/// Workflows are discovered from `.stencila/workflows/*/WORKFLOW.md`.
pub async fn discover(cwd: &Path) -> Vec<WorkflowInstance> {
    if let Some(stencila_dir) = stencila_dirs::closest_dot_dir(cwd, ".stencila") {
        let workflows_dir = stencila_dir.join("workflows");
        list(&workflows_dir).await
    } else {
        Vec::new()
    }
}

/// Find a workflow by name.
pub async fn get_by_name(cwd: &Path, name: &str) -> Result<WorkflowInstance> {
    let workflows = discover(cwd).await;
    workflows
        .into_iter()
        .find(|wf| wf.name == name)
        .ok_or_else(|| eyre::eyre!("Unable to find workflow with name `{name}`"))
}

/// List all workflows found in a workflows directory
pub async fn list(workflows_dir: &Path) -> Vec<WorkflowInstance> {
    if !workflows_dir.exists() {
        return Vec::new();
    }

    match list_dir(workflows_dir).await {
        Ok(mut workflows) => {
            workflows.sort_by(|a, b| a.name.cmp(&b.name));
            workflows
        }
        Err(error) => {
            tracing::error!(
                "While listing workflows in `{}`: {error}",
                workflows_dir.display()
            );
            Vec::new()
        }
    }
}

/// List workflows in a directory.
///
/// Globs for `*/WORKFLOW.md` files (one level deep), decodes each as a Workflow.
async fn list_dir(workflows_dir: &Path) -> Result<Vec<WorkflowInstance>> {
    tracing::trace!(
        "Attempting to read workflows from `{}`",
        workflows_dir.display()
    );

    let mut workflows = vec![];
    for path in glob(&format!("{}/*/WORKFLOW.md", workflows_dir.display()))?.flatten() {
        match load_workflow(&path).await {
            Ok(instance) => workflows.push(instance),
            Err(error) => {
                tracing::warn!("Skipping `{}`: {error}", path.display());
            }
        }
    }

    Ok(workflows)
}

/// Load a single workflow from a WORKFLOW.md path.
pub(crate) async fn load_workflow(path: &Path) -> Result<WorkflowInstance> {
    let content = read_to_string(path).await?;

    let node = stencila_codecs::from_str(
        &content,
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            node_type: Some(NodeType::Workflow),
            ..Default::default()
        }),
    )
    .await?;

    if let Node::Workflow(workflow) = node {
        WorkflowInstance::new(workflow, path.to_path_buf())
            .map(|wf| wf.with_source(WorkflowSource::Workspace))
    } else {
        bail!(
            "Expected `{}` to be a `Workflow`, got a `{}`",
            path.display(),
            node.to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper to create a workflow directory --

    fn create_workflow(base: &Path, name: &str, description: &str, dot: Option<&str>) {
        let wf_dir = base.join(format!(".stencila/workflows/{name}"));
        std::fs::create_dir_all(&wf_dir).expect("create workflow dir");
        let dot_block = if let Some(dot) = dot {
            format!("\n```dot\n{dot}\n```\n")
        } else {
            String::new()
        };
        let content = format!("---\nname: {name}\ndescription: {description}\n---\n{dot_block}");
        std::fs::write(wf_dir.join("WORKFLOW.md"), content).expect("write WORKFLOW.md");
    }

    // -- discover() tests --

    #[tokio::test]
    async fn discover_finds_workflows_sorted() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workflow(tmp.path(), "beta-wf", "second workflow", None);
        create_workflow(tmp.path(), "alpha-wf", "first workflow", None);

        let workflows = discover(tmp.path()).await;

        assert_eq!(workflows.len(), 2);
        assert_eq!(workflows[0].name, "alpha-wf");
        assert_eq!(workflows[1].name, "beta-wf");
    }

    #[tokio::test]
    async fn discover_empty_when_no_workflows_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let workflows = discover(tmp.path()).await;
        assert!(workflows.is_empty());
    }

    // -- get_by_name() tests --

    #[tokio::test]
    async fn get_by_name_finds_workflow() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workflow(tmp.path(), "my-workflow", "a test workflow", None);

        let wf = get_by_name(tmp.path(), "my-workflow").await;
        assert!(wf.is_ok());
        assert_eq!(wf.expect("found").name, "my-workflow");
    }

    #[tokio::test]
    async fn get_by_name_returns_error_when_not_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = get_by_name(tmp.path(), "nonexistent").await;
        assert!(result.is_err());
    }

    // -- load_workflow() tests --

    #[tokio::test]
    async fn load_workflow_with_pipeline() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dot = r#"digraph test {
    node [shape=box]
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    work  [agent="code-engineer", prompt="Do something"]
    start -> work -> exit
}"#;
        create_workflow(tmp.path(), "test-wf", "A test workflow", Some(dot));

        let path = tmp.path().join(".stencila/workflows/test-wf/WORKFLOW.md");
        let instance = load_workflow(&path).await.expect("load");

        assert_eq!(instance.name, "test-wf");
        assert_eq!(instance.description, "A test workflow");
        assert!(instance.pipeline.is_some());
        assert!(instance.content.is_some());
    }

    #[tokio::test]
    async fn load_workflow_without_dot_block() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workflow(tmp.path(), "no-dot", "A workflow without pipeline", None);

        let path = tmp.path().join(".stencila/workflows/no-dot/WORKFLOW.md");
        let instance = load_workflow(&path).await.expect("load");

        assert_eq!(instance.name, "no-dot");
        assert!(instance.pipeline.is_none());
    }

    // -- graph() tests --

    #[tokio::test]
    async fn graph_returns_parsed_graph() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dot = r#"digraph test {
    node [shape=box]
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    work  [agent="code-engineer", prompt="Do something"]
    start -> work -> exit
}"#;
        create_workflow(tmp.path(), "graph-wf", "A test workflow", Some(dot));

        let path = tmp.path().join(".stencila/workflows/graph-wf/WORKFLOW.md");
        let instance = load_workflow(&path).await.expect("load");

        let graph = instance.graph();
        assert!(graph.is_ok());
    }

    #[tokio::test]
    async fn graph_returns_error_when_pipeline_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_workflow(tmp.path(), "no-pipe", "No pipeline", None);

        let path = tmp.path().join(".stencila/workflows/no-pipe/WORKFLOW.md");
        let instance = load_workflow(&path).await.expect("load");

        let graph = instance.graph();
        assert!(graph.is_err());
        match graph {
            Ok(..) => panic!("Expected error"),
            Err(error) => assert!(error.to_string().contains("has no pipeline defined")),
        }
    }

    // -- agent_references() tests --

    #[tokio::test]
    async fn agent_references_returns_agent_names() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dot = r#"digraph test {
    node [shape=box]
    start [shape=Mdiamond]
    exit  [shape=Msquare]
    review  [agent="code-reviewer", prompt="Review code"]
    write   [agent="code-engineer", prompt="Write code"]
    start -> write -> review -> exit
}"#;
        create_workflow(tmp.path(), "refs-wf", "Agent refs test", Some(dot));

        let path = tmp.path().join(".stencila/workflows/refs-wf/WORKFLOW.md");
        let instance = load_workflow(&path).await.expect("load");

        let agents = instance.agent_references();
        assert!(agents.contains(&"code-reviewer".to_string()));
        assert!(agents.contains(&"code-engineer".to_string()));
        assert_eq!(agents.len(), 2);
    }
}
