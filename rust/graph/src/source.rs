//! Source metadata for graph roots.
//!
//! Graph edges explain relationships inside a graph. Repository metadata belongs
//! on the graph itself because it describes the source snapshot for the whole
//! graph rather than evidence for any particular edge.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use eyre::Result;
use stencila_codec_utils::{closest_git_repo, git_head_sha, git_repo_info};
use stencila_schema::{Graph, Node, WorktreeStatus};

/// Populate root `Graph` source metadata from a filesystem path when it is in a Git repo.
pub(crate) fn set_graph_source_metadata_from_path(graph: &mut Graph, path: &Path) -> Result<()> {
    let Ok(repo_root) = closest_git_repo(path) else {
        return Ok(());
    };

    let Some(relative_path) = git_relative_path(path, &repo_root) else {
        return Ok(());
    };

    graph.options.path = Some(relative_path.clone());

    if let Ok(repo_info) = git_repo_info(&repo_root) {
        graph.options.repository = repo_info.origin;
    }

    graph.options.commit = git_head_sha(&repo_root);
    graph.options.worktree_status = git_worktree_status(&repo_root, &relative_path);

    Ok(())
}

/// Populate root `Graph` source metadata from the input root node, when already available.
pub(crate) fn set_graph_source_metadata_from_node(graph: &mut Graph, node: &Node) {
    if let Some(metadata) = SourceMetadata::from_node(node) {
        metadata.apply_to(graph);
    }
}

#[derive(Debug, Clone)]
struct SourceMetadata {
    repository: Option<String>,
    path: Option<String>,
    commit: Option<String>,
    worktree_status: Option<WorktreeStatus>,
}

impl SourceMetadata {
    fn from_node(node: &Node) -> Option<Self> {
        macro_rules! from_options {
            ($work:expr) => {
                Self {
                    repository: $work.options.repository.clone(),
                    path: $work.options.path.clone(),
                    commit: $work.options.commit.clone(),
                    worktree_status: $work.options.worktree_status,
                }
            };
        }

        macro_rules! options_variants {
            ($( $variant:ident ),* $(,)?) => {
                match node {
                    $(
                        Node::$variant(work) => Some(from_options!(work)),
                    )*
                    Node::SoftwareSourceCode(work) => Some(Self {
                        repository: work.repository.clone(),
                        path: work.path.clone(),
                        commit: work.commit.clone(),
                        worktree_status: work.options.worktree_status,
                    }),
                    Node::File(work) => Some(Self {
                        repository: work.options.repository.clone(),
                        path: Some(work.path.clone()),
                        commit: work.options.commit.clone(),
                        worktree_status: work.options.worktree_status,
                    }),
                    _ => None,
                }
            };
        }

        options_variants!(
            Agent,
            Article,
            AudioObject,
            Chat,
            Claim,
            Collection,
            Comment,
            CreativeWork,
            Datatable,
            Figure,
            Graph,
            ImageObject,
            MediaObject,
            Periodical,
            Prompt,
            PublicationIssue,
            PublicationVolume,
            Review,
            Skill,
            SoftwareApplication,
            Table,
            VideoObject,
            Workflow,
        )
    }

    fn apply_to(self, graph: &mut Graph) {
        graph.options.repository = self.repository;
        graph.options.path = self.path;
        graph.options.commit = self.commit;
        graph.options.worktree_status = self.worktree_status;
    }
}

fn git_relative_path(path: &Path, repo_root: &Path) -> Option<String> {
    let relative = path.strip_prefix(repo_root).ok()?;
    if relative.as_os_str().is_empty() {
        return Some(".".to_string());
    }

    relative
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .map(|components| components.join("/"))
}

fn git_worktree_status(repo_root: &Path, relative_path: &str) -> Option<WorktreeStatus> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["status", "--porcelain", "--untracked-files=normal", "--"])
        .arg(relative_path)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output = String::from_utf8(output.stdout).ok()?;
    let mut saw_untracked = false;

    for line in output.lines() {
        if line.as_bytes().starts_with(b"??") {
            saw_untracked = true;
        } else {
            return Some(WorktreeStatus::Dirty);
        }
    }

    Some(if saw_untracked {
        WorktreeStatus::Untracked
    } else {
        WorktreeStatus::Clean
    })
}
