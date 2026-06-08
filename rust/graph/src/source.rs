//! Source metadata for graph roots.
//!
//! Graph edges explain relationships inside a graph. Repository metadata belongs
//! on the graph itself because it describes the source snapshot for the whole
//! graph rather than evidence for any particular edge.

use std::{
    collections::BTreeMap,
    path::Path,
    process::{Command, Stdio},
};

use eyre::Result;
use stencila_codec_utils::{closest_git_repo, git_head_sha, git_repo_info};
use stencila_schema::{Author, Graph, Node, Person, WorktreeStatus};

const GIT_LOG_RECORD_SEPARATOR: u8 = 0x1e;
const GIT_LOG_UNIT_SEPARATOR: u8 = 0x1f;

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

/// Collect unique Git commit authors for files under a workspace path.
///
/// The returned map is keyed by workspace-relative path. History is inspected
/// once for the whole workspace subtree, and authors are ordered by their first
/// appearance in Git's default reverse-chronological log order.
pub(crate) fn git_authors_by_workspace_path(root: &Path) -> BTreeMap<String, Vec<Author>> {
    let Ok(repo_root) = closest_git_repo(root) else {
        return BTreeMap::new();
    };

    let Some(workspace_repo_path) = git_relative_path(root, &repo_root) else {
        return BTreeMap::new();
    };

    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["log", "--format=%x1e%aN%x1f%aE", "--name-only", "-z", "--"])
        .arg(&workspace_repo_path)
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else {
        return BTreeMap::new();
    };
    if !output.status.success() {
        return BTreeMap::new();
    }

    parse_git_authors_by_workspace_path(&output.stdout, &workspace_repo_path)
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitAuthor {
    name: String,
    email: String,
}

fn parse_git_authors_by_workspace_path(
    output: &[u8],
    workspace_repo_path: &str,
) -> BTreeMap<String, Vec<Author>> {
    let mut current_author = None;
    let mut authors_by_path = BTreeMap::<String, Vec<GitAuthor>>::new();

    for record in output.split(|byte| *byte == 0) {
        let record = trim_git_log_record(record);
        if record.is_empty() {
            continue;
        }

        if let Some(author) = parse_git_author_record(record) {
            current_author = Some(author);
            continue;
        }

        let Some(author) = current_author.clone() else {
            continue;
        };
        let Ok(repo_path) = std::str::from_utf8(record) else {
            continue;
        };
        let Some(workspace_path) = workspace_relative_git_path(repo_path, workspace_repo_path)
        else {
            continue;
        };

        let authors = authors_by_path
            .entry(workspace_path.to_string())
            .or_default();
        if !authors.contains(&author) {
            authors.push(author);
        }
    }

    authors_by_path
        .into_iter()
        .filter_map(|(path, authors)| {
            let authors = authors
                .into_iter()
                .map(GitAuthor::into_schema_author)
                .collect::<Vec<_>>();
            (!authors.is_empty()).then_some((path, authors))
        })
        .collect()
}

fn trim_git_log_record(mut record: &[u8]) -> &[u8] {
    while matches!(record.first(), Some(b'\n' | b'\r')) {
        record = &record[1..];
    }
    while matches!(record.last(), Some(b'\n' | b'\r')) {
        record = &record[..record.len() - 1];
    }
    record
}

fn parse_git_author_record(record: &[u8]) -> Option<GitAuthor> {
    let record = record.strip_prefix(&[GIT_LOG_RECORD_SEPARATOR])?;
    let separator = record
        .iter()
        .position(|byte| *byte == GIT_LOG_UNIT_SEPARATOR)?;
    let name = String::from_utf8_lossy(&record[..separator])
        .trim()
        .to_string();
    let email = String::from_utf8_lossy(&record[separator + 1..])
        .trim()
        .to_string();

    (!name.is_empty() || !email.is_empty()).then_some(GitAuthor { name, email })
}

fn workspace_relative_git_path<'a>(
    repo_path: &'a str,
    workspace_repo_path: &str,
) -> Option<&'a str> {
    if workspace_repo_path == "." {
        return Some(repo_path);
    }

    repo_path
        .strip_prefix(workspace_repo_path)
        .and_then(|path| path.strip_prefix('/'))
        .filter(|path| !path.is_empty())
}

impl GitAuthor {
    fn into_schema_author(self) -> Author {
        let mut person = Person::new();
        if !self.name.is_empty() {
            person.options.name = Some(self.name);
        }
        if !self.email.is_empty() {
            person.options.emails = Some(vec![self.email]);
        }
        Author::Person(person)
    }
}
