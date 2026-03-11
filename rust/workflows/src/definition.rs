//! Workflow definition loading and discovery.
//!
//! Parses `WORKFLOW.md` files (YAML frontmatter + optional Markdown body with
//! DOT pipeline) from workspace `.stencila/workflows/<name>/WORKFLOW.md`
//! and provides a public API mirroring the agents crate pattern.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Block, Node, NodeType, Workflow};

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
        let mut state = serializer.serialize_struct("WorkflowInstance", 7)?;

        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("goal", &self.inner.goal)?;
        state.serialize_field("source", &self.source.map(|s| s.to_string()))?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("ephemeral", &self.is_ephemeral())?;
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
        let mut graph = stencila_attractor::parse_dot(pipeline).map_err(|e| {
            eyre::eyre!("Failed to parse pipeline for workflow `{}`: {e}", self.name)
        })?;
        self.resolve_content_references(&mut graph)?;
        Ok(graph)
    }

    /// Whether this workflow directory is marked ephemeral.
    ///
    /// Ephemeral workflows have a `.gitignore` file containing `*` in their
    /// directory, indicating they were created by an agent for one-time use
    /// and can be discarded later.
    pub fn is_ephemeral(&self) -> bool {
        is_ephemeral_dir(&self.home)
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

    fn resolve_content_references(&self, graph: &mut stencila_attractor::Graph) -> Result<()> {
        const REF_ATTRS: &[(&str, &[&str])] = &[
            ("prompt", &["prompt_ref", "prompt-ref"]),
            ("shell", &["shell_ref", "shell-ref"]),
            ("ask", &["ask_ref", "ask-ref"]),
            ("interview", &["interview_ref", "interview-ref"]),
        ];

        let defs = self.content_blocks_by_id()?;

        for node in graph.nodes.values_mut() {
            for &(attr, ref_attrs) in REF_ATTRS {
                if let Some((ref_attr, reference)) = ref_attrs.iter().find_map(|ref_attr| {
                    node.get_str_attr(ref_attr)
                        .map(|v| (*ref_attr, v.to_string()))
                }) {
                    if node.attrs.contains_key(attr) {
                        bail!(
                            "Workflow `{}` node `{}` can not set both `{attr}` and `{ref_attr}`",
                            self.name,
                            node.id
                        );
                    }

                    let id = reference.trim_start_matches('#');
                    let value = defs.get(id).ok_or_else(|| {
                        eyre::eyre!(
                            "Workflow `{}` node `{}` references missing content block `#{id}` via `{ref_attr}`",
                            self.name,
                            node.id
                        )
                    })?;

                    node.attrs.insert(attr.to_string(), value.clone().into());
                }

                for ref_attr in ref_attrs {
                    node.attrs.shift_remove(*ref_attr);
                }
            }
        }

        Ok(())
    }

    fn content_blocks_by_id(&self) -> Result<BTreeMap<String, String>> {
        let mut defs = BTreeMap::new();

        for block in self.content.iter().flatten() {
            let (id, code) = match block {
                Block::CodeBlock(cb) => (&cb.id, &cb.code),
                Block::CodeChunk(cc) => (&cc.id, &cc.code),
                _ => continue,
            };

            let Some(id) = id else { continue };

            if defs.insert(id.clone(), code.to_string()).is_some() {
                bail!(
                    "Workflow `{}` has duplicate content block id `#{id}`",
                    self.name
                );
            }
        }

        Ok(defs)
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

/// The sentinel file used to mark a workflow directory as ephemeral.
const EPHEMERAL_SENTINEL: &str = ".gitignore";

/// The content written to the ephemeral sentinel file.
const EPHEMERAL_SENTINEL_CONTENT: &str = "*\n";

/// Check whether a workflow directory is marked as ephemeral.
fn is_ephemeral_dir(dir: &Path) -> bool {
    let sentinel = dir.join(EPHEMERAL_SENTINEL);
    sentinel
        .is_file()
        .then(|| std::fs::read_to_string(&sentinel).ok())
        .flatten()
        .is_some_and(|content| content.trim() == "*")
}

/// Get the workflow directory for a given name relative to the current working directory.
fn workflow_dir(cwd: &Path) -> Option<PathBuf> {
    let stencila_dir = stencila_dirs::closest_dot_dir(cwd, ".stencila")?;
    Some(stencila_dir.join("workflows"))
}

/// Check if a named workflow is ephemeral.
pub fn is_ephemeral(cwd: &Path, name: &str) -> bool {
    workflow_dir(cwd)
        .map(|dir| dir.join(name))
        .is_some_and(|dir| is_ephemeral_dir(&dir))
}

/// Save an ephemeral workflow by removing its ephemeral sentinel.
///
/// Returns `Ok(true)` if the workflow was ephemeral and is now saved,
/// `Ok(false)` if it was not ephemeral or doesn't exist.
pub fn save_ephemeral(cwd: &Path, name: &str) -> Result<bool> {
    let Some(workflows_dir) = workflow_dir(cwd) else {
        return Ok(false);
    };
    let dir = workflows_dir.join(name);
    if !is_ephemeral_dir(&dir) {
        return Ok(false);
    }
    std::fs::remove_file(dir.join(EPHEMERAL_SENTINEL))?;
    Ok(true)
}

/// Discard an ephemeral workflow by removing its entire directory.
///
/// Returns `Ok(true)` if the workflow was ephemeral and has been removed,
/// `Ok(false)` if it was not ephemeral or doesn't exist.
pub fn discard_ephemeral(cwd: &Path, name: &str) -> Result<bool> {
    let Some(workflows_dir) = workflow_dir(cwd) else {
        return Ok(false);
    };
    let dir = workflows_dir.join(name);
    if !is_ephemeral_dir(&dir) {
        return Ok(false);
    }
    std::fs::remove_dir_all(dir)?;
    Ok(true)
}

/// Create the ephemeral sentinel file (`.gitignore` with `*`) in a directory.
pub fn mark_ephemeral(dir: &Path) -> Result<()> {
    std::fs::write(dir.join(EPHEMERAL_SENTINEL), EPHEMERAL_SENTINEL_CONTENT)?;
    Ok(())
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

    async fn load_inline_workflow(
        name: &str,
        content: &str,
    ) -> (tempfile::TempDir, WorkflowInstance) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wf_dir = tmp.path().join(format!(".stencila/workflows/{name}"));
        std::fs::create_dir_all(&wf_dir).expect("create workflow dir");
        std::fs::write(wf_dir.join("WORKFLOW.md"), content).expect("write WORKFLOW.md");
        let instance = load_workflow(&wf_dir.join("WORKFLOW.md"))
            .await
            .expect("load");
        (tmp, instance)
    }

    #[tokio::test]
    async fn graph_resolves_prompt_shell_and_ask_refs() {
        let (_tmp, instance) = load_inline_workflow(
            "refs-content",
            r##"---
name: refs-content
description: Test refs
---

```text #creator-prompt
Create or revise the draft using:
$goal
```

```sh #run-script
echo hello
echo world
```

```text #human-question
What should change next?
```

```dot
digraph refs_content {
    Start -> Create -> Run -> Ask -> End

    Create [agent="writer", prompt-ref="#creator-prompt"]
    Run    [shell-ref="#run-script"]
    Ask    [ask-ref="#human-question", question-type="freeform"]
}
```
"##,
        )
        .await;
        let graph = instance.graph().expect("graph");

        let create = graph.nodes.get("Create").expect("Create node");
        assert_eq!(
            create.get_str_attr("prompt"),
            Some("Create or revise the draft using:\n$goal")
        );
        assert_eq!(create.shape(), stencila_attractor::Graph::CODERGEN_SHAPE);

        let run = graph.nodes.get("Run").expect("Run node");
        assert_eq!(run.get_str_attr("shell"), Some("echo hello\necho world"));

        let ask = graph.nodes.get("Ask").expect("Ask node");
        assert_eq!(ask.get_str_attr("ask"), Some("What should change next?"));
    }

    #[tokio::test]
    async fn graph_errors_on_missing_ref_target() {
        let (_tmp, instance) = load_inline_workflow(
            "missing-ref",
            r##"---
name: missing-ref
description: Test missing ref
---

```dot
digraph missing_ref {
    Start -> Create -> End
    Create [agent="writer", prompt-ref="#missing-prompt"]
}
```
"##,
        )
        .await;
        let error = instance.graph().expect_err("missing ref should error");
        assert!(
            error
                .to_string()
                .contains("missing content block `#missing-prompt`")
        );
    }

    #[tokio::test]
    async fn graph_errors_when_literal_and_ref_both_present() {
        let (_tmp, instance) = load_inline_workflow(
            "conflicting-ref",
            r##"---
name: conflicting-ref
description: Test conflicting literal and ref attrs
---

```text #creator-prompt
Create a draft.
```

```dot
digraph conflicting_ref {
    Start -> Create -> End
    Create [prompt="literal prompt", prompt-ref="#creator-prompt"]
}
```
"##,
        )
        .await;
        let error = instance
            .graph()
            .expect_err("conflicting attrs should error");
        let message = error.to_string();
        assert!(message.contains("can not set both `prompt`"), "{message}");
    }

    #[tokio::test]
    async fn graph_resolves_interview_ref() {
        let (_tmp, instance) = load_inline_workflow(
            "interview-ref",
            r##"---
name: interview-ref
description: Test interview ref
---

```yaml #review-interview
questions:
  - question: "What areas need improvement?"
    type: freeform
    store: review.improvements
  - question: "Are there any blocking issues?"
    type: yes-no
```

```dot
digraph interview_ref {
    Start -> Review -> End
    Review [interview-ref="#review-interview"]
}
```
"##,
        )
        .await;
        let graph = instance.graph().expect("graph");

        let review = graph.nodes.get("Review").expect("Review node");
        assert_eq!(
            review.get_str_attr("interview"),
            Some(
                "questions:\n  - question: \"What areas need improvement?\"\n    type: freeform\n    store: review.improvements\n  - question: \"Are there any blocking issues?\"\n    type: yes-no"
            )
        );
    }

    #[tokio::test]
    async fn graph_errors_on_interview_and_interview_ref_conflict() {
        let (_tmp, instance) = load_inline_workflow(
            "interview-conflict",
            r##"---
name: interview-conflict
description: Test conflicting interview attrs
---

```yaml #review-interview
questions:
  - question: "What areas need improvement?"
```

```dot
digraph interview_conflict {
    Start -> Review -> End
    Review [interview="inline interview", interview-ref="#review-interview"]
}
```
"##,
        )
        .await;
        let error = instance
            .graph()
            .expect_err("conflicting attrs should error");
        let message = error.to_string();
        assert!(
            message.contains("can not set both `interview`"),
            "{message}"
        );
    }
}
